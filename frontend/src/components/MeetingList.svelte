<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { api } from '../lib/api';
  import type { PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  export let buildingId: string | null = null;

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

    // Listen for page show events to reload data when navigating back (client-side only)
    if (typeof window !== 'undefined') {
      window.addEventListener('pageshow', handlePageShow);
      window.addEventListener('focus', handleWindowFocus);
    }
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('pageshow', handlePageShow);
      window.removeEventListener('focus', handleWindowFocus);
    }
  });

  function handlePageShow(event: PageTransitionEvent) {
    // Reload data when navigating back to this page
    if (event.persisted) {
      loadMeetings();
    }
  }

  function handleWindowFocus() {
    // Reload data when window regains focus
    loadMeetings();
  }

  async function loadMeetings() {
    try {
      loading = true;

      if (buildingId) {
        // Endpoint without pagination for building-specific meetings
        const response = await api.get<Meeting[]>(`/buildings/${buildingId}/meetings`);
        meetings = response;
        totalItems = response.length;
        totalPages = 1;
        currentPage = 1;
      } else {
        // Paginated endpoint for all meetings
        const endpoint = `/meetings?page=${currentPage}&per_page=${perPage}`;
        const response = await api.get<PageResponse<Meeting>>(endpoint);
        meetings = response.data;
        totalItems = response.pagination.total_items;
        totalPages = response.pagination.total_pages;
        currentPage = response.pagination.current_page;
        perPage = response.pagination.per_page;
      }

      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : $_('meetings.error_loading_meetings');
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

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'Scheduled': { class: 'bg-blue-100 text-blue-800', label: $_('meetings.status_scheduled') },
      'Completed': { class: 'bg-green-100 text-green-800', label: $_('meetings.status_completed') },
      'Cancelled': { class: 'bg-red-100 text-red-800', label: $_('meetings.status_cancelled') }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function getMeetingTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'Ordinary': $_('meetings.type_ordinary'),
      'Extraordinary': $_('meetings.type_extraordinary'),
      'ordinary': $_('meetings.type_ordinary'),
      'extraordinary': $_('meetings.type_extraordinary')
    };
    return labels[type] || type;
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {$_('meetings.count', { values: { count: totalItems } })}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">{$_('common.loading')}</p>
  {:else if meetings.length === 0}
    <p class="text-center text-gray-600 py-8">
      {$_('meetings.no_meetings')}
    </p>
  {:else}
    <div class="grid gap-4">
      {#each meetings as meeting (meeting.id)}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div>
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">
                  {meeting.title}
                </h3>
                <span class="text-xs px-2 py-1 rounded-full {getStatusBadge(meeting.status).class}">
                  {getStatusBadge(meeting.status).label}
                </span>
              </div>
              <p class="text-gray-600 text-sm">
                📋 {getMeetingTypeLabel(meeting.meeting_type)}
              </p>
              <p class="text-gray-500 text-sm">
                📅 {formatDate(meeting.scheduled_date)}
              </p>
              {#if meeting.attendees_count}
                <p class="text-gray-500 text-sm">
                  👥 {$_('meetings.participants_count', { values: { count: meeting.attendees_count } })}
                </p>
              {/if}
            </div>
            <a href="/meeting-detail?id={meeting.id}" class="text-primary-600 hover:text-primary-700 text-sm font-medium">
              {$_('common.details')} →
            </a>
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
