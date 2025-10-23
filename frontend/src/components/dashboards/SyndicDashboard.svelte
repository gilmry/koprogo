<script lang="ts">
  import { authStore } from '../../stores/auth';

  $: user = $authStore.user;
</script>

<div>
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      Bienvenue, {user?.firstName} 👋
    </h1>
    <p class="text-gray-600">
      Dashboard Syndic - Gestion de vos copropriétés
    </p>
  </div>

  <!-- Stats Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Immeubles gérés</span>
        <span class="text-2xl">🏢</span>
      </div>
      <p class="text-3xl font-bold text-gray-900">8</p>
      <p class="text-sm text-gray-500 mt-1">245 lots au total</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Copropriétaires</span>
        <span class="text-2xl">👥</span>
      </div>
      <p class="text-3xl font-bold text-gray-900">184</p>
      <p class="text-sm text-green-600 mt-1">+5 ce mois</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Charges en attente</span>
        <span class="text-2xl">💰</span>
      </div>
      <p class="text-3xl font-bold text-gray-900">12</p>
      <p class="text-sm text-orange-600 mt-1">4 580€</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Prochaine AG</span>
        <span class="text-2xl">📅</span>
      </div>
      <p class="text-xl font-bold text-gray-900">15 Nov</p>
      <p class="text-sm text-gray-500 mt-1">Résidence Les Jardins</p>
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
        <div class="space-y-4">
          <div class="flex items-start space-x-3 p-4 bg-red-50 border border-red-200 rounded-lg">
            <span class="text-2xl">🚨</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Fuite d'eau - Bâtiment A</p>
              <p class="text-sm text-gray-600">Intervention plombier requise</p>
              <p class="text-xs text-red-600 mt-1">Urgent</p>
            </div>
          </div>
          <div class="flex items-start space-x-3 p-4 bg-orange-50 border border-orange-200 rounded-lg">
            <span class="text-2xl">📄</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Convocation AG à envoyer</p>
              <p class="text-sm text-gray-600">Résidence Le Parc</p>
              <p class="text-xs text-orange-600 mt-1">Dans 3 jours</p>
            </div>
          </div>
          <div class="flex items-start space-x-3 p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
            <span class="text-2xl">💰</span>
            <div class="flex-1">
              <p class="text-sm font-medium text-gray-900">Relance paiements</p>
              <p class="text-sm text-gray-600">8 copropriétaires en retard</p>
              <p class="text-xs text-yellow-600 mt-1">À traiter</p>
            </div>
          </div>
        </div>
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
</div>
