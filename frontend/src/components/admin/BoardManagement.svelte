<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../../lib/api';
  import { toast } from '../../stores/toast';
  import type { Building, Owner, BoardMemberResponse } from '../../lib/types';

  interface Meeting {
    id: string;
    title: string;
    scheduled_date: string;
    meeting_type: string;
    status: string;
  }

  let buildings: Building[] = [];
  let selectedBuildingId: string = '';
  let boardMembers: BoardMemberResponse[] = [];
  let owners: Owner[] = [];
  let meetings: Meeting[] = [];
  let loading = true;
  let loadingMembers = false;
  let showElectModal = false;

  // Election form
  let electForm = {
    owner_id: '',
    building_id: '',
    meeting_id: '',
    position: 'president' as 'president' | 'treasurer' | 'secretary',
    mandate_start: '',
    mandate_end: '',
  };

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    try {
      loading = true;
      const response = await api.get<{ data: Building[] }>('/buildings?per_page=100');
      buildings = response.data;
      if (buildings.length > 0) {
        selectedBuildingId = buildings[0].id;
        await loadBoardMembers();
      }
      loading = false;
    } catch (err) {
      console.error('Error loading buildings:', err);
      toast.error('Erreur lors du chargement des immeubles');
      loading = false;
    }
  }

  async function loadBoardMembers() {
    if (!selectedBuildingId) return;

    try {
      loadingMembers = true;
      boardMembers = await api.get<BoardMemberResponse[]>(
        `/buildings/${selectedBuildingId}/board-members/active`
      );
      loadingMembers = false;
    } catch (err) {
      console.error('Error loading board members:', err);
      toast.error('Erreur lors du chargement des membres du conseil');
      loadingMembers = false;
    }
  }

  async function loadOwners() {
    try {
      const response = await api.get<{ data: Owner[] }>('/owners?per_page=100');
      owners = response.data;
    } catch (err) {
      console.error('Error loading owners:', err);
      toast.error('Erreur lors du chargement des copropriétaires');
    }
  }

  async function loadMeetings() {
    if (!selectedBuildingId) return;

    try {
      const allMeetings = await api.get<Meeting[]>(
        `/buildings/${selectedBuildingId}/meetings?per_page=100`
      );
      // Filter to show only completed and scheduled meetings (not cancelled)
      // Elections are typically recorded in completed meetings
      // Note: API returns capitalized status values (Completed, Scheduled, Cancelled)
      meetings = (allMeetings || []).filter(m => {
        const status = m.status.toLowerCase();
        return status === 'completed' || status === 'scheduled';
      });
      // Sort by date descending (most recent first)
      meetings.sort((a, b) => new Date(b.scheduled_date).getTime() - new Date(a.scheduled_date).getTime());
    } catch (err) {
      console.error('Error loading meetings:', err);
      toast.error('Erreur lors du chargement des réunions');
      meetings = [];
    }
  }

  async function handleBuildingChange() {
    await loadBoardMembers();
  }

  function openElectModal() {
    electForm = {
      owner_id: '',
      building_id: selectedBuildingId,
      meeting_id: '',
      position: 'president',
      mandate_start: new Date().toISOString().split('T')[0],
      mandate_end: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], // +1 year (Belgian law)
    };
    loadOwners();
    loadMeetings();
    showElectModal = true;
  }

  function closeElectModal() {
    showElectModal = false;
  }

  async function handleElect() {
    try {
      // Map form fields to API expected format
      // Add time to dates (ISO 8601 format with timezone)
      const payload = {
        owner_id: electForm.owner_id,
        building_id: electForm.building_id,
        elected_by_meeting_id: electForm.meeting_id, // API expects elected_by_meeting_id
        position: electForm.position,
        mandate_start: `${electForm.mandate_start}T00:00:00Z`,
        mandate_end: `${electForm.mandate_end}T23:59:59Z`,
      };
      await api.post('/board-members', payload);
      toast.success('Membre du conseil élu avec succès');
      closeElectModal();
      await loadBoardMembers();
    } catch (err) {
      console.error('Error electing board member:', err);
      toast.error(err instanceof Error ? err.message : 'Erreur lors de l\'élection');
    }
  }

  async function handleRemove(memberId: string) {
    if (!confirm('Êtes-vous sûr de vouloir retirer ce membre du conseil ?')) return;

    try {
      await api.delete(`/board-members/${memberId}`);
      toast.success('Membre retiré avec succès');
      await loadBoardMembers();
    } catch (err) {
      console.error('Error removing board member:', err);
      toast.error('Erreur lors du retrait du membre');
    }
  }

  function getPositionLabel(position: string): string {
    const labels: Record<string, string> = {
      president: 'Président',
      treasurer: 'Trésorier',
      secretary: 'Secrétaire',
    };
    return labels[position] || position;
  }

  function getPositionIcon(position: string): string {
    const icons: Record<string, string> = {
      president: '👑',
      treasurer: '💰',
      secretary: '📝',
    };
    return icons[position] || '🎯';
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR');
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900">Gestion du Conseil d'Administration</h1>
    <p class="mt-2 text-gray-600">Gérez les membres du conseil de copropriété</p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement...</p>
      </div>
    </div>
  {:else}
    <!-- Building Selector -->
    <div class="bg-white shadow rounded-lg p-6 mb-6">
      <div class="flex items-center justify-between">
        <div class="flex-1 max-w-md">
          <label for="building-select" class="block text-sm font-medium text-gray-700 mb-2">
            Sélectionner un immeuble
          </label>
          <select
            id="building-select"
            bind:value={selectedBuildingId}
            on:change={handleBuildingChange}
            class="block w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
          >
            {#each buildings as building}
              <option value={building.id}>{building.name} - {building.address}</option>
            {/each}
          </select>
        </div>
        <button
          on:click={openElectModal}
          class="ml-4 px-4 py-2 bg-primary-600 text-white rounded-md hover:bg-primary-700 transition"
        >
          ➕ Élire un membre
        </button>
      </div>
    </div>

    <!-- Board Members List -->
    {#if loadingMembers}
      <div class="flex justify-center py-12">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    {:else if boardMembers.length === 0}
      <div class="bg-gray-50 border border-gray-200 rounded-lg p-8 text-center">
        <p class="text-gray-500 text-lg">Aucun membre du conseil pour cet immeuble</p>
        <button
          on:click={openElectModal}
          class="mt-4 px-4 py-2 bg-primary-600 text-white rounded-md hover:bg-primary-700 transition"
        >
          Élire le premier membre
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {#each boardMembers as member}
          <div class="bg-white border border-gray-200 rounded-lg shadow p-6 hover:shadow-lg transition">
            <div class="flex items-start justify-between mb-4">
              <div class="flex items-center gap-3">
                <span class="text-4xl">{getPositionIcon(member.position)}</span>
                <div>
                  <h3 class="text-lg font-bold text-gray-900">{getPositionLabel(member.position)}</h3>
                  {#if member.expires_soon}
                    <span class="inline-block mt-1 px-2 py-0.5 bg-orange-100 text-orange-800 text-xs font-medium rounded">
                      ⚠️ Expire dans {member.days_remaining} jours
                    </span>
                  {/if}
                </div>
              </div>
            </div>

            <div class="space-y-2 text-sm">
              <div class="flex justify-between">
                <span class="text-gray-600">Owner ID:</span>
                <span class="font-medium text-gray-900 text-xs">{member.owner_id}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Début:</span>
                <span class="font-medium text-gray-900">{formatDate(member.mandate_start)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Fin:</span>
                <span class="font-medium text-gray-900">{formatDate(member.mandate_end)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Jours restants:</span>
                <span class="font-medium {member.expires_soon ? 'text-orange-600' : 'text-green-600'}">
                  {member.days_remaining}
                </span>
              </div>
            </div>

            <div class="mt-4 pt-4 border-t border-gray-200 flex gap-2">
              <button
                on:click={() => handleRemove(member.id)}
                class="flex-1 px-3 py-2 bg-red-50 text-red-700 rounded hover:bg-red-100 transition text-sm font-medium"
              >
                🗑️ Retirer
              </button>
              <a
                href="/board-dashboard?building_id={member.building_id}"
                class="flex-1 px-3 py-2 bg-primary-50 text-primary-700 rounded hover:bg-primary-100 transition text-sm font-medium text-center"
              >
                📊 Dashboard
              </a>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- Election Modal -->
{#if showElectModal}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
    <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
      <h2 class="text-2xl font-bold text-gray-900 mb-4">Élire un membre du conseil</h2>

      <form on:submit|preventDefault={handleElect} class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Copropriétaire</label>
          <select
            bind:value={electForm.owner_id}
            required
            class="w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
          >
            <option value="">-- Sélectionner un copropriétaire --</option>
            {#each owners as owner}
              <option value={owner.id}>{owner.first_name} {owner.last_name} ({owner.email})</option>
            {/each}
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Position</label>
          <select
            bind:value={electForm.position}
            class="w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
          >
            <option value="president">👑 Président</option>
            <option value="treasurer">💰 Trésorier</option>
            <option value="secretary">📝 Secrétaire</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Assemblée générale (PV)</label>
          <select
            bind:value={electForm.meeting_id}
            required
            class="w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
          >
            <option value="">-- Sélectionner une assemblée générale --</option>
            {#each meetings as meeting}
              <option value={meeting.id}>
                {#if meeting.status.toLowerCase() === 'completed'}✓{:else}📅{/if}
                {meeting.title} - {new Date(meeting.scheduled_date).toLocaleDateString('fr-BE')}
                ({meeting.meeting_type.toLowerCase() === 'ordinary' ? 'AGO' : 'AGE'})
                {#if meeting.status.toLowerCase() === 'completed'}- Terminée{/if}
              </option>
            {/each}
          </select>
          <p class="mt-1 text-xs text-gray-500">
            {#if meetings && meetings.length === 0}
              ⚠️ Aucune assemblée trouvée pour cet immeuble. Créez d'abord une AG.
            {:else}
              L'élection doit être actée dans un PV d'assemblée générale
            {/if}
          </p>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Début du mandat</label>
            <input
              type="date"
              bind:value={electForm.mandate_start}
              required
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Fin du mandat</label>
            <input
              type="date"
              bind:value={electForm.mandate_end}
              required
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
            />
          </div>
        </div>

        <div class="flex gap-3 pt-4">
          <button
            type="button"
            on:click={closeElectModal}
            class="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition"
          >
            Annuler
          </button>
          <button
            type="submit"
            class="flex-1 px-4 py-2 bg-primary-600 text-white rounded-md hover:bg-primary-700 transition"
          >
            Élire
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
