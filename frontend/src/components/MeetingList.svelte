<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  interface Meeting {
    id: string;
    title: string;
    meeting_type: string;
    scheduled_date: string;
    status: string;
    attendees_count?: number;
  }

  let meetings: Meeting[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadMeetings();
  });

  async function loadMeetings() {
    try {
      loading = true;
      const response = await api.get<PageResponse<Meeting>>(
        `/meetings?page=${currentPage}&per_page=${perPage}`
      );

      meetings = response.data;
      totalItems = response.total;
      totalPages = response.total_pages;
      currentPage = response.page;
      perPage = response.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des assemblées';
      console.error('Error loading meetings:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadMeetings();
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function getStatusBadge(status: string): string {
    const badges: Record<string, string> = {
      'Scheduled': 'bg-blue-100 text-blue-800',
      'Completed': 'bg-green-100 text-green-800',
      'Cancelled': 'bg-red-100 text-red-800'
    };
    return badges[status] || 'bg-gray-100 text-gray-800';
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} assemblée{totalItems !== 1 ? 's' : ''}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if meetings.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucune assemblée enregistrée.
    </p>
  {:else}
    <div class="grid gap-4">
      {#each meetings as meeting}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div>
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">
                  {meeting.title}
                </h3>
                <span class="text-xs px-2 py-1 rounded-full {getStatusBadge(meeting.status)}">
                  {meeting.status}
                </span>
              </div>
              <p class="text-gray-600 text-sm">
                📋 {meeting.meeting_type}
              </p>
              <p class="text-gray-500 text-sm">
                📅 {formatDate(meeting.scheduled_date)}
              </p>
              {#if meeting.attendees_count}
                <p class="text-gray-500 text-sm">
                  👥 {meeting.attendees_count} participant{meeting.attendees_count > 1 ? 's' : ''}
                </p>
              {/if}
            </div>
            <button class="text-primary-600 hover:text-primary-700 text-sm font-medium">
              Détails →
            </button>
          </div>
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <Pagination
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        perPage={perPage}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>
