<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import {
    inspectionsApi,
    inspectionTypeLabels,
    inspectionStatusLabels,
    InspectionStatus,
    type TechnicalInspection,
    type UpdateInspectionDto,
  } from "../../lib/api/inspections";
  import { toast } from "../../stores/toast";
  import Modal from "../ui/Modal.svelte";

  export let isOpen = false;
  export let inspection: TechnicalInspection;

  const dispatch = createEventDispatcher();

  let editMode = false;
  let submitting = false;

  interface InspectionEditForm {
    title: string;
    description: string;
    inspector_name: string;
    inspector_company: string;
    inspector_certification: string;
    inspection_date_str: string;
    status: InspectionStatus;
    result_summary: string;
    defects_found: string;
    recommendations: string;
    compliant: boolean | null;
    compliance_certificate_number: string;
    cost: number | undefined;
    invoice_number: string;
    notes: string;
  }

  let form: InspectionEditForm = resetForm();

  function resetForm(): InspectionEditForm {
    return {
      title: inspection?.title ?? "",
      description: inspection?.description ?? "",
      inspector_name: inspection?.inspector_name ?? "",
      inspector_company: inspection?.inspector_company ?? "",
      inspector_certification: inspection?.inspector_certification ?? "",
      inspection_date_str: inspection?.inspection_date?.slice(0, 10) ?? "",
      status: inspection?.status ?? InspectionStatus.Pending,
      result_summary: inspection?.result_summary ?? "",
      defects_found: inspection?.defects_found ?? "",
      recommendations: inspection?.recommendations ?? "",
      compliant: inspection?.compliant ?? null,
      compliance_certificate_number: inspection?.compliance_certificate_number ?? "",
      cost: inspection?.cost,
      invoice_number: inspection?.invoice_number ?? "",
      notes: inspection?.notes ?? "",
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
    if (!form.title.trim() || !form.inspector_name.trim()) {
      toast.error("Le titre et l'inspecteur sont requis");
      return;
    }
    try {
      submitting = true;
      const dto: UpdateInspectionDto = {
        title: form.title,
        description: form.description || undefined,
        inspector_name: form.inspector_name,
        inspector_company: form.inspector_company || undefined,
        inspector_certification: form.inspector_certification || undefined,
        inspection_date: form.inspection_date_str
          ? new Date(form.inspection_date_str).toISOString()
          : undefined,
        status: form.status,
        result_summary: form.result_summary || undefined,
        defects_found: form.defects_found || undefined,
        recommendations: form.recommendations || undefined,
        compliant: form.compliant ?? undefined,
        compliance_certificate_number: form.compliance_certificate_number || undefined,
        cost: form.cost || undefined,
        invoice_number: form.invoice_number || undefined,
        notes: form.notes || undefined,
      };
      const updated = await inspectionsApi.update(inspection.id, dto);
      toast.success("Inspection mise à jour");
      inspection = updated;
      editMode = false;
      dispatch("updated", updated);
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la mise à jour");
    } finally {
      submitting = false;
    }
  }

  async function quickStatusUpdate(
    status: InspectionStatus,
    compliant: boolean | undefined,
  ) {
    try {
      const updated = await inspectionsApi.update(inspection.id, {
        status,
        compliant,
      });
      toast.success(`Statut mis à jour : ${inspectionStatusLabels[status]}`);
      inspection = updated;
      dispatch("updated", updated);
    } catch (err: any) {
      toast.error(err.message || "Erreur lors de la mise à jour du statut");
    }
  }

  async function handleDelete() {
    if (!confirm("Supprimer définitivement cette inspection ?")) return;
    try {
      await inspectionsApi.delete(inspection.id);
      toast.success("Inspection supprimée");
      dispatch("deleted", inspection.id);
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

  function statusColor(status: InspectionStatus): string {
    switch (status) {
      case InspectionStatus.Completed:
        return "bg-green-100 text-green-800";
      case InspectionStatus.Failed:
        return "bg-red-100 text-red-800";
      case InspectionStatus.PassedWithRemarks:
        return "bg-yellow-100 text-yellow-800";
      default:
        return "bg-gray-100 text-gray-800";
    }
  }
</script>

<Modal
  {isOpen}
  title={editMode ? "Modifier l'inspection" : "Inspection technique"}
  size="lg"
  on:close={handleClose}
>
  {#if inspection}
    {#if !editMode}
      <!-- Vue lecture -->
      <div class="space-y-4">
        <!-- Header avec actions -->
        <div class="flex items-start justify-between">
          <div>
            <h3 class="text-xl font-semibold text-gray-900">{inspection.title}</h3>
            <div class="flex items-center gap-2 mt-1">
              <span
                class="inline-flex items-center px-2.5 py-1 rounded-full text-sm font-medium {statusColor(inspection.status)}"
              >
                {inspectionStatusLabels[inspection.status] || inspection.status}
              </span>
              {#if inspection.is_overdue}
                <span
                  class="inline-flex items-center px-2.5 py-1 rounded-full text-sm font-medium bg-red-100 text-red-800"
                >
                  En retard
                </span>
              {/if}
            </div>
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

        <!-- Actions rapides de statut (seulement si En attente) -->
        {#if inspection.status === InspectionStatus.Pending}
          <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
            <p class="text-sm font-medium text-blue-800 mb-2">Enregistrer le résultat :</p>
            <div class="flex flex-wrap gap-2">
              <button
                on:click={() => quickStatusUpdate(InspectionStatus.Completed, true)}
                class="px-3 py-1.5 text-xs bg-green-600 text-white rounded hover:bg-green-700 transition"
              >
                ✅ Conforme
              </button>
              <button
                on:click={() => quickStatusUpdate(InspectionStatus.PassedWithRemarks, true)}
                class="px-3 py-1.5 text-xs bg-yellow-600 text-white rounded hover:bg-yellow-700 transition"
              >
                ⚠️ Conforme avec remarques
              </button>
              <button
                on:click={() => quickStatusUpdate(InspectionStatus.Failed, false)}
                class="px-3 py-1.5 text-xs bg-red-600 text-white rounded hover:bg-red-700 transition"
              >
                ❌ Non conforme
              </button>
            </div>
          </div>
        {/if}

        <!-- Grille d'infos -->
        <div class="grid grid-cols-2 gap-4 bg-gray-50 rounded-lg p-4 text-sm">
          <div>
            <p class="text-gray-500">Type d'inspection</p>
            <p class="font-medium text-gray-900">
              {inspectionTypeLabels[inspection.inspection_type] || inspection.inspection_type}
            </p>
          </div>
          <div>
            <p class="text-gray-500">Inspecteur</p>
            <p class="font-medium text-gray-900">{inspection.inspector_name}</p>
            {#if inspection.inspector_company}
              <p class="text-gray-600 text-xs mt-0.5">{inspection.inspector_company}</p>
            {/if}
          </div>
          <div>
            <p class="text-gray-500">Date de l'inspection</p>
            <p class="font-medium text-gray-900">{formatDate(inspection.inspection_date)}</p>
          </div>
          <div>
            <p class="text-gray-500">Prochaine inspection</p>
            <p
              class="font-medium {inspection.is_overdue
                ? 'text-red-700'
                : inspection.days_until_due < 30
                  ? 'text-yellow-700'
                  : 'text-gray-900'}"
            >
              {formatDate(inspection.next_due_date)}
            </p>
            <p class="text-xs text-gray-500 mt-0.5">
              {inspection.days_until_due > 0
                ? `dans ${inspection.days_until_due} j`
                : `${Math.abs(inspection.days_until_due)} j de retard`}
            </p>
          </div>
          {#if inspection.compliant !== null && inspection.compliant !== undefined}
            <div>
              <p class="text-gray-500">Conformité</p>
              <span
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {inspection.compliant
                  ? 'bg-green-100 text-green-800'
                  : 'bg-red-100 text-red-800'}"
              >
                {inspection.compliant ? "Conforme" : "Non conforme"}
              </span>
            </div>
          {/if}
          {#if inspection.cost}
            <div>
              <p class="text-gray-500">Coût</p>
              <p class="font-medium text-gray-900">
                {new Intl.NumberFormat("fr-BE", { style: "currency", currency: "EUR" }).format(
                  inspection.cost,
                )}
              </p>
              {#if inspection.invoice_number}
                <p class="text-gray-600 text-xs mt-0.5">Facture n° {inspection.invoice_number}</p>
              {/if}
            </div>
          {/if}
          {#if inspection.compliance_certificate_number}
            <div>
              <p class="text-gray-500">N° Certificat</p>
              <p class="font-medium text-gray-900">{inspection.compliance_certificate_number}</p>
            </div>
          {/if}
        </div>

        {#if inspection.description}
          <div>
            <p class="text-sm font-medium text-gray-700 mb-1">Description</p>
            <p class="text-sm text-gray-600">{inspection.description}</p>
          </div>
        {/if}

        {#if inspection.result_summary}
          <div>
            <p class="text-sm font-medium text-gray-700 mb-1">Résumé du résultat</p>
            <p class="text-sm text-gray-600">{inspection.result_summary}</p>
          </div>
        {/if}

        {#if inspection.defects_found}
          <div class="bg-red-50 border border-red-200 rounded-lg p-3">
            <p class="text-sm font-medium text-red-800 mb-1">Défauts constatés</p>
            <p class="text-sm text-red-700">{inspection.defects_found}</p>
          </div>
        {/if}

        {#if inspection.recommendations}
          <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
            <p class="text-sm font-medium text-yellow-800 mb-1">Recommandations</p>
            <p class="text-sm text-yellow-700">{inspection.recommendations}</p>
          </div>
        {/if}

        {#if inspection.notes}
          <div>
            <p class="text-sm font-medium text-gray-700 mb-1">Notes</p>
            <p class="text-sm text-gray-600">{inspection.notes}</p>
          </div>
        {/if}
      </div>

    {:else}
      <!-- Vue édition -->
      <div class="space-y-3">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          <div>
            <label for="insp-title" class="block text-sm text-gray-600 mb-1">Titre *</label>
            <input
              id="insp-title"
              bind:value={form.title}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="insp-status" class="block text-sm text-gray-600 mb-1">Statut</label>
            <select
              id="insp-status"
              bind:value={form.status}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              {#each Object.entries(inspectionStatusLabels) as [val, label]}
                <option value={val}>{label}</option>
              {/each}
            </select>
          </div>
          <div>
            <label for="insp-inspector" class="block text-sm text-gray-600 mb-1">Inspecteur *</label>
            <input
              id="insp-inspector"
              bind:value={form.inspector_name}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="insp-company" class="block text-sm text-gray-600 mb-1">Société</label>
            <input
              id="insp-company"
              bind:value={form.inspector_company}
              placeholder="Optionnel"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="insp-date" class="block text-sm text-gray-600 mb-1">Date de l'inspection</label>
            <input
              id="insp-date"
              type="date"
              bind:value={form.inspection_date_str}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="insp-compliant" class="block text-sm text-gray-600 mb-1">Conformité</label>
            <select
              id="insp-compliant"
              bind:value={form.compliant}
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            >
              <option value={null}>Non renseignée</option>
              <option value={true}>Conforme</option>
              <option value={false}>Non conforme</option>
            </select>
          </div>
          <div>
            <label for="insp-cost" class="block text-sm text-gray-600 mb-1">Coût (EUR)</label>
            <input
              id="insp-cost"
              type="number"
              bind:value={form.cost}
              min="0"
              step="0.01"
              placeholder="Optionnel"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div>
            <label for="insp-cert" class="block text-sm text-gray-600 mb-1">N° Certificat</label>
            <input
              id="insp-cert"
              bind:value={form.compliance_certificate_number}
              placeholder="Optionnel"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            />
          </div>
          <div class="md:col-span-2">
            <label for="insp-result" class="block text-sm text-gray-600 mb-1">Résumé du résultat</label>
            <textarea
              id="insp-result"
              bind:value={form.result_summary}
              rows="2"
              placeholder="Résumé de l'inspection"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>
          <div class="md:col-span-2">
            <label for="insp-defects" class="block text-sm text-gray-600 mb-1">Défauts constatés</label>
            <textarea
              id="insp-defects"
              bind:value={form.defects_found}
              rows="2"
              placeholder="Liste des défauts observés"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>
          <div class="md:col-span-2">
            <label for="insp-reco" class="block text-sm text-gray-600 mb-1">Recommandations</label>
            <textarea
              id="insp-reco"
              bind:value={form.recommendations}
              rows="2"
              placeholder="Actions correctives recommandées"
              class="w-full border rounded px-3 py-1.5 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
            ></textarea>
          </div>
          <div class="md:col-span-2">
            <label for="insp-notes" class="block text-sm text-gray-600 mb-1">Notes</label>
            <textarea
              id="insp-notes"
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
