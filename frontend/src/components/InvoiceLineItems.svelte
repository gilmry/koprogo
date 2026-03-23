<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../lib/i18n';

  export let lineItems: LineItem[] = [];
  export let disabled = false;

  const dispatch = createEventDispatcher();

  interface LineItem {
    id?: string;
    description: string;
    quantity: number;
    unit_price: number;
    vat_rate: number;
    amount_excl_vat: number;
    vat_amount: number;
    amount_incl_vat: number;
  }

  const vatRates = [
    { value: 0, label: $_('invoices.vat_0') },
    { value: 6, label: $_('invoices.vat_6') },
    { value: 12, label: $_('invoices.vat_12') },
    { value: 21, label: $_('invoices.vat_21') }
  ];

  function createEmptyLine(): LineItem {
    return {
      description: '',
      quantity: 1,
      unit_price: 0,
      vat_rate: 21,
      amount_excl_vat: 0,
      vat_amount: 0,
      amount_incl_vat: 0
    };
  }

  function addLine() {
    lineItems = [...lineItems, createEmptyLine()];
    notifyChange();
  }

  function removeLine(index: number) {
    lineItems = lineItems.filter((_, i) => i !== index);
    notifyChange();
  }

  function calculateLine(line: LineItem) {
    line.amount_excl_vat = Math.round(line.quantity * line.unit_price * 100) / 100;
    line.vat_amount = Math.round(line.amount_excl_vat * line.vat_rate) / 100;
    line.amount_incl_vat = Math.round((line.amount_excl_vat + line.vat_amount) * 100) / 100;
  }

  function handleLineChange(index: number) {
    calculateLine(lineItems[index]);
    lineItems = lineItems; // Trigger reactivity
    notifyChange();
  }

  function notifyChange() {
    dispatch('change', lineItems);
  }

  // Calculate totals
  $: totalExclVat = lineItems.reduce((sum, item) => sum + item.amount_excl_vat, 0);
  $: totalVat = lineItems.reduce((sum, item) => sum + item.vat_amount, 0);
  $: totalInclVat = lineItems.reduce((sum, item) => sum + item.amount_incl_vat, 0);

  // Initialize with one line if empty
  $: if (lineItems.length === 0 && !disabled) {
    lineItems = [createEmptyLine()];
    notifyChange();
  }

  function formatCurrency(amount: number): string {
    return amount.toFixed(2);
  }
</script>

