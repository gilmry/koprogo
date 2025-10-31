<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Owner, User } from '../lib/types';

  interface OwnerWithUser extends Owner {
    linkedUser?: {
      id: string;
      email: string;
      first_name: string;
      last_name: string;
    };
  }

  let owners: OwnerWithUser[] = [];
  let ownerUsers: User[] = []; // Users with role='owner'
  let loading = true;
  let error: string | null = null;
  let successMessage: string | null = null;

  // Pagination
  let currentPage = 1;
  let totalPages = 1;
  let perPage = 10;

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = null;

      // Load owners (paginated)
      const ownersResponse = await api.get<{ data: Owner[]; pagination: any }>(
        `/owners?page=${currentPage}&per_page=${perPage}`
      );

      totalPages = ownersResponse.pagination.total_pages;

      // Load all users with role='owner'
      const usersResponse = await api.get<{ data: User[] }>('/users?per_page=1000');
      ownerUsers = usersResponse.data.filter((u: User) => u.role === 'owner');

      // Enrich owners with linked user info
      const enrichedOwners = await Promise.all(
        ownersResponse.data.map(async (owner: Owner) => {
          if (owner.user_id) {
            const linkedUser = ownerUsers.find(u => u.id === owner.user_id);
            console.log('Owner with user_id:', owner.id, 'user_id:', owner.user_id, 'linkedUser found:', !!linkedUser);
            return {
              ...owner,
              linkedUser: linkedUser ? {
                id: linkedUser.id,
                email: linkedUser.email,
                first_name: linkedUser.firstName,
                last_name: linkedUser.lastName
              } : undefined
            };
          }
          return owner;
        })
      );

      console.log('Enriched owners:', enrichedOwners);
      console.log('Total owner users:', ownerUsers.length);

      // Force Svelte reactivity by creating a new array reference
      owners = [...enrichedOwners];

      loading = false;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur lors du chargement des données';
      loading = false;
    }
  }

  async function linkOwnerToUser(ownerId: string, userId: string | null) {
    try {
      error = null;
      successMessage = null;

      await api.put(`/owners/${ownerId}/link-user`, {
        user_id: userId
      });

      successMessage = userId
        ? 'Owner lié au user avec succès !'
        : 'Owner délié du user avec succès !';

      // Force reload with a small delay to ensure DB transaction is committed
      await new Promise(resolve => setTimeout(resolve, 300));
      await loadData();

      // Clear success message after 3 seconds
      setTimeout(() => {
        successMessage = null;
      }, 3000);
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur lors de la liaison';
    }
  }

  function getAvailableUsers(currentOwnerId: string): User[] {
    // Show only users that are not already linked to another owner
    const linkedUserIds = owners
      .filter(o => o.id !== currentOwnerId && o.user_id)
      .map(o => o.user_id);

    return ownerUsers.filter(u => !linkedUserIds.includes(u.id));
  }

  async function goToPage(page: number) {
    currentPage = page;
    await loadData();
  }
</script>

<div class="bg-white rounded-lg shadow">
  <div class="p-6 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      Gestion des liens User ↔ Owner
    </h2>
    <p class="text-sm text-gray-600 mt-1">
      Associez les comptes utilisateurs (role=owner) aux entités Owner pour donner accès au portail
    </p>
  </div>

  <div class="p-6">
    {#if error}
      <div class="mb-4 bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded">
        {error}
      </div>
    {/if}

    {#if successMessage}
      <div class="mb-4 bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded">
        {successMessage}
      </div>
    {/if}

    {#if loading}
      <div class="flex items-center justify-center py-12">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Owner
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Email
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                User lié
              </th>
              <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Action
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each owners as owner (owner.id)}
              <tr class="hover:bg-gray-50">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm font-medium text-gray-900">
                    {owner.first_name} {owner.last_name}
                  </div>
                  <div class="text-xs text-gray-500">
                    ID: {owner.id.substring(0, 8)}...
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="text-sm text-gray-900">{owner.email}</div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  {#if owner.linkedUser}
                    <div class="flex items-center">
                      <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                        ✓ {owner.linkedUser.first_name} {owner.linkedUser.last_name}
                      </span>
                    </div>
                    <div class="text-xs text-gray-500 mt-1">
                      {owner.linkedUser.email}
                    </div>
                  {:else}
                    <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                      Aucun lien
                    </span>
                  {/if}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm">
                  {#if owner.linkedUser}
                    <button
                      on:click={() => linkOwnerToUser(owner.id, null)}
                      class="text-red-600 hover:text-red-900 font-medium"
                    >
                      Délier
                    </button>
                  {:else}
                    <select
                      on:change={(e) => {
                        const userId = e.currentTarget.value;
                        if (userId) {
                          linkOwnerToUser(owner.id, userId);
                          e.currentTarget.value = ''; // Reset
                        }
                      }}
                      class="block w-full pl-3 pr-10 py-2 text-sm border-gray-300 focus:outline-none focus:ring-primary-500 focus:border-primary-500 rounded-md"
                    >
                      <option value="">Sélectionner un user...</option>
                      {#each getAvailableUsers(owner.id) as user (user.id)}
                        <option value={user.id}>
                          {user.firstName} {user.lastName} ({user.email})
                        </option>
                      {/each}
                    </select>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>

      <!-- Pagination -->
      {#if totalPages > 1}
        <div class="mt-6 flex items-center justify-between border-t border-gray-200 pt-4">
          <div class="text-sm text-gray-700">
            Page {currentPage} sur {totalPages}
          </div>
          <div class="flex gap-2">
            <button
              on:click={() => goToPage(currentPage - 1)}
              disabled={currentPage === 1}
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Précédent
            </button>
            <button
              on:click={() => goToPage(currentPage + 1)}
              disabled={currentPage === totalPages}
              class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Suivant
            </button>
          </div>
        </div>
      {/if}
    {/if}
  </div>
</div>
