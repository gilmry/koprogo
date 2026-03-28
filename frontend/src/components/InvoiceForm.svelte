<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import InvoiceLineItems from './InvoiceLineItems.svelte';
  import { todayISO, defaultDueDate, toISODateNoon } from '../lib/utils/date.utils';
  import { calculateVAT as calcVAT, formatCurrency, aggregateLineItems } from '../lib/utils/finance.utils';
  import { withLoadingState, withErrorHandling } from '../lib/utils/error.utils';

  export let buildingId: string = '';
  export let organizationId: string = ''; // Organization ID for multi-tenant
  export let invoiceId: string | null = null; // null for create, UUID for edit
  export let onSaved: ((invoice: any) => void) | null = null;
  export let onCancel: (() => void) | null = null;

  // Form mode: 'simple' for single amount, 'detailed' for line items
  let mode: 'simple' | 'detailed' = 'simple';

  // Form fields
  let description = '';
  let category = 'Maintenance';
  let amountExclVat = '';
  let vatRate = '21.00';
  let invoiceDate = '';
  let dueDate = '';
  let supplier = '';
  let invoiceNumber = '';
  let accountCode = ''; // Code compte PCMN

  // Line items for detailed mode
  let lineItems: any[] = [];

  // Liste des comptes PCMN
  let accounts: any[] = [];

  // Liste des bâtiments (si buildingId n'est pas fourni)
  let buildings: any[] = [];
  let selectedBuildingId = buildingId;

  // Calculated fields
  let vatAmount = 0;
  let amountInclVat = 0;

  // State
  let loading = false;
  let error = '';
  let isEditMode = false;

  const categories = [
    { value: 'Maintenance', label: $_('invoices.category_maintenance') },
    { value: 'Repairs', label: $_('invoices.category_repairs') },
    { value: 'Insurance', label: $_('invoices.category_insurance') },
    { value: 'Utilities', label: $_('invoices.category_utilities') },
    { value: 'Cleaning', label: $_('invoices.category_cleaning') },
    { value: 'Administration', label: $_('invoices.category_admin') },
    { value: 'Works', label: $_('invoices.category_works') },
    { value: 'Other', label: $_('invoices.category_other') }
  ];

  const vatRates = [
    { value: '0.00', label: $_('invoices.vat_0') },
    { value: '6.00', label: $_('invoices.vat_6') },
    { value: '12.00', label: $_('invoices.vat_12') },
    { value: '21.00', label: $_('invoices.vat_21') }
  ];

  onMount(async () => {
    invoiceDate = todayISO();
    dueDate = defaultDueDate();

    // Load buildings if no buildingId provided
    if (!buildingId || buildingId === '') {
      await loadBuildings();
    }

    // Load accounts list
    await loadAccounts();

    // Load invoice if editing
    if (invoiceId) {
      isEditMode = true;
      await loadInvoice();
    }
  });

  async function loadBuildings() {
    await withErrorHandling({
      action: async () => {
        const response = await api.get('/buildings');
        const data = Array.isArray(response) ? response : [];
        buildings = data;
        if (buildings.length > 0 && !selectedBuildingId) {
          selectedBuildingId = buildings[0].id;
        }
      },
      errorMessage: 'Failed to load buildings',
    });
  }

  async function loadAccounts() {
    await withErrorHandling({
      action: async () => {
        const response = await api.get('/accounts');
        const data = Array.isArray(response) ? response : [];
        accounts = data
          .filter((acc: any) => acc.code.startsWith('6'))
          .sort((a: any, b: any) => a.code.localeCompare(b.code));
      },
      errorMessage: 'Failed to load accounts',
    });
  }

  async function loadInvoice() {
    await withLoadingState({
      action: () => api.get(`/invoices/${invoiceId}`),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      errorMessage: $_('invoices.load_error'),
      onSuccess: (invoice: any) => {
        description = invoice.description;
        category = invoice.category;
        amountExclVat = invoice.amount_excl_vat?.toString() || '';
        vatRate = invoice.vat_rate?.toString() || '21.00';
        invoiceDate = invoice.invoice_date?.split('T')[0] || '';
        dueDate = invoice.due_date?.split('T')[0] || '';
        supplier = invoice.supplier || '';
        invoiceNumber = invoice.invoice_number || '';
        accountCode = invoice.account_code || '';
        recalculateVAT();
      },
    });
  }

  function recalculateVAT() {
    const ht = parseFloat(amountExclVat) || 0;
    const rate = parseFloat(vatRate) || 0;
    const result = calcVAT(ht, rate);
    vatAmount = result.vatAmount;
    amountInclVat = result.amountInclVat;
  }

  // Recalculate VAT when amount or rate changes
  $: {
    if (amountExclVat || vatRate) {
      recalculateVAT();
    }
  }

  function handleLineItemsChange(event: CustomEvent) {
    lineItems = event.detail;
  }

  function toggleMode() {
    if (mode === 'simple') {
      mode = 'detailed';
    } else {
      mode = 'simple';
      lineItems = [];
    }
  }

  async function handleSubmit() {
    loading = true;
    error = '';

    // Validation
    if (mode === 'simple') {
      if (!description.trim()) {
        error = $_('invoices.description_required');
        loading = false;
        return;
      }
      if (parseFloat(amountExclVat) <= 0) {
        error = $_('invoices.amount_required');
        loading = false;
        return;
      }
    } else {
      if (lineItems.length === 0) {
        error = $_('invoices.add_line_item');
        loading = false;
        return;
      }
      for (const item of lineItems) {
        if (!item.description.trim()) {
          error = $_('invoices.line_description_required');
          loading = false;
          return;
        }
      }
    }

    // Get organization_id from authStore if not provided
    const orgId = organizationId || $authStore.user?.activeRole?.organizationId || '';

    if (!orgId) {
      error = $_('common.org_id_missing');
      loading = false;
      return;
    }

    let dto: any = {
      organization_id: orgId,
      building_id: selectedBuildingId,
      description: mode === 'simple' ? description : lineItems.map(l => l.description).join(', '),
      category,
      expense_date: toISODateNoon(invoiceDate),
      supplier: supplier || null,
      invoice_number: invoiceNumber || null,
      account_code: accountCode || null
    };

    if (mode === 'simple') {
      const amountHT = parseFloat(amountExclVat);
      const vat = parseFloat(vatRate);
      const result = calcVAT(amountHT, vat);
      dto.amount = result.amountInclVat;
      dto.amount_excl_vat = amountHT;
      dto.vat_rate = vat;
    } else {
      const totals = aggregateLineItems(lineItems);
      dto.amount = totals.totalTTC;
      dto.amount_excl_vat = totals.totalHT;
      dto.vat_rate = totals.totalHT > 0 ? (totals.totalVAT / totals.totalHT) * 100 : 0;
      dto.line_items = lineItems.map(item => ({
        description: item.description,
        quantity: item.quantity,
        unit_price: item.unit_price,
        vat_rate: item.vat_rate
      }));
    }

    await withErrorHandling({
      action: async () => {
        if (isEditMode && invoiceId) {
          const updated = await api.put(`/expenses/${invoiceId}`, dto);
          if (onSaved) onSaved(updated);
        } else {
          const created = await api.post('/expenses', dto);
          if (onSaved) onSaved(created);
        }
      },
      setLoading: (v) => loading = v,
      errorMessage: $_('invoices.save_error'),
    });
  }
