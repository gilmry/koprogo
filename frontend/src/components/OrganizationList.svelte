<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  interface Organization {
    id: string;
    name: string;
    slug: string;
    contact_email: string;
    contact_phone?: string;
    subscription_plan: string;
    max_buildings: number;
    max_users: number;
    is_active: boolean;
    created_at: string;
  }

  let organizations: Organization[] = [];
  let loading = true;
  let error = '';
  let searchTerm = '';

  onMount(async () => {
    await loadOrganizations();
  });

  async function loadOrganizations() {
    try {
      loading = true;
      error = '';
      // Pour le SuperAdmin, on veut TOUTES les organisations sans filtre
      const response = await api.get<{data: Organization[]}>('/organizations?per_page=1000');
      organizations = response.data;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des organisations';
      console.error('Error loading organizations:', e);
    } finally {
      loading = false;
    }
  }

  $: filteredOrganizations = organizations.filter(org =>
    org.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    org.contact_email.toLowerCase().includes(searchTerm.toLowerCase()) ||
    org.slug.toLowerCase().includes(searchTerm.toLowerCase())
  );

  function getPlanBadgeClass(plan: string): string {
    const classes = {
      free: 'bg-gray-100 text-gray-800',
      professional: 'bg-blue-100 text-blue-800',
      enterprise: 'bg-purple-100 text-purple-800',
    };
    return classes[plan as keyof typeof classes] || 'bg-gray-100 text-gray-800';
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', { 
      year: 'numeric', 
      month: 'short', 
      day: 'numeric' 
    });
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-3xl font-bold text-gray-900">Organisations</h1>
      <p class="mt-1 text-sm text-gray-600">
        Gérer toutes les organisations de la plateforme
      </p>
    </div>
    <button class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition">
      ➕ Nouvelle organisation
    </button>
  </div>

  <!-- Search Bar -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="relative">
      <input
        type="text"
        bind:value={searchTerm}
        placeholder="Rechercher par nom, email ou slug..."
        class="w-full px-4 py-2 pl-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      />
      <span class="absolute left-3 top-2.5 text-gray-400">🔍</span>
    </div>
  </div>

  <!-- Error Message -->
  {#if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
      ⚠️ {error}
    </div>
  {/if}

  <!-- Table -->
  <div class="bg-white rounded-lg shadow overflow-hidden">
    {#if loading}
      <div class="p-12 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
        <p class="mt-2 text-gray-600">Chargement...</p>
      </div>
    {:else if filteredOrganizations.length === 0}
      <div class="p-12 text-center text-gray-500">
        {searchTerm ? 'Aucune organisation trouvée pour cette recherche.' : 'Aucune organisation enregistrée.'}
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Organisation
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Contact
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Plan
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Limites
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Statut
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Créée le
              </th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each filteredOrganizations as org (org.id)}
              <tr class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div>
                    <div class="text-sm font-medium text-gray-900">{org.name}</div>
                    <div class="text-sm text-gray-500">/{org.slug}</div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900">{org.contact_email}</div>
                  {#if org.contact_phone}
                    <div class="text-sm text-gray-500">{org.contact_phone}</div>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getPlanBadgeClass(org.subscription_plan)}">
                    {org.subscription_plan}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  <div>{org.max_buildings} immeubles</div>
                  <div>{org.max_users} utilisateurs</div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  {#if org.is_active}
                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                      ✓ Active
                    </span>
                  {:else}
                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">
                      ✗ Inactive
                    </span>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {formatDate(org.created_at)}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <button class="text-primary-600 hover:text-primary-900 mr-3">
                    👁️ Voir
                  </button>
                  <button class="text-gray-600 hover:text-gray-900">
                    ✏️ Modifier
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Footer -->
      <div class="bg-gray-50 px-6 py-3 border-t border-gray-200">
        <p class="text-sm text-gray-700">
          <span class="font-medium">{filteredOrganizations.length}</span>
          {filteredOrganizations.length === 1 ? 'organisation' : 'organisations'}
          {searchTerm ? ' (filtrées)' : ''}
        </p>
      </div>
    {/if}
  </div>
</div>
