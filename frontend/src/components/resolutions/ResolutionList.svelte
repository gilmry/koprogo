<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
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
  import { withErrorHandling, withLoadingState } from '../../lib/utils/error.utils';

  let {
    meetingId,
    meetingStatus = 'Scheduled',
  }: {
    meetingId: string;
    meetingStatus?: string;
  } = $props();

  let resolutions = $state<Resolution[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showCreateForm = $state(false);

  let isAdmin = $derived($authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN);
  let canAddResolution = $derived(meetingStatus === 'Scheduled' && isAdmin);

  $effect(() => {
    loadResolutions();
  });

  async function loadResolutions() {
    await withLoadingState({
      action: () => resolutionsApi.listByMeeting(meetingId),
      setLoading: (v: boolean) => loading = v,
      setError: (v: string) => error = v,
      onSuccess: (data: Resolution[]) => { resolutions = data; },
      errorMessage: $_("resolutions.list.loadingError"),
    });
  }

  async function handleResolutionCreated(detail: Resolution | null) {
    showCreateForm = false;
    if (detail) {
      await loadResolutions();
    }
  }

  async function handleDeleteResolution(id: string) {
    if (!confirm($_("resolutions.list.deleteConfirm"))) return;

    await withErrorHandling({
      action: () => resolutionsApi.delete(id),
      successMessage: $_("resolutions.list.deleteSuccess"),
      errorMessage: $_("resolutions.list.deleteError"),
      onSuccess: () => { loadResolutions(); },
    });
  }

  let pendingCount = $derived(resolutions.filter(r => r.status === ResolutionStatus.Pending).length);
  let adoptedCount = $derived(resolutions.filter(r => r.status === ResolutionStatus.Adopted).length);
  let rejectedCount = $derived(resolutions.filter(r => r.status === ResolutionStatus.Rejected).length);
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden" data-testid="resolution-list">
  <div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-white">{$_("resolutions.list.title")}</h2>
        {#if resolutions.length > 0}
          <p class="text-indigo-200 text-sm mt-1">
            {resolutions.length} {$_("resolutions.list.resolution", { values: { count: resolutions.length } })}
            {#if pendingCount > 0}
              &middot; <span data-testid="resolution-pending-count">{pendingCount} {$_("resolutions.list.pending")}</span>
            {/if}
            {#if adoptedCount > 0}
              &middot; <span data-testid="resolution-adopted-count">{adoptedCount} {$_("resolutions.list.adopted", { values: { count: adoptedCount } })}</span>
            {/if}
            {#if rejectedCount > 0}
              &middot; <span data-testid="resolution-rejected-count">{rejectedCount} {$_("resolutions.list.rejected", { values: { count: rejectedCount } })}</span>
            {/if}
          </p>
        {/if}
      </div>
      {#if canAddResolution && !showCreateForm}
        <button
          type="button"
          onclick={() => showCreateForm = true}
          class="inline-flex items-center px-3 py-1.5 bg-white/20 hover:bg-white/30 text-white rounded-lg text-sm font-medium transition-colors"
          data-testid="resolution-create-btn"
        >
          <span class="mr-1">+</span> {$_("common.add")}
        </button>
      {/if}
    </div>
  </div>

  <div class="p-6">
    {#if showCreateForm}
      <div class="mb-4">
        <ResolutionCreateForm {meetingId} oncreated={handleResolutionCreated} />
      </div>
    {/if}

    {#if loading}
      <div class="py-8 text-center" data-testid="resolution-list-loading">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
        <p class="mt-2 text-sm text-gray-500">{$_("resolutions.list.loading")}</p>
      </div>

    {:else if error}
      <div class="p-4 bg-red-50 border border-red-200 rounded-md">
        <p class="text-sm text-red-800">{error}</p>
        <button onclick={loadResolutions} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
          {$_("common.retry")}
        </button>
      </div>

    {:else if resolutions.length === 0}
      <div class="py-8 text-center">
        <p class="text-gray-500">{$_("resolutions.list.notFound")}</p>
        {#if canAddResolution}
          <p class="mt-2 text-sm text-gray-400">
            {$_("resolutions.list.emptyMessage")}
          </p>
        {/if}
      </div>

    {:else}
      <div class="space-y-4">
        {#each resolutions as resolution, index (resolution.id)}
          <div class="relative" data-testid="resolution-item">
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
                  onclick={() => handleDeleteResolution(resolution.id)}
                  class="text-gray-400 hover:text-red-500 p-1 shrink-0"
                  aria-label={$_("common.delete")}
                  title={$_("common.delete")}
                  data-testid="resolution-delete-btn"
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

    {#if resolutions.length > 0 || showCreateForm}
      <div class="mt-6 p-3 bg-blue-50 border-l-4 border-blue-400 rounded-md text-xs text-blue-800">
        <strong>{$_("resolutions.list.belgianLaw")}:</strong> {$_("resolutions.list.legalText")}
      </div>
    {/if}
  </div>
</div>
