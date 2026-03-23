<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { etatsDatesApi, type EtatDate } from '../../lib/api/etats-dates';
  import EtatDateStatusBadge from './EtatDateStatusBadge.svelte';
  import { toast } from '../../stores/toast';

  let etatDate: EtatDate | null = null;
  let loading = true;
  let error = '';
  let actionLoading = false;
  let etatDateId = '';

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    etatDateId = params.get('id') || '';
    if (etatDateId) loadEtatDate();
  });

  async function loadEtatDate() {
    try {
      loading = true;
      error = '';
      etatDate = await etatsDatesApi.getById(etatDateId);
    } catch (err: any) {
      error = err.message || $_('common.loadingError');
    } finally {
      loading = false;
    }
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function formatDate(dateString: string | null): string {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric', month: 'long', day: 'numeric'
    });
  }

  function formatPercent(value: number): string {
    return `${value.toFixed(2)}%`;
  }

  async function markInProgress() {
    if (!confirm($_('etatsDate.confirms.startProcessing'))) return;
    try {
      actionLoading = true;
      etatDate = await etatsDatesApi.markInProgress(etatDateId);
    } catch (err: any) {
      toast.error($_('etatsDate.errors.markingInProgress') + ': ' + (err.message || $_('common.error')));
    } finally {
      actionLoading = false;
    }
  }

  async function markGenerated() {
    const pdfPath = prompt($_('etatsDate.prompts.pdfPath'));
    if (!pdfPath) return;
    try {
      actionLoading = true;
      etatDate = await etatsDatesApi.markGenerated(etatDateId, pdfPath);
    } catch (err: any) {
      toast.error($_('etatsDate.errors.markingGenerated') + ': ' + (err.message || $_('common.error')));
    } finally {
      actionLoading = false;
    }
  }

  async function markDelivered() {
    if (!confirm($_('etatsDate.confirms.confirmDelivery'))) return;
    try {
      actionLoading = true;
      etatDate = await etatsDatesApi.markDelivered(etatDateId);
    } catch (err: any) {
      toast.error($_('etatsDate.errors.markingDelivered') + ': ' + (err.message || $_('common.error')));
    } finally {
      actionLoading = false;
    }
  }

  async function deleteEtatDate() {
    if (!confirm($_('etatsDate.confirms.deleteEtatDate'))) return;
    try {
      await etatsDatesApi.delete(etatDateId);
      window.location.href = '/etats-dates';
    } catch (err: any) {
      toast.error($_('etatsDate.errors.deletion') + ': ' + (err.message || $_('common.error')));
    }
  }
</script>

