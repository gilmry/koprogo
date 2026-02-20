<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    workReportsApi,
    workTypeLabels,
    warrantyTypeLabels,
    WorkType,
    WarrantyType,
    type WorkReport,
  } from "../../lib/api/work-reports";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";

  export let isOpen = false;
  export let report: WorkReport;

  const dispatch = createEventDispatcher();

  let editMode = false;
  let submitting = false;

  interface WorkReportEditForm {
    title: string;
    description: string;
    work_type: WorkType;
    contractor_name: string;
    contractor_contact: string;
    work_date_str: string;
    completion_date_str: string;
    cost: number;
    invoice_number: string;
    warranty_type: WarrantyType;
    notes: string;
  }

  let form: WorkReportEditForm = resetForm();

  function resetForm(): WorkReportEditForm {
    return {
      title: report?.title ?? "",
      description: report?.description ?? "",
      work_type: report?.work_type ?? WorkType.Maintenance,
      contractor_name: report?.contractor_name ?? "",
      contractor_contact: report?.contractor_contact ?? "",
      work_date_str: report?.work_date?.slice(0, 10) ?? "",
      completion_date_str: report?.completion_date?.slice(0, 10) ?? "",
      cost: report?.cost ?? 0,
      invoice_number: report?.invoice_number ?? "",
      warranty_type: report?.warranty_type ?? WarrantyType.Standard,
      notes: report?.notes ?? "",
    };
  }

  function enterEditMode() {
    form = resetForm();
    editMode = true;
  }

  function cancelEdit() {
    editMode = false;
  }

  async function saveEdit() {
    if (!form.title.trim() || !form.contractor_name.trim()) {
      toast.error("Le titre et l'entrepreneur sont requis");
      return;
    }
    try {
      submitting = true;
      const updated = await workReportsApi.update(report.id, {
        title: form.title,
        description: form.description || undefined,
        work_type: form.work_type,
        contractor_name: form.contractor_name,
        contractor_contact: form.contractor_contact || undefined,
        work_date: form.work_date_str ? new Date(form.work_date_str).toISOString() : undefined,
        completion_date: form.completion_date_str ? new Date(form.completion_date_str).toISOString() : undefined,
        cost: form.cost,
        invoice_number: form.invoice_number || undefined,
        warranty_type: form.warranty_type,
        notes: form.notes || undefined,
      });
      toast.success("Rapport mis à jour");
      report = updated;
      editMode = false;
      dispatch("updated", updated);
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la mise à jour");
    } finally {
      submitting = false;
    }
  }

  async function handleDelete() {
    if (!confirm("Supprimer définitivement ce rapport de travaux ?")) return;
    try {
      await workReportsApi.delete(report.id);
      toast.success("Rapport supprimé");
      dispatch("deleted", report.id);
      handleClose();
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la suppression");
    }
  }

  function handleClose() {
    editMode = false;
    dispatch("close");
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE", {
      day: "numeric",
      month: "long",
      year: "numeric",
    });
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat("fr-BE", {
      style: "currency",
      currency: "EUR",
    }).format(amount);
  }
</script>

