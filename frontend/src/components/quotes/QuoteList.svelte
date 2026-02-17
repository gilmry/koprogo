<script lang="ts">
  import { onMount } from 'svelte';
  import {
    quotesApi,
    type Quote,
    type CreateQuoteDto,
    QuoteStatus,
  } from '../../lib/api/quotes';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import QuoteStatusBadge from './QuoteStatusBadge.svelte';
  import QuoteDetail from './QuoteDetail.svelte';

  export let buildingId: string;

  let quotes: Quote[] = [];
  let filteredQuotes: Quote[] = [];
  let loading = true;
  let error = '';
  let statusFilter: QuoteStatus | 'all' = 'all';
  let expandedId: string | null = null;
  let showCreateForm = false;
  let createLoading = false;

  // Compare mode
  let compareMode = false;
  let selectedForCompare: Set<string> = new Set();

  // Create form fields
  let newContractorId = '';
  let newProjectTitle = '';
  let newProjectDescription = '';
  let newWorkCategory = '';

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;

  onMount(async () => {
    await loadQuotes();
  });

  async function loadQuotes() {
    try {
      loading = true;
      error = '';
      quotes = await quotesApi.listByBuilding(buildingId);
      applyFilters();
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des devis';
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredQuotes = quotes.filter(q => {
      if (statusFilter === 'all') return true;
      return q.status === statusFilter;
    });
  }

  $: if (statusFilter) applyFilters();

  function formatAmount(amountCents: number | undefined): string {
    if (!amountCents) return '-';
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amountCents / 100);
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    });
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function toggleCompareSelect(id: string) {
    if (selectedForCompare.has(id)) {
      selectedForCompare.delete(id);
    } else {
      selectedForCompare.add(id);
    }
    selectedForCompare = selectedForCompare; // trigger reactivity
  }

  function goToCompare() {
    if (selectedForCompare.size < 2) {
      toast.error('Sélectionnez au moins 2 devis pour comparer');
      return;
    }
    const ids = Array.from(selectedForCompare).join(',');
    window.location.href = `/quotes/compare?ids=${ids}`;
  }

  async function handleCreate() {
    if (!newContractorId || !newProjectTitle || !newWorkCategory) {
      toast.error('Veuillez remplir tous les champs obligatoires');
      return;
    }

    try {
      createLoading = true;
      const data: CreateQuoteDto = {
        building_id: buildingId,
        contractor_id: newContractorId,
        project_title: newProjectTitle,
        project_description: newProjectDescription,
        work_category: newWorkCategory,
      };
      await quotesApi.create(data);
      toast.success('Demande de devis créée');
      showCreateForm = false;
      newContractorId = '';
      newProjectTitle = '';
      newProjectDescription = '';
      newWorkCategory = '';
      await loadQuotes();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la création');
    } finally {
      createLoading = false;
    }
  }

  function handleQuoteUpdated(event: CustomEvent<Quote>) {
    const updated = event.detail;
    quotes = quotes.map(q => q.id === updated.id ? updated : q);
    applyFilters();
  }

  function handleQuoteDeleted(quoteId: string) {
    quotes = quotes.filter(q => q.id !== quoteId);
    expandedId = null;
    applyFilters();
  }

  // Status counts
  $: statusCounts = {
    total: quotes.length,
    requested: quotes.filter(q => q.status === QuoteStatus.Requested).length,
    received: quotes.filter(q => q.status === QuoteStatus.Received).length,
    underReview: quotes.filter(q => q.status === QuoteStatus.UnderReview).length,
    accepted: quotes.filter(q => q.status === QuoteStatus.Accepted).length,
    rejected: quotes.filter(q => q.status === QuoteStatus.Rejected).length,
  };
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">
          Devis entrepreneurs
        </h3>
        <p class="mt-1 text-sm text-gray-500">
          Gestion des devis conformément à la loi belge (3 devis obligatoires pour travaux &gt;5000EUR).
        </p>
      </div>
      {#if isAdmin}
        <div class="flex gap-2">
          {#if compareMode}
            <button on:click={goToCompare}
              class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors disabled:opacity-50"
              disabled={selectedForCompare.size < 2}>
              Comparer ({selectedForCompare.size})
            </button>
            <button on:click={() => { compareMode = false; selectedForCompare = new Set(); }}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
              Annuler
            </button>
          {:else}
            <button on:click={() => compareMode = true}
              class="px-3 py-1.5 bg-blue-100 text-blue-700 rounded-lg text-sm font-medium hover:bg-blue-200 transition-colors"
              disabled={quotes.length < 2}>
              Comparer
            </button>
            <button on:click={() => showCreateForm = !showCreateForm}
              class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 transition-colors">
              + Demander un devis
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Status summary -->
  {#if quotes.length > 0}
    <div class="px-4 py-2 bg-gray-50 border-b border-gray-200 flex flex-wrap gap-3 text-xs text-gray-600">
      <span>{statusCounts.total} total</span>
      {#if statusCounts.requested > 0}<span class="text-blue-600">{statusCounts.requested} demandé{statusCounts.requested > 1 ? 's' : ''}</span>{/if}
      {#if statusCounts.received > 0}<span class="text-purple-600">{statusCounts.received} reçu{statusCounts.received > 1 ? 's' : ''}</span>{/if}
      {#if statusCounts.underReview > 0}<span class="text-yellow-600">{statusCounts.underReview} en revue</span>{/if}
      {#if statusCounts.accepted > 0}<span class="text-green-600">{statusCounts.accepted} accepté{statusCounts.accepted > 1 ? 's' : ''}</span>{/if}
      {#if statusCounts.rejected > 0}<span class="text-red-600">{statusCounts.rejected} refusé{statusCounts.rejected > 1 ? 's' : ''}</span>{/if}
    </div>
  {/if}

  <!-- Filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex items-center space-x-4">
      <label class="text-sm font-medium text-gray-700">Statut :</label>
      <select bind:value={statusFilter}
        class="text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
        <option value="all">Tous</option>
        <option value={QuoteStatus.Requested}>Demandé</option>
        <option value={QuoteStatus.Received}>Reçu</option>
        <option value={QuoteStatus.UnderReview}>En revue</option>
        <option value={QuoteStatus.Accepted}>Accepté</option>
        <option value={QuoteStatus.Rejected}>Refusé</option>
        <option value={QuoteStatus.Expired}>Expiré</option>
        <option value={QuoteStatus.Withdrawn}>Retiré</option>
      </select>
    </div>
  </div>

  <!-- Create form -->
  {#if showCreateForm}
    <div class="p-4 border-b border-gray-200 bg-amber-50">
      <h4 class="text-sm font-semibold text-gray-900 mb-3">Nouvelle demande de devis</h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label for="contractorId" class="block text-xs text-gray-600 mb-1">ID Entrepreneur *</label>
          <input id="contractorId" type="text" bind:value={newContractorId} placeholder="UUID de l'entrepreneur"
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div>
          <label for="workCategory" class="block text-xs text-gray-600 mb-1">Catégorie de travaux *</label>
          <select id="workCategory" bind:value={newWorkCategory}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
            <option value="">Sélectionner...</option>
            <option value="plumbing">Plomberie</option>
            <option value="electrical">Électricité</option>
            <option value="heating">Chauffage</option>
            <option value="painting">Peinture</option>
            <option value="roofing">Toiture</option>
            <option value="facade">Façade</option>
            <option value="elevator">Ascenseur</option>
            <option value="general">Général</option>
          </select>
        </div>
        <div class="md:col-span-2">
          <label for="projectTitle" class="block text-xs text-gray-600 mb-1">Titre du projet *</label>
          <input id="projectTitle" type="text" bind:value={newProjectTitle} placeholder="Ex: Rénovation façade arrière"
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div class="md:col-span-2">
          <label for="projectDesc" class="block text-xs text-gray-600 mb-1">Description du projet</label>
          <textarea id="projectDesc" rows="2" bind:value={newProjectDescription} placeholder="Détails des travaux..."
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"></textarea>
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button on:click={handleCreate} disabled={createLoading}
          class="px-4 py-2 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors">
          {createLoading ? 'Création...' : 'Créer la demande'}
        </button>
        <button on:click={() => showCreateForm = false}
          class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
          Annuler
        </button>
      </div>
      <p class="mt-2 text-xs text-gray-400">
        Loi belge : 3 devis minimum obligatoires pour travaux &gt;5000EUR (conformité copropriété).
      </p>
    </div>
  {/if}

  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">Chargement des devis...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadQuotes} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
        Réessayer
      </button>
    </div>
  {:else if filteredQuotes.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucun devis trouvé</p>
      {#if isAdmin}
        <p class="mt-2 text-sm text-gray-400">
          Cliquez sur "Demander un devis" pour créer une nouvelle demande.
        </p>
      {/if}
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredQuotes as quote (quote.id)}
        <li class="hover:bg-gray-50">
          <div class="px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between cursor-pointer" on:click={() => toggleExpand(quote.id)}>
              <div class="flex items-center space-x-3 flex-1 min-w-0">
                {#if compareMode}
                  <input type="checkbox" checked={selectedForCompare.has(quote.id)}
                    on:click|stopPropagation={() => toggleCompareSelect(quote.id)}
                    class="h-4 w-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                {/if}
                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-3 mb-1">
                    <h4 class="text-sm font-medium text-gray-900 truncate">{quote.project_title}</h4>
                    <QuoteStatusBadge status={quote.status} />
                  </div>
                  <div class="flex items-center text-sm text-gray-500 flex-wrap gap-x-4 gap-y-1">
                    <span>{quote.contractor_name || quote.contractor_id.slice(0, 8)}</span>
                    <span>{quote.work_category}</span>
                    {#if quote.amount_incl_vat_cents}
                      <span class="font-medium text-gray-700">{formatAmount(quote.amount_incl_vat_cents)}</span>
                    {/if}
                    <span class="text-xs text-gray-400">{formatDate(quote.created_at)}</span>
                  </div>
                </div>
              </div>
              <div class="ml-4">
                <svg class="h-5 w-5 text-gray-400 transition-transform {expandedId === quote.id ? 'rotate-90' : ''}"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </div>
            </div>

            {#if expandedId === quote.id}
              <div class="mt-3">
                <QuoteDetail {quote}
                  on:updated={handleQuoteUpdated}
                  on:deleted={() => handleQuoteDeleted(quote.id)} />
              </div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Belgian law notice -->
  {#if quotes.length > 0 && quotes.length < 3}
    <div class="px-4 py-3 bg-yellow-50 border-t border-yellow-200">
      <p class="text-xs text-yellow-800">
        <strong>Rappel légal belge :</strong> Pour des travaux &gt;5000EUR, la loi belge exige au minimum 3 devis comparables.
        Vous avez actuellement {quotes.length} devis.
      </p>
    </div>
  {/if}
</div>
