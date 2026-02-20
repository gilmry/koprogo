<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../stores/auth';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';

  let invoices: any[] = [];
  let filteredInvoices: any[] = [];
  let loading = true;
  let error = '';
  let selectedInvoice: any = null;
  let showApprovalModal = false;
  let showRejectionModal = false;
  let rejectionReason = '';
  let submitting = false;

  // Filtres
  let filterStatus: string = 'all';
  let filterPaymentStatus: string = 'all';

  onMount(async () => {
    // Initialize auth store from localStorage
    await authStore.init();

    console.log('🔐 Auth state on mount:', {
      user: $authStore.user,
      isAuthenticated: $authStore.isAuthenticated,
      hasActiveRole: !!$authStore.user?.activeRole,
      activeRole: $authStore.user?.activeRole,
      directRole: $authStore.user?.role,
    });
    loadInvoices();
  });

  async function loadInvoices() {
    loading = true;
    error = '';
    try {
      const response = await api.get('/expenses');
      const data = Array.isArray(response) ? response : (response?.data || []);
      console.log('📊 Invoices loaded:', data.length, data);
      invoices = data.sort((a: any, b: any) =>
        new Date(b.expense_date || b.created_at).getTime() - new Date(a.expense_date || a.created_at).getTime()
      );
      console.log('📊 After sort:', invoices.length);
    } catch (err: any) {
      console.error('❌ Error loading invoices:', err);
      error = err.message || 'Erreur lors du chargement des factures';
    } finally {
      loading = false;
    }
  }

  async function submitForApproval(invoiceId: string) {
    if (!confirm('Soumettre cette facture pour approbation ?')) return;

    submitting = true;
    try {
      console.log('🔄 Submitting invoice for approval:', invoiceId);
      const response = await api.put(`/invoices/${invoiceId}/submit`, {});
      console.log('✅ Invoice submitted successfully:', response);
      console.log('📋 New approval_status:', response.approval_status);
      await loadInvoices();
    } catch (err: any) {
      console.error('❌ Error submitting invoice:', err);
      toast.error(err.message || 'Erreur lors de la soumission');
    } finally {
      submitting = false;
    }
  }

  async function approveInvoice() {
    if (!selectedInvoice) return;

    submitting = true;
    try {
      await api.put(`/invoices/${selectedInvoice.id}/approve`, {
        approved_by_user_id: $authStore.user?.id || '',
      });
      showApprovalModal = false;
      selectedInvoice = null;
      await loadInvoices();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'approbation');
    } finally {
      submitting = false;
    }
  }

  async function rejectInvoice() {
    if (!selectedInvoice || !rejectionReason.trim()) {
      toast.error('Veuillez saisir une raison de rejet');
      return;
    }

    submitting = true;
    try {
      await api.put(`/invoices/${selectedInvoice.id}/reject`, {
        rejected_by_user_id: $authStore.user?.id || '',
        rejection_reason: rejectionReason,
      });
      showRejectionModal = false;
      selectedInvoice = null;
      rejectionReason = '';
      await loadInvoices();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors du rejet');
    } finally {
      submitting = false;
    }
  }

  async function markAsPaid(invoiceId: string) {
    if (!confirm('Marquer cette facture comme payée ? Cela générera automatiquement une écriture comptable de paiement (FIN).')) return;

    submitting = true;
    try {
      await api.put(`/expenses/${invoiceId}/mark-paid`, {});
      await loadInvoices();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors du paiement');
    } finally {
      submitting = false;
    }
  }

  function openApprovalModal(invoice: any) {
    selectedInvoice = invoice;
    showApprovalModal = true;
  }

  function openRejectionModal(invoice: any) {
    selectedInvoice = invoice;
    showRejectionModal = true;
  }

  function closeModals() {
    showApprovalModal = false;
    showRejectionModal = false;
    selectedInvoice = null;
    rejectionReason = '';
  }

  function getApprovalStatusBadge(status: string) {
    const badges: Record<string, { class: string; label: string }> = {
      draft: { class: 'bg-gray-200 text-gray-800', label: 'Brouillon' },
      pending_approval: { class: 'bg-yellow-200 text-yellow-900', label: 'En attente' },
      approved: { class: 'bg-green-200 text-green-900', label: 'Approuvée' },
      rejected: { class: 'bg-red-200 text-red-900', label: 'Rejetée' },
    };
    return badges[status] || { class: 'bg-gray-200 text-gray-800', label: status };
  }

  function getPaymentStatusBadge(status: string) {
    const badges: Record<string, { class: string; label: string }> = {
      pending: { class: 'bg-blue-200 text-blue-900', label: 'En attente' },
      paid: { class: 'bg-green-200 text-green-900', label: 'Payée' },
      overdue: { class: 'bg-red-200 text-red-900', label: 'En retard' },
      cancelled: { class: 'bg-gray-200 text-gray-800', label: 'Annulée' },
    };
    return badges[status] || { class: 'bg-gray-200 text-gray-800', label: status };
  }

  function canSubmitForApproval(invoice: any): boolean {
    // Use activeRole if available, otherwise fallback to user.role
    const role = $authStore.user?.activeRole?.role ?? $authStore.user?.role;
    const canSubmit = (
      invoice.approval_status === 'draft' &&
      (role === 'accountant' || role === 'syndic' || role === 'superadmin')
    );
    console.log('🔍 canSubmitForApproval:', invoice.id, 'status:', invoice.approval_status, 'role:', role, 'result:', canSubmit);
    return canSubmit;
  }

  function canApprove(invoice: any): boolean {
    // Use activeRole if available, otherwise fallback to user.role
    const role = $authStore.user?.activeRole?.role ?? $authStore.user?.role;
    const canApprove = (
      invoice.approval_status === 'pending_approval' &&
      (role === 'syndic' || role === 'superadmin')
    );
    console.log('🔍 canApprove:', invoice.id, 'status:', invoice.approval_status, 'role:', role, 'result:', canApprove);
    return canApprove;
  }

  function canReject(invoice: any): boolean {
    // Use activeRole if available, otherwise fallback to user.role
    const role = $authStore.user?.activeRole?.role ?? $authStore.user?.role;
    const canReject = (
      invoice.approval_status === 'pending_approval' &&
      (role === 'syndic' || role === 'superadmin')
    );
    console.log('🔍 canReject:', invoice.id, 'status:', invoice.approval_status, 'role:', role, 'result:', canReject);
    return canReject;
  }

  function canMarkAsPaid(invoice: any): boolean {
    // Use activeRole if available, otherwise fallback to user.role
    const role = $authStore.user?.activeRole?.role ?? $authStore.user?.role;
    const canPay = (
      invoice.approval_status === 'approved' &&
      invoice.payment_status === 'pending' &&
      (role === 'accountant' || role === 'syndic' || role === 'superadmin')
    );
    console.log('🔍 canMarkAsPaid:', invoice.id, 'status:', invoice.approval_status, 'payment:', invoice.payment_status, 'role:', role, 'result:', canPay);
    return canPay;
  }

  $: {
    filteredInvoices = invoices.filter((inv) => {
      if (filterStatus !== 'all' && inv.approval_status !== filterStatus) {
        console.log('❌ Filtered out by approval_status:', inv.approval_status, '!==', filterStatus);
        return false;
      }
      if (filterPaymentStatus !== 'all' && inv.payment_status !== filterPaymentStatus) {
        console.log('❌ Filtered out by payment_status:', inv.payment_status, '!==', filterPaymentStatus);
        return false;
      }
      return true;
    });
    console.log('✅ Filtered invoices:', filteredInvoices.length, 'filters:', filterStatus, filterPaymentStatus);
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', {
      style: 'currency',
      currency: 'EUR',
    }).format(amount);
  }

  function formatDate(date: string): string {
    return new Date(date).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  }
