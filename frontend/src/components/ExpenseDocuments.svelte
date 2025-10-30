<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Document } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let expenseId: string;
  export let expenseStatus: string;

  let documents: Document[] = [];
  let loading = true;
  let error = '';
  let uploading = false;

  // Upload form state
  let showUploadForm = false;
  let uploadFile: File | null = null;
  let uploadTitle = '';
  let uploadDescription = '';
  let uploadDocumentType: string = 'Invoice';

  onMount(async () => {
    await loadDocuments();
  });

  async function loadDocuments() {
    try {
      loading = true;
      error = '';
      documents = await api.get<Document[]>(`/expenses/${expenseId}/documents`);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des documents';
      console.error('Error loading documents:', e);
    } finally {
      loading = false;
    }
  }

  function handleFileChange(event: Event) {
    const target = event.target as HTMLInputElement;
    if (target.files && target.files.length > 0) {
      uploadFile = target.files[0];
    }
  }

  async function handleUpload() {
    if (!uploadFile || !uploadTitle) {
      alert('Veuillez remplir tous les champs obligatoires');
      return;
    }

    try {
      uploading = true;
      error = '';

      // Get user info and expense details
      const userInfo = await api.get<any>('/auth/me');
      const expenseResponse = await api.get<any>(`/expenses/${expenseId}`);
      const buildingId = expenseResponse.building_id;

      // Step 1: Upload document using api helper
      const uploadedDoc = await api.uploadDocument({
        file: uploadFile,
        buildingId: buildingId,
        documentType: uploadDocumentType as any,
        title: uploadTitle,
        description: uploadDescription || undefined,
        uploadedBy: userInfo.id
      });

      // Step 2: Link document to expense
      await api.put(`/documents/${uploadedDoc.id}/link-expense`, {
        expense_id: expenseId
      });

      // Reset form
      uploadFile = null;
      uploadTitle = '';
      uploadDescription = '';
      uploadDocumentType = 'Invoice';
      showUploadForm = false;

      // Reload documents
      await loadDocuments();
      alert('Document ajouté avec succès');
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de l\'upload';
      console.error('Error uploading document:', e);
      alert(`Erreur: ${error}`);
    } finally {
      uploading = false;
    }
  }

  async function handleDownload(documentId: string, title: string) {
    try {
      const response = await fetch(`${import.meta.env.PUBLIC_API_URL}/documents/${documentId}/download`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
        },
      });

      if (!response.ok) {
        throw new Error('Erreur lors du téléchargement');
      }

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = title;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);
    } catch (e) {
      alert('Erreur lors du téléchargement du document');
      console.error('Error downloading document:', e);
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function getDocumentTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'MeetingMinutes': 'Procès-verbal',
      'FinancialStatement': 'Bilan financier',
      'Invoice': 'Facture',
      'Contract': 'Contrat',
      'Regulation': 'Règlement',
      'WorksQuote': 'Devis travaux',
      'Other': 'Autre'
    };
    return labels[type] || type;
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return bytes + ' B';
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
  }
</script>

<div class="bg-white rounded-lg shadow p-6">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-lg font-semibold text-gray-900">Documents liés</h3>
    {#if expenseStatus !== 'Cancelled'}
      <Button variant="primary" on:click={() => showUploadForm = !showUploadForm}>
        {showUploadForm ? 'Annuler' : '+ Ajouter un document'}
      </Button>
    {/if}
  </div>

  {#if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
      {error}
    </div>
  {/if}

  <!-- Upload Form -->
  {#if showUploadForm}
    <div class="bg-gray-50 rounded-lg p-4 mb-6">
      <h4 class="font-medium text-gray-900 mb-4">Ajouter un document</h4>
      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Type de document *
          </label>
          <select
            bind:value={uploadDocumentType}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          >
            <option value="Invoice">Facture</option>
            <option value="FinancialStatement">Bilan financier</option>
            <option value="MeetingMinutes">Procès-verbal</option>
            <option value="Contract">Contrat</option>
            <option value="Regulation">Règlement</option>
            <option value="WorksQuote">Devis travaux</option>
            <option value="Other">Autre</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Titre *
          </label>
          <input
            type="text"
            bind:value={uploadTitle}
            placeholder="Ex: Procès-verbal AG 2029"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Description
          </label>
          <textarea
            bind:value={uploadDescription}
            rows="3"
            placeholder="Description du document..."
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Fichier *
          </label>
          <input
            type="file"
            on:change={handleFileChange}
            accept=".pdf,.doc,.docx,.xls,.xlsx,.jpg,.jpeg,.png"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
          {#if uploadFile}
            <p class="text-sm text-gray-500 mt-1">
              Fichier sélectionné: {uploadFile.name} ({formatFileSize(uploadFile.size)})
            </p>
          {/if}
        </div>

        <div class="flex gap-2">
          <Button variant="primary" on:click={handleUpload} disabled={uploading}>
            {uploading ? 'Upload en cours...' : 'Ajouter le document'}
          </Button>
          <Button variant="outline" on:click={() => showUploadForm = false}>
            Annuler
          </Button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Documents List -->
  {#if loading}
    <div class="text-center text-gray-500 py-8">
      <p>Chargement des documents...</p>
    </div>
  {:else if documents.length === 0}
    <div class="text-center text-gray-500 py-8">
      <p>Aucun document lié pour le moment</p>
      <p class="text-sm mt-2">Ajoutez des documents pour cette dépense (factures, justificatifs, etc.)</p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each documents as doc (doc.id)}
        <div class="border border-gray-200 rounded-lg p-4 hover:bg-gray-50 transition">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <span class="text-2xl">📄</span>
                <h4 class="font-medium text-gray-900">{doc.title}</h4>
                <span class="text-xs px-2 py-1 rounded-full bg-blue-100 text-blue-800">
                  {getDocumentTypeLabel(doc.document_type)}
                </span>
              </div>
              {#if doc.description}
                <p class="text-sm text-gray-600 ml-8">{doc.description}</p>
              {/if}
              <div class="flex gap-4 text-xs text-gray-500 ml-8 mt-1">
                <span>📅 {formatDate(doc.created_at)}</span>
                <span>💾 {formatFileSize(doc.file_size)}</span>
              </div>
            </div>
            <Button variant="outline" on:click={() => handleDownload(doc.id, doc.title)}>
              Télécharger
            </Button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
