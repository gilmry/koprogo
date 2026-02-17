<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    quotesApi,
    type Quote,
    type SubmitQuoteDto,
    QuoteStatus,
  } from '../../lib/api/quotes';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import QuoteStatusBadge from './QuoteStatusBadge.svelte';

  export let quote: Quote;

  const dispatch = createEventDispatcher<{ updated: Quote; deleted: void }>();

  let actionLoading = false;
  let showSubmitForm = false;

  // Submit form fields
  let amountExclVat = '';
  let vatRate = '21';
  let validityDate = '';
  let estimatedDays = '';
  let warrantyYears = '2';

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;

  function formatAmount(amountCents: number | undefined): string {
    if (!amountCents) return '-';
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amountCents / 100);
  }

  function formatDate(dateStr: string | undefined): string {
    if (!dateStr) return '-';
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    });
  }

  async function handleSubmitQuote() {
    if (!amountExclVat || !validityDate || !estimatedDays) {
      toast.error('Veuillez remplir tous les champs obligatoires');
      return;
    }

    try {
      actionLoading = true;
      const data: SubmitQuoteDto = {
        amount_excl_vat_cents: Math.round(parseFloat(amountExclVat) * 100),
        vat_rate: parseFloat(vatRate),
        validity_date: validityDate,
        estimated_duration_days: parseInt(estimatedDays),
        warranty_years: parseInt(warrantyYears),
      };
      const updated = await quotesApi.submit(quote.id, data);
      toast.success('Devis soumis');
      showSubmitForm = false;
      dispatch('updated', updated);
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la soumission');
    } finally {
      actionLoading = false;
    }
  }

  async function handleStartReview() {
    try {
      actionLoading = true;
      const updated = await quotesApi.startReview(quote.id);
      toast.success('Devis en cours de revue');
      dispatch('updated', updated);
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      actionLoading = false;
    }
  }

  async function handleAccept() {
    const notes = prompt('Notes de décision (optionnel) :') || '';
    try {
      actionLoading = true;
      const updated = await quotesApi.accept(quote.id, {
        decision_by: $authStore.user?.id || '',
        decision_notes: notes || undefined,
      });
      toast.success('Devis accepté');
      dispatch('updated', updated);
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      actionLoading = false;
    }
  }

  async function handleReject() {
    const notes = prompt('Raison du refus :');
    if (!notes) return;
    try {
      actionLoading = true;
      const updated = await quotesApi.reject(quote.id, {
        decision_by: $authStore.user?.id || '',
        decision_notes: notes,
      });
      toast.success('Devis refusé');
      dispatch('updated', updated);
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      actionLoading = false;
    }
  }

  async function handleWithdraw() {
    if (!confirm('Retirer ce devis ?')) return;
    try {
      actionLoading = true;
      const updated = await quotesApi.withdraw(quote.id);
      toast.success('Devis retiré');
      dispatch('updated', updated);
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (!confirm('Supprimer ce devis ? Cette action est irréversible.')) return;
    try {
      actionLoading = true;
      await quotesApi.delete(quote.id);
      toast.success('Devis supprimé');
      dispatch('deleted');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    } finally {
      actionLoading = false;
    }
  }
</script>

<div class="bg-gray-50 border border-gray-200 rounded-lg p-4 space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h4 class="text-sm font-semibold text-gray-900">{quote.project_title}</h4>
      <p class="text-xs text-gray-500 mt-0.5">
        {quote.contractor_name || quote.contractor_id.slice(0, 8)} - {quote.work_category}
      </p>
    </div>
    <QuoteStatusBadge status={quote.status} />
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
          <p class="text-xs text-gray-500">Montant HTVA</p>
          <p class="text-sm font-medium text-gray-900">{formatAmount(quote.amount_excl_vat_cents)}</p>
        </div>
      {/if}
      {#if quote.amount_incl_vat_cents}
        <div>
          <p class="text-xs text-gray-500">Montant TVAC ({quote.vat_rate}%)</p>
          <p class="text-sm font-medium text-gray-900">{formatAmount(quote.amount_incl_vat_cents)}</p>
        </div>
      {/if}
      {#if quote.estimated_duration_days}
        <div>
          <p class="text-xs text-gray-500">Durée estimée</p>
          <p class="text-sm font-medium text-gray-900">{quote.estimated_duration_days} jours</p>
        </div>
      {/if}
      {#if quote.warranty_years}
        <div>
          <p class="text-xs text-gray-500">Garantie</p>
          <p class="text-sm font-medium text-gray-900">{quote.warranty_years} ans</p>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Dates -->
  <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-500">
    <span>Créé le {formatDate(quote.created_at)}</span>
    {#if quote.validity_date}
      <span>Valide jusqu'au {formatDate(quote.validity_date)}</span>
    {/if}
    {#if quote.submitted_at}
      <span>Soumis le {formatDate(quote.submitted_at)}</span>
    {/if}
    {#if quote.decision_at}
      <span>Décision le {formatDate(quote.decision_at)}</span>
    {/if}
    {#if quote.contractor_rating !== undefined && quote.contractor_rating !== null}
      <span>Note entrepreneur : {quote.contractor_rating}/100</span>
    {/if}
  </div>

  {#if quote.decision_notes}
    <div class="p-2 bg-white border border-gray-100 rounded text-sm text-gray-700">
      <span class="font-medium">Notes de décision :</span> {quote.decision_notes}
    </div>
  {/if}

  <!-- Submit Form (contractor) -->
  {#if showSubmitForm}
    <div class="p-4 bg-white border border-amber-200 rounded-lg space-y-3">
      <h5 class="text-sm font-medium text-gray-900">Soumettre le devis</h5>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label for="amount" class="block text-xs text-gray-600 mb-1">Montant HTVA (EUR) *</label>
          <input id="amount" type="number" step="0.01" min="0" bind:value={amountExclVat}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div>
          <label for="vat" class="block text-xs text-gray-600 mb-1">Taux TVA (%)</label>
          <select id="vat" bind:value={vatRate}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
            <option value="6">6% (rénovation)</option>
            <option value="12">12%</option>
            <option value="21">21% (standard)</option>
          </select>
        </div>
        <div>
          <label for="validity" class="block text-xs text-gray-600 mb-1">Date de validité *</label>
          <input id="validity" type="date" bind:value={validityDate}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div>
          <label for="duration" class="block text-xs text-gray-600 mb-1">Durée estimée (jours) *</label>
          <input id="duration" type="number" min="1" bind:value={estimatedDays}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div>
          <label for="warranty" class="block text-xs text-gray-600 mb-1">Garantie (années)</label>
          <select id="warranty" bind:value={warrantyYears}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
            <option value="1">1 an</option>
            <option value="2">2 ans (défauts apparents)</option>
            <option value="5">5 ans</option>
            <option value="10">10 ans (décennale)</option>
          </select>
        </div>
      </div>
      <div class="flex gap-2">
        <button on:click={handleSubmitQuote} disabled={actionLoading}
          class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors">
          {actionLoading ? 'Envoi...' : 'Soumettre'}
        </button>
        <button on:click={() => showSubmitForm = false}
          class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
          Annuler
        </button>
      </div>
    </div>
  {/if}

  <!-- Actions -->
  <div class="flex flex-wrap gap-2 pt-2 border-t border-gray-100">
    {#if quote.status === QuoteStatus.Requested}
      <button on:click={() => showSubmitForm = !showSubmitForm} disabled={actionLoading}
        class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors">
        Soumettre le devis
      </button>
      <button on:click={handleDelete} disabled={actionLoading}
        class="px-3 py-1.5 bg-red-100 text-red-700 rounded-lg text-sm font-medium hover:bg-red-200 disabled:opacity-50 transition-colors">
        Supprimer
      </button>
    {:else if quote.status === QuoteStatus.Received && isAdmin}
      <button on:click={handleStartReview} disabled={actionLoading}
        class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors">
        Passer en revue
      </button>
      <button on:click={handleWithdraw} disabled={actionLoading}
        class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors">
        Retirer
      </button>
    {:else if quote.status === QuoteStatus.UnderReview && isAdmin}
      <button on:click={handleAccept} disabled={actionLoading}
        class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors">
        Accepter
      </button>
      <button on:click={handleReject} disabled={actionLoading}
        class="px-3 py-1.5 bg-red-600 text-white rounded-lg text-sm font-medium hover:bg-red-700 disabled:opacity-50 transition-colors">
        Refuser
      </button>
    {:else if quote.status === QuoteStatus.Received || quote.status === QuoteStatus.Requested}
      <button on:click={handleWithdraw} disabled={actionLoading}
        class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors">
        Retirer
      </button>
    {/if}
  </div>
</div>
