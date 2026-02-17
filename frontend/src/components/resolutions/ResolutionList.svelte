<script lang="ts">
  import { onMount } from 'svelte';
  import {
    resolutionsApi,
    type Resolution,
    ResolutionStatus,
  } from '../../lib/api/resolutions';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import ResolutionVotePanel from './ResolutionVotePanel.svelte';
  import ResolutionCreateForm from './ResolutionCreateForm.svelte';

  export let meetingId: string;
  export let meetingStatus: string = 'Scheduled';

  let resolutions: Resolution[] = [];
  let loading = true;
  let error = '';
  let showCreateForm = false;

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;
  $: canAddResolution = meetingStatus === 'Scheduled' && isAdmin;

  onMount(async () => {
    await loadResolutions();
  });

  async function loadResolutions() {
    try {
      loading = true;
      error = '';
      resolutions = await resolutionsApi.listByMeeting(meetingId);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des résolutions';
      console.error('Failed to load resolutions:', err);
    } finally {
      loading = false;
    }
  }

  async function handleResolutionCreated(event: CustomEvent<Resolution | null>) {
    showCreateForm = false;
    if (event.detail) {
      await loadResolutions();
    }
  }

  async function handleDeleteResolution(id: string) {
    if (!confirm('Supprimer cette résolution ?')) return;

    try {
      await resolutionsApi.delete(id);
      toast.success('Résolution supprimée');
      await loadResolutions();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    }
  }

  $: pendingCount = resolutions.filter(r => r.status === ResolutionStatus.Pending).length;
  $: adoptedCount = resolutions.filter(r => r.status === ResolutionStatus.Adopted).length;
  $: rejectedCount = resolutions.filter(r => r.status === ResolutionStatus.Rejected).length;
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden">
  <div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">Résolutions & Votes</h2>
        {#if resolutions.length > 0}
          <p class="text-indigo-200 text-sm mt-1">
            {resolutions.length} résolution{resolutions.length > 1 ? 's' : ''}
            {#if pendingCount > 0}
              &middot; {pendingCount} en attente
            {/if}
            {#if adoptedCount > 0}
              &middot; {adoptedCount} adoptée{adoptedCount > 1 ? 's' : ''}
            {/if}
            {#if rejectedCount > 0}
              &middot; {rejectedCount} rejetée{rejectedCount > 1 ? 's' : ''}
            {/if}
          </p>
        {/if}
      </div>
      {#if canAddResolution && !showCreateForm}
        <button
          on:click={() => showCreateForm = true}
          class="inline-flex items-center px-3 py-1.5 bg-white/20 hover:bg-white/30 text-white rounded-lg text-sm font-medium transition-colors"
        >
          <span class="mr-1">+</span> Ajouter
        </button>
      {/if}
    </div>
  </div>

  <div class="p-6">
    <!-- Create Form -->
    {#if showCreateForm}
      <div class="mb-4">
        <ResolutionCreateForm {meetingId} on:created={handleResolutionCreated} />
      </div>
    {/if}

    <!-- Loading -->
    {#if loading}
      <div class="py-8 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
        <p class="mt-2 text-sm text-gray-500">Chargement des résolutions...</p>
      </div>

    <!-- Error -->
    {:else if error}
      <div class="p-4 bg-red-50 border border-red-200 rounded-md">
        <p class="text-sm text-red-800">{error}</p>
        <button on:click={loadResolutions} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
          Réessayer
        </button>
      </div>

    <!-- Empty State -->
    {:else if resolutions.length === 0}
      <div class="py-8 text-center">
        <p class="text-gray-500">Aucune résolution pour cette assemblée</p>
        {#if canAddResolution}
          <p class="mt-2 text-sm text-gray-400">
            Ajoutez des résolutions à l'ordre du jour pour permettre le vote des copropriétaires.
          </p>
        {/if}
      </div>

    <!-- Resolution List -->
    {:else}
      <div class="space-y-4">
        {#each resolutions as resolution, index (resolution.id)}
          <div class="relative">
            <div class="flex items-start gap-2 mb-2">
              <span class="text-xs text-gray-400 font-mono mt-1">#{index + 1}</span>
              <div class="flex-1">
                <ResolutionVotePanel
                  {resolution}
                  {meetingStatus}
                  {isAdmin}
                />
              </div>
              {#if isAdmin && resolution.status === ResolutionStatus.Pending}
                <button
                  on:click={() => handleDeleteResolution(resolution.id)}
                  class="text-gray-400 hover:text-red-500 p-1 shrink-0"
                  title="Supprimer"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- Legal notice -->
    {#if resolutions.length > 0 || showCreateForm}
      <div class="mt-6 p-3 bg-blue-50 border-l-4 border-blue-400 rounded-md text-xs text-blue-800">
        <strong>Loi belge de la copropriété :</strong> Les votes sont comptés en millièmes (tantièmes)
        conformément à l'article 577-6 du Code Civil. Chaque lot dispose d'un pouvoir de vote proportionnel
        à sa quote-part dans les parties communes. Le vote par procuration est autorisé.
      </div>
    {/if}
  </div>
</div>
