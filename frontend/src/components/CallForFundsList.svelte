<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../lib/i18n";
  import { api, callForFundsApi } from "../lib/api";
  import type { Building } from "../lib/types";
  import { formatDate } from "../lib/utils/date.utils";
  import { formatCurrency } from "../lib/utils/finance.utils";
  import { withErrorHandling } from "../lib/utils/error.utils";
  // Track H Story H2 — validate-before-compute (FR-H2).
  import ConformityBanner from "../lib/components/shared/ConformityBanner.svelte";
  import {
    buildConformityStatus,
    showConformityToast,
  } from "../lib/utils/conformity";
  import ConfirmDialog from "./ui/ConfirmDialog.svelte";

  let {
    buildingId = undefined,
    statusFilter = undefined,
    onCreate = () => {},
  }: {
    buildingId?: string | undefined;
    statusFilter?: string | undefined;
    onCreate?: () => void;
  } = $props();

  let calls = $state<any[]>([]);
  let loading = $state(true);

  /// L'action en attente, et l'appel de fonds qu'elle vise.
  ///
  /// Les trois `confirm()` remplacés étaient des dialogues du NAVIGATEUR : un
  /// navigateur piloté les supprime, et l'action prend la forme exacte d'une
  /// panne (#844).
  ///
  /// Envoyer un appel de fonds le notifie à tous les copropriétaires,
  /// l'annuler défait cette notification, le supprimer l'efface. Trois actes
  /// qui engagent la trésorerie de la copropriété.
  let actionEnAttente = $state<"envoyer" | "annuler" | "supprimer" | null>(
    null,
  );
  let cibleEnAttente = $state<string | null>(null);
  // Track H Story H2 — building enrichi pour gating UI.
  let building = $state<Building | null>(null);

  let filteredCalls = $derived.by(() => {
    if (statusFilter && statusFilter !== "all") {
      if (statusFilter === "overdue") {
        return calls.filter((c) => c.is_overdue);
      } else {
        return calls.filter((c) => c.status === statusFilter);
      }
    }
    return calls;
  });

  $effect(() => {
    loadCalls();
    if (buildingId) {
      loadBuilding();
    }
  });

  // Track H Story H2 — Statut conformité dérivé. Cf. pattern Story H1
  // BuildingDetail.svelte. `null` si building pas chargé → canCompute=true
  // (BE 422 reste source de vérité défense profondeur).
  let conformityStatus = $derived(
    building && building.is_conformant !== undefined
      ? buildConformityStatus({
          is_conformant: !!building.is_conformant,
          total_units: building.total_units,
          units_count: building.units_count ?? 0,
          total_tantiemes: building.total_tantiemes,
          quota_delta: building.quota_delta ?? "0",
        })
      : null,
  );
  let canCompute = $derived(
    conformityStatus ? conformityStatus.is_conformant : true,
  );

  async function loadBuilding() {
    if (!buildingId) return;
    try {
      building = await api.get<Building>(`/buildings/${buildingId}`);
    } catch (e) {
      console.error("Failed to load building metrics:", e);
    }
  }

  async function loadCalls() {
    loading = true;
    const result = await withErrorHandling({
      action: () => callForFundsApi.list(buildingId),
      errorMessage: $_("callForFunds.loadError"),
    });
    if (result) calls = result;
    loading = false;
  }

  function handleSend(id: string) {
    cibleEnAttente = id;
    actionEnAttente = "envoyer";
  }

  async function executer_envoyer() {
    const id = cibleEnAttente;
    actionEnAttente = null;
    cibleEnAttente = null;
    if (!id) return;
    try {
      await callForFundsApi.send(id);
      await loadCalls();
    } catch (err) {
      // Track H Story H2 — toast narratif si 422 BUILDING_NOT_CONFORMANT.
      // Sinon, fallback message standard.
      if (!showConformityToast(err)) {
        const { toast } = await import("../stores/toast");
        toast.error($_("callForFunds.sendError"));
      }
    }
  }

  function handleCancel(id: string) {
    cibleEnAttente = id;
    actionEnAttente = "annuler";
  }

  async function executer_annuler() {
    const id = cibleEnAttente;
    actionEnAttente = null;
    cibleEnAttente = null;
    if (!id) return;
    const result = await withErrorHandling({
      action: () => callForFundsApi.cancel(id),
      successMessage: $_("callForFunds.cancelled"),
      errorMessage: $_("callForFunds.cancelError"),
    });
    if (result !== undefined) await loadCalls();
  }

  function handleDelete(id: string) {
    cibleEnAttente = id;
    actionEnAttente = "supprimer";
  }

  async function executer_supprimer() {
    const id = cibleEnAttente;
    actionEnAttente = null;
    cibleEnAttente = null;
    if (!id) return;
    const result = await withErrorHandling({
      action: () => callForFundsApi.delete(id),
      successMessage: $_("callForFunds.deleted"),
      errorMessage: $_("callForFunds.deleteError"),
    });
    if (result !== undefined) await loadCalls();
  }

  function getStatusBadgeClass(status: string): string {
    const classes: Record<string, string> = {
      draft: "bg-gray-100 text-gray-800",
      sent: "bg-blue-100 text-blue-800",
      partial: "bg-yellow-100 text-yellow-800",
      completed: "bg-green-100 text-green-800",
      cancelled: "bg-red-100 text-red-800",
    };
    return classes[status] || "bg-gray-100 text-gray-800";
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      draft: "Brouillon",
      sent: "Envoyé",
      partial: "Partiellement payé",
      completed: "Complété",
      cancelled: "Annulé",
    };
    return labels[status] || status;
  }

  function getContributionTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      regular: "Charges régulières",
      extraordinary: "Charges extraordinaires",
      advance: "Avance",
      adjustment: "Régularisation",
    };
    return labels[type] || type;
  }
