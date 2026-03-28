<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';
  import { formatDateTime } from '../../lib/utils/date.utils';
  import { withErrorHandling } from '../../lib/utils/error.utils';
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
    const responseData = await withErrorHandling({
      action: () => api.get<{ data: User[] }>('/users'),
      setLoading: (v) => loading = v,
      errorMessage: $_('admin.errors.failedToLoadUsers'),
    });
    if (responseData) {
      users = responseData.data || [];
      filteredUsers = users;
    }
  }

  async function handleAdminExport(userId: string, userEmail: string) {
    selectedUserId = userId;
    selectedUserEmail = userEmail;

    const data = await withErrorHandling({
      action: () => api.get<GdprExport>(`/admin/gdpr/users/${userId}/export`),
      setLoading: (v) => loading = v,
      successMessage: `Data exported for ${userEmail} - User will be notified`,
      errorMessage: 'Failed to export data',
    });
    if (data) {
      exportData = data;
      showExportModal = true;
      await loadAuditLogs();
    }
  }

  async function handleAdminErase(userId: string, userEmail: string) {
    selectedUserId = userId;
    selectedUserEmail = userEmail;

    const result = await withErrorHandling({
      action: () => api.delete<GdprEraseResponse>(`/admin/gdpr/users/${userId}/erase`),
      setLoading: (v) => loading = v,
      successMessage: `Data erased for ${userEmail} - User will be notified`,
      errorMessage: 'Failed to erase data',
    });
    showEraseConfirmation = false;
    if (result) {
      erasureResult = result;
      await Promise.all([loadUsers(), loadAuditLogs()]);
    }
  }

  async function loadAuditLogs(page = 1) {
    const params = new URLSearchParams({
      page: page.toString(),
      per_page: '20',
      event_type: 'Gdpr',
    });

    const data = await withErrorHandling({
      action: () => api.get<{ logs: any[]; total: number }>(`/admin/gdpr/audit-logs?${params}`),
      setLoading: (v) => loading = v,
      errorMessage: 'Failed to load audit logs',
    });
    if (data) {
      auditLogs = data.logs || [];
      auditLogsPage = page;
      auditLogsTotalPages = Math.ceil((data.total || 0) / 20);
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
          {$_('admin.gdpr.title')}
        </h2>
        <p class="mt-1 text-sm text-gray-500">
          {$_('admin.gdpr.description')}
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
        {showAuditLogs ? $_('common.hide') : $_('common.show')} {$_('admin.gdpr.auditLogs')}
      </button>
    </div>

    <!-- Search Bar -->
    <div class="mt-4">
      <label for="user-search" class="sr-only">{$_('common.searchUsers')}</label>
      <input
        type="text"
        id="user-search"
        bind:value={searchQuery}
        data-testid="admin-gdpr-search"
        placeholder={$_('admin.gdpr.searchPlaceholder')}
        class="shadow-sm focus:ring-indigo-500 focus:border-indigo-500 block w-full sm:text-sm border-gray-300 rounded-md"
      />
    </div>
  </div>

  <!-- Users Table -->
  <div class="bg-white shadow rounded-lg overflow-hidden" data-testid="admin-gdpr-users-table">
    <div class="px-6 py-4 border-b border-gray-200">
      <h3 class="text-lg font-medium text-gray-900">
        {$_('common.users')} ({filteredUsers.length})
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
        <p class="mt-2 text-sm text-gray-500">{$_('common.loadingUsers')}</p>
      </div>
    {:else if filteredUsers.length === 0}
      <div class="p-6 text-center">
        <p class="text-sm text-gray-500">
          {searchQuery ? $_('admin.gdpr.noUsersFound') : $_('admin.gdpr.noUsersAvailable')}
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
                {$_('common.user')}
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_('common.role')}
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_('common.organization')}
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_('common.status')}
              </th>
              <th
                scope="col"
                class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider"
              >
                {$_('common.actions')}
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
                    {user.is_active ? $_('common.active') : $_('common.inactive')}
                  </span>
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                  <button
                    on:click={() => handleAdminExport(user.id, user.email)}
                    disabled={loading}
                    data-testid="admin-gdpr-export-user"
                    class="text-blue-600 hover:text-blue-900 mr-4 disabled:opacity-50"
                  >
                    {$_('admin.gdpr.export')}
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
                    {$_('admin.gdpr.erase')}
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
          {$_('admin.gdpr.auditLogsTitle')}
        </h3>
        <button
          on:click={() => loadAuditLogs(auditLogsPage)}
          disabled={loading}
          data-testid="admin-gdpr-refresh-logs"
          class="text-sm text-indigo-600 hover:text-indigo-900 disabled:opacity-50"
        >
          {$_('common.refresh')}
        </button>
      </div>

      {#if loading && auditLogs.length === 0}
        <div class="p-6 text-center">
          <p class="text-sm text-gray-500">{$_('admin.gdpr.loadingAuditLogs')}</p>
        </div>
      {:else if auditLogs.length === 0}
        <div class="p-6 text-center">
          <p class="text-sm text-gray-500">{$_('admin.gdpr.noAuditLogs')}</p>
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
                  {$_('common.timestamp')}
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  {$_('common.eventType')}
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  {$_('common.userId')}
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  {$_('common.ipAddress')}
                </th>
                <th
                  scope="col"
                  class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  {$_('admin.gdpr.adminInitiated')}
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              {#each auditLogs as log (log.id)}
                <tr data-testid="admin-gdpr-audit-log-row">
                  <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {formatDateTime(log.timestamp)}
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
                        {$_('admin.gdpr.admin')}
                      </span>
                    {:else}
                      <span class="text-sm text-gray-500">{$_('admin.gdpr.selfService')}</span>
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
                {$_('common.previous')}
              </button>
              <button
                on:click={() => loadAuditLogs(auditLogsPage + 1)}
                disabled={auditLogsPage === auditLogsTotalPages || loading}
                class="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
              >
                {$_('common.next')}
              </button>
            </div>
            <div class="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
              <div>
                <p class="text-sm text-gray-700">
                  {$_('common.page')} <span class="font-medium">{auditLogsPage}</span> {$_('common.of')}
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
                    ← {$_('common.previous')}
                  </button>
                  <button
                    on:click={() => loadAuditLogs(auditLogsPage + 1)}
                    disabled={auditLogsPage === auditLogsTotalPages || loading}
                    class="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50"
                  >
                    {$_('common.next')} →
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
            {$_('admin.gdpr.anonymizedSuccessfully')}
          </h3>
          <div class="mt-2 text-sm text-green-700">
            <p>{$_('admin.gdpr.user')}: {erasureResult.user_email}</p>
            <p>{$_('admin.gdpr.ownersAnonymized')}: {erasureResult.owners_anonymized}</p>
            <p>{$_('common.timestamp')}: {formatDateTime(erasureResult.anonymized_at)}</p>
          </div>
          <button
            on:click={() => (erasureResult = null)}
            class="mt-2 text-sm font-medium text-green-800 hover:text-green-900"
          >
            {$_('common.dismiss')}
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
                {$_('admin.gdpr.adminDataExport')}: {selectedUserEmail}
              </h3>
              <div class="mt-2 space-y-4 max-h-96 overflow-y-auto">
                <div>
                  <h4 class="font-semibold text-gray-700">{$_('admin.gdpr.exportDate')}:</h4>
                  <p class="text-sm text-gray-600">
                    {formatDateTime(exportData.export_date)}
                  </p>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">{$_('admin.gdpr.userInformation')}:</h4>
                  <pre
                    class="text-xs bg-gray-50 p-2 rounded overflow-x-auto">{JSON.stringify(exportData.user, null, 2)}</pre>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">
                    {$_('admin.gdpr.summary')}:
                  </h4>
                  <ul class="text-sm text-gray-600 list-disc list-inside">
                    <li>{$_('admin.gdpr.owners')}: {exportData.owners.length}</li>
                    <li>{$_('admin.gdpr.units')}: {exportData.units.length}</li>
                    <li>{$_('admin.gdpr.expenses')}: {exportData.expenses.length}</li>
                    <li>{$_('admin.gdpr.documents')}: {exportData.documents.length}</li>
                    <li>{$_('admin.gdpr.meetings')}: {exportData.meetings.length}</li>
                    <li>{$_('admin.gdpr.totalItems')}: {exportData.total_items}</li>
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
            {$_('admin.gdpr.downloadJson')}
          </button>
          <button
            type="button"
            on:click={() => (showExportModal = false)}
            data-testid="admin-gdpr-modal-close"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
          >
            {$_('common.close')}
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
                {$_('admin.gdpr.eraseUserData')}
              </h3>
              <div class="mt-2">
                <p class="text-sm text-gray-500">
                  {$_('admin.gdpr.aboutToAnonymize')}
                </p>
                <p class="mt-2 text-sm font-semibold text-gray-900">
                  {selectedUserEmail}
                </p>
                <ul class="mt-3 text-sm text-gray-500 list-disc list-inside">
                  <li>{$_('admin.gdpr.userAccountAnonymized')}</li>
                  <li>{$_('admin.gdpr.ownerRecordsAnonymized')}</li>
                  <li>{$_('admin.gdpr.userNotifiedViaEmail')}</li>
                  <li>{$_('admin.gdpr.operationLoggedInAuditTrail')}</li>
                </ul>
                <p class="mt-3 text-sm font-semibold text-red-600">
                  {$_('admin.gdpr.cannotBeUndone')}
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
              {$_('admin.gdpr.erasing')}...
            {:else}
              {$_('admin.gdpr.yesEraseUserData')}
            {/if}
          </button>
          <button
            type="button"
            on:click={() => (showEraseConfirmation = false)}
            disabled={loading}
            data-testid="admin-gdpr-erase-cancel"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:w-auto sm:text-sm disabled:opacity-50"
          >
            {$_('common.cancel')}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
