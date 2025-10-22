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
      Mon Espace Copropriétaire
    </p>
  </div>

  <!-- Stats Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Mes lots</span>
        <span class="text-2xl">🚪</span>
      </div>
      <p class="text-3xl font-bold text-gray-900">2</p>
      <p class="text-sm text-gray-500 mt-1">Appartement + Parking</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Charges à payer</span>
        <span class="text-2xl">💰</span>
      </div>
      <p class="text-3xl font-bold text-orange-600">850€</p>
      <p class="text-sm text-gray-500 mt-1">Échéance: 30 Nov</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Prochaine AG</span>
        <span class="text-2xl">📅</span>
      </div>
      <p class="text-xl font-bold text-gray-900">15 Nov</p>
      <p class="text-sm text-gray-500 mt-1">à 18h00</p>
    </div>
  </div>

  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- My Units -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Mes lots</h2>
      </div>
      <div class="p-6">
        <div class="space-y-4">
          <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
            <div class="flex items-center justify-between mb-2">
              <h3 class="font-semibold text-gray-900">Lot 24 - Appartement</h3>
              <span class="text-2xl">🏠</span>
            </div>
            <p class="text-sm text-gray-600">3ème étage, 75m²</p>
            <p class="text-sm text-gray-500 mt-1">Quote-part: 850/10000</p>
          </div>
          <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
            <div class="flex items-center justify-between mb-2">
              <h3 class="font-semibold text-gray-900">Lot 103 - Parking</h3>
              <span class="text-2xl">🚗</span>
            </div>
            <p class="text-sm text-gray-600">Sous-sol -1</p>
            <p class="text-sm text-gray-500 mt-1">Quote-part: 50/10000</p>
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
          <a href="/owner/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
            <span class="text-sm font-medium text-gray-700">Mes charges</span>
          </a>
          <a href="/owner/documents" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📄</span>
            <span class="text-sm font-medium text-gray-700">Documents</span>
          </a>
          <a href="/owner/profile" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">👤</span>
            <span class="text-sm font-medium text-gray-700">Mon profil</span>
          </a>
          <a href="/owner/contact" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">✉️</span>
            <span class="text-sm font-medium text-gray-700">Contact</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
