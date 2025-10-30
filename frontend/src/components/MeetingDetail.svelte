<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Meeting, Building } from '../lib/types';
  import Button from './ui/Button.svelte';
  import MeetingDocuments from './MeetingDocuments.svelte';

  let meeting: Meeting | null = null;
  let building: Building | null = null;
  let loading = true;
  let error = '';
  let meetingId: string = '';

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    meetingId = urlParams.get('id') || '';

    if (!meetingId) {
      error = 'ID de l\'assemblée manquant';
      loading = false;
      return;
    }

    loadMeeting();
  });

  async function loadMeeting() {
    try {
      loading = true;
      error = '';
      meeting = await api.get<Meeting>(`/meetings/${meetingId}`);

      // Load building info
      if (meeting && meeting.building_id) {
        try {
          building = await api.get<Building>(`/buildings/${meeting.building_id}`);
        } catch (e) {
          console.error('Error loading building:', e);
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement de l\'assemblée';
      console.error('Error loading meeting:', e);
    } finally {
      loading = false;
    }
  }

  const handleGoBack = () => {
    window.history.back();
  };

  const handleComplete = async () => {
    if (!meeting) return;

    const attendees = prompt('Nombre de participants présents :');
    if (!attendees) return;

    const attendeesCount = parseInt(attendees);
    if (isNaN(attendeesCount) || attendeesCount < 0) {
      alert('Veuillez entrer un nombre valide');
      return;
    }

    try {
      await api.post(`/meetings/${meeting.id}/complete`, { attendees_count: attendeesCount });
      await loadMeeting();
      alert('Assemblée marquée comme terminée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de la mise à jour';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error completing meeting:', e);
    }
  };

  const handleCancel = async () => {
    if (!meeting) return;

    if (!confirm('Êtes-vous sûr de vouloir annuler cette assemblée ?')) {
      return;
    }

    try {
      await api.post(`/meetings/${meeting.id}/cancel`, {});
      await loadMeeting();
      alert('Assemblée annulée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de l\'annulation';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error cancelling meeting:', e);
    }
  };

  const handleReschedule = async () => {
    if (!meeting) return;

    const newDate = prompt('Nouvelle date (format: YYYY-MM-DD HH:MM):');
    if (!newDate) return;

    try {
      // Parse and format the date as ISO 8601
      const date = new Date(newDate);
      if (isNaN(date.getTime())) {
        alert('Format de date invalide. Utilisez: YYYY-MM-DD HH:MM');
        return;
      }

      await api.post(`/meetings/${meeting.id}/reschedule`, {
        scheduled_date: date.toISOString()
      });
      await loadMeeting();
      alert('Assemblée reprogrammée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de la reprogrammation';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error rescheduling meeting:', e);
    }
  };

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
      'Scheduled': { class: 'bg-blue-100 text-blue-800', label: 'Planifiée' },
      'Completed': { class: 'bg-green-100 text-green-800', label: 'Terminée' },
      'Cancelled': { class: 'bg-red-100 text-red-800', label: 'Annulée' }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function getMeetingTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'ordinary': 'Assemblée Générale Ordinaire',
      'extraordinary': 'Assemblée Générale Extraordinaire'
    };
    return labels[type] || type;
  }

  function isUpcoming(date: string): boolean {
    return new Date(date) > new Date();
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement...</p>
      </div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
      {error}
    </div>
    <div class="mt-4">
      <Button variant="outline" on:click={handleGoBack}>
        Retour
      </Button>
    </div>
  {:else if meeting}
    <!-- Header -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <button
            on:click={handleGoBack}
            class="text-gray-600 hover:text-gray-900"
          >
            Retour
          </button>
          <h1 class="text-3xl font-bold text-gray-900">{meeting.title}</h1>
        </div>
        <div class="flex gap-2">
          {#if meeting.status === 'Scheduled'}
            <Button variant="primary" on:click={handleComplete}>
              Marquer comme terminée
            </Button>
            <Button variant="outline" on:click={handleCancel}>
              Annuler
            </Button>
            <Button variant="outline" on:click={handleReschedule}>
              Reprogrammer
            </Button>
          {:else if meeting.status === 'Cancelled'}
            <Button variant="primary" on:click={handleReschedule}>
              Reprogrammer
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Main Info Card -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">Informations générales</h2>
          <span class="px-3 py-1 rounded-full text-sm font-medium {getStatusBadge(meeting.status).class}">
            {getStatusBadge(meeting.status).label}
          </span>
        </div>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Type -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Type d'assemblée</h3>
            <p class="text-lg text-gray-900">{getMeetingTypeLabel(meeting.meeting_type)}</p>
          </div>

          <!-- Date -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Date et heure</h3>
            <p class="text-lg text-gray-900">{formatDate(meeting.scheduled_date)}</p>
            {#if isUpcoming(meeting.scheduled_date) && meeting.status === 'Scheduled'}
              <span class="inline-flex items-center px-2 py-1 text-xs font-medium bg-blue-100 text-blue-800 rounded-full mt-1">
                À venir
              </span>
            {/if}
          </div>

          <!-- Location -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Lieu</h3>
            <p class="text-lg text-gray-900">{meeting.location}</p>
          </div>

          <!-- Building -->
          {#if building}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Immeuble</h3>
              <a href="/building-detail?id={building.id}" class="text-lg text-primary-600 hover:text-primary-700 hover:underline">
                {building.name}
              </a>
              <p class="text-sm text-gray-600">{building.address}</p>
            </div>
          {/if}

          {#if meeting.attendees_count !== undefined && meeting.attendees_count !== null}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Participants</h3>
              <p class="text-lg text-gray-900">{meeting.attendees_count} participant{meeting.attendees_count > 1 ? 's' : ''}</p>
            </div>
          {/if}

          <!-- Description -->
          {#if meeting.description}
            <div class="md:col-span-2">
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Description</h3>
              <p class="text-gray-900 whitespace-pre-line">{meeting.description}</p>
            </div>
          {/if}

          <!-- Agenda -->
          {#if meeting.agenda && meeting.agenda.length > 0}
            <div class="md:col-span-2">
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Ordre du jour</h3>
              <div class="bg-gray-50 rounded-lg p-4">
                <ol class="list-decimal list-inside space-y-2">
                  {#each meeting.agenda as item, index}
                    <li class="text-gray-900">
                      <span class="ml-2">{item}</span>
                    </li>
                  {/each}
                </ol>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Documents Section -->
    <div class="mb-8">
      <MeetingDocuments meetingId={meetingId} meetingStatus={meeting.status} />
    </div>
  {/if}
</div>
