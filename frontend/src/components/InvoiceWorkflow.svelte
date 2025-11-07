<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  export let invoiceId: string;
  export let onStatusChanged: ((invoice: any) => void) | null = null;

  // Invoice data
  let invoice: any = null;
  let loading = false;
  let error = '';

  // Rejection modal
  let showRejectModal = false;
  let rejectionReason = '';

  onMount(async () => {
    await loadInvoice();
  });

  async function loadInvoice() {
    try {
      loading = true;
      error = '';
      invoice = await api.get(`/invoices/${invoiceId}`);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement';
    } finally {
      loading = false;
    }
  }

  async function submitForApproval() {
    if (!confirm('Soumettre cette facture pour validation ?')) {
      return;
    }

    try {
      loading = true;
      error = '';
      const updated = await api.put(`/invoices/${invoiceId}/submit`, {});
      invoice = updated;
      if (onStatusChanged) onStatusChanged(updated);
    } catch (err: any) {
      error = err.message || 'Erreur lors de la soumission';
    } finally {
      loading = false;
    }
  }

  async function approve() {
    if (!confirm('Approuver cette facture ?')) {
      return;
    }

    try {
      loading = true;
      error = '';
      const updated = await api.put(`/invoices/${invoiceId}/approve`, {});
      invoice = updated;
      if (onStatusChanged) onStatusChanged(updated);
    } catch (err: any) {
      error = err.message || 'Erreur lors de l\'approbation';
    } finally {
      loading = false;
    }
  }

  function openRejectModal() {
    showRejectModal = true;
    rejectionReason = '';
  }

  function closeRejectModal() {
    showRejectModal = false;
    rejectionReason = '';
  }

  async function confirmReject() {
    if (!rejectionReason.trim()) {
      alert('Veuillez fournir une raison de rejet');
      return;
    }

    try {
      loading = true;
      error = '';
      const updated = await api.put(`/invoices/${invoiceId}/reject`, {
        rejection_reason: rejectionReason
      });
      invoice = updated;
      closeRejectModal();
      if (onStatusChanged) onStatusChanged(updated);
    } catch (err: any) {
      error = err.message || 'Erreur lors du rejet';
    } finally {
      loading = false;
    }
  }

  function getStatusBadgeClass(status: string): string {
    const s = status.toLowerCase();
    if (s.includes('draft')) return 'badge-draft';
    if (s.includes('pending')) return 'badge-pending';
    if (s.includes('approved')) return 'badge-approved';
    if (s.includes('rejected')) return 'badge-rejected';
    return '';
  }

  function getStatusLabel(status: string): string {
    const s = status.toLowerCase();
    if (s.includes('draft')) return 'Brouillon';
    if (s.includes('pending')) return 'En attente';
    if (s.includes('approved')) return 'Approuvée';
    if (s.includes('rejected')) return 'Rejetée';
    return status;
  }

  function canBeModified(status: string): boolean {
    const s = status.toLowerCase();
    return s.includes('draft') || s.includes('rejected');
  }

  function canBeSubmitted(status: string): boolean {
    const s = status.toLowerCase();
    return s.includes('draft') || s.includes('rejected');
  }

  function canBeApproved(status: string): boolean {
    const s = status.toLowerCase();
    return s.includes('pending');
  }
</script>

