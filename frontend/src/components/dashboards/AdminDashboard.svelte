<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';

  interface Stats {
    totalOrganizations: number;
    totalUsers: number;
    totalBuildings: number;
    activeSubscriptions: number;
    monthlyRevenue: number;
  }

  let stats: Stats = {
    totalOrganizations: 0,
    totalUsers: 0,
    totalBuildings: 0,
    activeSubscriptions: 0,
    monthlyRevenue: 0,
  };
  let loading = true;

  $: user = $authStore.user;

  onMount(async () => {
    // TODO: Fetch real stats from API
    // Simulated data for now
    setTimeout(() => {
      stats = {
        totalOrganizations: 24,
        totalUsers: 187,
        totalBuildings: 98,
        activeSubscriptions: 22,
        monthlyRevenue: 4580,
      };
      loading = false;
    }, 500);
  });
</script>

<div>
  <!-- Header -->
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      Bienvenue, {user?.firstName} 👋
    </h1>
    <p class="text-gray-600">
      Dashboard SuperAdmin - Vue d'ensemble de la plateforme KoproGo
    </p>
  </div>

  <!-- Stats Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6 mb-8">
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Organisations</span>
        <span class="text-2xl">🏛️</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalOrganizations}</p>
        <p class="text-sm text-green-600 mt-1">+3 ce mois</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Utilisateurs</span>
        <span class="text-2xl">👥</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalUsers}</p>
        <p class="text-sm text-green-600 mt-1">+12 ce mois</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Immeubles</span>
        <span class="text-2xl">🏢</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.totalBuildings}</p>
        <p class="text-sm text-green-600 mt-1">+5 ce mois</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Abonnements actifs</span>
        <span class="text-2xl">✅</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.activeSubscriptions}</p>
        <p class="text-sm text-gray-500 mt-1">sur {stats.totalOrganizations} orgs</p>
      {/if}
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">MRR</span>
        <span class="text-2xl">💰</span>
      </div>
      {#if loading}
        <div class="h-8 bg-gray-200 animate-pulse rounded"></div>
      {:else}
        <p class="text-3xl font-bold text-gray-900">{stats.monthlyRevenue}€</p>
        <p class="text-sm text-green-600 mt-1">+8.5% ce mois</p>
      {/if}
    </div>
  </div>

  <!-- Quick Actions -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Recent Activity -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Activité récente</h2>
      </div>
      <div class="p-6">
        <div class="space-y-4">
          <div class="flex items-start space-x-3">
            <span class="text-2xl">🏛️</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Nouvelle organisation créée</p>
              <p class="text-sm text-gray-600">Copropriété Les Jardins - Paris 15e</p>
              <p class="text-xs text-gray-400 mt-1">Il y a 2 heures</p>
            </div>
          </div>
          <div class="flex items-start space-x-3">
            <span class="text-2xl">👤</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Nouvel utilisateur</p>
              <p class="text-sm text-gray-600">jean.dupont@example.com (Syndic)</p>
              <p class="text-xs text-gray-400 mt-1">Il y a 5 heures</p>
            </div>
          </div>
          <div class="flex items-start space-x-3">
            <span class="text-2xl">🏢</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Immeuble ajouté</p>
              <p class="text-sm text-gray-600">Résidence Le Parc - Lyon 3e</p>
              <p class="text-xs text-gray-400 mt-1">Hier à 14:32</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Links -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Actions rapides</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <a
            href="/admin/organizations"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏛️</span>
            <span class="text-sm font-medium text-gray-700">Organisations</span>
          </a>
          <a
            href="/admin/users"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">👥</span>
            <span class="text-sm font-medium text-gray-700">Utilisateurs</span>
          </a>
          <a
            href="/buildings"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">Immeubles</span>
          </a>
          <a
            href="/admin/subscriptions"
            class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group"
          >
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💳</span>
            <span class="text-sm font-medium text-gray-700">Abonnements</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