<div class="line-items-container">
  <div class="header">
    <h3>{$_('invoices.line_items_title')}</h3>
    <button
      type="button"
      class="btn-add"
      on:click={addLine}
      disabled={disabled}
    >
      {$_('invoices.add_line')}
    </button>
  </div>

  <div class="lines-list">
    {#each lineItems as line, index}
      <div class="line-item">
        <div class="line-number">{index + 1}</div>

        <div class="line-content">
          <!-- Description -->
          <div class="form-group full-width">
            <label for="desc-{index}">{$_('common.description')} *</label>
            <input
              type="text"
              id="desc-{index}"
              bind:value={line.description}
              on:input={() => handleLineChange(index)}
              placeholder={$_('invoices.line_description_placeholder')}
              disabled={disabled}
              required
            />
          </div>

          <div class="form-row">
            <!-- Quantity -->
            <div class="form-group">
              <label for="qty-{index}">{$_('invoices.quantity')} *</label>
              <input
                type="number"
                id="qty-{index}"
                bind:value={line.quantity}
                on:input={() => handleLineChange(index)}
                step="0.01"
                min="0.01"
                disabled={disabled}
                required
              />
            </div>

            <!-- Unit Price -->
            <div class="form-group">
              <label for="price-{index}">{$_('invoices.unit_price')} *</label>
              <input
                type="number"
                id="price-{index}"
                bind:value={line.unit_price}
                on:input={() => handleLineChange(index)}
                step="0.01"
                min="0"
                disabled={disabled}
                required
              />
            </div>

            <!-- VAT Rate -->
            <div class="form-group">
              <label for="vat-{index}">{$_('invoices.vat')}</label>
              <select
                id="vat-{index}"
                bind:value={line.vat_rate}
                on:change={() => handleLineChange(index)}
                disabled={disabled}
              >
                {#each vatRates as rate}
                  <option value={rate.value}>{rate.label}</option>
                {/each}
              </select>
            </div>

            <!-- Calculated Amounts -->
            <div class="form-group calculated">
              <label>{$_('invoices.total_excl')}</label>
              <div class="amount">{formatCurrency(line.amount_excl_vat)} €</div>
            </div>

            <div class="form-group calculated">
              <label>{$_('invoices.vat')}</label>
              <div class="amount">{formatCurrency(line.vat_amount)} €</div>
            </div>

            <div class="form-group calculated total">
              <label>{$_('invoices.total_incl')}</label>
              <div class="amount">{formatCurrency(line.amount_incl_vat)} €</div>
            </div>
          </div>
        </div>

        <!-- Remove Button -->
        {#if lineItems.length > 1 && !disabled}
          <button
            type="button"
            class="btn-remove"
            on:click={() => removeLine(index)}
            title={$_('invoices.remove_line')}
          >
            ✕
          </button>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Grand Total -->
  {#if lineItems.length > 0}
    <div class="grand-total">
      <div class="total-row">
        <span>{$_('invoices.total_excl')}:</span>
        <strong>{formatCurrency(totalExclVat)} €</strong>
      </div>
      <div class="total-row">
        <span>{$_('invoices.total_vat')}:</span>
        <strong>{formatCurrency(totalVat)} €</strong>
      </div>
      <div class="total-row grand">
        <span>{$_('invoices.total_incl')}:</span>
        <strong>{formatCurrency(totalInclVat)} €</strong>
      </div>
    </div>
  {/if}
</div>

<style>
  .line-items-container {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    border: 2px solid #e5e7eb;
    margin-bottom: 1.5rem;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .header h3 {
    margin: 0;
    color: #374151;
    font-size: 1.25rem;
  }

  .btn-add {
    padding: 0.5rem 1rem;
    background-color: #059669;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .btn-add:hover:not(:disabled) {
    background-color: #059669;
  }

  .btn-add:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .lines-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .line-item {
    position: relative;
    display: flex;
    gap: 1rem;
    padding: 1.5rem;
    background: #f9fafb;
    border-radius: 8px;
    border: 1px solid #e5e7eb;
  }

  .line-number {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2rem;
    height: 2rem;
    background: #3b82f6;
    color: white;
    border-radius: 50%;
    font-weight: bold;
    font-size: 0.875rem;
    flex-shrink: 0;
  }

  .line-content {
    flex: 1;
  }

  .form-group {
    margin-bottom: 0;
  }

  .form-group.full-width {
    width: 100%;
    margin-bottom: 1rem;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1.5fr 1fr 1fr 1fr 1.2fr;
    gap: 0.75rem;
    align-items: end;
  }

  label {
    display: block;
    margin-bottom: 0.25rem;
    font-weight: 500;
    color: #6b7280;
    font-size: 0.875rem;
  }

  input,
  select {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 0.875rem;
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  input:disabled,
  select:disabled {
    background-color: #f3f4f6;
    cursor: not-allowed;
  }

  .form-group.calculated {
    display: flex;
    flex-direction: column;
  }

  .amount {
    padding: 0.5rem;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    text-align: right;
    font-weight: 500;
    color: #374151;
    font-size: 0.875rem;
  }

  .form-group.total .amount {
    background: #eff6ff;
    border-color: #3b82f6;
    color: #1e40af;
    font-weight: 600;
  }

  .btn-remove {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    width: 2rem;
    height: 2rem;
    background: #ef4444;
    color: white;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1rem;
    line-height: 1;
    transition: background-color 0.2s;
  }

  .btn-remove:hover {
    background: #dc2626;
  }

  .grand-total {
    margin-top: 1.5rem;
    padding: 1rem;
    background: #f3f4f6;
    border-radius: 8px;
    border: 2px solid #d1d5db;
  }

  .total-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    font-size: 0.95rem;
    color: #4b5563;
  }

  .total-row.grand {
    margin-top: 0.5rem;
    padding-top: 0.75rem;
    border-top: 2px solid #9ca3af;
    font-size: 1.25rem;
    color: #1f2937;
  }

  .total-row strong {
    font-weight: 700;
  }

  .total-row.grand strong {
    color: #3b82f6;
  }

  @media (max-width: 1024px) {
    .form-row {
      grid-template-columns: 1fr 1fr;
      gap: 0.5rem;
    }

    .form-group.full-width {
      grid-column: 1 / -1;
    }
  }
</style>
