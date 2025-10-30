<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { toastStore } from '../stores/toast';
  import type { GdprExport, GdprEraseResponse } from '../lib/types';

  let loading = false;
  let canErase = false;
  let checkingErasure = false;
  let exportData: GdprExport | null = null;
  let showExportModal = false;
  let showEraseConfirmation = false;
  let erasureResult: GdprEraseResponse | null = null;

  onMount(async () => {
    await checkCanErase();
  });

  async function checkCanErase() {
    checkingErasure = true;
    try {
      const token = $authStore.token;
      if (!token) throw new Error('Not authenticated');

      const response = await fetch('/api/v1/gdpr/can-erase', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        canErase = data.can_erase;
      }
    } catch (error) {
      console.error('Failed to check erasure eligibility:', error);
    } finally {
      checkingErasure = false;
    }
  }

  async function handleExportData() {
    loading = true;
    try {
      const token = $authStore.token;
      if (!token) throw new Error('Not authenticated');

      const response = await fetch('/api/v1/gdpr/export', {
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Export failed');
      }

      exportData = await response.json();
      showExportModal = true;
      toastStore.success('Your personal data has been exported successfully');
    } catch (error) {
      toastStore.error(error instanceof Error ? error.message : 'Failed to export data');
    } finally {
      loading = false;
    }
  }

  async function handleEraseData() {
    loading = true;
    try {
      const token = $authStore.token;
      if (!token) throw new Error('Not authenticated');

      const response = await fetch('/api/v1/gdpr/erase', {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Erasure failed');
      }

      erasureResult = await response.json();
      showEraseConfirmation = false;
      toastStore.success('Your personal data has been anonymized');

      // Logout after erasure
      setTimeout(() => {
        authStore.logout();
        window.location.href = '/login';
      }, 3000);
    } catch (error) {
      toastStore.error(error instanceof Error ? error.message : 'Failed to erase data');
      showEraseConfirmation = false;
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
    a.download = `gdpr-export-${new Date().toISOString()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
</script>

<div class="bg-white shadow rounded-lg p-6" data-testid="gdpr-data-panel">
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold text-gray-900" data-testid="gdpr-panel-title">My Personal Data</h2>
    <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800" data-testid="gdpr-rights-badge">
      GDPR Rights
    </span>
  </div>

  <div class="space-y-6">
    <!-- Export Data Section -->
    <div class="border-l-4 border-blue-500 bg-blue-50 p-4">
      <div class="flex items-start">
        <div class="flex-shrink-0">
          <svg class="h-6 w-6 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
          </svg>
        </div>
        <div class="ml-3 flex-1">
          <h3 class="text-lg font-medium text-blue-900">Right to Access (Article 15)</h3>
          <p class="mt-2 text-sm text-blue-700">
            Export all your personal data in a machine-readable format. This includes your account information,
            owner records, unit ownerships, expenses, documents, and meeting participation.
          </p>
          <button
            on:click={handleExportData}
            disabled={loading}
            data-testid="gdpr-export-button"
            class="mt-4 inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
          >
            {#if loading}
              <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              Exporting...
            {:else}
              Export My Data
            {/if}
          </button>
        </div>
      </div>
    </div>

    <!-- Erase Data Section -->
    <div class="border-l-4 border-red-500 bg-red-50 p-4">
      <div class="flex items-start">
        <div class="flex-shrink-0">
          <svg class="h-6 w-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
          </svg>
        </div>
        <div class="ml-3 flex-1">
          <h3 class="text-lg font-medium text-red-900">Right to Erasure (Article 17)</h3>
          <p class="mt-2 text-sm text-red-700">
            Request the permanent anonymization of your personal data. This action is irreversible and will
            replace your identifiable information with anonymized placeholders.
          </p>
          {#if checkingErasure}
            <p class="mt-2 text-sm text-red-600">Checking eligibility...</p>
          {:else if !canErase}
            <div class="mt-3 bg-yellow-50 border border-yellow-200 rounded-md p-3">
              <p class="text-sm text-yellow-800">
                ⚠️ Your data cannot be erased at this time due to legal holds or active obligations.
                Please contact support for more information.
              </p>
            </div>
          {:else}
            <button
              on:click={() => showEraseConfirmation = true}
              disabled={loading}
              data-testid="gdpr-erase-button"
              class="mt-4 inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50"
            >
              Request Data Erasure
            </button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Erasure Result -->
    {#if erasureResult}
      <div class="bg-green-50 border border-green-200 rounded-md p-4" data-testid="gdpr-erasure-result">
        <div class="flex">
          <div class="flex-shrink-0">
            <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
            </svg>
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-green-800">Data Anonymized Successfully</h3>
            <div class="mt-2 text-sm text-green-700">
              <p>Your user account and {erasureResult.owners_anonymized} owner record(s) have been anonymized.</p>
              <p class="mt-1">Anonymized at: {new Date(erasureResult.anonymized_at).toLocaleString()}</p>
              <p class="mt-1 font-semibold">You will be logged out in 3 seconds...</p>
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Export Modal -->
{#if showExportModal && exportData}
  <div class="fixed z-50 inset-0 overflow-y-auto" data-testid="gdpr-export-modal">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" on:click={() => showExportModal = false}></div>

      <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-4xl sm:w-full" data-testid="gdpr-export-modal-content">
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="mt-3 text-center sm:mt-0 sm:text-left w-full">
              <h3 class="text-lg leading-6 font-medium text-gray-900 mb-4">
                Personal Data Export
              </h3>
              <div class="mt-2 space-y-4 max-h-96 overflow-y-auto">
                <div>
                  <h4 class="font-semibold text-gray-700">Export Date:</h4>
                  <p class="text-sm text-gray-600">{new Date(exportData.export_date).toLocaleString()}</p>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">User Information:</h4>
                  <pre class="text-xs bg-gray-50 p-2 rounded overflow-x-auto">{JSON.stringify(exportData.user, null, 2)}</pre>
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">Owner Records ({exportData.owners.length}):</h4>
                  {#if exportData.owners.length > 0}
                    <pre class="text-xs bg-gray-50 p-2 rounded overflow-x-auto">{JSON.stringify(exportData.owners, null, 2)}</pre>
                  {:else}
                    <p class="text-sm text-gray-500 italic">No owner records</p>
                  {/if}
                </div>

                <div>
                  <h4 class="font-semibold text-gray-700">Unit Ownerships ({exportData.units.length}):</h4>
                  {#if exportData.units.length > 0}
                    <pre class="text-xs bg-gray-50 p-2 rounded overflow-x-auto">{JSON.stringify(exportData.units, null, 2)}</pre>
                  {:else}
                    <p class="text-sm text-gray-500 italic">No unit ownerships</p>
                  {/if}
                </div>

                <div class="text-sm text-gray-600">
                  <p><strong>Total Items:</strong> {exportData.total_items}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
          <button
            type="button"
            on:click={downloadExport}
            data-testid="gdpr-download-export-button"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm"
          >
            Download JSON
          </button>
          <button
            type="button"
            on:click={() => showExportModal = false}
            data-testid="gdpr-export-modal-close"
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
  <div class="fixed z-50 inset-0 overflow-y-auto" data-testid="gdpr-erase-confirm-modal">
    <div class="flex items-center justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0">
      <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity" on:click={() => showEraseConfirmation = false}></div>

      <div class="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full" data-testid="gdpr-erase-confirm-content">
        <div class="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
          <div class="sm:flex sm:items-start">
            <div class="mx-auto flex-shrink-0 flex items-center justify-center h-12 w-12 rounded-full bg-red-100 sm:mx-0 sm:h-10 sm:w-10">
              <svg class="h-6 w-6 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
              </svg>
            </div>
            <div class="mt-3 text-center sm:mt-0 sm:ml-4 sm:text-left">
              <h3 class="text-lg leading-6 font-medium text-gray-900">
                Confirm Data Erasure
              </h3>
              <div class="mt-2">
                <p class="text-sm text-gray-500">
                  Are you absolutely sure you want to erase your personal data? This action is <strong>irreversible</strong> and will:
                </p>
                <ul class="mt-2 text-sm text-gray-500 list-disc list-inside">
                  <li>Anonymize your user account</li>
                  <li>Anonymize all your owner records</li>
                  <li>Replace identifiable information with placeholders</li>
                  <li>Log you out immediately</li>
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
            on:click={handleEraseData}
            disabled={loading}
            data-testid="gdpr-erase-confirm-button"
            class="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
          >
            {#if loading}
              Erasing...
            {:else}
              Yes, Erase My Data
            {/if}
          </button>
          <button
            type="button"
            on:click={() => showEraseConfirmation = false}
            disabled={loading}
            data-testid="gdpr-erase-cancel-button"
            class="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:w-auto sm:text-sm disabled:opacity-50"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
