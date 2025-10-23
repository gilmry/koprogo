<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';
  import { apiEndpoint } from '../../lib/config';

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
  let seedLoading = false;
  let clearLoading = false;
  let seedMessage = '';
  let seedError = '';

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
    console.log('API Endpoint:', apiEndpoint('/api/v1/seed/demo'));
    console.log('============================');

    try {
      const response = await fetch(apiEndpoint('/api/v1/seed/demo'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$authStore.token}`,
        },
      });

      const data = await response.json();

      if (response.ok) {
        seedMessage = data.message || 'Données de démonstration créées avec succès!';
        // Reload stats after seeding
        setTimeout(() => seedMessage = '', 5000);
      } else {
        seedError = data.error || 'Erreur lors de la création des données';
        setTimeout(() => seedError = '', 5000);
      }
    } catch (error) {
      console.error('Seed error:', error);
      seedError = 'Erreur de connexion au serveur';
      setTimeout(() => seedError = '', 5000);
    } finally {
      seedLoading = false;
    }
  };

  const handleClearDemoData = async () => {
    if (!confirm('Êtes-vous sûr de vouloir supprimer toutes les données de démonstration?')) {
      return;
    }

    clearLoading = true;
    seedMessage = '';
    seedError = '';

    try {
      const response = await fetch(apiEndpoint('/api/v1/seed/clear'), {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$authStore.token}`,
        },
      });

      const data = await response.json();

      if (response.ok) {
        seedMessage = data.message || 'Données de démonstration supprimées avec succès!';
        setTimeout(() => seedMessage = '', 5000);
      } else {
        seedError = data.error || 'Erreur lors de la suppression des données';
        setTimeout(() => seedError = '', 5000);
      }
    } catch (error) {
      console.error('Clear error:', error);
      seedError = 'Erreur de connexion au serveur';
      setTimeout(() => seedError = '', 5000);
    } finally {
      clearLoading = false;
    }
  };
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

  <!-- Database Management -->
  <div class="bg-white rounded-lg shadow mb-8">
    <div class="p-6 border-b border-gray-200">
      <h2 class="text-lg font-semibold text-gray-900">Gestion de la base de données</h2>
      <p class="text-sm text-gray-600 mt-1">Gérer les données de démonstration pour les tests et la présentation</p>
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
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="border-2 border-gray-200 rounded-lg p-6">
          <div class="flex items-center mb-4">
            <span class="text-3xl mr-3">🌱</span>
            <div>
              <h3 class="font-semibold text-gray-900">Générer les données de démo</h3>
              <p class="text-sm text-gray-600">Crée une organisation complète avec utilisateurs, immeubles et charges</p>
            </div>
          </div>
          <button
            on:click={handleSeedDemoData}
            disabled={seedLoading || clearLoading}
            class="w-full bg-green-600 text-white py-3 rounded-lg hover:bg-green-700 transition font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {seedLoading ? 'Génération en cours...' : 'Générer les données'}
          </button>
          <div class="mt-3 text-xs text-gray-500">
            <p class="font-semibold mb-1">Comptes créés:</p>
            <ul class="space-y-0.5 ml-4">
              <li>• Syndic: syndic@copro-demo.fr / syndic123</li>
              <li>• Comptable: comptable@copro-demo.fr / comptable123</li>
              <li>• Propriétaire 1: proprietaire1@copro-demo.fr / owner123</li>
              <li>• Propriétaire 2: proprietaire2@copro-demo.fr / owner123</li>
            </ul>
          </div>
        </div>
        <div class="border-2 border-gray-200 rounded-lg p-6">
          <div class="flex items-center mb-4">
            <span class="text-3xl mr-3">🗑️</span>
            <div>
              <h3 class="font-semibold text-gray-900">Supprimer les données de démo</h3>
              <p class="text-sm text-gray-600">Supprime toutes les données de démonstration (préserve le SuperAdmin)</p>
            </div>
          </div>
          <button
            on:click={handleClearDemoData}
            disabled={seedLoading || clearLoading}
            class="w-full bg-red-600 text-white py-3 rounded-lg hover:bg-red-700 transition font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {clearLoading ? 'Suppression en cours...' : 'Supprimer les données'}
          </button>
          <p class="mt-3 text-xs text-gray-500">
            ⚠️ Cette action supprimera toutes les organisations, utilisateurs, immeubles, propriétaires, lots et charges de démonstration.
          </p>
        </div>
      </div>
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
