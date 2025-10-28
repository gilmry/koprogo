<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { api } from '../lib/api';
  import Pagination from './Pagination.svelte';
  import DocumentUploadModal from './DocumentUploadModal.svelte';
  import {
    DOCUMENT_TYPE_OPTIONS as DOCUMENT_TYPES,
    type Building,
    type Document,
    type PageResponse,
    UserRole,
    type User,
  } from '../lib/types';
  import { authStore } from '../stores/auth';

  export let allowUpload: boolean | null = null;
  export let allowDelete: boolean | null = null;

  let documents: Document[] = [];
  let loading = true;
  let error = '';
  let downloadError = '';
  let deleteError = '';
  let infoMessage = '';

  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  let buildings: Building[] = [];
  let buildingsLoading = false;
  let buildingsError: string | null = null;
  let buildingNameMap = new Map<string, string>();

  let showUploadModal = false;
  let deletingId: string | null = null;

  let user: User | null = null;
  let unsubscribe: () => void;

  onMount(async () => {
    authStore.init();
    unsubscribe = authStore.subscribe((state) => {
      user = state.user;
    });
    await loadDocuments();
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

  $: computedAllowUpload =
    allowUpload ?? (user?.role === UserRole.SUPERADMIN || user?.role === UserRole.SYNDIC);

  $: computedAllowDelete =
    allowDelete ?? (user?.role === UserRole.SUPERADMIN || user?.role === UserRole.SYNDIC);

  $: if (computedAllowUpload && user && buildings.length === 0 && !buildingsLoading) {
    loadBuildings();
  }

  async function loadBuildings() {
    try {
      buildingsLoading = true;
      buildingsError = null;
      const response = await api.get<PageResponse<Building>>('/buildings?per_page=100');
      buildings = response.data;
      buildingNameMap = new Map(response.data.map((b) => [b.id, b.name]));
    } catch (err) {
      buildingsError = err instanceof Error ? err.message : 'Impossible de charger les bâtiments';
      console.error('Failed to load buildings', err);
    } finally {
      buildingsLoading = false;
    }
  }

  async function loadDocuments() {
    try {
      loading = true;
      error = '';
      const response = await api.get<PageResponse<Document>>(
        `/documents?page=${currentPage}&per_page=${perPage}`,
      );

      documents = response.data;
      totalItems = response.pagination.total_items;
      totalPages = response.pagination.total_pages;
      currentPage = response.pagination.current_page;
      perPage = response.pagination.per_page;
    } catch (e) {
      error =
        e instanceof Error ? e.message : 'Erreur lors du chargement des documents';
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
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  }

  function getDocumentLabel(type: string): string {
    const match = DOCUMENT_TYPES.find((opt) => opt.value === type);
    return match ? match.label : type;
  }

  function getBuildingName(id: string): string {
    return buildingNameMap.get(id) ?? buildings.find((b) => b.id === id)?.name ?? '—';
  }

  function getDocumentIcon(mimeType: string): string {
    if (mimeType.includes('pdf')) return '📄';
    if (mimeType.includes('image')) return '🖼️';
    if (mimeType.includes('word') || mimeType.includes('document')) return '📝';
    if (mimeType.includes('excel') || mimeType.includes('spreadsheet')) return '📊';
    return '📎';
  }

  function getDownloadFilename(doc: Document): string {
    const extension = doc.file_path.includes('.')
      ? `.${doc.file_path.split('.').pop()}`
      : '';
    const safeTitle = doc.title.replace(/[^a-z0-9\-_]+/gi, '_');
    return `${safeTitle || 'document'}${extension}`;
  }

  async function handleDownload(doc: Document) {
    downloadError = '';
    try {
      await api.download(`/documents/${doc.id}/download`, getDownloadFilename(doc));
    } catch (err) {
      downloadError =
        err instanceof Error ? err.message : 'Erreur lors du téléchargement';
      console.error('Download failed', err);
    }
  }

  async function handleDelete(doc: Document) {
    if (!computedAllowDelete) return;
    const confirmed = window.confirm(
      `Supprimer définitivement le document « ${doc.title} » ?`,
    );
    if (!confirmed) return;

    deleteError = '';
    infoMessage = '';
    deletingId = doc.id;

    try {
      await api.deleteDocument(doc.id);
      infoMessage = 'Document supprimé avec succès.';
      await loadDocuments();
    } catch (err) {
      deleteError =
        err instanceof Error ? err.message : 'Erreur lors de la suppression du document';
      console.error('Delete failed', err);
    } finally {
      deletingId = null;
    }
  }

  function handleUploadSuccess() {
    showUploadModal = false;
    infoMessage = 'Document téléversé avec succès.';
    loadDocuments();
  }

  function handleOpenUpload() {
    if (!buildings.length && !buildingsLoading) {
      loadBuildings();
    }
    infoMessage = '';
    deleteError = '';
    showUploadModal = true;
  }
</script>

<div class="space-y-6">
  <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
    <div>
      <p class="text-gray-600">
        {totalItems} document{totalItems !== 1 ? 's' : ''}
      </p>
      {#if buildingsError}
        <p class="text-sm text-red-500">{buildingsError}</p>
      {/if}
    </div>
    {#if computedAllowUpload}
      <button
        class="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-primary-600 text-white hover:bg-primary-700 transition"
        on:click={handleOpenUpload}
      >
        <span>📤</span>
        <span>Téléverser un document</span>
      </button>
    {/if}
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if infoMessage}
    <div class="bg-emerald-50 border border-emerald-200 text-emerald-700 px-4 py-3 rounded">
      {infoMessage}
    </div>
  {/if}

  {#if downloadError}
    <div class="bg-yellow-50 border border-yellow-400 text-yellow-700 px-4 py-3 rounded">
      {downloadError}
    </div>
  {/if}

  {#if deleteError}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {deleteError}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement…</p>
  {:else if documents.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucun document enregistré pour le moment.
    </p>
  {:else}
    <div class="overflow-x-auto bg-white border border-gray-200 rounded-xl shadow-sm">
      <table class="min-w-full divide-y divide-gray-100 text-sm">
        <thead class="bg-gray-50">
          <tr class="text-left text-gray-500 uppercase text-xs tracking-wider">
            <th class="px-5 py-3">Titre</th>
            <th class="px-5 py-3">Bâtiment</th>
            <th class="px-5 py-3">Type</th>
            <th class="px-5 py-3">Taille</th>
            <th class="px-5 py-3">Ajouté le</th>
            <th class="px-5 py-3 text-right">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          {#each documents as doc}
            <tr class="hover:bg-gray-50">
              <td class="px-5 py-3">
                <div class="flex items-center gap-3">
                  <span class="text-xl">{getDocumentIcon(doc.mime_type)}</span>
                  <div>
                    <p class="font-medium text-gray-900">{doc.title}</p>
                    {#if doc.description}
                      <p class="text-xs text-gray-500">{doc.description}</p>
                    {/if}
                  </div>
                </div>
              </td>
              <td class="px-5 py-3 text-gray-700">{getBuildingName(doc.building_id)}</td>
              <td class="px-5 py-3 text-gray-700">{getDocumentLabel(doc.document_type)}</td>
              <td class="px-5 py-3 text-gray-700">{formatFileSize(doc.file_size)}</td>
              <td class="px-5 py-3 text-gray-700">{formatDate(doc.created_at)}</td>
              <td class="px-5 py-3 text-right">
                <div class="flex justify-end gap-2">
                  <button
                    class="inline-flex items-center gap-1 text-primary-600 hover:text-primary-700 font-medium"
                    on:click={() => handleDownload(doc)}
                  >
                    <span>⬇️</span>
                    <span>Télécharger</span>
                  </button>
                  {#if computedAllowDelete}
                    <button
                      class="inline-flex items-center gap-1 text-red-600 hover:text-red-700 font-medium disabled:opacity-60"
                      on:click={() => handleDelete(doc)}
                      disabled={deletingId === doc.id}
                    >
                      <span>🗑️</span>
                      <span>{deletingId === doc.id ? 'Suppression…' : 'Supprimer'}</span>
                    </button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
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

  {#if computedAllowUpload}
    <DocumentUploadModal
      bind:open={showUploadModal}
      {buildings}
      {user}
      loadingBuildings={buildingsLoading}
      on:close={() => (showUploadModal = false)}
      on:uploaded={handleUploadSuccess}
    />
  {/if}
</div>
