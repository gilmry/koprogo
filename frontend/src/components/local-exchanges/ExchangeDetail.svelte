<script lang="ts">
  import {
    localExchangesApi,
    type LocalExchange,
    ExchangeStatus,
    exchangeTypeLabels,
    exchangeTypeIcons,
    exchangeStatusLabels,
    exchangeStatusColors,
    formatCredits,
    formatRating,
  } from '../../lib/api/local-exchanges';
  import { toast } from '../../stores/toast';

  export let exchange: LocalExchange;
  export let currentUserId: string = '';

  let actionLoading = false;
  let ratingValue = 0;
  let showRatingForm = false;
  let cancelReason = '';
  let showCancelForm = false;

  $: isProvider = exchange.provider_id === currentUserId;
  $: isRequester = exchange.requester_id === currentUserId;
  $: statusColors = exchangeStatusColors[exchange.status];

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  async function handleRequest() {
    if (!confirm('Voulez-vous demander cet échange ?')) return;
    actionLoading = true;
    try {
      exchange = await localExchangesApi.request(exchange.id);
      toast.success('Échange demandé avec succès !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la demande');
    } finally {
      actionLoading = false;
    }
  }

  async function handleStart() {
    if (!confirm('Démarrer cet échange ?')) return;
    actionLoading = true;
    try {
      exchange = await localExchangesApi.start(exchange.id);
      toast.success('Échange démarré !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors du démarrage');
    } finally {
      actionLoading = false;
    }
  }

  async function handleComplete() {
    if (!confirm('Confirmer la fin de cet échange ? Les crédits seront transférés automatiquement.')) return;
    actionLoading = true;
    try {
      exchange = await localExchangesApi.complete(exchange.id);
      toast.success('Échange terminé ! Crédits transférés.');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la complétion');
    } finally {
      actionLoading = false;
    }
  }

  async function handleCancel() {
    if (!cancelReason.trim()) {
      toast.error('Veuillez indiquer une raison d\'annulation');
      return;
    }
    actionLoading = true;
    try {
      exchange = await localExchangesApi.cancel(exchange.id, { reason: cancelReason });
      showCancelForm = false;
      toast.success('Échange annulé');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'annulation');
    } finally {
      actionLoading = false;
    }
  }

  async function handleRate(asProvider: boolean) {
    if (ratingValue < 1 || ratingValue > 5) {
      toast.error('Veuillez donner une note entre 1 et 5');
      return;
    }
    actionLoading = true;
    try {
      if (asProvider) {
        await localExchangesApi.rateRequester(exchange.id, { rating: ratingValue });
        exchange.requester_rating = ratingValue;
      } else {
        await localExchangesApi.rateProvider(exchange.id, { rating: ratingValue });
        exchange.provider_rating = ratingValue;
      }
      showRatingForm = false;
      ratingValue = 0;
      toast.success('Note enregistrée !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la notation');
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (!confirm('Supprimer cette offre d\'échange ? Cette action est irréversible.')) return;
    actionLoading = true;
    try {
      await localExchangesApi.delete(exchange.id);
      toast.success('Échange supprimé');
      window.location.href = '/exchanges';
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    } finally {
      actionLoading = false;
    }
  }

  function canRate(): { canRateProvider: boolean; canRateRequester: boolean } {
    if (exchange.status !== ExchangeStatus.Completed) return { canRateProvider: false, canRateRequester: false };
    return {
      canRateProvider: isRequester && !exchange.provider_rating,
      canRateRequester: isProvider && !exchange.requester_rating,
    };
  }
</script>

<div class="space-y-6">
  <!-- Header Card -->
  <div class="bg-white shadow-md rounded-lg p-6">
    <div class="flex items-start justify-between">
      <div class="flex items-start gap-4">
        <span class="text-4xl">{exchangeTypeIcons[exchange.exchange_type]}</span>
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h2 class="text-2xl font-bold text-gray-900">{exchange.title}</h2>
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusColors.bg} {statusColors.text}">
              {exchangeStatusLabels[exchange.status]}
            </span>
          </div>
          <p class="text-sm text-gray-500">
            {exchangeTypeLabels[exchange.exchange_type]} - {formatCredits(exchange.credits)}
          </p>
        </div>
      </div>
    </div>

    <p class="mt-4 text-gray-700">{exchange.description}</p>

    <!-- Metadata Grid -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
      <div class="p-3 bg-blue-50 rounded-lg">
        <div class="text-xs text-blue-600 font-medium">Fournisseur</div>
        <div class="text-sm text-blue-900 font-medium">{exchange.provider_name}</div>
        {#if isProvider}
          <span class="text-xs text-blue-600">(vous)</span>
        {/if}
      </div>
      <div class="p-3 bg-green-50 rounded-lg">
        <div class="text-xs text-green-600 font-medium">Demandeur</div>
        <div class="text-sm text-green-900 font-medium">
          {exchange.requester_name || 'En attente'}
        </div>
        {#if isRequester}
          <span class="text-xs text-green-600">(vous)</span>
        {/if}
      </div>
      <div class="p-3 bg-amber-50 rounded-lg">
        <div class="text-xs text-amber-600 font-medium">Crédits</div>
        <div class="text-sm text-amber-900 font-bold">{formatCredits(exchange.credits)}</div>
      </div>
      <div class="p-3 bg-purple-50 rounded-lg">
        <div class="text-xs text-purple-600 font-medium">Créé le</div>
        <div class="text-sm text-purple-900">{formatDate(exchange.created_at)}</div>
      </div>
    </div>

    <!-- Timeline -->
    <div class="mt-6 border-t border-gray-200 pt-4">
      <h4 class="text-sm font-medium text-gray-700 mb-3">Historique</h4>
      <div class="space-y-2 text-sm">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-green-500"></span>
          <span class="text-gray-600">Offert le {formatDate(exchange.offered_at)}</span>
        </div>
        {#if exchange.requested_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-blue-500"></span>
            <span class="text-gray-600">Demandé le {formatDate(exchange.requested_at)}</span>
          </div>
        {/if}
        {#if exchange.started_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-yellow-500"></span>
            <span class="text-gray-600">Démarré le {formatDate(exchange.started_at)}</span>
          </div>
        {/if}
        {#if exchange.completed_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-green-600"></span>
            <span class="text-gray-600">Terminé le {formatDate(exchange.completed_at)}</span>
          </div>
        {/if}
        {#if exchange.cancelled_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-red-500"></span>
            <span class="text-gray-600">Annulé le {formatDate(exchange.cancelled_at)}</span>
            {#if exchange.cancellation_reason}
              <span class="text-gray-500">- {exchange.cancellation_reason}</span>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Ratings Card (if completed) -->
  {#if exchange.status === ExchangeStatus.Completed}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">Évaluations</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="p-4 bg-gray-50 rounded-lg">
          <div class="text-sm font-medium text-gray-700 mb-1">Note du fournisseur</div>
          <div class="text-lg">{formatRating(exchange.provider_rating)}</div>
        </div>
        <div class="p-4 bg-gray-50 rounded-lg">
          <div class="text-sm font-medium text-gray-700 mb-1">Note du demandeur</div>
          <div class="text-lg">{formatRating(exchange.requester_rating)}</div>
        </div>
      </div>

      {#if canRate().canRateProvider || canRate().canRateRequester}
        {#if !showRatingForm}
          <button
            on:click={() => { showRatingForm = true; }}
            class="mt-4 px-4 py-2 bg-amber-600 text-white text-sm rounded-md hover:bg-amber-700"
          >
            Noter {canRate().canRateProvider ? 'le fournisseur' : 'le demandeur'}
          </button>
        {:else}
          <div class="mt-4 p-4 border border-amber-200 rounded-lg bg-amber-50">
            <div class="text-sm font-medium text-gray-700 mb-2">
              Votre note ({canRate().canRateProvider ? 'fournisseur' : 'demandeur'})
            </div>
            <div class="flex items-center gap-2 mb-3">
              {#each [1, 2, 3, 4, 5] as star}
                <button
                  type="button"
                  on:click={() => ratingValue = star}
                  class="text-3xl transition-colors {ratingValue >= star ? 'text-yellow-400' : 'text-gray-300'} hover:text-yellow-300"
                >
                  &#9733;
                </button>
              {/each}
            </div>
            <div class="flex gap-2">
              <button
                on:click={() => handleRate(canRate().canRateRequester)}
                disabled={actionLoading || ratingValue === 0}
                class="px-4 py-2 bg-amber-600 text-white text-sm rounded-md hover:bg-amber-700 disabled:opacity-50"
              >
                Confirmer
              </button>
              <button
                on:click={() => { showRatingForm = false; ratingValue = 0; }}
                class="px-4 py-2 bg-gray-200 text-gray-700 text-sm rounded-md hover:bg-gray-300"
              >
                Annuler
              </button>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Actions Card -->
  <div class="bg-white shadow-md rounded-lg p-6">
    <h3 class="text-lg font-medium text-gray-900 mb-4">Actions</h3>
    <div class="flex flex-wrap gap-3">
      {#if exchange.status === ExchangeStatus.Offered && !isProvider}
        <button
          on:click={handleRequest}
          disabled={actionLoading}
          class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
        >
          Demander cet échange
        </button>
      {/if}

      {#if exchange.status === ExchangeStatus.Requested && isProvider}
        <button
          on:click={handleStart}
          disabled={actionLoading}
          class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
        >
          Accepter et démarrer
        </button>
      {/if}

      {#if exchange.status === ExchangeStatus.InProgress && isProvider}
        <button
          on:click={handleComplete}
          disabled={actionLoading}
          class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
        >
          Marquer comme terminé
        </button>
      {/if}

      {#if exchange.status !== ExchangeStatus.Completed && exchange.status !== ExchangeStatus.Cancelled}
        {#if !showCancelForm}
          <button
            on:click={() => showCancelForm = true}
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200"
          >
            Annuler
          </button>
        {:else}
          <div class="w-full p-4 border border-red-200 rounded-lg bg-red-50">
            <label class="block text-sm font-medium text-red-800 mb-1">Raison de l'annulation</label>
            <textarea
              bind:value={cancelReason}
              rows="2"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-red-500 focus:ring-red-500 text-sm"
              placeholder="Indiquez la raison..."
            ></textarea>
            <div class="flex gap-2 mt-2">
              <button
                on:click={handleCancel}
                disabled={actionLoading}
                class="px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 disabled:opacity-50"
              >
                Confirmer l'annulation
              </button>
              <button
                on:click={() => { showCancelForm = false; cancelReason = ''; }}
                class="px-4 py-2 bg-gray-200 text-gray-700 text-sm rounded-md hover:bg-gray-300"
              >
                Retour
              </button>
            </div>
          </div>
        {/if}
      {/if}

      {#if exchange.status === ExchangeStatus.Offered && isProvider}
        <button
          on:click={handleDelete}
          disabled={actionLoading}
          class="px-4 py-2 bg-gray-100 text-gray-700 text-sm font-medium rounded-md hover:bg-gray-200"
        >
          Supprimer l'offre
        </button>
      {/if}
    </div>
  </div>
</div>
