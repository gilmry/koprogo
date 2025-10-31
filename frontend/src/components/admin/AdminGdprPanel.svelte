<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';
  import { toast } from '../../stores/toast';
  import { api } from '../../lib/api';
  import { UserRole } from '../../lib/types';
  import type {
    GdprExport,
    GdprEraseResponse,
    User,
  } from '../../lib/types';

  let users: User[] = [];
  let filteredUsers: User[] = [];
  let searchQuery = '';
  let loading = false;
  let selectedUserId: string | null = null;
  let selectedUserEmail = '';
  let exportData: GdprExport | null = null;
  let erasureResult: GdprEraseResponse | null = null;
  let showExportModal = false;
  let showEraseConfirmation = false;
  let auditLogs: any[] = [];
  let showAuditLogs = false;
  let auditLogsPage = 1;
  let auditLogsTotalPages = 1;

  onMount(async () => {
    // Ensure auth store is initialized before loading users
    await authStore.init();
    await loadUsers();
  });

  $: {
    if (searchQuery) {
      filteredUsers = users.filter(
        (u) =>
          u.email.toLowerCase().includes(searchQuery.toLowerCase()) ||
          u.first_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          u.last_name.toLowerCase().includes(searchQuery.toLowerCase()),
      );
    } else {
      filteredUsers = users;
    }
  }

  async function loadUsers() {
    loading = true;
    try {
      const responseData = await api.get<{ data: User[] }>('/users');
      users = responseData.data || [];
      filteredUsers = users;
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to load users',
      );
    } finally {
      loading = false;
    }
  }

  async function handleAdminExport(userId: string, userEmail: string) {
    loading = true;
    selectedUserId = userId;
    selectedUserEmail = userEmail;

    try {
      exportData = await api.get<GdprExport>(`/admin/gdpr/users/${userId}/export`);
      showExportModal = true;
      toast.success(
        `Data exported for ${userEmail} - User will be notified`,
      );

      // Reload audit logs after operation
      await loadAuditLogs();
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to export data',
      );
    } finally {
      loading = false;
    }
  }

  async function handleAdminErase(userId: string, userEmail: string) {
    loading = true;
    selectedUserId = userId;
    selectedUserEmail = userEmail;

    try {
      erasureResult = await api.delete<GdprEraseResponse>(`/admin/gdpr/users/${userId}/erase`);
      showEraseConfirmation = false;
      toast.success(
        `Data erased for ${userEmail} - User will be notified`,
      );

      // Reload users and audit logs
      await Promise.all([loadUsers(), loadAuditLogs()]);
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to erase data',
      );
      showEraseConfirmation = false;
    } finally {
      loading = false;
    }
  }

  async function loadAuditLogs(page = 1) {
    loading = true;
    try {
      const params = new URLSearchParams({
        page: page.toString(),
        per_page: '20',
        event_type: 'Gdpr',
      });

      const data = await api.get<{ logs: any[]; total: number }>(`/admin/gdpr/audit-logs?${params}`);
      auditLogs = data.logs || [];
      auditLogsPage = page;
      auditLogsTotalPages = Math.ceil((data.total || 0) / 20);
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : 'Failed to load audit logs',
      );
    } finally {
      loading = false;
    }
  }

  function downloadExport() {
    if (!exportData) return;

    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `admin-gdpr-export-${selectedUserId}-${new Date().toISOString()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function formatEventType(eventType: string): string {
    return eventType.replace(/([A-Z])/g, ' $1').trim();
  }
</script>

<div class="space-y-6" data-testid="admin-gdpr-panel">
  <!-- Header -->
  <div class="bg-white shadow rounded-lg p-6">
    <div class="flex items-center justify-between mb-4">
      <div>
        <h2 class="text-2xl font-bold text-gray-900" data-testid="admin-gdpr-title">
          GDPR Administration
        </h2>
        <p class="mt-1 text-sm text-gray-500">
          Manage user data exports and erasure requests (SuperAdmin only)
        </p>
      </div>
      <button
        on:click={() => {
          showAuditLogs = !showAuditLogs;
          if (showAuditLogs && auditLogs.length === 0) loadAuditLogs();
        }}
        data-testid="admin-gdpr-audit-toggle"
        class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
      >
        {showAuditLogs ? 'Hide' : 'Show'} Audit Logs
      </button>
    </div>

    <!-- Search Bar -->
    <div class="mt-4">
      <label for="user-search" class="sr-only">Search users</label>
      <input
        type="text"
        id="user-search"
        bind:value={searchQuery}
        data-testid="admin-gdpr-search"
        placeholder="Search by email, first name, or last name..."
        class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
      />
    </div>
  </div>

  <!-- Users Table -->
  <div class="bg-white shadow rounded-lg overflow-hidden" data-testid="admin-gdpr-users-table">
    <div class="px-6 py-4 border-b border-gray-200">
      <h3 class="text-lg font-medium text-gray-900">
        Users ({filteredUsers.length})
      </h3>
    </div>

    {#if loading && users.length === 0}
      <div class="p-6 text-center">
        <svg
          class="animate-spin h-8 w-8 mx-auto text-indigo-600"
          fill="none"
          viewBox="0 0 24 24"
        >
          <circle
            class="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            stroke-width="4"
          ></circle>
          <path
            class="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          ></path>
        </svg>
        <p class="mt-2 text-sm text-gray-500">Loading users...</p>
      </div>
    {:else if filteredUsers.length === 0}
      <div class="p-6 text-center">
        <p class="text-sm text-gray-500">
          {searchQuery ? 'No users found matching your search' : 'No users available'}
        </p>
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="min-w-full divide-y divide-gray-200">
          <thead class="bg-gray-50">
            <tr>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                User
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Role
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Organization
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Status
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                Actions
              </th>
            </tr>
          </thead>
          <tbody class="bg-white divide-y divide-gray-200">
            {#each filteredUsers as user (user.id)}
              <tr data-testid="admin-gdpr-user-row">
                <td class="px-6 py-4 whitespace-nowrap">
                  <div class="flex items-center">
                    <div>
                      <div class="text-sm font-medium text-gray-900" data-testid="user-name">
                        {user.first_name}
                        {user.last_name}
                      </div>
                      <div class="text-sm text-gray-500" data-testid="user-email">
                        {user.email}
                      </div>
                    </div>
                  </div>
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span
                    class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {user.role ===
                    UserRole.SUPERADMIN
                      ? 'bg-purple-100 text-purple-800'
                      : user.role === UserRole.SYNDIC
                        ? 'bg-blue-100 text-blue-800'
                        : 'bg-gray-100 text-gray-800'}"
                  >
                    {user.role}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {user.organizationId || '-'}
                </td>
                <td class="px-6 py-4 whitespace-nowrap">
                  <span
                    class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {user.is_active
                      ? 'bg-green-100 text-green-800'
                      : 'bg-red-100 text-red-800'}"
                  >
                    {user.is_active ? 'Active' : 'Inactive'}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <button
                    on:click={() => handleAdminExport(user.id, user.email)}
                    disabled={loading}
                    data-testid="admin-gdpr-export-user"
                    class="text-blue-600 hover:text-blue-900 mr-4 disabled:opacity-50"
                  >
                    Export
                  </button>
                  <button
                    on:click={() => {
                      selectedUserId = user.id;
                      selectedUserEmail = user.email;
                      showEraseConfirmation = true;
                    }}
                    disabled={loading}
                    data-testid="admin-gdpr-erase-user"
                    class="text-red-600 hover:text-red-900 disabled:opacity-50"
                  >
                    Erase
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

  <!-- Audit Logs Section -->
  {#if showAuditLogs}
    <div
      class="bg-white shadow rounded-lg overflow-hidden"
      data-testid="admin-gdpr-audit-logs"
    >
      <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
        <h3 class="text-lg font-medium text-gray-900">
          GDPR Audit Logs (7-year retention)
        </h3>
        <button
          on:click={() => loadAuditLogs(auditLogsPage)}
          disabled={loading}
          data-testid="admin-gdpr-refresh-logs"
          class="text-sm text-indigo-600 hover:text-indigo-900 disabled:opacity-50"
        >
          Refresh
        </button>
      </div>

      {#if loading && auditLogs.length === 0}
        <div class="p-6 text-center">
          <p class="text-sm text-gray-500">Loading audit logs...</p>
        </div>
      {:else if auditLogs.length === 0}
        <div class="p-6 text-center">
          <p class="text-sm text-gray-500">No audit logs found</p>
        </div>
      {:else}
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Timestamp
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Event Type
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  User ID
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  IP Address
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  Admin Initiated
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              {#each auditLogs as log (log.id)}
                <tr data-testid="admin-gdpr-audit-log-row">
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {new Date(log.timestamp).toLocaleString()}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    <span class="text-sm font-medium text-gray-900">
                      {formatEventType(log.event_type)}
                    </span>
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500 font-mono">
                    {log.user_id ? `${log.user_id.substring(0, 8)}...` : '-'}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {log.ip_address || '-'}
                  </td>
                  <td class="px-6 py-4 whitespace-nowrap">
                    {#if log.metadata?.admin_initiated}
                      <span
                        class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full bg-yellow-100 text-yellow-800"
                      >
                        Admin
                      </span>
                    {:else}
                      <span class="text-sm text-gray-500">Self-service</span>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>

        <!-- Pagination -->
        {#if auditLogsTotalPages > 1}
          <div class="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
            <div class="flex-1 flex justify-between sm:hidden">
              <button
                on:click={() => loadAuditLogs(auditLogsPage - 1)}
                disabled={auditLogsPage === 1 || loading}
                class="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
              >
                Previous
              </button>
              <button
                on:click={() => loadAuditLogs(auditLogsPage + 1)}
                disabled={auditLogsPage === auditLogsTotalPages || loading}
                class="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
              >
                Next
              </button>
            </div>
            <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
              <div>
                <p class="text-sm text-gray-700">
                  Page <span class="font-medium">{auditLogsPage}</span> of
                  <span class="font-medium">{auditLogsTotalPages}</span>
                </p>
              </div>
              <div>
                <nav class="relative z-0 inline-flex rounded-md shadow-sm -space-x-px">
                  <button
                    on:click={() => loadAuditLogs(auditLogsPage - 1)}
                    disabled={auditLogsPage === 1 || loading}
                    class="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50"
                  >
                    ← Previous
                  </button>
                  <button
                    on:click={() => loadAuditLogs(auditLogsPage + 1)}
                    disabled={auditLogsPage === auditLogsTotalPages || loading}
                    class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50"
                  >
                    Next →
                  </button>
                </nav>
              </div>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Erasure Result Banner -->
  {#if erasureResult}
    <div
      class="bg-green-50 border border-green-200 rounded-md p-4"
      data-testid="admin-gdpr-erasure-result"
    >
      <div class="flex">
        <div class="flex-shrink-0">
          <svg
            class="h-5 w-5 text-green-400"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fill-rule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clip-rule="evenodd"
            />
          </svg>
        </div>
        <div class="ml-3">
          <h3 class="text-sm font-medium text-green-800">
            Data Anonymized Successfully
          </h3>
          <div class="mt-2 text-sm text-green-700">
            <p>User: {erasureResult.user_email}</p>
            <p>Owners anonymized: {erasureResult.owners_anonymized}</p>
            <p>Timestamp: {new Date(erasureResult.anonymized_at).toLocaleString()}</p>
          </div>
          <button
            on:click={() => (erasureResult = null)}
            class="mt-2 text-sm font-medium text-green-800 hover:text-green-900"
          >
            Dismiss
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Export Modal -->
{#if showExportModal && exportData}
  <div class="fixed z-50 inset-0 overflow-y-auto" data-testid="admin-gdpr-export-modal">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <div
        class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
        on:click={() => (showExportModal = false)}
        aria-hidden="true"
      ></div>

      <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-4xl sm:w-full relative z-10">
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="mt-3 text-center sm:mt-0 sm:text-left w-full">
              <h3 class="text-lg leading-6 font-medium text-gray-900 mb-4">
                Admin Data Export: {selectedUserEmail}
              </h3>
              <div class="mt-2 space-y-4 max-h-96 overflow-y-auto">
                <div>
                  <h4 class="font-semibold text-gray-700">Export Date:</h4>
                  <p class="text-sm text-gray-600">
                    {new Date(exportData.export_date).toLocaleString()}
                  </p>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">User Information:</h4>
                  <pre
                    class="text-xs bg-gray-50 p-2 rounded overflow-x-auto">{JSON.stringify(exportData.user, null, 2)}</pre>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">
                    Summary:
                  </h4>
                  <ul class="text-sm text-gray-600 list-disc list-inside">
                    <li>Owners: {exportData.owners.length}</li>
                    <li>Units: {exportData.units.length}</li>
                    <li>Expenses: {exportData.expenses.length}</li>
                    <li>Documents: {exportData.documents.length}</li>
                    <li>Meetings: {exportData.meetings.length}</li>
                    <li>Total Items: {exportData.total_items}</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
          <button
            type="button"
            on:click={downloadExport}
            data-testid="admin-gdpr-download-button"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Download JSON
          </button>
          <button
            type="button"
            on:click={() => (showExportModal = false)}
            data-testid="admin-gdpr-modal-close"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Erase Confirmation Modal -->
{#if showEraseConfirmation}
  <div class="fixed z-50 inset-0 overflow-y-auto" data-testid="admin-gdpr-erase-modal">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <div
        class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
        on:click={() => (showEraseConfirmation = false)}
        aria-hidden="true"
      ></div>

      <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full relative z-10">
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div
              class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10"
            >
              <svg
                class="h-6 w-6 text-red-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
              </svg>
            </div>
            <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
              <h3 class="text-lg leading-6 font-medium text-gray-900">
                Admin Erase User Data
              </h3>
              <div class="mt-2">
                <p class="text-sm text-gray-500">
                  You are about to <strong class="text-red-600">permanently anonymize</strong> data for:
                </p>
                <p class="mt-2 text-sm font-semibold text-gray-900">
                  {selectedUserEmail}
                </p>
                <ul class="mt-3 text-sm text-gray-500 list-disc list-inside">
                  <li>User account will be anonymized</li>
                  <li>All owner records will be anonymized</li>
                  <li>User will be notified via email</li>
                  <li>Operation will be logged in audit trail</li>
                </ul>
                <p class="mt-3 text-sm font-semibold text-red-600">
                  This action cannot be undone!
                </p>
              </div>
            </div>
          </div>
        </div>
        <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
          <button
            type="button"
            on:click={() =>
              selectedUserId && handleAdminErase(selectedUserId, selectedUserEmail)}
            disabled={loading}
            data-testid="admin-gdpr-erase-confirm"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
          >
            {#if loading}
              Erasing...
            {:else}
              Yes, Erase User Data
            {/if}
          </button>
          <button
            type="button"
            on:click={() => (showEraseConfirmation = false)}
            disabled={loading}
            data-testid="admin-gdpr-erase-cancel"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:w-auto sm:text-sm disabled:opacity-50"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
