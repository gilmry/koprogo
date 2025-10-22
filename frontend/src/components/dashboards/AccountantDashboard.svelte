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
      Dashboard Comptable - Gestion financière
    </p>
  </div>

  <!-- Stats Cards -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Charges totales</span>
        <span class="text-2xl">💰</span>
      </div>
      <p class="text-3xl font-bold text-gray-900">45 850€</p>
      <p class="text-sm text-gray-500 mt-1">Ce mois</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">Payé</span>
        <span class="text-2xl">✅</span>
      </div>
      <p class="text-3xl font-bold text-green-600">41 270€</p>
      <p class="text-sm text-gray-500 mt-1">90% collecté</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">En attente</span>
        <span class="text-2xl">⏳</span>
      </div>
      <p class="text-3xl font-bold text-orange-600">4 580€</p>
      <p class="text-sm text-gray-500 mt-1">10% restant</p>
    </div>

    <div class="bg-white rounded-lg shadow p-6">
      <div class="flex items-center justify-between mb-2">
        <span class="text-gray-600 text-sm font-medium">En retard</span>
        <span class="text-2xl">🚨</span>
      </div>
      <p class="text-3xl font-bold text-red-600">8</p>
      <p class="text-sm text-gray-500 mt-1">Copropriétaires</p>
    </div>
  </div>

  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Recent Transactions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Dernières transactions</h2>
      </div>
      <div class="p-6">
        <div class="space-y-4">
          <div class="flex items-center justify-between p-4 bg-green-50 border border-green-200 rounded-lg">
            <div class="flex items-center space-x-3">
              <span class="text-2xl">✅</span>
              <div>
                <p class="text-sm font-medium text-gray-900">Paiement reçu</p>
                <p class="text-sm text-gray-600">M. Durand - Lot 24</p>
              </div>
            </div>
            <p class="text-lg font-bold text-green-600">+850€</p>
          </div>
          <div class="flex items-center justify-between p-4 bg-red-50 border border-red-200 rounded-lg">
            <div class="flex items-center space-x-3">
              <span class="text-2xl">💸</span>
              <div>
                <p class="text-sm font-medium text-gray-900">Paiement effectué</p>
                <p class="text-sm text-gray-600">Entretien ascenseur</p>
              </div>
            </div>
            <p class="text-lg font-bold text-red-600">-2 400€</p>
          </div>
          <div class="flex items-center justify-between p-4 bg-green-50 border border-green-200 rounded-lg">
            <div class="flex items-center space-x-3">
              <span class="text-2xl">✅</span>
              <div>
                <p class="text-sm font-medium text-gray-900">Paiement reçu</p>
                <p class="text-sm text-gray-600">Mme Martin - Lot 12</p>
              </div>
            </div>
            <p class="text-lg font-bold text-green-600">+720€</p>
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
          <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
            <span class="text-sm font-medium text-gray-700">Charges</span>
          </a>
          <a href="/reports" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📈</span>
            <span class="text-sm font-medium text-gray-700">Rapports</span>
          </a>
          <a href="/invoices" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📄</span>
            <span class="text-sm font-medium text-gray-700">Factures</span>
          </a>
          <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">Immeubles</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
