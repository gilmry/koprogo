<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  export let buildingId: string = '';
  export let invoiceId: string | null = null; // null for create, UUID for edit
  export let onSaved: ((invoice: any) => void) | null = null;
  export let onCancel: (() => void) | null = null;

  // Form fields
  let description = '';
  let category = 'Maintenance';
  let amountExclVat = '';
  let vatRate = '21.00';
  let invoiceDate = '';
  let dueDate = '';
  let supplier = '';
  let invoiceNumber = '';

  // Calculated fields
  let vatAmount = 0;
  let amountInclVat = 0;

  // State
  let loading = false;
  let error = '';
  let isEditMode = false;

  const categories = [
    { value: 'Maintenance', label: 'Entretien' },
    { value: 'Repairs', label: 'Réparations' },
    { value: 'Insurance', label: 'Assurance' },
    { value: 'Utilities', label: 'Charges courantes' },
    { value: 'Cleaning', label: 'Nettoyage' },
    { value: 'Administration', label: 'Administration' },
    { value: 'Works', label: 'Travaux' },
    { value: 'Other', label: 'Autre' }
  ];

  const vatRates = [
    { value: '0.00', label: '0% (Exonéré)' },
    { value: '6.00', label: '6% (Taux réduit)' },
    { value: '12.00', label: '12% (Taux parking)' },
    { value: '21.00', label: '21% (Taux normal)' }
  ];

  onMount(async () => {
    // Set default dates
    const today = new Date().toISOString().split('T')[0];
    invoiceDate = today;

    const nextMonth = new Date();
    nextMonth.setMonth(nextMonth.getMonth() + 1);
    dueDate = nextMonth.toISOString().split('T')[0];

    // Load invoice if editing
    if (invoiceId) {
      isEditMode = true;
      await loadInvoice();
    }
  });

  async function loadInvoice() {
    try {
      loading = true;
      error = '';
      const invoice = await api.get(`/invoices/${invoiceId}`);

      // Populate form
      description = invoice.description;
      category = invoice.category;
      amountExclVat = invoice.amount_excl_vat?.toString() || '';
      vatRate = invoice.vat_rate?.toString() || '21.00';
      invoiceDate = invoice.invoice_date?.split('T')[0] || '';
      dueDate = invoice.due_date?.split('T')[0] || '';
      supplier = invoice.supplier || '';
      invoiceNumber = invoice.invoice_number || '';

      calculateVAT();
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement de la facture';
    } finally {
      loading = false;
    }
  }

  function calculateVAT() {
    const ht = parseFloat(amountExclVat) || 0;
    const rate = parseFloat(vatRate) || 0;

    vatAmount = Math.round((ht * rate) / 100 * 100) / 100;
    amountInclVat = Math.round((ht + vatAmount) * 100) / 100;
  }

  // Recalculate VAT when amount or rate changes
  $: {
    if (amountExclVat || vatRate) {
      calculateVAT();
    }
  }

  async function handleSubmit() {
    try {
      loading = true;
      error = '';

      // Validation
      if (!description.trim()) {
        error = 'La description est requise';
        return;
      }
      if (parseFloat(amountExclVat) <= 0) {
        error = 'Le montant doit être supérieur à 0';
        return;
      }

      if (isEditMode && invoiceId) {
        // Update existing invoice
        const dto = {
          description,
          category,
          amount_excl_vat: parseFloat(amountExclVat),
          vat_rate: parseFloat(vatRate),
          invoice_date: `${invoiceDate}T12:00:00Z`,
          due_date: dueDate ? `${dueDate}T12:00:00Z` : null,
          supplier: supplier || null,
          invoice_number: invoiceNumber || null
        };

        const updated = await api.put(`/invoices/${invoiceId}`, dto);
        if (onSaved) onSaved(updated);
      } else {
        // Create new invoice draft
        const dto = {
          building_id: buildingId,
          description,
          category,
          amount_excl_vat: parseFloat(amountExclVat),
          vat_rate: parseFloat(vatRate),
          invoice_date: `${invoiceDate}T12:00:00Z`,
          due_date: dueDate ? `${dueDate}T12:00:00Z` : null,
          supplier: supplier || null,
          invoice_number: invoiceNumber || null
        };

        const created = await api.post('/invoices/draft', dto);
        if (onSaved) onSaved(created);
      }
    } catch (err: any) {
      error = err.message || 'Erreur lors de l\'enregistrement';
    } finally {
      loading = false;
    }
  }
</script>

<div class="invoice-form">
  <h2>{isEditMode ? 'Modifier' : 'Créer'} une facture</h2>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <!-- Description -->
    <div class="form-group">
      <label for="description">Description *</label>
      <input
        type="text"
        id="description"
        bind:value={description}
        placeholder="Ex: Réparation ascenseur"
        required
        disabled={loading}
      />
    </div>

    <!-- Category -->
    <div class="form-group">
      <label for="category">Catégorie</label>
      <select id="category" bind:value={category} disabled={loading}>
        {#each categories as cat}
          <option value={cat.value}>{cat.label}</option>
        {/each}
      </select>
    </div>

    <!-- Amount HT and VAT -->
    <div class="form-row">
      <div class="form-group">
        <label for="amountExclVat">Montant HT (€) *</label>
        <input
          type="number"
          id="amountExclVat"
          bind:value={amountExclVat}
          step="0.01"
          min="0.01"
          placeholder="1000.00"
          required
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="vatRate">Taux TVA</label>
        <select id="vatRate" bind:value={vatRate} disabled={loading}>
          {#each vatRates as rate}
            <option value={rate.value}>{rate.label}</option>
          {/each}
        </select>
      </div>
    </div>

    <!-- Calculated VAT -->
    <div class="calculated-amounts">
      <div class="amount-row">
        <span>Montant HT:</span>
        <strong>{parseFloat(amountExclVat || '0').toFixed(2)} €</strong>
      </div>
      <div class="amount-row">
        <span>TVA ({vatRate}%):</span>
        <strong>{vatAmount.toFixed(2)} €</strong>
      </div>
      <div class="amount-row total">
        <span>Montant TTC:</span>
        <strong>{amountInclVat.toFixed(2)} €</strong>
      </div>
    </div>

    <!-- Dates -->
    <div class="form-row">
      <div class="form-group">
        <label for="invoiceDate">Date facture *</label>
        <input
          type="date"
          id="invoiceDate"
          bind:value={invoiceDate}
          required
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="dueDate">Date d'échéance</label>
        <input
          type="date"
          id="dueDate"
          bind:value={dueDate}
          disabled={loading}
        />
      </div>
    </div>

    <!-- Supplier and Invoice Number -->
    <div class="form-row">
      <div class="form-group">
        <label for="supplier">Fournisseur</label>
        <input
          type="text"
          id="supplier"
          bind:value={supplier}
          placeholder="ACME Elevators SPRL"
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="invoiceNumber">N° facture</label>
        <input
          type="text"
          id="invoiceNumber"
          bind:value={invoiceNumber}
          placeholder="INV-2025-042"
          disabled={loading}
        />
      </div>
    </div>

    <!-- Actions -->
    <div class="form-actions">
      {#if onCancel}
        <button type="button" class="btn btn-secondary" on:click={onCancel} disabled={loading}>
          Annuler
        </button>
      {/if}
      <button type="submit" class="btn btn-primary" disabled={loading}>
        {#if loading}
          Enregistrement...
        {:else}
          {isEditMode ? 'Mettre à jour' : 'Créer brouillon'}
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

  h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    color: #333;
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
