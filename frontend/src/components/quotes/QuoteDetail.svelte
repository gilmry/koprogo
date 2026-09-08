<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import {
    quotesApi,
    type Quote,
    type SubmitQuoteDto,
    QuoteStatus,
  } from "../../lib/api/quotes";
  import { toast } from "../../stores/toast";
  import { authStore } from "../../stores/auth";
  import { UserRole } from "../../lib/types";
  import QuoteStatusBadge from "./QuoteStatusBadge.svelte";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import { formatDate } from "../../lib/utils/date.utils";
  import { formatAmount } from "../../lib/utils/finance.utils";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";
  import Modal from "../ui/Modal.svelte";
  import Button from "../ui/Button.svelte";

  let {
    quote,
    onupdated,
    ondeleted,
  }: {
    quote: Quote;
    onupdated?: (updated: Quote) => void;
    ondeleted?: () => void;
  } = $props();

  let actionLoading = $state(false);

  /// L'action en attente, et la note de décision saisie.
  ///
  /// Deux `prompt()` et deux `confirm()` NATIFS. Un navigateur piloté les
  /// supprime, et l'action prend la forme exacte d'une panne — aucun dialogue,
  /// aucune requête, aucun message (#844).
  ///
  /// Accepter ou refuser un devis est une DÉCISION du syndic sur un marché,
  /// et la note qui l'accompagne est la trace de son motif. La perdre dans un
  /// dialogue que l'outil de recette avale, c'est perdre la justification
  /// d'un choix engageant la copropriété.
  let actionEnAttente = $state<"retirer" | "supprimer" | null>(null);
  let decisionEnAttente = $state<"accepter" | "refuser" | null>(null);
  let noteDeDecision = $state("");
  let showSubmitForm = $state(false);

  let amountExclVat = $state("");
  let vatRate = $state("21");
  let validityDate = $state("");
  let estimatedDays = $state("");
  let warrantyYears = $state("2");

  let isAdmin = $derived(
    $authStore.user?.role === UserRole.SYNDIC ||
      $authStore.user?.role === UserRole.SUPERADMIN,
  );

  function formatQuoteAmount(amountCents: number | undefined): string {
    if (!amountCents) return "-";
    return formatAmount(amountCents);
  }

  async function handleSubmitQuote() {
    if (!amountExclVat || !validityDate || !estimatedDays) {
      toast.error($_("quotes.detail.fillRequired"));
      return;
    }

    const updated = await withErrorHandling({
      action: () => {
        const data: SubmitQuoteDto = {
          amount_excl_vat_cents: Math.round(parseFloat(amountExclVat) * 100),
          vat_rate: parseFloat(vatRate),
          validity_date: validityDate,
          estimated_duration_days: parseInt(estimatedDays),
          warranty_years: parseInt(warrantyYears),
        };
        return quotesApi.submit(quote.id, data);
      },
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.submitSuccess"),
      errorMessage: $_("quotes.detail.submitError"),
    });
    if (updated) {
      showSubmitForm = false;
      onupdated?.(updated);
    }
  }

  async function handleStartReview() {
    const updated = await withErrorHandling({
      action: () => quotesApi.startReview(quote.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.reviewSuccess"),
      errorMessage: $_("common.error"),
    });
    if (updated) onupdated?.(updated);
  }

  function handleAccept() {
    decisionEnAttente = "accepter";
  }

  async function executerAcceptation() {
    const notes = noteDeDecision;
    decisionEnAttente = null;
    noteDeDecision = "";
    const updated = await withErrorHandling({
      action: () =>
        quotesApi.accept(quote.id, {
          decision_by: $authStore.user?.id || "",
          decision_notes: notes || undefined,
        }),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.acceptSuccess"),
      errorMessage: $_("common.error"),
    });
    if (updated) onupdated?.(updated);
  }

  function handleReject() {
    decisionEnAttente = "refuser";
  }

  async function executerRefus() {
    const notes = noteDeDecision;
    decisionEnAttente = null;
    noteDeDecision = "";
    if (!notes) return;
    const updated = await withErrorHandling({
      action: () =>
        quotesApi.reject(quote.id, {
          decision_by: $authStore.user?.id || "",
          decision_notes: notes,
        }),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.rejectSuccess"),
      errorMessage: $_("common.error"),
    });
    if (updated) onupdated?.(updated);
  }

  function handleWithdraw() {
    actionEnAttente = "retirer";
  }

  async function executerRetrait() {
    actionEnAttente = null;
    const updated = await withErrorHandling({
      action: () => quotesApi.withdraw(quote.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.withdrawSuccess"),
      errorMessage: $_("common.error"),
    });
    if (updated) onupdated?.(updated);
  }

  function handleDelete() {
    actionEnAttente = "supprimer";
  }

  async function executerSuppression() {
    actionEnAttente = null;
    await withErrorHandling({
      action: () => quotesApi.delete(quote.id),
      setLoading: (v: boolean) => (actionLoading = v),
      successMessage: $_("quotes.detail.deleteSuccess"),
      errorMessage: $_("quotes.detail.deleteError"),
      onSuccess: () => ondeleted?.(),
    });
  }
</script>

<div
  class="bg-gray-50 border border-gray-200 rounded-lg p-4 space-y-4"
  data-testid="quote-detail"
>
  <div class="flex items-center justify-between">
    <div>
      <h4 class="text-sm font-semibold text-gray-900" data-testid="quote-title">
        {quote.project_title}
      </h4>
      <p class="text-xs text-gray-500 mt-0.5">
        {quote.contractor_name || quote.contractor_id.slice(0, 8)} - {quote.work_category}
      </p>
    </div>
    <span data-testid="quote-status-badge"
      ><QuoteStatusBadge status={quote.status} /></span
    >
  </div>

  <!-- Description -->
  {#if quote.project_description}
    <p class="text-sm text-gray-600">{quote.project_description}</p>
  {/if}

  <!-- Info grid -->
  {#if quote.amount_excl_vat_cents || quote.estimated_duration_days || quote.warranty_years}
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
      {#if quote.amount_excl_vat_cents}
        <div>
          <p class="text-xs text-gray-500">
            {$_("quotes.detail.amountExclVat")}
          </p>
          <p class="text-sm font-medium text-gray-900">
            {formatQuoteAmount(quote.amount_excl_vat_cents)}
          </p>
        </div>
      {/if}
      {#if quote.amount_incl_vat_cents}
        <div>
          <p class="text-xs text-gray-500">
            {$_("quotes.detail.amountInclVat")} ({quote.vat_rate}%)
          </p>
          <p class="text-sm font-medium text-gray-900">
            {formatQuoteAmount(quote.amount_incl_vat_cents)}
          </p>
        </div>
      {/if}
      {#if quote.estimated_duration_days}
        <div>
          <p class="text-xs text-gray-500">
            {$_("quotes.detail.estimatedDuration")}
          </p>
          <p class="text-sm font-medium text-gray-900">
            {quote.estimated_duration_days}
            {$_("quotes.detail.days")}
          </p>
        </div>
      {/if}
      {#if quote.warranty_years}
        <div>
          <p class="text-xs text-gray-500">{$_("quotes.detail.warranty")}</p>
          <p class="text-sm font-medium text-gray-900">
            {quote.warranty_years}
            {$_("quotes.detail.years")}
          </p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Dates -->
  <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-500">
    <span>{$_("quotes.detail.createdOn")} {formatDate(quote.created_at)}</span>
    {#if quote.validity_date}
      <span
        >{$_("quotes.detail.validUntil")}
        {formatDate(quote.validity_date)}</span
      >
    {/if}
    {#if quote.submitted_at}
      <span
        >{$_("quotes.detail.submittedOn")}
        {formatDate(quote.submitted_at)}</span
      >
    {/if}
    {#if quote.decision_at}
      <span
        >{$_("quotes.detail.decisionOn")} {formatDate(quote.decision_at)}</span
      >
    {/if}
    {#if quote.contractor_rating !== undefined && quote.contractor_rating !== null}
      <span
        >{$_("quotes.detail.contractorRating")}: {quote.contractor_rating}/100</span
      >
    {/if}
  </div>

  {#if quote.decision_notes}
    <div
      class="p-2 bg-white border border-gray-100 rounded text-sm text-gray-700"
    >
      <span class="font-medium">{$_("quotes.detail.decisionNotes")}:</span>
      {quote.decision_notes}
    </div>
  {/if}

  <!-- Submit Form (contractor) -->
  {#if showSubmitForm}
    <div class="p-4 bg-white border border-amber-200 rounded-lg space-y-3">
      <h5 class="text-sm font-medium text-gray-900">
        {$_("quotes.detail.submitFormTitle")}
      </h5>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label for="amount" class="block text-xs text-gray-600 mb-1"
            >{$_("quotes.detail.amountExclVat")} (EUR) *</label
          >
          <input
            id="amount"
            type="number"
            step="0.01"
            min="0"
            bind:value={amountExclVat}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
          />
        </div>
        <div>
          <label for="vat" class="block text-xs text-gray-600 mb-1"
            >{$_("quotes.detail.vatRate")}</label
          >
          <select
            id="vat"
            bind:value={vatRate}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
          >
            <option value="6">6% ({$_("quotes.detail.renovation")})</option>
            <option value="12">12%</option>
            <option value="21">21% ({$_("quotes.detail.standard")})</option>
          </select>
        </div>
        <div>
          <label for="validity" class="block text-xs text-gray-600 mb-1"
            >{$_("quotes.detail.validityDate")} *</label
          >
          <input
            id="validity"
            type="date"
            bind:value={validityDate}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
          />
        </div>
        <div>
          <label for="duration" class="block text-xs text-gray-600 mb-1"
            >{$_("quotes.detail.estimatedDurationDays")} *</label
          >
          <input
            id="duration"
            type="number"
            min="1"
            bind:value={estimatedDays}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
          />
        </div>
        <div>
          <label for="warranty" class="block text-xs text-gray-600 mb-1"
            >{$_("quotes.detail.warrantyYears")}</label
          >
          <select
            id="warranty"
            bind:value={warrantyYears}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
          >
            <option value="1">1 {$_("quotes.detail.year")}</option>
            <option value="2"
              >2 {$_("quotes.detail.years")} ({$_(
                "quotes.detail.apparentDefects",
              )})</option
            >
            <option value="5">5 {$_("quotes.detail.years")}</option>
            <option value="10"
              >10 {$_("quotes.detail.years")} ({$_(
                "quotes.detail.decennial",
              )})</option
            >
          </select>
        </div>
      </div>
      <div class="flex gap-2">
        <button
          onclick={handleSubmitQuote}
          disabled={actionLoading}
          class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors"
        >
          {actionLoading
            ? $_("quotes.detail.submitting")
            : $_("quotes.detail.submit")}
        </button>
        <button
          onclick={() => (showSubmitForm = false)}
          class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors"
        >
          {$_("common.cancel")}
        </button>
      </div>
    </div>
  {/if}

  <!-- Actions -->
  <div
    class="flex flex-wrap gap-2 pt-2 border-t border-gray-100"
    data-testid="quote-actions"
  >
    {#if quote.status === QuoteStatus.Requested}
      <button
        onclick={() => (showSubmitForm = !showSubmitForm)}
        disabled={actionLoading}
        data-testid="submit-quote-button"
        class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.submitQuote")}
      </button>
      <button
        onclick={handleDelete}
        disabled={actionLoading}
        data-testid="delete-quote-button"
        class="px-3 py-1.5 bg-red-100 text-red-700 rounded-lg text-sm font-medium hover:bg-red-200 disabled:opacity-50 transition-colors"
      >
        {$_("common.delete")}
      </button>
    {:else if quote.status === QuoteStatus.Received && isAdmin}
      <button
        onclick={handleStartReview}
        disabled={actionLoading}
        data-testid="review-quote-button"
        class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.review")}
      </button>
      <button
        onclick={handleWithdraw}
        disabled={actionLoading}
        data-testid="withdraw-quote-button"
        class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.withdraw")}
      </button>
    {:else if quote.status === QuoteStatus.UnderReview && isAdmin}
      <button
        onclick={handleAccept}
        disabled={actionLoading}
        data-testid="accept-quote-button"
        class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.accept")}
      </button>
      <button
        onclick={handleReject}
        disabled={actionLoading}
        data-testid="reject-quote-button"
        class="px-3 py-1.5 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.reject")}
      </button>
    {:else if quote.status === QuoteStatus.UnderReview}
      <button
        onclick={handleWithdraw}
        disabled={actionLoading}
        data-testid="withdraw-quote-button"
        class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors"
      >
        {$_("quotes.detail.withdraw")}
      </button>
    {/if}
  </div>
</div>

<!-- Les dialogues qui remplacent deux `confirm()` et deux `prompt()` natifs
     (#844). La note de décision se saisit dans la page : c'est la trace du
     motif d'un choix qui engage la copropriété, elle ne peut pas vivre dans
     une boîte que l'outil de recette fait disparaître. -->
<ConfirmDialog
  isOpen={actionEnAttente !== null}
  title={$_("common.confirm")}
  message={actionEnAttente === "retirer"
    ? $_("quotes.detail.withdrawConfirm")
    : actionEnAttente === "supprimer"
      ? $_("quotes.detail.deleteConfirm")
      : ""}
  variant="danger"
  loading={actionLoading}
  onconfirm={() => {
    if (actionEnAttente === "retirer") executerRetrait();
    else if (actionEnAttente === "supprimer") executerSuppression();
  }}
  oncancel={() => (actionEnAttente = null)}
/>

<Modal
  isOpen={decisionEnAttente !== null}
  title={decisionEnAttente === "accepter"
    ? $_("quotes.detail.decisionNotesPrompt")
    : $_("quotes.detail.rejectReasonPrompt")}
  size="sm"
  onclose={() => {
    decisionEnAttente = null;
    noteDeDecision = "";
  }}
>
  <label
    class="block text-sm font-medium text-gray-700"
    for="quote-decision-notes"
  >
    {decisionEnAttente === "accepter"
      ? $_("quotes.detail.decisionNotesPrompt")
      : $_("quotes.detail.rejectReasonPrompt")}
  </label>
  <textarea
    id="quote-decision-notes"
    rows="3"
    bind:value={noteDeDecision}
    data-testid="quote-decision-notes-input"
    class="mt-1 block w-full rounded-md border-gray-300 shadow-sm sm:text-sm"
  ></textarea>

  {#snippet footer()}
    <div class="flex justify-end space-x-3">
      <Button
        variant="outline"
        onclick={() => {
          decisionEnAttente = null;
          noteDeDecision = "";
        }}
        data-testid="quote-decision-cancel"
      >
        {$_("common.cancel")}
      </Button>
      <Button
        variant="primary"
        onclick={() => {
          if (decisionEnAttente === "accepter") executerAcceptation();
          else executerRefus();
        }}
        disabled={actionLoading ||
          (decisionEnAttente === "refuser" && noteDeDecision.length === 0)}
        data-testid="quote-decision-submit"
      >
        {$_("common.confirm")}
      </Button>
    </div>
  {/snippet}
</Modal>
