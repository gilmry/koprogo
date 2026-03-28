<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { authStore } from '../../stores/auth';
  import { apiEndpoint } from '../../lib/config';
  import { api } from '../../lib/api';
  import { withErrorHandling } from "../../lib/utils/error.utils";

  interface Stats {
    totalOrganizations: number;
    totalUsers: number;
    totalBuildings: number;
    activeSubscriptions: number;
    totalOwners: number;
    totalUnits: number;
    totalExpenses: number;
    totalMeetings: number;
  }

  let stats: Stats = {
    totalOrganizations: 0,
    totalUsers: 0,
    totalBuildings: 0,
    activeSubscriptions: 0,
    totalOwners: 0,
    totalUnits: 0,
    totalExpenses: 0,
    totalMeetings: 0,
  };
  let loading = true;
  let statsError = '';
  let seedLoading = false;
  let clearLoading = false;
  let seedMessage = '';
  let seedError = '';

  $: user = $authStore.user;

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    loading = true;
    statsError = '';
    const data = await withErrorHandling({
      action: () => api.get<{
        total_organizations: number;
        total_users: number;
        total_buildings: number;
        active_subscriptions: number;
        total_owners: number;
        total_units: number;
        total_expenses: number;
        total_meetings: number;
      }>('/stats/dashboard'),
    });
    if (data) {
      stats = {
        totalOrganizations: data.total_organizations,
        totalUsers: data.total_users,
        totalBuildings: data.total_buildings,
        activeSubscriptions: data.active_subscriptions,
        totalOwners: data.total_owners,
        totalUnits: data.total_units,
        totalExpenses: data.total_expenses,
        totalMeetings: data.total_meetings,
      };
    } else {
      statsError = $_('common.error.loadStats');
    }
    loading = false;
  }

  const handleSeedDemoData = async () => {
    seedLoading = true;
    seedMessage = '';
    seedError = '';

    // DEBUG: Log token state
    console.log('=== DEBUG: Seed Demo Data ===');
    console.log('Auth Store State:', $authStore);
    console.log('Token:', $authStore.token);
    console.log('Is Authenticated:', $authStore.isAuthenticated);
    if (typeof window !== 'undefined') {
      console.log('LocalStorage Token:', localStorage.getItem('koprogo_token'));
      console.log('LocalStorage User:', localStorage.getItem('koprogo_user'));
    }
    console.log('API Endpoint:', apiEndpoint('/seed/demo'));
    console.log('============================');

    try {
      const response = await fetch(apiEndpoint('/seed/demo'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$authStore.token}`,
        },
      });

      const data = await response.json();

      if (response.ok) {
        seedMessage = data.message || $_('dashboards.admin.seed.successMessage');
        // Reload stats after seeding
        await loadStats();
        setTimeout(() => seedMessage = '', 5000);
      } else {
        seedError = data.error || $_('dashboards.admin.seed.errorMessage');
        setTimeout(() => seedError = '', 5000);
      }
    } catch (error) {
      console.error('Seed error:', error);
      seedError = $_('common.error.serverConnection');
      setTimeout(() => seedError = '', 5000);
    } finally {
      seedLoading = false;
    }
  };

  const handleClearDemoData = async () => {
    if (!confirm($_('dashboards.admin.seed.confirmDelete'))) {
      return;
    }

    clearLoading = true;
    seedMessage = '';
    seedError = '';

    try {
      const response = await fetch(apiEndpoint('/seed/clear'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$authStore.token}`,
        },
      });

      const data = await response.json();

      if (response.ok) {
        seedMessage = data.message || $_('dashboards.admin.seed.deleteSuccessMessage');
        // Reload stats after clearing
        await loadStats();
        setTimeout(() => seedMessage = '', 5000);
      } else {
        seedError = data.error || $_('dashboards.admin.seed.deleteErrorMessage');
        setTimeout(() => seedError = '', 5000);
      }
    } catch (error) {
      console.error('Clear error:', error);
      seedError = $_('common.error.serverConnection');
      setTimeout(() => seedError = '', 5000);
    } finally {
      clearLoading = false;
    }
  };
</script>

