<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';
  import type { Owner } from '../../lib/types';
  import OwnerEditModal from '../OwnerEditModal.svelte';

  $: user = $authStore.user;

  interface SyndicStats {
    total_buildings: number;
    total_units: number;
    total_owners: number;
    pending_expenses_count: number;
    pending_expenses_amount: number;
    next_meeting: {
      id: string;
      date: string;
      building_name: string;
    } | null;
  }

  interface UrgentTask {
    task_type: string;
    title: string;
    description: string;
    priority: string;
    building_name: string | null;
    entity_id: string | null;
    due_date: string | null;
  }

  let stats: SyndicStats | null = null;
  let urgentTasks: UrgentTask[] = [];
  let recentOwners: Owner[] = [];
  let loading = true;
  let error: string | null = null;

  // Modal state
  let isModalOpen = false;
  let selectedOwner: Owner | null = null;

  onMount(async () => {
    await loadDashboardData();
  });

  async function loadDashboardData() {
    try {
      loading = true;
      const [statsData, tasksData, ownersData] = await Promise.all([
        api.get<SyndicStats>('/stats/syndic'),
        api.get<UrgentTask[]>('/stats/syndic/urgent-tasks'),
        api.get<{ data: Owner[] }>('/owners?page=1&per_page=5'),
      ]);
      stats = statsData;
      urgentTasks = tasksData;
      recentOwners = ownersData.data;
      loading = false;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur lors du chargement des statistiques';
      loading = false;
      console.error('Error fetching stats:', err);
    }
  }

  function openEditModal(owner: Owner) {
    selectedOwner = owner;
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
    selectedOwner = null;
  }

  async function handleOwnerSaved() {
    await loadDashboardData();
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', { day: 'numeric', month: 'short' });
  }

  function formatAmount(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function getTaskIcon(taskType: string): string {
    switch (taskType) {
      case 'expense': return '💰';
      case 'meeting': return '📄';
      default: return '📋';
    }
  }

  function getTaskStyles(priority: string): { bg: string; border: string; text: string } {
    switch (priority) {
      case 'urgent':
        return { bg: 'bg-red-50', border: 'border-red-200', text: 'text-red-600' };
      case 'high':
        return { bg: 'bg-orange-50', border: 'border-orange-200', text: 'text-orange-600' };
      default:
        return { bg: 'bg-yellow-50', border: 'border-yellow-200', text: 'text-yellow-600' };
    }
  }

  function getPriorityLabel(priority: string): string {
    switch (priority) {
      case 'urgent': return 'Urgent';
      case 'high': return 'Important';
      default: return 'À traiter';
    }
  }
</script>

<div>
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      Bienvenue, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      Dashboard Syndic - Gestion de vos copropriétés
    </p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
      <p class="text-red-800 font-medium">Erreur</p>
      <p class="text-red-600 text-sm">{error}</p>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Immeubles gérés</span>
          <span class="text-2xl">🏢</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_buildings}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.total_units} lots au total</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Copropriétaires</span>
          <span class="text-2xl">👥</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_owners}</p>
        <p class="text-sm text-gray-500 mt-1">actifs</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Charges en attente</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.pending_expenses_count}</p>
        <p class="text-sm text-orange-600 mt-1">{formatAmount(stats.pending_expenses_amount)}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Prochaine AG</span>
          <span class="text-2xl">📅</span>
        </div>
        {#if stats.next_meeting}
          <p class="text-xl font-bold text-gray-900">{formatDate(stats.next_meeting.date)}</p>
          <p class="text-sm text-gray-500 mt-1">{stats.next_meeting.building_name}</p>
        {:else}
          <p class="text-lg font-medium text-gray-500">Aucune AG prévue</p>
        {/if}
      </div>
    </div>

    <!-- Main Content -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Urgent Tasks -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">Tâches urgentes</h2>
        </div>
        <div class="p-6">
          {#if urgentTasks.length > 0}
            <div class="space-y-4">
              {#each urgentTasks as task}
                {@const styles = getTaskStyles(task.priority)}
                <div class="flex items-start space-x-3 p-4 {styles.bg} border {styles.border} rounded-lg">
                  <span class="text-2xl">{getTaskIcon(task.task_type)}</span>
                  <div class="flex-1">
                    <p class="text-sm font-medium text-gray-900">{task.title}</p>
                    <p class="text-sm text-gray-600">{task.description}</p>
                    {#if task.building_name}
                      <p class="text-xs text-gray-500 mt-1">{task.building_name}</p>
                    {/if}
                    <p class="text-xs {styles.text} mt-1">{getPriorityLabel(task.priority)}</p>
                  </div>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">Aucune tâche urgente pour le moment</p>
              <p class="text-sm text-gray-400 mt-2">Tout est sous contrôle ! 🎉</p>
            </div>
          {/if}
        </div>
      </div>

    <!-- Quick Actions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Actions rapides</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">Immeubles</span>
          </a>
          <a href="/owners" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">👥</span>
            <span class="text-sm font-medium text-gray-700">Copropriétaires</span>
          </a>
          <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
            <span class="text-sm font-medium text-gray-700">Charges</span>
          </a>
          <a href="/meetings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📅</span>
            <span class="text-sm font-medium text-gray-700">Assemblées</span>
          </a>
        </div>
      </div>
    </div>
    </div>

    <!-- Recent Owners Section -->
    <div class="mt-8">
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-lg font-semibold text-gray-900">Copropriétaires récents</h2>
          <a href="/owners" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            Voir tout →
          </a>
        </div>
        <div class="p-6">
          {#if recentOwners.length > 0}
            <div class="space-y-3">
              {#each recentOwners as owner}
                <div class="flex items-center justify-between p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition">
                  <div class="flex-1">
                    <h3 class="font-medium text-gray-900">
                      {owner.first_name} {owner.last_name}
                    </h3>
                    <p class="text-sm text-gray-600">
                      📧 {owner.email}
                    </p>
                    {#if owner.phone}
                      <p class="text-sm text-gray-500">
                        📞 {owner.phone}
                      </p>
                    {/if}
                  </div>
                  <button
                    on:click={() => openEditModal(owner)}
                    class="ml-4 px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
                  >
                    Modifier
                  </button>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">Aucun copropriétaire enregistré</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}

  <!-- Owner Edit Modal -->
  <OwnerEditModal
    owner={selectedOwner}
    isOpen={isModalOpen}
    on:close={closeModal}
    on:save={handleOwnerSaved}
  />
</div>
