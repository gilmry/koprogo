<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  interface Document {
    id: string;
    title: string;
    document_type: string;
    file_size: number;
    mime_type: string;
    created_at: string;
  }

  let documents: Document[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadDocuments();
  });

  async function loadDocuments() {
    try {
      loading = true;
      const response = await api.get<PageResponse<Document>>(
        `/documents?page=${currentPage}&per_page=${perPage}`
      );

      documents = response.data;
      totalItems = response.pagination.total_items;
      totalPages = response.pagination.total_pages;
      currentPage = response.pagination.current_page;
      perPage = response.pagination.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des documents';
      console.error('Error loading documents:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadDocuments();
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function getDocumentIcon(mimeType: string): string {
    if (mimeType.includes('pdf')) return '📄';
    if (mimeType.includes('image')) return '🖼️';
    if (mimeType.includes('word') || mimeType.includes('document')) return '📝';
    if (mimeType.includes('excel') || mimeType.includes('spreadsheet')) return '📊';
    return '📎';
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} document{totalItems !== 1 ? 's' : ''}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if documents.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucun document enregistré.
    </p>
  {:else}
    <div class="grid gap-4">
      {#each documents as doc}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
              <span class="text-3xl">{getDocumentIcon(doc.mime_type)}</span>
              <div>
                <h3 class="text-lg font-semibold text-gray-900">
                  {doc.title}
                </h3>
                <p class="text-gray-600 text-sm">
                  {doc.document_type} · {formatFileSize(doc.file_size)}
                </p>
                <p class="text-gray-500 text-xs">
                  {formatDate(doc.created_at)}
                </p>
              </div>
            </div>
            <div class="flex gap-2">
              <button
                on:click={() => api.download(`/documents/${doc.id}/download`, doc.title)}
                class="text-primary-600 hover:text-primary-700 text-sm font-medium"
              >
                Télécharger
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <Pagination
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        perPage={perPage}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>
