<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  export let buildingId: string | null = null;
  export let onInvoiceSelected: ((invoice: any) => void) | null = null;
  export let filterByStatus: string | null = null; // 'pending', 'approved', 'rejected', 'draft'
  export let showPendingOnly = false; // Special mode for syndic dashboard

  // Invoice list
  let invoices: any[] = [];
  let filteredInvoices: any[] = [];
  let loading = false;
  let error = '';

  // Filters
  let statusFilter = filterByStatus || '';
  let searchQuery = '';
  let dateFrom = '';
  let dateTo = '';

  // Pagination
  let currentPage = 1;
  let pageSize = 10;
  let totalPages = 1;
  let paginatedInvoices: any[] = [];

  onMount(async () => {
    await loadInvoices();
  });

  async function loadInvoices() {
    try {
      loading = true;
      error = '';

      let url = '/invoices';
      const params = new URLSearchParams();

      if (buildingId) {
        params.append('building_id', buildingId);
      }

      if (showPendingOnly) {
        params.append('approval_status', 'pending_approval');
      } else if (statusFilter) {
        params.append('approval_status', statusFilter);
      }

      if (dateFrom) {
        params.append('date_from', dateFrom);
      }
      if (dateTo) {
        params.append('date_to', dateTo);
      }

      const queryString = params.toString();
      if (queryString) {
        url += `?${queryString}`;
      }

      invoices = await api.get(url);
      applyFilters();
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des factures';
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredInvoices = invoices.filter((invoice) => {
      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matchesDescription = invoice.description?.toLowerCase().includes(query);
        const matchesSupplier = invoice.supplier?.toLowerCase().includes(query);
        const matchesInvoiceNumber = invoice.invoice_number?.toLowerCase().includes(query);
        if (!matchesDescription && !matchesSupplier && !matchesInvoiceNumber) {
          return false;
        }
      }

      return true;
    });

    // Calculate pagination
    totalPages = Math.ceil(filteredInvoices.length / pageSize);
    currentPage = Math.min(currentPage, totalPages || 1);
    updatePagination();
  }

  function updatePagination() {
    const startIndex = (currentPage - 1) * pageSize;
    const endIndex = startIndex + pageSize;
    paginatedInvoices = filteredInvoices.slice(startIndex, endIndex);
  }

  function handleStatusFilterChange() {
    currentPage = 1;
    loadInvoices();
  }

  function handleDateFilterChange() {
    currentPage = 1;
    loadInvoices();
  }

  function handleSearchChange() {
    currentPage = 1;
    applyFilters();
  }

  function handlePageChange(newPage: number) {
    currentPage = newPage;
    updatePagination();
  }

  function selectInvoice(invoice: any) {
    if (onInvoiceSelected) {
      onInvoiceSelected(invoice);
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

  function formatDate(dateString: string | null | undefined): string {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleDateString('fr-BE');
  }

  function formatAmount(amount: number | null | undefined): string {
    if (amount === null || amount === undefined) return '-';
    return `${amount.toFixed(2)} €`;
  }

  // Reactive statements
  $: if (searchQuery !== undefined) {
    handleSearchChange();
  }
</script>

<div class="invoice-list">
  <!-- Header -->
  <div class="list-header">
    <h2>
      {#if showPendingOnly}
        Factures en attente de validation
      {:else}
        Liste des factures
      {/if}
    </h2>
    <button class="btn btn-primary" on:click={loadInvoices} disabled={loading}>
      🔄 Actualiser
    </button>
  </div>

  <!-- Filters -->
  {#if !showPendingOnly}
    <div class="filters">
      <div class="filter-group">
        <label for="status-filter">Statut:</label>
        <select
          id="status-filter"
          bind:value={statusFilter}
          on:change={handleStatusFilterChange}
          disabled={loading}
        >
          <option value="">Tous</option>
          <option value="draft">Brouillon</option>
          <option value="pending_approval">En attente</option>
          <option value="approved">Approuvée</option>
          <option value="rejected">Rejetée</option>
        </select>
      </div>

      <div class="filter-group">
        <label for="search">Recherche:</label>
        <input
          id="search"
          type="text"
          bind:value={searchQuery}
          placeholder="Description, fournisseur, n° facture..."
          disabled={loading}
        />
      </div>

      <div class="filter-group">
        <label for="date-from">Du:</label>
        <input
          id="date-from"
          type="date"
          bind:value={dateFrom}
          on:change={handleDateFilterChange}
          disabled={loading}
        />
      </div>

      <div class="filter-group">
        <label for="date-to">Au:</label>
        <input
          id="date-to"
          type="date"
          bind:value={dateTo}
          on:change={handleDateFilterChange}
          disabled={loading}
        />
      </div>
    </div>
  {/if}

  <!-- Loading/Error States -->
  {#if loading}
    <p class="loading">Chargement...</p>
  {:else if error}
    <div class="alert alert-error">{error}</div>
  {:else if paginatedInvoices.length === 0}
    <div class="empty-state">
      <p>Aucune facture trouvée.</p>
    </div>
  {:else}
    <!-- Invoice Table -->
    <div class="table-container">
      <table class="invoice-table">
        <thead>
          <tr>
            <th>Date</th>
            <th>Description</th>
            <th>Fournisseur</th>
            <th>N° Facture</th>
            <th>Montant HT</th>
            <th>TVA</th>
            <th>Montant TTC</th>
            <th>Statut</th>
            <th>Échéance</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each paginatedInvoices as invoice}
            <tr class="invoice-row" on:click={() => selectInvoice(invoice)}>
              <td>{formatDate(invoice.invoice_date)}</td>
              <td class="description-cell">{invoice.description}</td>
              <td>{invoice.supplier || '-'}</td>
              <td>{invoice.invoice_number || '-'}</td>
              <td class="amount-cell">{formatAmount(invoice.amount_excl_vat)}</td>
              <td class="amount-cell">{invoice.vat_rate ? `${invoice.vat_rate}%` : '-'}</td>
              <td class="amount-cell total">
                {formatAmount(invoice.amount_incl_vat || invoice.amount)}
              </td>
              <td>
                <span class="badge {getStatusBadgeClass(invoice.approval_status)}">
                  {getStatusLabel(invoice.approval_status)}
                </span>
              </td>
              <td>{formatDate(invoice.due_date)}</td>
              <td>
                <button
                  class="btn btn-sm btn-secondary"
                  on:click|stopPropagation={() => selectInvoice(invoice)}
                >
                  Voir
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="pagination">
        <button
          class="btn btn-sm"
          on:click={() => handlePageChange(currentPage - 1)}
          disabled={currentPage === 1}
        >
          ← Précédent
        </button>

        <span class="page-info">
          Page {currentPage} sur {totalPages} ({filteredInvoices.length} facture{filteredInvoices.length > 1 ? 's' : ''})
        </span>

        <button
          class="btn btn-sm"
          on:click={() => handlePageChange(currentPage + 1)}
          disabled={currentPage === totalPages}
        >
          Suivant →
        </button>
      </div>
    {/if}

    <!-- Summary -->
    <div class="summary">
      <p>
        <strong>Total affiché:</strong>
        {filteredInvoices.length} facture{filteredInvoices.length > 1 ? 's' : ''}
      </p>
    </div>
  {/if}
</div>

<style>
  .invoice-list {
    background: white;
    padding: 2rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  .list-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
  }

  .list-header h2 {
    margin: 0;
    color: #333;
  }

  .filters {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 1rem;
    background: #f9f9f9;
    border-radius: 4px;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .filter-group label {
    font-weight: 600;
    font-size: 0.9rem;
    color: #555;
  }

  .filter-group select,
  .filter-group input {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 0.95rem;
  }

  .filter-group select:focus,
  .filter-group input:focus {
    outline: none;
    border-color: #4a90e2;
    box-shadow: 0 0 0 2px rgba(74, 144, 226, 0.1);
  }

  .loading {
    text-align: center;
    padding: 2rem;
    color: #666;
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

  .empty-state {
    text-align: center;
    padding: 3rem;
    color: #999;
  }

  .table-container {
    overflow-x: auto;
    margin-bottom: 1rem;
  }

  .invoice-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .invoice-table thead {
    background: #f5f5f5;
    border-bottom: 2px solid #ddd;
  }

  .invoice-table th {
    padding: 0.75rem;
    text-align: left;
    font-weight: 600;
    color: #555;
  }

  .invoice-table td {
    padding: 0.75rem;
    border-bottom: 1px solid #eee;
  }

  .invoice-row {
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .invoice-row:hover {
    background-color: #f9f9f9;
  }

  .description-cell {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .amount-cell {
    text-align: right;
    font-family: 'Courier New', monospace;
  }

  .amount-cell.total {
    font-weight: 600;
  }

  .badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-weight: 600;
    font-size: 0.8rem;
    white-space: nowrap;
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

  .btn {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-size: 0.95rem;
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

  .btn-secondary {
    background-color: #e0e0e0;
    color: #333;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #d0d0d0;
  }

  .btn-sm {
    padding: 0.4rem 0.8rem;
    font-size: 0.85rem;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 1rem;
    margin-top: 1.5rem;
    padding: 1rem;
    border-top: 1px solid #eee;
  }

  .page-info {
    color: #666;
    font-size: 0.9rem;
  }

  .summary {
    margin-top: 1rem;
    padding: 1rem;
    background: #f9f9f9;
    border-radius: 4px;
    text-align: right;
  }

  .summary p {
    margin: 0;
    color: #555;
  }

  /* Responsive */
  @media (max-width: 768px) {
    .invoice-list {
      padding: 1rem;
    }

    .filters {
      grid-template-columns: 1fr;
    }

    .table-container {
      overflow-x: scroll;
    }

    .invoice-table {
      font-size: 0.8rem;
    }

    .invoice-table th,
    .invoice-table td {
      padding: 0.5rem;
    }

    .description-cell {
      max-width: 120px;
    }
  }
</style>
