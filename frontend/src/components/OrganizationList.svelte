<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { Organization } from '../lib/types';
  import OrganizationForm from './admin/OrganizationForm.svelte';
  import ConfirmDialog from './ui/ConfirmDialog.svelte';
  import Button from './ui/Button.svelte';

  let organizations: Organization[] = [];
  let loading = true;
  let error = '';
  let searchTerm = '';
  let showFormModal = false;
  let showConfirmDialog = false;
  let selectedOrganization: Organization | null = null;
  let formMode: 'create' | 'edit' = 'create';
  let actionLoading = false;

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

  const handleCreate = () => {
    selectedOrganization = null;
    formMode = 'create';
    showFormModal = true;
  };

  const handleEdit = (org: Organization) => {
    selectedOrganization = org;
    formMode = 'edit';
    showFormModal = true;
  };

  const handleToggleActive = async (org: Organization) => {
    actionLoading = true;
    try {
      const endpoint = org.is_active
        ? `/organizations/${org.id}/suspend`
        : `/organizations/${org.id}/activate`;

      await api.put(endpoint, {});

      toast.show(
        `Organisation ${org.is_active ? 'désactivée' : 'activée'} avec succès`,
        'success'
      );

      await loadOrganizations();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
    } finally {
      actionLoading = false;
    }
  };

  const handleDeleteClick = (org: Organization) => {
    selectedOrganization = org;
    showConfirmDialog = true;
  };

  const handleDeleteConfirm = async () => {
    if (!selectedOrganization) return;

    actionLoading = true;
    try {
      await api.delete(`/organizations/${selectedOrganization.id}`);
      toast.show('Organisation supprimée avec succès', 'success');
      showConfirmDialog = false;
      selectedOrganization = null;
      await loadOrganizations();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
    } finally {
      actionLoading = false;
    }
  };

  const handleFormSuccess = async () => {
    await loadOrganizations();
  };
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
    <Button variant="primary" on:click={handleCreate} data-testid="create-organization-button">
      ➕ Nouvelle organisation
    </Button>
  </div>

  <!-- Search Bar -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="relative">
      <input
        type="text"
        bind:value={searchTerm}
        placeholder="Rechercher par nom, email ou slug..."
        data-testid="organization-search-input"
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
          <tbody class="bg-white divide-y divide-gray-200" data-testid="organizations-table-body">
            {#each filteredOrganizations as org (org.id)}
              <tr class="hover:bg-gray-50" data-testid="organization-row" data-org-id={org.id} data-org-name={org.name}>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div>
                    <div class="text-sm font-medium text-gray-900" data-testid="organization-name">{org.name}</div>
                    <div class="text-sm text-gray-500" data-testid="organization-slug">/{org.slug}</div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900" data-testid="organization-email">{org.contact_email}</div>
                  {#if org.contact_phone}
                    <div class="text-sm text-gray-500" data-testid="organization-phone">{org.contact_phone}</div>
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
                  <div class="flex justify-end space-x-2">
                    <button
                      on:click={() => handleEdit(org)}
                      class="text-primary-600 hover:text-primary-900"
                      title="Modifier"
                      data-testid="edit-organization-button"
                      disabled={actionLoading}
                    >
                      ✏️
                    </button>
                    <button
                      on:click={() => handleToggleActive(org)}
                      class={org.is_active ? 'text-orange-600 hover:text-orange-900' : 'text-green-600 hover:text-green-900'}
                      title={org.is_active ? 'Désactiver' : 'Activer'}
                      data-testid="toggle-organization-button"
                      disabled={actionLoading}
                    >
                      {org.is_active ? '⏸️' : '▶️'}
                    </button>
                    <button
                      on:click={() => handleDeleteClick(org)}
                      class="text-red-600 hover:text-red-900"
                      title="Supprimer"
                      data-testid="delete-organization-button"
                      disabled={actionLoading}
                    >
                      🗑️
                    </button>
                  </div>
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

<!-- Organization Form Modal -->
<OrganizationForm
  bind:isOpen={showFormModal}
  organization={selectedOrganization}
  mode={formMode}
  on:success={handleFormSuccess}
  on:close={() => {
    showFormModal = false;
    selectedOrganization = null;
  }}
/>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:isOpen={showConfirmDialog}
  title="Confirmer la suppression"
  message="Êtes-vous sûr de vouloir supprimer l'organisation '{selectedOrganization?.name}' ? Cette action est irréversible et supprimera également tous les utilisateurs, immeubles et données associés."
  confirmText="Supprimer"
  cancelText="Annuler"
  variant="danger"
  loading={actionLoading}
  on:confirm={handleDeleteConfirm}
  on:cancel={() => {
    showConfirmDialog = false;
    selectedOrganization = null;
  }}
/>