{#if loading}
  <div class="flex justify-center py-12">
    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
  </div>
{:else if error}
  <div class="bg-red-50 border border-red-200 rounded-lg p-4">
    <p class="text-red-700">{error}</p>
    <button on:click={loadEtatDate} class="mt-2 text-sm text-red-600 underline">{$_('common.retry')}</button>
  </div>
{:else if etatDate}
  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-white">{$_('etatsDate.title')}</h1>
            <p class="text-indigo-100 font-mono">{etatDate.reference_number}</p>
          </div>
          <div class="flex items-center gap-2">
            <EtatDateStatusBadge status={etatDate.status} />
            {#if etatDate.is_overdue}
              <span class="px-2 py-1 bg-red-500 text-white text-xs rounded-full font-bold">{$_('etatsDate.overdue')}</span>
            {/if}
          </div>
        </div>
      </div>
    </div>

    <!-- Property Info -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('etatsDate.propertyInfo')}</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <p class="text-sm text-gray-600">{$_('etatsDate.building')}</p>
          <p class="font-medium text-gray-900">{etatDate.building_name}</p>
          <p class="text-sm text-gray-500">{etatDate.building_address}</p>
        </div>
        <div>
          <p class="text-sm text-gray-600">{$_('etatsDate.unit')}</p>
          <p class="font-medium text-gray-900">N. {etatDate.unit_number}</p>
          {#if etatDate.unit_floor}
            <p class="text-sm text-gray-500">{$_('etatsDate.floor')} {etatDate.unit_floor}</p>
          {/if}
          {#if etatDate.unit_area}
            <p class="text-sm text-gray-500">{etatDate.unit_area} m2</p>
          {/if}
        </div>
      </div>
      <div class="mt-4 grid grid-cols-2 gap-4">
        <div class="bg-gray-50 rounded-lg p-3">
          <p class="text-sm text-gray-600">{$_('etatsDate.ordinaryChargesQuota')}</p>
          <p class="text-lg font-bold text-gray-900">{formatPercent(etatDate.ordinary_charges_quota)}</p>
        </div>
        <div class="bg-gray-50 rounded-lg p-3">
          <p class="text-sm text-gray-600">{$_('etatsDate.extraordinaryChargesQuota')}</p>
          <p class="text-lg font-bold text-gray-900">{formatPercent(etatDate.extraordinary_charges_quota)}</p>
        </div>
      </div>
    </div>

    <!-- Financial Data -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('etatsDate.financialData')}</h2>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <p class="text-sm text-gray-600">{$_('etatsDate.ownerBalance')}</p>
          <p class="text-2xl font-bold {etatDate.owner_balance >= 0 ? 'text-green-600' : 'text-red-600'}">
            {formatCurrency(etatDate.owner_balance)}
          </p>
        </div>
        <div class="bg-orange-50 border border-orange-200 rounded-lg p-4">
          <p class="text-sm text-gray-600">{$_('etatsDate.arrears')}</p>
          <p class="text-2xl font-bold text-orange-600">{formatCurrency(etatDate.arrears_amount)}</p>
        </div>
        <div class="bg-green-50 border border-green-200 rounded-lg p-4">
          <p class="text-sm text-gray-600">{$_('etatsDate.monthlyProvision')}</p>
          <p class="text-2xl font-bold text-green-600">{formatCurrency(etatDate.monthly_provision_amount)}</p>
        </div>
      </div>
      <div class="mt-4 grid grid-cols-2 gap-4">
        <div class="p-3 border rounded-lg">
          <p class="text-sm text-gray-600">{$_('etatsDate.totalBalance')}</p>
          <p class="text-xl font-bold {etatDate.total_balance >= 0 ? 'text-green-600' : 'text-red-600'}">
            {formatCurrency(etatDate.total_balance)}
          </p>
        </div>
        <div class="p-3 border rounded-lg">
          <p class="text-sm text-gray-600">{$_('etatsDate.approvedWorksUnpaid')}</p>
          <p class="text-xl font-bold text-gray-900">{formatCurrency(etatDate.approved_works_unpaid)}</p>
        </div>
      </div>
    </div>

    <!-- Notary Info -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('etatsDate.notaryInfo')}</h2>
      <div class="space-y-2">
        <p><span class="text-gray-600">{$_('common.name')}:</span> <span class="font-medium">{etatDate.notary_name}</span></p>
        <p><span class="text-gray-600">{$_('common.email')}:</span> <span class="font-medium">{etatDate.notary_email}</span></p>
        {#if etatDate.notary_phone}
          <p><span class="text-gray-600">{$_('common.phone')}:</span> <span class="font-medium">{etatDate.notary_phone}</span></p>
        {/if}
        <p><span class="text-gray-600">{$_('common.language')}:</span> <span class="font-medium">
          {etatDate.language === 'fr' ? $_('languages.fr') : etatDate.language === 'nl' ? $_('languages.nl') : $_('languages.de')}
        </span></p>
      </div>
    </div>

    <!-- Timeline -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('etatsDate.timeline')}</h2>
      <div class="space-y-3">
        <div class="flex justify-between py-2 border-b">
          <span class="text-gray-600">{$_('etatsDate.referenceDate')}</span>
          <span class="font-medium">{formatDate(etatDate.reference_date)}</span>
        </div>
        <div class="flex justify-between py-2 border-b">
          <span class="text-gray-600">{$_('etatsDate.requestedOn')}</span>
          <span class="font-medium">{formatDate(etatDate.requested_date)}</span>
        </div>
        {#if etatDate.generated_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">{$_('etatsDate.generatedOn')}</span>
            <span class="font-medium text-green-600">{formatDate(etatDate.generated_date)}</span>
          </div>
        {/if}
        {#if etatDate.delivered_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">{$_('etatsDate.deliveredOn')}</span>
            <span class="font-medium text-purple-600">{formatDate(etatDate.delivered_date)}</span>
          </div>
        {/if}
        <div class="flex justify-between py-2">
          <span class="text-gray-600">{$_('etatsDate.daysSinceRequest')}</span>
          <span class="font-bold {etatDate.days_since_request > 10 ? 'text-red-600' : 'text-gray-900'}">
            {etatDate.days_since_request} {$_('common.days')}
          </span>
        </div>
      </div>
    </div>

    <!-- PDF -->
    {#if etatDate.pdf_file_path}
      <div class="bg-white rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-3">{$_('etatsDate.pdfDocument')}</h2>
        <a href={etatDate.pdf_file_path} class="text-primary-600 hover:text-primary-700 font-medium">
          {$_('etatsDate.downloadPdf')}
        </a>
      </div>
    {/if}

    <!-- Actions -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('common.actions')}</h2>
      <div class="flex flex-wrap gap-3">
        {#if etatDate.status === 'requested'}
          <button
            on:click={markInProgress}
            disabled={actionLoading}
            class="px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition disabled:opacity-50"
          >
            {$_('etatsDate.actions.startProcessing')}
          </button>
        {/if}

        {#if etatDate.status === 'in_progress'}
          <button
            on:click={markGenerated}
            disabled={actionLoading}
            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition disabled:opacity-50"
          >
            {$_('etatsDate.actions.markAsGenerated')}
          </button>
        {/if}

        {#if etatDate.status === 'generated'}
          <button
            on:click={markDelivered}
            disabled={actionLoading}
            class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition disabled:opacity-50"
          >
            {$_('etatsDate.actions.confirmDelivery')}
          </button>
        {/if}

        {#if etatDate.status === 'requested' || etatDate.status === 'in_progress'}
          <button
            on:click={deleteEtatDate}
            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition"
          >
            {$_('common.delete')}
          </button>
        {/if}

        <a href="/etats-dates" class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition">
          {$_('common.backToList')}
        </a>
      </div>
    </div>
  </div>
{/if}
