<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { User } from '../lib/types';
  import UserForm from './admin/UserForm.svelte';
  import ConfirmDialog from './ui/ConfirmDialog.svelte';
  import Button from './ui/Button.svelte';

  // Backend user format (snake_case)
  interface BackendUser {
    id: string;
    email: string;
    first_name: string;
    last_name: string;
    role: string;
    organization_id?: string;
    is_active: boolean;
    created_at: string;
  }

  let users: User[] = [];
  let loading = true;
  let error = '';
  let searchTerm = '';
  let roleFilter = 'all';
  let showFormModal = false;
  let showConfirmDialog = false;
  let selectedUser: User | null = null;
  let formMode: 'create' | 'edit' = 'create';
  let actionLoading = false;

  onMount(async () => {
    await loadUsers();
  });

  async function loadUsers() {
    try {
      loading = true;
      error = '';
      const response = await api.get<{ data: BackendUser[] }>('/users?per_page=1000');

      // Map backend format to frontend format
      users = response.data.map((u) => ({
        id: u.id,
        email: u.email,
        firstName: u.first_name,
        lastName: u.last_name,
        role: u.role as any,
        organizationId: u.organization_id,
        is_active: u.is_active,
        created_at: u.created_at,
      }));
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des utilisateurs';
      console.error('Error loading users:', e);
    } finally {
      loading = false;
    }
  }

  $: filteredUsers = users.filter((user) => {
    const matchesSearch =
      user.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.firstName.toLowerCase().includes(searchTerm.toLowerCase()) ||
      user.lastName.toLowerCase().includes(searchTerm.toLowerCase());

    const matchesRole = roleFilter === 'all' || user.role === roleFilter;

    return matchesSearch && matchesRole;
  });

  function getRoleBadgeClass(role: string): string {
    const classes = {
      superadmin: 'bg-purple-100 text-purple-800',
      syndic: 'bg-blue-100 text-blue-800',
      accountant: 'bg-green-100 text-green-800',
      owner: 'bg-yellow-100 text-yellow-800',
    };
    return classes[role as keyof typeof classes] || 'bg-gray-100 text-gray-800';
  }

  function getRoleLabel(role: string): string {
    const labels = {
      superadmin: '👑 SuperAdmin',
      syndic: '🏢 Syndic',
      accountant: '📊 Comptable',
      owner: '👤 Propriétaire',
    };
    return labels[role as keyof typeof labels] || role;
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
    selectedUser = null;
    formMode = 'create';
    showFormModal = true;
  };

  const handleEdit = (user: User) => {
    selectedUser = user;
    formMode = 'edit';
    showFormModal = true;
  };

  const handleToggleActive = async (user: User) => {
    actionLoading = true;
    try {
      const endpoint = user.is_active
        ? `/users/${user.id}/deactivate`
        : `/users/${user.id}/activate`;

      await api.put(endpoint, {});

      toast.show(
        `Utilisateur ${user.is_active ? 'désactivé' : 'activé'} avec succès`,
        'success'
      );

      await loadUsers();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
    } finally {
      actionLoading = false;
    }
  };

  const handleDeleteClick = (user: User) => {
    selectedUser = user;
    showConfirmDialog = true;
  };

  const handleDeleteConfirm = async () => {
    if (!selectedUser) return;

    actionLoading = true;
    try {
      await api.delete(`/users/${selectedUser.id}`);
      toast.show('Utilisateur supprimé avec succès', 'success');
      showConfirmDialog = false;
      selectedUser = null;
      await loadUsers();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
    } finally {
      actionLoading = false;
    }
  };

  const handleFormSuccess = async () => {
    await loadUsers();
  };
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-3xl font-bold text-gray-900">Utilisateurs</h1>
      <p class="mt-1 text-sm text-gray-600">
        Gérer tous les utilisateurs de la plateforme
      </p>
    </div>
    <Button variant="primary" on:click={handleCreate}>
      ➕ Nouvel utilisateur
    </Button>
  </div>

  <!-- Search and Filters -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="flex flex-col md:flex-row gap-4">
      <div class="flex-1 relative">
        <input
          type="text"
          bind:value={searchTerm}
          placeholder="Rechercher par nom ou email..."
          class="w-full px-4 py-2 pl-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        />
        <span class="absolute left-3 top-2.5 text-gray-400">🔍</span>
      </div>
      <div>
        <select
          bind:value={roleFilter}
          class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
        >
          <option value="all">Tous les rôles</option>
          <option value="superadmin">SuperAdmin</option>
          <option value="syndic">Syndic</option>
          <option value="accountant">Comptable</option>
          <option value="owner">Propriétaire</option>
        </select>
      </div>
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
    {:else if filteredUsers.length === 0}
      <div class="p-12 text-center text-gray-500">
        {searchTerm || roleFilter !== 'all' ? 'Aucun utilisateur trouvé pour cette recherche.' : 'Aucun utilisateur enregistré.'}
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Utilisateur
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Email
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Rôle
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Organisation
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Statut
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Inscrit le
              </th>
              <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each filteredUsers as user (user.id)}
              <tr class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="flex items-center">
                    <div class="flex-shrink-0 h-10 w-10 bg-primary-100 rounded-full flex items-center justify-center">
                      <span class="text-primary-600 font-semibold">
                        {user.firstName[0]}{user.lastName[0]}
                      </span>
                    </div>
                    <div class="ml-4">
                      <div class="text-sm font-medium text-gray-900">
                        {user.firstName} {user.lastName}
                      </div>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900">{user.email}</div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getRoleBadgeClass(user.role)}">
                    {getRoleLabel(user.role)}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {user.organizationId ? user.organizationId.substring(0, 8) + '...' : '-'}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  {#if user.is_active}
                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-green-100 text-green-800">
                      ✓ Actif
                    </span>
                  {:else}
                    <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-red-100 text-red-800">
                      ✗ Inactif
                    </span>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {formatDate(user.created_at)}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <div class="flex justify-end space-x-2">
                    <button
                      on:click={() => handleEdit(user)}
                      class="text-primary-600 hover:text-primary-900"
                      title="Modifier"
                      disabled={actionLoading}
                    >
                      ✏️
                    </button>
                    <button
                      on:click={() => handleToggleActive(user)}
                      class={user.is_active ? 'text-orange-600 hover:text-orange-900' : 'text-green-600 hover:text-green-900'}
                      title={user.is_active ? 'Désactiver' : 'Activer'}
                      disabled={actionLoading}
                    >
                      {user.is_active ? '⏸️' : '▶️'}
                    </button>
                    <button
                      on:click={() => handleDeleteClick(user)}
                      class="text-red-600 hover:text-red-900"
                      title="Supprimer"
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
          <span class="font-medium">{filteredUsers.length}</span>
          {filteredUsers.length === 1 ? 'utilisateur' : 'utilisateurs'}
          {searchTerm || roleFilter !== 'all' ? ' (filtrés)' : ''}
        </p>
      </div>
    {/if}
  </div>
</div>

<!-- User Form Modal -->
<UserForm
  bind:isOpen={showFormModal}
  user={selectedUser}
  mode={formMode}
  on:success={handleFormSuccess}
  on:close={() => {
    showFormModal = false;
    selectedUser = null;
  }}
/>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:isOpen={showConfirmDialog}
  title="Confirmer la suppression"
  message="Êtes-vous sûr de vouloir supprimer l'utilisateur '{selectedUser?.firstName} {selectedUser?.lastName}' ? Cette action est irréversible."
  confirmText="Supprimer"
  cancelText="Annuler"
  variant="danger"
  loading={actionLoading}
  on:confirm={handleDeleteConfirm}
  on:cancel={() => {
    showConfirmDialog = false;
    selectedUser = null;
  }}
/>