<Modal {isOpen} title={editMode ? "Modifier le rapport" : "Rapport de travaux"} size="lg" on:close={handleClose}>
  {#if report}
    {#if !editMode}
      <!-- Vue lecture -->
      <div class="space-y-4">
        <!-- Header avec actions -->
        <div class="flex items-start justify-between">
          <div>
            <h3 class="text-xl font-semibold text-gray-900">{report.title}</h3>
            <span class="inline-flex items-center px-2.5 py-1 rounded-full text-sm font-medium mt-1
              {report.work_type === 'emergency' ? 'bg-red-100 text-red-800' :
               report.work_type === 'repair' ? 'bg-orange-100 text-orange-800' :
               report.work_type === 'renovation' ? 'bg-purple-100 text-purple-800' :
               report.work_type === 'installation' ? 'bg-blue-100 text-blue-800' :
               'bg-gray-100 text-gray-800'}">
              {workTypeLabels[report.work_type] || report.work_type}
            </span>
          </div>
          <div class="flex gap-2 shrink-0">
            <button
              on:click={enterEditMode}
              class="px-3 py-1.5 text-sm text-blue-600 border border-blue-300 rounded-lg hover:bg-blue-50 transition"
            >
              Modifier
            </button>
            <button
              on:click={handleDelete}
              class="px-3 py-1.5 text-sm text-red-600 border border-red-300 rounded-lg hover:bg-red-50 transition"
            >
              Supprimer
            </button>
          </div>
        </div>

        <!-- Infos principales -->
        <div class="grid grid-cols-2 gap-4 bg-gray-50 rounded-lg p-4 text-sm">
          <div>
            <p class="text-gray-500">Entrepreneur</p>
            <p class="font-medium text-gray-900">{report.contractor_name}</p>
            {#if report.contractor_contact}
              <p class="text-gray-600 text-xs mt-0.5">{report.contractor_contact}</p>
            {/if}
          </div>
          <div>
            <p class="text-gray-500">Date des travaux</p>
            <p class="font-medium text-gray-900">{formatDate(report.work_date)}</p>
            {#if report.completion_date}
              <p class="text-gray-600 text-xs mt-0.5">Fin : {formatDate(report.completion_date)}</p>
            {/if}
          </div>
          <div>
            <p class="text-gray-500">Coût</p>
            <p class="font-medium text-gray-900">{formatCurrency(report.cost)}</p>
            {#if report.invoice_number}
              <p class="text-gray-600 text-xs mt-0.5">Facture n° {report.invoice_number}</p>
            {/if}
          </div>
          <div>
            <p class="text-gray-500">Garantie</p>
            <p class="font-medium {report.is_warranty_valid ? 'text-green-700' : 'text-red-700'}">
              {warrantyTypeLabels[report.warranty_type] || report.warranty_type}
            </p>
            {#if report.warranty_type !== 'none'}
              <p class="text-xs {report.is_warranty_valid ? 'text-green-600' : 'text-red-600'} mt-0.5">
                {#if report.is_warranty_valid}
                  Expire dans {report.warranty_days_remaining} j (le {formatDate(report.warranty_expiry)})
                {:else}
                  Expirée le {formatDate(report.warranty_expiry)}
                {/if}
              </p>
            {/if}
          </div>
        </div>

        {#if report.description}
          <div>
            <p class="text-sm font-medium text-gray-700 mb-1">Description</p>
            <p class="text-sm text-gray-600 whitespace-pre-wrap">{report.description}</p>
          </div>
        {/if}

        {#if report.notes}
          <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
            <p class="text-sm font-medium text-yellow-800 mb-1">Notes</p>
            <p class="text-sm text-yellow-700">{report.notes}</p>
          </div>
        {/if}

        {#if report.photos.length > 0 || report.documents.length > 0}
          <p class="text-xs text-gray-500">
            {#if report.photos.length > 0}{report.photos.length} photo(s){/if}
            {#if report.documents.length > 0}&nbsp;· {report.documents.length} document(s){/if}
          </p>
        {/if}
      </div>

    {:else}
      <!-- Vue édition -->
      <div class="space-y-3">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div>
            <label for="wr-title" class="block text-sm text-gray-600 mb-1">Titre *</label>
            <input
              id="wr-title"
              bind:value={form.title}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-contractor" class="block text-sm text-gray-600 mb-1">Entrepreneur *</label>
            <input
              id="wr-contractor"
              bind:value={form.contractor_name}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-type" class="block text-sm text-gray-600 mb-1">Type de travaux</label>
            <select
              id="wr-type"
              bind:value={form.work_type}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.entries(workTypeLabels) as [val, label]}
                <option value={val}>{label}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="wr-date" class="block text-sm text-gray-600 mb-1">Date des travaux</label>
            <input
              id="wr-date"
              type="date"
              bind:value={form.work_date_str}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-completion" class="block text-sm text-gray-600 mb-1">Date de fin (optionnel)</label>
            <input
              id="wr-completion"
              type="date"
              bind:value={form.completion_date_str}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-cost" class="block text-sm text-gray-600 mb-1">Coût (EUR)</label>
            <input
              id="wr-cost"
              type="number"
              bind:value={form.cost}
              min="0"
              step="0.01"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-warranty" class="block text-sm text-gray-600 mb-1">Garantie</label>
            <select
              id="wr-warranty"
              bind:value={form.warranty_type}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.entries(warrantyTypeLabels) as [val, label]}
                <option value={val}>{label}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="wr-invoice" class="block text-sm text-gray-600 mb-1">N° Facture</label>
            <input
              id="wr-invoice"
              bind:value={form.invoice_number}
              placeholder="Optionnel"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="wr-contact" class="block text-sm text-gray-600 mb-1">Contact entrepreneur</label>
            <input
              id="wr-contact"
              bind:value={form.contractor_contact}
              placeholder="Téléphone ou email"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div class="md:col-span-2">
            <label for="wr-description" class="block text-sm text-gray-600 mb-1">Description</label>
            <textarea
              id="wr-description"
              bind:value={form.description}
              rows="3"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>
          <div class="md:col-span-2">
            <label for="wr-notes" class="block text-sm text-gray-600 mb-1">Notes</label>
            <textarea
              id="wr-notes"
              bind:value={form.notes}
              rows="2"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>
        </div>

        <div class="flex gap-2 pt-2">
          <button
            on:click={saveEdit}
            disabled={submitting}
            class="px-4 py-1.5 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50 transition"
          >
            {submitting ? "Enregistrement…" : "Enregistrer"}
          </button>
          <button
            on:click={cancelEdit}
            class="px-4 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300 transition"
          >
            Annuler
          </button>
        </div>
      </div>
    {/if}
  {/if}
</Modal>