<div class="invoice-workflow">
  {#if loading && !invoice}
    <p>Chargement...</p>
  {:else if error && !invoice}
    <div class="alert alert-error">{error}</div>
  {:else if invoice}
    <!-- Status Badge -->
    <div class="status-section">
      <span class="badge {getStatusBadgeClass(invoice.approval_status)}">
        {getStatusLabel(invoice.approval_status)}
      </span>
    </div>

    <!-- Invoice Details -->
    <div class="invoice-details">
      <h3>{invoice.description}</h3>
      <div class="detail-row">
        <span>Montant HT:</span>
        <strong>{invoice.amount_excl_vat?.toFixed(2) || '0.00'} €</strong>
      </div>
      <div class="detail-row">
        <span>TVA ({invoice.vat_rate}%):</span>
        <strong>{invoice.vat_amount?.toFixed(2) || '0.00'} €</strong>
      </div>
      <div class="detail-row total">
        <span>Montant TTC:</span>
        <strong>{invoice.amount_incl_vat?.toFixed(2) || invoice.amount?.toFixed(2) || '0.00'} €</strong>
      </div>
      {#if invoice.invoice_date}
        <div class="detail-row">
          <span>Date facture:</span>
          <span>{new Date(invoice.invoice_date).toLocaleDateString('fr-BE')}</span>
        </div>
      {/if}
      {#if invoice.supplier}
        <div class="detail-row">
          <span>Fournisseur:</span>
          <span>{invoice.supplier}</span>
        </div>
      {/if}
    </div>

    <!-- Workflow Info -->
    {#if invoice.submitted_at}
      <div class="workflow-info">
        <p>
          <strong>Soumis le:</strong>
          {new Date(invoice.submitted_at).toLocaleString('fr-BE')}
        </p>
      </div>
    {/if}

    {#if invoice.approved_at}
      <div class="workflow-info">
        <p>
          <strong>{invoice.approval_status?.toLowerCase().includes('approved') ? 'Approuvé' : 'Rejeté'} le:</strong>
          {new Date(invoice.approved_at).toLocaleString('fr-BE')}
        </p>
      </div>
    {/if}

    {#if invoice.rejection_reason}
      <div class="alert alert-warning">
        <strong>Raison du rejet:</strong>
        <p>{invoice.rejection_reason}</p>
      </div>
    {/if}

    <!-- Error Display -->
    {#if error}
      <div class="alert alert-error">{error}</div>
    {/if}

    <!-- Workflow Actions -->
    <div class="workflow-actions">
      {#if canBeModified(invoice.approval_status)}
        <div class="info-box">
          ℹ️ Cette facture peut encore être modifiée
        </div>
      {/if}

      {#if canBeSubmitted(invoice.approval_status)}
        <button
          class="btn btn-primary"
          on:click={submitForApproval}
          disabled={loading}
        >
          📤 Soumettre pour validation
        </button>
      {/if}

      {#if canBeApproved(invoice.approval_status)}
        <div class="approval-actions">
          <button
            class="btn btn-success"
            on:click={approve}
            disabled={loading}
          >
            ✓ Approuver
          </button>
          <button
            class="btn btn-danger"
            on:click={openRejectModal}
            disabled={loading}
          >
            ✗ Rejeter
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Reject Modal -->
{#if showRejectModal}
  <div class="modal-overlay" on:click={closeRejectModal}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Rejeter la facture</h3>
      <p>Veuillez indiquer la raison du rejet:</p>

      <textarea
        bind:value={rejectionReason}
        placeholder="Ex: Montant incorrect, devis non respecté, facture en double..."
        rows="4"
        disabled={loading}
      ></textarea>

      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={closeRejectModal} disabled={loading}>
          Annuler
        </button>
        <button class="btn btn-danger" on:click={confirmReject} disabled={loading || !rejectionReason.trim()}>
          {#if loading}
            Rejet en cours...
          {:else}
            Confirmer le rejet
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .invoice-workflow {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .status-section {
    margin-bottom: 1.5rem;
  }

  .badge {
    display: inline-block;
    padding: 0.5rem 1rem;
    border-radius: 20px;
    font-weight: 600;
    font-size: 0.9rem;
  }

  .badge-draft {
    background-color: #e0e0e0;
    color: #666;
  }

  .badge-pending {
    background-color: #fff3cd;
    color: #856404;
    border: 1px solid #ffc107;
  }

  .badge-approved {
    background-color: #d4edda;
    color: #155724;
    border: 1px solid #28a745;
  }

  .badge-rejected {
    background-color: #f8d7da;
    color: #721c24;
    border: 1px solid #dc3545;
  }

  .invoice-details {
    background: #f9f9f9;
    padding: 1.5rem;
    border-radius: 4px;
    margin-bottom: 1.5rem;
  }

  .invoice-details h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #333;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid #eee;
  }

  .detail-row:last-child {
    border-bottom: none;
  }

  .detail-row.total {
    margin-top: 0.5rem;
    padding-top: 0.75rem;
    border-top: 2px solid #ddd;
    border-bottom: none;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .workflow-info {
    background: #e8f4f8;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .workflow-info p {
    margin: 0.5rem 0;
  }

  .alert {
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .alert-error {
    background-color: #fee;
    border: 1px solid #fcc;
    color: #c33;
  }

  .alert-warning {
    background-color: #fff3cd;
    border: 1px solid #ffc107;
    color: #856404;
  }

  .info-box {
    background-color: #e7f3ff;
    border: 1px solid #b3d9ff;
    color: #004085;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }

  .workflow-actions {
    margin-top: 1.5rem;
  }

  .approval-actions {
    display: flex;
    gap: 1rem;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s;
    font-weight: 500;
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

  .btn-success {
    background-color: #28a745;
    color: white;
    flex: 1;
  }

  .btn-success:hover:not(:disabled) {
    background-color: #218838;
  }

  .btn-danger {
    background-color: #dc3545;
    color: white;
    flex: 1;
  }

  .btn-danger:hover:not(:disabled) {
    background-color: #c82333;
  }

  .btn-secondary {
    background-color: #e0e0e0;
    color: #333;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #d0d0d0;
  }

  /* Modal */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    max-width: 500px;
    width: 90%;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .modal-content h3 {
    margin-top: 0;
    margin-bottom: 1rem;
    color: #333;
  }

  .modal-content p {
    margin-bottom: 1rem;
    color: #666;
  }

  .modal-content textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
    font-family: inherit;
    resize: vertical;
  }

  .modal-content textarea:focus {
    outline: none;
    border-color: #4a90e2;
    box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.1);
  }

  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
    margin-top: 1.5rem;
  }
</style>
