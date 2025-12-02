<script lang="ts">
  // Journal Entry Form Component (Noalyss-inspired)
  //
  // CREDITS & ATTRIBUTION:
  // This UI is inspired by the Noalyss project (https://gitlab.com/noalyss/noalyss)
  // Noalyss is a free accounting software for Belgian and French accounting
  // License: GPL-2.0-or-later (GNU General Public License version 2 or later)
  // Copyright: Dany De Bontridder <dany@alchimerys.eu>
  //
  // Noalyss-inspired features:
  // - Journal type selector (ACH, VEN, FIN, ODS)
  // - Debit/Credit column layout
  // - Real-time balance validation
  // - Multi-line entry with dynamic addition
  // - Quick account code entry

  import { onMount } from 'svelte';

  const API_URL = import.meta.env.PUBLIC_API_URL || 'http://localhost:8080/api/v1';

  // Props
  export let buildingId: string | null = null;
  export let onSuccess: (() => void) | null = null;

  // Journal types (Noalyss-inspired)
  const journalTypes = [
    { code: 'ACH', label: 'ACH - Achats (Purchases)', description: 'Factures fournisseurs' },
    { code: 'VEN', label: 'VEN - Ventes (Sales)', description: 'Factures clients' },
    { code: 'FIN', label: 'FIN - Financier (Financial)', description: 'Opérations bancaires' },
    { code: 'ODS', label: 'ODS - Opérations Diverses (Miscellaneous)', description: 'Autres opérations' },
  ];

  // Form state
  let journalType = 'ODS';
  let entryDate = new Date().toISOString().split('T')[0];
  let description = '';
  let documentRef = '';
  let lines: Array<{
    accountCode: string;
    debit: string;
    credit: string;
    description: string;
  }> = [
    { accountCode: '', debit: '', credit: '', description: '' },
    { accountCode: '', debit: '', credit: '', description: '' },
  ];

  let loading = false;
  let error = '';
  let success = false;

  // Calculated balances
  $: totalDebits = lines.reduce((sum, line) => sum + (parseFloat(line.debit) || 0), 0);
  $: totalCredits = lines.reduce((sum, line) => sum + (parseFloat(line.credit) || 0), 0);
  $: difference = Math.abs(totalDebits - totalCredits);
  $: isBalanced = difference < 0.01 && totalDebits > 0;

  function addLine() {
    lines = [...lines, { accountCode: '', debit: '', credit: '', description: '' }];
  }

  function removeLine(index: number) {
    if (lines.length > 2) {
      lines = lines.filter((_, i) => i !== index);
    }
  }

  async function handleSubmit() {
    error = '';
    success = false;

    // Validation
    if (!description.trim()) {
      error = 'La description est requise';
      return;
    }

    const validLines = lines.filter(
      (line) => line.accountCode.trim() && (parseFloat(line.debit) > 0 || parseFloat(line.credit) > 0)
    );

    if (validLines.length < 2) {
      error = 'Au moins 2 lignes avec montants sont requises';
      return;
    }

    if (!isBalanced) {
      error = `Écriture déséquilibrée: débits=${totalDebits.toFixed(2)}€, crédits=${totalCredits.toFixed(2)}€`;
      return;
    }

    loading = true;

    try {
      const token = localStorage.getItem('token');
      const response = await fetch(`${API_URL}/journal-entries`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          building_id: buildingId,
          journal_type: journalType,
          entry_date: `${entryDate}T12:00:00Z`,
          description: description.trim(),
          document_ref: documentRef.trim() || null,
          lines: validLines.map((line) => ({
            account_code: line.accountCode.trim(),
            debit: parseFloat(line.debit) || 0,
            credit: parseFloat(line.credit) || 0,
            description: line.description.trim(),
          })),
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Échec de la création de l\'écriture comptable');
      }

      success = true;
      resetForm();
      if (onSuccess) onSuccess();
    } catch (err: any) {
      error = err.message;
    } finally {
      loading = false;
    }
  }

  function resetForm() {
    journalType = 'ODS';
    entryDate = new Date().toISOString().split('T')[0];
    description = '';
    documentRef = '';
    lines = [
      { accountCode: '', debit: '', credit: '', description: '' },
      { accountCode: '', debit: '', credit: '', description: '' },
    ];
  }

  function handleDebitInput(index: number) {
    if (lines[index].debit) {
      lines[index].credit = ''; // Clear credit if debit is entered
    }
  }

  function handleCreditInput(index: number) {
    if (lines[index].credit) {
      lines[index].debit = ''; // Clear debit if credit is entered
    }
  }