<div data-testid="admin-dashboard">
  <!-- Header -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      {$_('common.welcome')}, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      {$_('dashboards.admin.title')} - {$_('dashboards.admin.subtitle')}
    </p>
  </div>

  <!-- Stats Cards -->
  {#if statsError}
    <div class="mb-6 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
      ⚠️ {statsError}
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.organizations')}</span>
        <span class="text-2xl">🏛️</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalOrganizations}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.activeSubscriptions} {$_('dashboards.admin.stats.active')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.users')}</span>
        <span class="text-2xl">👥</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalUsers}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.allOrganizations')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.buildings')}</span>
        <span class="text-2xl">🏢</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalBuildings}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.totalUnits} {$_('dashboards.admin.stats.units')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.owners')}</span>
        <span class="text-2xl">👨‍👩‍👧</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalOwners}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.database')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.lots')}</span>
        <span class="text-2xl">🏠</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalUnits}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.allBuildings')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.expenses')}</span>
        <span class="text-2xl">💶</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalExpenses}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.totalRecorded')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.meetings')}</span>
        <span class="text-2xl">📅</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalMeetings}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.plannedMeetings')}</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">{$_('dashboards.admin.stats.subscriptions')}</span>
        <span class="text-2xl">✅</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.activeSubscriptions}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.admin.stats.outOf')} {stats.totalOrganizations} {$_('dashboards.admin.stats.orgs')}</p>
      {/if}
    </div>
  </div>

  <!-- Database Management -->
  <div class="bg-white rounded-lg shadow mb-8">
    <div class="p-6 border-b border-gray-200">
      <div class="flex justify-between items-start">
        <div>
          <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.admin.seed.title')}</h2>
          <p class="text-sm text-gray-600 mt-1">{$_('dashboards.admin.seed.subtitle')}</p>
        </div>
        <a
          href="/admin/seed"
          class="inline-flex items-center gap-2 px-4 py-2 bg-blue-50 text-blue-700 rounded-lg hover:bg-blue-100 transition text-sm font-medium"
        >
          <span>⚙️</span>
          {$_('dashboards.admin.seed.advancedManagement')}
        </a>
      </div>
    </div>
    <div class="p-6">
      {#if seedMessage}
        <div class="mb-4 p-4 bg-green-50 border border-green-200 text-green-700 rounded-lg">
          ✓ {seedMessage}
        </div>
      {/if}
      {#if seedError}
        <div class="mb-4 p-4 bg-red-50 border border-red-200 text-red-700 rounded-lg">
          ✗ {seedError}
        </div>
      {/if}

      <!-- Info Banner -->
      <div class="mb-6 p-4 bg-blue-50 border-l-4 border-blue-500 rounded">
        <p class="text-sm text-blue-900">
          <strong>ℹ️ {$_('dashboards.admin.seed.infoBanner')}</strong> {$_('dashboards.admin.seed.infoDetails')}
          <code class="bg-blue-100 px-1 rounded font-mono text-xs">is_seed_data=true</code> {$_('dashboards.admin.seed.infoProtection')}
        </p>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <!-- Generate Seed -->
        <div class="border-2 border-green-200 rounded-lg p-6 bg-green-50">
          <div class="flex items-center gap-3 mb-4">
            <span class="text-4xl">🌱</span>
            <div>
              <h3 class="font-semibold text-lg text-green-900">{$_('dashboards.admin.seed.generateTitle')}</h3>
              <p class="text-xs text-green-700">{$_('dashboards.admin.seed.generateSubtitle')}</p>
            </div>
          </div>
          <ul class="text-sm text-gray-700 mb-4 space-y-2">
            <li class="flex items-start gap-2">
              <span class="text-green-600 font-bold">✓</span>
              <span>{$_('dashboards.admin.seed.list1')}</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-green-600 font-bold">✓</span>
              <span>{$_('dashboards.admin.seed.list2')}</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-green-600 font-bold">✓</span>
              <span>{$_('dashboards.admin.seed.list3')}</span>
            </li>
          </ul>
          <button
            on:click={handleSeedDemoData}
            disabled={seedLoading || clearLoading}
            class="w-full bg-green-600 text-white py-3 rounded-lg hover:bg-green-700 transition font-semibold disabled:opacity-50 disabled:cursor-not-allowed shadow-md"
          >
            {seedLoading ? '⏳ ' + $_('dashboards.admin.seed.generating') : '🚀 ' + $_('dashboards.admin.seed.generateButton')}
          </button>
        </div>

        <!-- Clear Seed -->
        <div class="border-2 border-red-200 rounded-lg p-6 bg-red-50">
          <div class="flex items-center gap-3 mb-4">
            <span class="text-4xl">🗑️</span>
            <div>
              <h3 class="font-semibold text-lg text-red-900">{$_('dashboards.admin.seed.deleteTitle')}</h3>
              <p class="text-xs text-red-700">{$_('dashboards.admin.seed.deleteSubtitle')}</p>
            </div>
          </div>
          <ul class="text-sm text-gray-700 mb-4 space-y-2">
            <li class="flex items-start gap-2">
              <span class="text-blue-600 font-bold">🛡️</span>
              <span><strong>{$_('dashboards.admin.seed.preserveLabel')}</strong> {$_('dashboards.admin.seed.preserveProduction')}</span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-red-600 font-bold">🗑️</span>
              <span>{$_('dashboards.admin.seed.deleteOnly')} <code class="bg-red-100 px-1 rounded text-xs">is_seed_data=true</code></span>
            </li>
            <li class="flex items-start gap-2">
              <span class="text-blue-600 font-bold">🔒</span>
              <span>{$_('dashboards.admin.seed.superAdminPreserved')}</span>
            </li>
          </ul>
          <button
            on:click={handleClearDemoData}
            disabled={seedLoading || clearLoading}
            class="w-full bg-red-600 text-white py-3 rounded-lg hover:bg-red-700 transition font-semibold disabled:opacity-50 disabled:cursor-not-allowed shadow-md"
          >
            {clearLoading ? '⏳ ' + $_('dashboards.admin.seed.deleting') : '🗑️ ' + $_('dashboards.admin.seed.deleteButton')}
          </button>
        </div>
      </div>

      <!-- Link to advanced management -->
      <div class="mt-6 p-4 bg-gray-50 border border-gray-200 rounded-lg">
        <p class="text-sm text-gray-600">
          💡 <strong>{$_('dashboards.admin.seed.needDetails')}</strong> {$_('dashboards.admin.seed.consultPage')}
          <a href="/admin/seed" class="text-blue-600 hover:text-blue-800 underline font-medium">{$_('dashboards.admin.seed.advancedPageLink')}</a>
          {$_('dashboards.admin.seed.toSeeDetails')}
        </p>
      </div>
    </div>
  </div>

  <!-- Quick Actions -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Recent Activity -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.admin.recentActivity')}</h2>
      </div>
      <div class="p-6">
        <div class="space-y-4">
          <div class="flex items-start space-x-3">
            <span class="text-2xl">🏛️</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">{$_('dashboards.admin.activity.newOrganization')}</p>
              <p class="text-sm text-gray-600">Copropriété Les Jardins - Paris 15e</p>
              <p class="text-xs text-gray-400 mt-1">{$_('dashboards.admin.activity.twoHoursAgo')}</p>
            </div>
          </div>
          <div class="flex items-start space-x-3">
            <span class="text-2xl">👤</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">{$_('dashboards.admin.activity.newUser')}</p>
              <p class="text-sm text-gray-600">jean.dupont@example.com (Syndic)</p>
              <p class="text-xs text-gray-400 mt-1">{$_('dashboards.admin.activity.fiveHoursAgo')}</p>
            </div>
          </div>
          <div class="flex items-start space-x-3">
            <span class="text-2xl">🏢</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">{$_('dashboards.admin.activity.buildingAdded')}</p>
              <p class="text-sm text-gray-600">Résidence Le Parc - Lyon 3e</p>
              <p class="text-xs text-gray-400 mt-1">{$_('dashboards.admin.activity.yesterday')}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Links -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.admin.quickActions')}</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <a
            href="/admin/organizations"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏛️</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.organizations')}</span>
          </a>
          <a
            href="/admin/users"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">👥</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.users')}</span>
          </a>
          <a
            href="/buildings"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.buildings')}</span>
          </a>
          <a
            href="/admin/subscriptions"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💳</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.subscriptions')}</span>
          </a>
          <a
            href="/admin/seed"
            class="flex flex-col items-center justify-center p-6 border-2 border-green-200 rounded-lg hover:border-green-500 hover:bg-green-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🌱</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.seedData')}</span>
          </a>
          <a
            href="/admin/user-owner-links"
            class="flex flex-col items-center justify-center p-6 border-2 border-blue-200 rounded-lg hover:border-blue-500 hover:bg-blue-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🔗</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.userOwnerLinks')}</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