</script>

<div class="workflow-container">
  <div class="header">
    <h1>Workflow de validation des factures</h1>
    <p class="subtitle">Gérez l'approbation et le paiement des factures</p>
  </div>

  {#if error}
    <div class="alert alert-error">
      <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
      </svg>
      {error}
    </div>
  {/if}

  <div class="filters">
    <div class="filter-group">
      <label for="filterStatus">Statut d'approbation</label>
      <select id="filterStatus" bind:value={filterStatus}>
        <option value="all">Tous</option>
        <option value="draft">Brouillon</option>
        <option value="pending_approval">En attente d'approbation</option>
        <option value="approved">Approuvée</option>
        <option value="rejected">Rejetée</option>
      </select>
    </div>

    <div class="filter-group">
      <label for="filterPaymentStatus">Statut de paiement</label>
      <select id="filterPaymentStatus" bind:value={filterPaymentStatus}>
        <option value="all">Tous</option>
        <option value="pending">En attente</option>
        <option value="paid">Payée</option>
        <option value="overdue">En retard</option>
        <option value="cancelled">Annulée</option>
      </select>
    </div>

    <button on:click={loadInvoices} class="btn-refresh" disabled={loading}>
      <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
      </svg>
      Actualiser
    </button>
  </div>

  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <p>Chargement des factures...</p>
    </div>
  {:else if filteredInvoices.length === 0}
    <div class="empty-state">
      <svg class="icon-large" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
      </svg>
      <p>Aucune facture trouvée</p>
    </div>
  {:else}
    <div class="invoice-grid">
      {#each filteredInvoices as invoice}
        <div class="invoice-card">
          <div class="card-header">
            <div>
              <h3 class="invoice-title">{invoice.description}</h3>
              <p class="invoice-meta">
                Date: {formatDate(invoice.expense_date)}
                {#if invoice.invoice_number}
                  • N° {invoice.invoice_number}
                {/if}
              </p>
            </div>
            <div class="invoice-amount">
              {formatCurrency(invoice.amount_incl_vat || invoice.amount || 0)}
            </div>
          </div>

          <div class="card-body">
            <div class="status-row">
              <div class="status-item">
                <span class="status-label">Approbation</span>
                <span class="badge {getApprovalStatusBadge(invoice.approval_status).class}">
                  {getApprovalStatusBadge(invoice.approval_status).label}
                </span>
              </div>
              <div class="status-item">
                <span class="status-label">Paiement</span>
                <span class="badge {getPaymentStatusBadge(invoice.payment_status).class}">
                  {getPaymentStatusBadge(invoice.payment_status).label}
                </span>
              </div>
            </div>

            {#if invoice.rejection_reason}
              <div class="rejection-info">
                <svg class="icon-small" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/>
                </svg>
                <div>
                  <strong>Raison du rejet:</strong>
                  <p>{invoice.rejection_reason}</p>
                </div>
              </div>
            {/if}

            {#if invoice.invoice_number}
              <div class="invoice-info">
                <span class="info-label">N° facture:</span>
                <span>{invoice.invoice_number}</span>
              </div>
            {/if}

            {#if invoice.supplier_name}
              <div class="invoice-info">
                <span class="info-label">Fournisseur:</span>
                <span>{invoice.supplier_name}</span>
              </div>
            {/if}
          </div>

          <div class="card-actions">
            {#if canSubmitForApproval(invoice)}
              <button
                on:click={() => submitForApproval(invoice.id)}
                class="btn btn-primary"
                disabled={submitting}
              >
                <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                Soumettre pour approbation
              </button>
            {/if}

            {#if canApprove(invoice)}
              <button
                on:click={() => openApprovalModal(invoice)}
                class="btn btn-success"
                disabled={submitting}
              >
                <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                Approuver
              </button>
            {/if}

            {#if canReject(invoice)}
              <button
                on:click={() => openRejectionModal(invoice)}
                class="btn btn-danger"
                disabled={submitting}
              >
                <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"/>
                </svg>
                Rejeter
              </button>
            {/if}

            {#if canMarkAsPaid(invoice)}
              <button
                on:click={() => markAsPaid(invoice.id)}
                class="btn btn-info"
                disabled={submitting}
              >
                <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z"/>
                </svg>
                Marquer comme payée
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Modal d'approbation -->
{#if showApprovalModal && selectedInvoice}
  <div class="modal-overlay" on:click={closeModals}>
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h2>Approuver la facture</h2>
        <button on:click={closeModals} class="btn-close">×</button>
      </div>
      <div class="modal-body">
        <p>Voulez-vous approuver cette facture ?</p>
        <div class="invoice-summary">
          <p><strong>Description:</strong> {selectedInvoice.description}</p>
          <p><strong>Montant:</strong> {formatCurrency(selectedInvoice.amount_incl_vat || selectedInvoice.amount || 0)}</p>
          {#if selectedInvoice.supplier_name}
            <p><strong>Fournisseur:</strong> {selectedInvoice.supplier_name}</p>
          {/if}
        </div>
        <div class="alert alert-info">
          <svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
          </svg>
          Une écriture comptable d'achat (ACH) sera automatiquement générée dans le journal.
        </div>
      </div>
      <div class="modal-footer">
        <button on:click={closeModals} class="btn btn-secondary" disabled={submitting}>
          Annuler
        </button>
        <button on:click={approveInvoice} class="btn btn-success" disabled={submitting}>
          {submitting ? 'Approbation en cours...' : 'Approuver'}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal de rejet -->
{#if showRejectionModal && selectedInvoice}
  <div class="modal-overlay" on:click={closeModals}>
    <div class="modal" on:click|stopPropagation>
      <div class="modal-header">
        <h2>Rejeter la facture</h2>
        <button on:click={closeModals} class="btn-close">×</button>
      </div>
      <div class="modal-body">
        <p>Veuillez indiquer la raison du rejet:</p>
        <div class="invoice-summary">
          <p><strong>Description:</strong> {selectedInvoice.description}</p>
          <p><strong>Montant:</strong> {formatCurrency(selectedInvoice.amount_incl_vat || selectedInvoice.amount || 0)}</p>
        </div>
        <textarea
          bind:value={rejectionReason}
          placeholder="Raison du rejet..."
          rows="4"
          class="textarea"
          disabled={submitting}
        ></textarea>
      </div>
      <div class="modal-footer">
        <button on:click={closeModals} class="btn btn-secondary" disabled={submitting}>
          Annuler
        </button>
        <button on:click={rejectInvoice} class="btn btn-danger" disabled={submitting || !rejectionReason.trim()}>
          {submitting ? 'Rejet en cours...' : 'Rejeter'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .workflow-container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
  }

  .header {
    margin-bottom: 2rem;
  }

  .header h1 {
    font-size: 2rem;
    font-weight: bold;
    color: #1f2937;
    margin-bottom: 0.5rem;
  }

  .subtitle {
    font-size: 1rem;
    color: #6b7280;
  }

  .alert {
    padding: 1rem;
    border-radius: 0.5rem;
    margin-bottom: 1.5rem;
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .alert-error {
    background-color: #fef2f2;
    color: #991b1b;
    border: 1px solid #fecaca;
  }

  .alert-info {
    background-color: #eff6ff;
    color: #1e40af;
    border: 1px solid #bfdbfe;
  }

  .filters {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
    flex-wrap: wrap;
    align-items: flex-end;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .filter-group label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .filter-group select {
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    background-color: white;
  }

  .btn-refresh {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #3b82f6;
    color: white;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    font-weight: 500;
    transition: background-color 0.2s;
  }

  .btn-refresh:hover:not(:disabled) {
    background-color: #2563eb;
  }

  .btn-refresh:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    gap: 1rem;
  }

  .spinner {
    border: 4px solid #f3f4f6;
    border-top: 4px solid #3b82f6;
    border-radius: 50%;
    width: 3rem;
    height: 3rem;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    gap: 1rem;
    color: #6b7280;
  }

  .icon {
    width: 1.25rem;
    height: 1.25rem;
  }

  .icon-small {
    width: 1rem;
    height: 1rem;
    flex-shrink: 0;
  }

  .icon-large {
    width: 4rem;
    height: 4rem;
  }

  .invoice-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 1.5rem;
  }

  .invoice-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    overflow: hidden;
    transition: box-shadow 0.2s;
  }

  .invoice-card:hover {
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  }

  .card-header {
    padding: 1.5rem;
    background-color: #f9fafb;
    border-bottom: 1px solid #e5e7eb;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .invoice-title {
    font-size: 1.125rem;
    font-weight: 600;
    color: #1f2937;
    margin-bottom: 0.25rem;
  }

  .invoice-meta {
    font-size: 0.875rem;
    color: #6b7280;
  }

  .invoice-amount {
    font-size: 1.25rem;
    font-weight: bold;
    color: #059669;
  }

  .card-body {
    padding: 1.5rem;
  }

  .status-row {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .status-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .status-label {
    font-size: 0.75rem;
    font-weight: 500;
    color: #6b7280;
    text-transform: uppercase;
  }

  .badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .rejection-info {
    margin-top: 1rem;
    padding: 1rem;
    background-color: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 0.375rem;
    display: flex;
    gap: 0.75rem;
    color: #991b1b;
  }

  .rejection-info strong {
    display: block;
    margin-bottom: 0.25rem;
  }

  .invoice-info {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem 0;
    border-bottom: 1px solid #f3f4f6;
    font-size: 0.875rem;
  }

  .invoice-info:last-child {
    border-bottom: none;
  }

  .info-label {
    font-weight: 500;
    color: #6b7280;
  }

  .card-actions {
    padding: 1rem 1.5rem;
    background-color: #f9fafb;
    border-top: 1px solid #e5e7eb;
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 0.375rem;
    cursor: pointer;
    font-weight: 500;
    font-size: 0.875rem;
    transition: all 0.2s;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    background-color: #3b82f6;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #2563eb;
  }

  .btn-success {
    background-color: #10b981;
    color: white;
  }

  .btn-success:hover:not(:disabled) {
    background-color: #059669;
  }

  .btn-danger {
    background-color: #ef4444;
    color: white;
  }

  .btn-danger:hover:not(:disabled) {
    background-color: #dc2626;
  }

  .btn-info {
    background-color: #06b6d4;
    color: white;
  }

  .btn-info:hover:not(:disabled) {
    background-color: #0891b2;
  }

  .btn-secondary {
    background-color: #6b7280;
    color: white;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #4b5563;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 50;
    padding: 1rem;
  }

  .modal {
    background: white;
    border-radius: 0.5rem;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
    max-width: 600px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem;
    border-bottom: 1px solid #e5e7eb;
  }

  .modal-header h2 {
    font-size: 1.5rem;
    font-weight: bold;
    color: #1f2937;
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 2rem;
    color: #6b7280;
    cursor: pointer;
    line-height: 1;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close:hover {
    color: #1f2937;
  }

  .modal-body {
    padding: 1.5rem;
  }

  .invoice-summary {
    background-color: #f9fafb;
    padding: 1rem;
    border-radius: 0.375rem;
    margin: 1rem 0;
  }

  .invoice-summary p {
    margin: 0.5rem 0;
    font-size: 0.875rem;
  }

  .textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-family: inherit;
    resize: vertical;
  }

  .textarea:focus {
    outline: none;
    border-color: #3b82f6;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding: 1.5rem;
    border-top: 1px solid #e5e7eb;
  }
</style>