</script>

<div class="invoice-form">
  <div class="form-header">
    <h2>{isEditMode ? $_('invoices.edit') : $_('invoices.create')} {$_('invoices.invoice')}</h2>
    {#if !isEditMode}
      <button type="button" class="btn-mode-toggle" on:click={toggleMode} disabled={loading} data-testid="mode-toggle">
        {mode === 'simple' ? $_('invoices.detailed_mode') : $_('invoices.simple_mode')}
      </button>
    {/if}
  </div>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    {#if mode === 'simple'}
      <!-- Simple Mode: Single Amount -->
      <!-- Building Selector (if no buildingId provided) -->
      {#if (!buildingId || buildingId === '') && buildings.length > 0}
      <div class="form-group">
        <label for="buildingSelect">{$_('common.building')} *</label>
        <select id="buildingSelect" bind:value={selectedBuildingId} disabled={loading} required data-testid="building-select">
          <option value="">{$_('invoices.select_building')}</option>
          {#each buildings as building}
            <option value={building.id}>{building.name} - {building.address}</option>
          {/each}
        </select>
      </div>
      {/if}

      <!-- Description -->
      <div class="form-group">
        <label for="description">{$_('common.description')} *</label>
        <input
          type="text"
          id="description"
          bind:value={description}
          placeholder={$_('invoices.description_placeholder')}
          required
          disabled={loading}
          data-testid="description-input"
        />
      </div>

      <!-- Category -->
      <div class="form-group">
        <label for="category">{$_('common.category')}</label>
        <select id="category" bind:value={category} disabled={loading} data-testid="category-select">
          {#each categories as cat}
            <option value={cat.value}>{cat.label}</option>
          {/each}
        </select>
      </div>

      <!-- Account Code (PCMN) -->
      <div class="form-group">
        <label for="accountCode">{$_('invoices.account_code')}</label>
        <select id="accountCode" bind:value={accountCode} disabled={loading}>
          <option value="">{$_('invoices.select_account')}</option>
          {#each accounts as account}
            <option value={account.code}>
              {account.code} - {account.label}
            </option>
          {/each}
        </select>
        <small class="form-help">{$_('invoices.account_help')}</small>
      </div>

      <!-- Amount HT and VAT -->
      <div class="form-row">
        <div class="form-group">
          <label for="amountExclVat">{$_('invoices.amount_excl_vat')} *</label>
          <input
            type="number"
            id="amountExclVat"
            bind:value={amountExclVat}
            step="0.01"
            min="0.01"
            placeholder="1000.00"
            required
            disabled={loading}
            data-testid="amount-input"
          />
        </div>

        <div class="form-group">
          <label for="vatRate">{$_('invoices.vat_rate')}</label>
          <select id="vatRate" bind:value={vatRate} disabled={loading} data-testid="vat-rate-select">
            {#each vatRates as rate}
              <option value={rate.value}>{rate.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Calculated VAT -->
      <div class="calculated-amounts">
        <div class="amount-row">
          <span>{$_('invoices.amount_excl_vat')}:</span>
          <strong>{formatCurrency(parseFloat(amountExclVat || '0'))}</strong>
        </div>
        <div class="amount-row">
          <span>{$_('invoices.vat')} ({vatRate}%):</span>
          <strong>{formatCurrency(vatAmount)}</strong>
        </div>
        <div class="amount-row total">
          <span>{$_('invoices.amount_incl_vat')}:</span>
          <strong>{formatCurrency(amountInclVat)}</strong>
        </div>
      </div>
    {:else}
      <!-- Detailed Mode: Line Items -->
      <!-- Category -->
      <div class="form-group">
        <label for="category">{$_('common.category')}</label>
        <select id="category" bind:value={category} disabled={loading} data-testid="category-select">
          {#each categories as cat}
            <option value={cat.value}>{cat.label}</option>
          {/each}
        </select>
      </div>

      <!-- Account Code (PCMN) -->
      <div class="form-group">
        <label for="accountCode">{$_('invoices.account_code')}</label>
        <select id="accountCode" bind:value={accountCode} disabled={loading}>
          <option value="">{$_('invoices.select_account')}</option>
          {#each accounts as account}
            <option value={account.code}>
              {account.code} - {account.label}
            </option>
          {/each}
        </select>
        <small class="form-help">{$_('invoices.account_help')}</small>
      </div>

      <InvoiceLineItems
        bind:lineItems={lineItems}
        disabled={loading}
        on:change={handleLineItemsChange}
      />
    {/if}

    <!-- Dates -->
    <div class="form-row">
      <div class="form-group">
        <label for="invoiceDate">{$_('invoices.invoice_date')} *</label>
        <input
          type="date"
          id="invoiceDate"
          bind:value={invoiceDate}
          required
          disabled={loading}
          data-testid="invoice-date-input"
        />
      </div>

      <div class="form-group">
        <label for="dueDate">{$_('invoices.due_date')}</label>
        <input
          type="date"
          id="dueDate"
          bind:value={dueDate}
          disabled={loading}
          data-testid="due-date-input"
        />
      </div>
    </div>

    <!-- Supplier and Invoice Number -->
    <div class="form-row">
      <div class="form-group">
        <label for="supplier">{$_('invoices.supplier')}</label>
        <input
          type="text"
          id="supplier"
          bind:value={supplier}
          placeholder={$_('invoices.supplier_placeholder')}
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="invoiceNumber">{$_('invoices.invoice_number')}</label>
        <input
          type="text"
          id="invoiceNumber"
          bind:value={invoiceNumber}
          placeholder={$_('invoices.invoice_number_placeholder')}
          disabled={loading}
        />
      </div>
    </div>

    <!-- Actions -->
    <div class="form-actions">
      {#if onCancel}
        <button type="button" class="btn btn-secondary" on:click={onCancel} disabled={loading} data-testid="cancel-button">
          {$_('common.cancel')}
        </button>
      {/if}
      <button type="submit" class="btn btn-primary" disabled={loading} data-testid="submit-button">
        {#if loading}
          {$_('invoices.saving')}
        {:else}
          {isEditMode ? $_('invoices.update') : $_('invoices.create_draft')}
        {/if}
      </button>
    </div>
  </form>
</div>

<style>
  .invoice-form {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  h2 {
    margin: 0;
    color: #333;
  }

  .btn-mode-toggle {
    padding: 0.5rem 1rem;
    background: #f3f4f6;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: all 0.2s;
  }

  .btn-mode-toggle:hover:not(:disabled) {
    background: #e5e7eb;
    border-color: #9ca3af;
  }

  .btn-mode-toggle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .alert {
    padding: 1rem;
    margin-bottom: 1rem;
    border-radius: 4px;
  }

  .alert-error {
    background-color: #fee;
    border: 1px solid #fcc;
    color: #c33;
  }

  .form-group {
    margin-bottom: 1rem;
    flex: 1;
  }

  .form-row {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 500;
    color: #555;
  }

  input,
  select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #4a90e2;
    box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.1);
  }

  input:disabled,
  select:disabled {
    background-color: #f5f5f5;
    cursor: not-allowed;
  }

  .calculated-amounts {
    background-color: #f9f9f9;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1.5rem;
  }

  .amount-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid #eee;
  }

  .amount-row:last-child {
    border-bottom: none;
  }

  .amount-row.total {
    font-size: 1.1rem;
    margin-top: 0.5rem;
    padding-top: 0.75rem;
    border-top: 2px solid #ddd;
    border-bottom: none;
    color: #333;
  }

  .form-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 2rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .form-help {
    display: block;
    margin-top: 0.25rem;
    font-size: 0.875rem;
    color: #6b7280;
    font-style: italic;
  }

  .btn-primary {
    background-color: #4a90e2;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #357abd;
  }

  .btn-secondary {
    background-color: #e0e0e0;
    color: #333;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #d0d0d0;
  }
</style>