</script>

<div class="journal-entry-form">
  <div class="form-header">
    <h3>🧾 Nouvelle Écriture Comptable</h3>
    <p class="text-sm text-gray-600">Interface inspirée de Noalyss - Saisie en partie double</p>
  </div>

  {#if error}
    <div class="alert alert-error">
      ❌ {error}
    </div>
  {/if}

  {#if success}
    <div class="alert alert-success">
      ✅ Écriture comptable créée avec succès!
    </div>
  {/if}

  <form on:submit|preventDefault={handleSubmit}>
    <!-- Header section -->
    <div class="form-section">
      <div class="grid grid-cols-2 gap-4">
        <div class="form-group">
          <label for="journal-type">Type de Journal *</label>
          <select id="journal-type" bind:value={journalType} class="form-control" required>
            {#each journalTypes as type}
              <option value={type.code}>
                {type.label}
              </option>
            {/each}
          </select>
          <small class="text-gray-600">
            {journalTypes.find((t) => t.code === journalType)?.description}
          </small>
        </div>

        <div class="form-group">
          <label for="entry-date">Date d'Opération *</label>
          <input
            type="date"
            id="entry-date"
            bind:value={entryDate}
            class="form-control"
            required
          />
        </div>
      </div>

      <div class="form-group">
        <label for="description">Description *</label>
        <input
          type="text"
          id="description"
          bind:value={description}
          class="form-control"
          placeholder="Ex: Facture eau janvier 2025"
          required
        />
      </div>

      <div class="form-group">
        <label for="document-ref">Référence Document</label>
        <input
          type="text"
          id="document-ref"
          bind:value={documentRef}
          class="form-control"
          placeholder="Ex: FA-2025-001"
        />
      </div>
    </div>

    <!-- Lines section (Noalyss-inspired layout) -->
    <div class="form-section">
      <div class="flex justify-between items-center mb-3">
        <h4 class="text-lg font-semibold">Lignes Comptables</h4>
        <button type="button" class="btn btn-secondary btn-sm" on:click={addLine}>
          ➕ Ajouter une ligne
        </button>
      </div>

      <div class="journal-lines-container">
        <table class="journal-lines-table">
          <thead>
            <tr>
              <th class="w-32">Compte</th>
              <th class="flex-1">Libellé</th>
              <th class="w-32 text-right">Débit (€)</th>
              <th class="w-32 text-right">Crédit (€)</th>
              <th class="w-20"></th>
            </tr>
          </thead>
          <tbody>
            {#each lines as line, index}
              <tr class="journal-line">
                <td>
                  <input
                    type="text"
                    bind:value={line.accountCode}
                    class="form-control form-control-sm"
                    placeholder="Ex: 6100"
                    maxlength="10"
                  />
                </td>
                <td>
                  <input
                    type="text"
                    bind:value={line.description}
                    class="form-control form-control-sm"
                    placeholder="Libellé de la ligne"
                  />
                </td>
                <td>
                  <input
                    type="number"
                    bind:value={line.debit}
                    on:input={() => handleDebitInput(index)}
                    class="form-control form-control-sm text-right"
                    placeholder="0.00"
                    step="0.01"
                    min="0"
                  />
                </td>
                <td>
                  <input
                    type="number"
                    bind:value={line.credit}
                    on:input={() => handleCreditInput(index)}
                    class="form-control form-control-sm text-right"
                    placeholder="0.00"
                    step="0.01"
                    min="0"
                  />
                </td>
                <td class="text-center">
                  {#if lines.length > 2}
                    <button
                      type="button"
                      class="btn-icon-danger"
                      on:click={() => removeLine(index)}
                      title="Supprimer cette ligne"
                    >
                      🗑️
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
          <tfoot>
            <tr class="totals-row">
              <td colspan="2" class="text-right font-bold">Totaux:</td>
              <td class="text-right font-bold text-blue-600">
                {totalDebits.toFixed(2)} €
              </td>
              <td class="text-right font-bold text-green-600">
                {totalCredits.toFixed(2)} €
              </td>
              <td></td>
            </tr>
            <tr class="balance-row">
              <td colspan="2" class="text-right font-bold">
                {#if isBalanced}
                  ✅ Équilibrée
                {:else if difference > 0}
                  ⚠️ Différence:
                {:else}
                  ℹ️ En attente
                {/if}
              </td>
              <td colspan="2" class="text-right font-bold" class:text-red-600={!isBalanced && totalDebits > 0}>
                {#if !isBalanced && totalDebits > 0}
                  {difference.toFixed(2)} €
                {/if}
              </td>
              <td></td>
            </tr>
          </tfoot>
        </table>
      </div>
    </div>

    <!-- Actions -->
    <div class="form-actions">
      <button type="button" class="btn btn-secondary" on:click={resetForm} disabled={loading}>
        🔄 Réinitialiser
      </button>
      <button type="submit" class="btn btn-primary" disabled={loading || !isBalanced}>
        {#if loading}
          ⏳ Création en cours...
        {:else}
          💾 Créer l'Écriture
        {/if}
      </button>
    </div>
  </form>
</div>

<style>
  .journal-entry-form {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .form-header {
    margin-bottom: 1.5rem;
    border-bottom: 2px solid #e5e7eb;
    padding-bottom: 1rem;
  }

  .form-header h3 {
    margin: 0 0 0.5rem 0;
    color: #1f2937;
    font-size: 1.5rem;
  }

  .form-section {
    margin-bottom: 2rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 600;
    color: #374151;
    font-size: 0.875rem;
  }

  .form-control {
    width: 100%;
    padding: 0.5rem 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    font-size: 1rem;
  }

  .form-control:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .form-control-sm {
    padding: 0.25rem 0.5rem;
    font-size: 0.875rem;
  }

  .journal-lines-container {
    overflow-x: auto;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
  }

  .journal-lines-table {
    width: 100%;
    border-collapse: collapse;
  }

  .journal-lines-table thead {
    background: #f9fafb;
    border-bottom: 2px solid #e5e7eb;
  }

  .journal-lines-table th {
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    color: #374151;
    font-size: 0.875rem;
  }

  .journal-lines-table td {
    padding: 0.5rem;
    border-bottom: 1px solid #f3f4f6;
  }

  .journal-line:hover {
    background: #f9fafb;
  }

  .totals-row {
    background: #f0f9ff;
    border-top: 2px solid #3b82f6;
  }

  .totals-row td {
    padding: 0.75rem;
    font-size: 1rem;
  }

  .balance-row {
    background: #ecfdf5;
  }

  .balance-row td {
    padding: 0.75rem;
    font-size: 1rem;
  }

  .btn-icon-danger {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1.25rem;
    padding: 0.25rem;
  }

  .btn-icon-danger:hover {
    opacity: 0.7;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #e5e7eb;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 4px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
    border: none;
    font-size: 1rem;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background: #3b82f6;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-secondary {
    background: #f3f4f6;
    color: #374151;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #e5e7eb;
  }

  .btn-sm {
    padding: 0.375rem 0.75rem;
    font-size: 0.875rem;
  }

  .alert {
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .alert-error {
    background: #fee2e2;
    color: #991b1b;
    border: 1px solid #fecaca;
  }

  .alert-success {
    background: #d1fae5;
    color: #065f46;
    border: 1px solid #a7f3d0;
  }

  .grid {
    display: grid;
  }

  .grid-cols-2 {
    grid-template-columns: repeat(2, 1fr);
  }

  .gap-4 {
    gap: 1rem;
  }

  .text-sm {
    font-size: 0.875rem;
  }

  .text-lg {
    font-size: 1.125rem;
  }

  .text-right {
    text-align: right;
  }

  .text-center {
    text-align: center;
  }

  .text-gray-600 {
    color: #4b5563;
  }

  .text-blue-600 {
    color: #2563eb;
  }

  .text-green-600 {
    color: #059669;
  }

  .text-red-600 {
    color: #dc2626;
  }

  .font-semibold {
    font-weight: 600;
  }

  .font-bold {
    font-weight: 700;
  }

  .flex {
    display: flex;
  }

  .flex-1 {
    flex: 1;
  }

  .justify-between {
    justify-content: space-between;
  }

  .items-center {
    align-items: center;
  }

  .mb-3 {
    margin-bottom: 0.75rem;
  }

  .w-20 {
    width: 5rem;
  }

  .w-32 {
    width: 8rem;
  }
</style>