</script>

<div class="space-y-4" data-testid="call-for-funds-list">
  <!-- Track H Story H2 — Banner conformité (FR-H2). -->
  {#if conformityStatus && building}
    <ConformityBanner
      status={conformityStatus}
      buildingId={building.id}
      buildingName={building.name}
    />
  {/if}

  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">{$_("callForFunds.title")}</h2>
    <button
      onclick={onCreate}
      disabled={!canCompute}
      aria-disabled={!canCompute}
      title={!canCompute ? $_("conformity.toast_title") : ""}
      class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      data-testid="call-for-funds-create-button"
      data-can-compute={canCompute}
    >
      + {$_("callForFunds.new")}
    </button>
  </div>

  {#if loading}
    <div class="text-center py-8">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"
      ></div>
      <p class="mt-2 text-gray-600">{$_("common.loading")}</p>
    </div>
  {:else if filteredCalls.length === 0}
    <div class="text-center py-12 bg-gray-50 rounded-lg">
      <svg
        class="mx-auto h-12 w-12 text-gray-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <p class="mt-2 text-gray-600">{$_("callForFunds.none")}</p>
      <button onclick={onCreate} class="mt-4 text-blue-600 hover:text-blue-800">
        {$_("callForFunds.createFirst")}
      </button>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.title")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.type")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.amount")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.callDate")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.dueDate")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("callForFunds.status")}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_("common.actions")}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredCalls as call (call.id)}
            <tr class:bg-red-50={call.is_overdue}>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">
                  {call.title}
                </div>
                <div class="text-sm text-gray-500">{call.description}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {getContributionTypeLabel(call.contribution_type)}
              </td>
              <td
                class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900"
              >
                {formatCurrency(call.total_amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.call_date)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.due_date)}
                {#if call.is_overdue}
                  <span
                    class="ml-2 px-2 py-1 text-xs font-semibold rounded-full bg-red-100 text-red-800"
                  >
                    {$_("callForFunds.overdue")}
                  </span>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span
                  class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getStatusBadgeClass(
                    call.status,
                  )}"
                >
                  {getStatusLabel(call.status)}
                </span>
              </td>
              <td
                class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2"
              >
                {#if call.status === "draft"}
                  <button
                    onclick={() => handleSend(call.id)}
                    class="text-blue-600 hover:text-blue-900"
                    aria-label={$_("callForFunds.sendTitle")}
                    title={$_("callForFunds.sendTitle")}
                  >
                    {$_("callForFunds.send")}
                  </button>
                  <button
                    onclick={() => handleDelete(call.id)}
                    class="text-red-600 hover:text-red-900"
                    aria-label={$_("callForFunds.deleteTitle")}
                    title={$_("callForFunds.deleteTitle")}
                  >
                    {$_("common.delete")}
                  </button>
                {:else if call.status === "sent" || call.status === "partial"}
                  <button
                    onclick={() => handleCancel(call.id)}
                    class="text-orange-600 hover:text-orange-900"
                    aria-label={$_("callForFunds.cancelTitle")}
                    title={$_("callForFunds.cancelTitle")}
                  >
                    {$_("common.cancel")}
                  </button>
                  <a
                    href="/owner-contributions?call_for_funds_id={call.id}"
                    class="text-green-600 hover:text-green-900"
                    aria-label={$_("callForFunds.viewContributions")}
                    title={$_("callForFunds.viewContributions")}
                  >
                    {$_("callForFunds.contributions")}
                  </a>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- Le dialogue qui remplace trois `confirm()` natifs (#844). -->
<ConfirmDialog
  isOpen={actionEnAttente !== null}
  title={$_("common.confirm")}
  message={actionEnAttente === "envoyer"
    ? $_("callForFunds.sendConfirm")
    : actionEnAttente === "annuler"
      ? $_("callForFunds.cancelConfirm")
      : actionEnAttente === "supprimer"
        ? $_("callForFunds.deleteConfirm")
        : ""}
  variant={actionEnAttente === "envoyer" ? "primary" : "danger"}
  onconfirm={() => {
    if (actionEnAttente === "envoyer") executer_envoyer();
    else if (actionEnAttente === "annuler") executer_annuler();
    else if (actionEnAttente === "supprimer") executer_supprimer();
  }}
  oncancel={() => {
    actionEnAttente = null;
    cibleEnAttente = null;
  }}
/>
