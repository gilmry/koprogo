<script lang="ts">
  import { onMount } from "svelte";
  import { inspectionsApi, inspectionTypeLabels, inspectionStatusLabels, inspectionFrequencyLabels } from "../../lib/api/inspections";
  import type { TechnicalInspection, CreateInspectionDto } from "../../lib/api/inspections";
  import { InspectionType, InspectionStatus } from "../../lib/api/inspections";
  import { toast } from "../../stores/toast";

  export let buildingId: string;
  export let organizationId: string = "";

  let inspections: TechnicalInspection[] = [];
  let loading = true;
  let error = "";
  let showCreateForm = false;
  let activeTab: "all" | "overdue" | "upcoming" = "all";

  // Create form
  let form: Partial<CreateInspectionDto> = resetForm();

  function resetForm(): Partial<CreateInspectionDto> {
    return {
      title: "",
      description: "",
      inspection_type: InspectionType.Elevator,
      inspector_name: "",
      inspector_company: "",
      inspection_date: new Date().toISOString().split("T")[0],
      cost: undefined,
      notes: "",
    };
  }

  async function loadInspections() {
    loading = true;
    error = "";
    try {
      if (activeTab === "overdue") {
        inspections = await inspectionsApi.getOverdue(buildingId);
      } else if (activeTab === "upcoming") {
        inspections = await inspectionsApi.getUpcoming(buildingId, 90);
      } else {
        inspections = await inspectionsApi.listByBuilding(buildingId);
      }
    } catch (e: any) {
      error = e.message || "Erreur lors du chargement";
    } finally {
      loading = false;
    }
  }

  async function createInspection() {
    if (!form.title || !form.inspector_name) {
      toast.error("Le titre et le nom de l'inspecteur sont requis");
      return;
    }
    try {
      const data: CreateInspectionDto = {
        organization_id: organizationId,
        building_id: buildingId,
        title: form.title!,
        description: form.description || undefined,
        inspection_type: form.inspection_type || InspectionType.Elevator,
        inspector_name: form.inspector_name!,
        inspector_company: form.inspector_company || undefined,
        inspection_date: new Date(form.inspection_date!).toISOString(),
        cost: form.cost || undefined,
        notes: form.notes || undefined,
      };
      await inspectionsApi.create(data);
      toast.success("Inspection créée");
      form = resetForm();
      showCreateForm = false;
      await loadInspections();
    } catch (e: any) {
      toast.error(e.message || "Erreur lors de la création");
    }
  }

  async function deleteInspection(id: string) {
    if (!confirm("Supprimer cette inspection ?")) return;
    try {
      await inspectionsApi.delete(id);
      toast.success("Inspection supprimée");
      await loadInspections();
    } catch (e: any) {
      toast.error(e.message || "Erreur lors de la suppression");
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  function statusColor(status: string): string {
    switch (status) {
      case "completed": return "bg-green-100 text-green-800";
      case "failed": return "bg-red-100 text-red-800";
      case "passed_with_remarks": return "bg-yellow-100 text-yellow-800";
      default: return "bg-gray-100 text-gray-800";
    }
  }

  function switchTab(tab: "all" | "overdue" | "upcoming") {
    activeTab = tab;
    loadInspections();
  }

  $: overdueCount = inspections.filter((i) => i.is_overdue).length;

  onMount(loadInspections);
</script>

<div class="space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold text-gray-800">Inspections techniques</h2>
    <button
      on:click={() => (showCreateForm = !showCreateForm)}
      class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700"
    >
      {showCreateForm ? "Annuler" : "+ Nouvelle inspection"}
    </button>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class="bg-white shadow rounded-lg p-4 border border-blue-200">
      <h3 class="font-medium text-gray-800 mb-3">Planifier une inspection</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label class="block text-sm text-gray-600 mb-1">Titre *</label>
          <input bind:value={form.title} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Ex: Inspection annuelle ascenseur" />
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Type d'inspection</label>
          <select bind:value={form.inspection_type} class="w-full border rounded px-3 py-1.5 text-sm">
            {#each Object.entries(inspectionTypeLabels) as [val, label]}
              <option value={val}>{label} ({inspectionFrequencyLabels[val] || ""})</option>
            {/each}
          </select>
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Inspecteur *</label>
          <input bind:value={form.inspector_name} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Nom de l'inspecteur" />
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Société</label>
          <input bind:value={form.inspector_company} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Société d'inspection" />
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Date de l'inspection</label>
          <input type="date" bind:value={form.inspection_date} class="w-full border rounded px-3 py-1.5 text-sm" />
        </div>
        <div>
          <label class="block text-sm text-gray-600 mb-1">Coût (EUR)</label>
          <input type="number" bind:value={form.cost} min="0" step="0.01" class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Optionnel" />
        </div>
        <div class="md:col-span-2">
          <label class="block text-sm text-gray-600 mb-1">Description</label>
          <textarea bind:value={form.description} rows="2" class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Détails de l'inspection"></textarea>
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button on:click={createInspection} class="px-4 py-1.5 bg-green-600 text-white text-sm rounded hover:bg-green-700">Créer</button>
        <button on:click={() => (showCreateForm = false)} class="px-4 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300">Annuler</button>
      </div>
    </div>
  {/if}

  <!-- Tabs -->
  <div class="flex gap-2 border-b border-gray-200">
    <button on:click={() => switchTab("all")}
      class="px-3 py-2 text-sm border-b-2 {activeTab === 'all' ? 'border-blue-500 text-blue-600 font-medium' : 'border-transparent text-gray-500 hover:text-gray-700'}">
      Toutes ({inspections.length})
    </button>
    <button on:click={() => switchTab("overdue")}
      class="px-3 py-2 text-sm border-b-2 {activeTab === 'overdue' ? 'border-red-500 text-red-600 font-medium' : 'border-transparent text-gray-500 hover:text-gray-700'}">
      En retard {#if overdueCount > 0}<span class="ml-1 bg-red-100 text-red-800 px-1.5 py-0.5 rounded-full text-xs">{overdueCount}</span>{/if}
    </button>
    <button on:click={() => switchTab("upcoming")}
      class="px-3 py-2 text-sm border-b-2 {activeTab === 'upcoming' ? 'border-yellow-500 text-yellow-600 font-medium' : 'border-transparent text-gray-500 hover:text-gray-700'}">
      A venir (90j)
    </button>
  </div>

  <!-- Loading / Error / Empty -->
  {#if loading}
    <div class="text-center py-8 text-gray-500">
      <div class="animate-spin inline-block w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full"></div>
      <p class="mt-2 text-sm">Chargement...</p>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
      {error}
      <button on:click={loadInspections} class="ml-2 underline">Réessayer</button>
    </div>
  {:else if inspections.length === 0}
    <div class="text-center py-8 text-gray-400 text-sm">
      {activeTab === "overdue" ? "Aucune inspection en retard" :
       activeTab === "upcoming" ? "Aucune inspection prévue dans les 90 jours" :
       "Aucune inspection technique"}
    </div>
  {:else}
    <!-- Inspections list -->
    <div class="space-y-3">
      {#each inspections as inspection}
        <div class="bg-white shadow-sm rounded-lg p-4 border border-gray-200 {inspection.is_overdue ? 'border-l-4 border-l-red-500' : ''}">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="font-medium text-gray-800">{inspection.title}</h3>
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {statusColor(inspection.status)}">
                  {inspectionStatusLabels[inspection.status] || inspection.status}
                </span>
                {#if inspection.is_overdue}
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                    En retard
                  </span>
                {/if}
              </div>
              <p class="text-sm text-gray-600">
                {inspectionTypeLabels[inspection.inspection_type] || inspection.inspection_type}
                - {inspection.inspector_name}
                {#if inspection.inspector_company}({inspection.inspector_company}){/if}
              </p>
              {#if inspection.description}
                <p class="text-sm text-gray-500 mt-1 line-clamp-2">{inspection.description}</p>
              {/if}
              <div class="flex flex-wrap items-center gap-4 mt-2 text-xs text-gray-500">
                <span>Inspection: {formatDate(inspection.inspection_date)}</span>
                <span class="{inspection.days_until_due < 0 ? 'text-red-600 font-medium' : inspection.days_until_due < 30 ? 'text-yellow-600' : ''}">
                  Prochaine: {formatDate(inspection.next_due_date)}
                  ({inspection.days_until_due > 0 ? `dans ${inspection.days_until_due}j` : `${Math.abs(inspection.days_until_due)}j de retard`})
                </span>
                {#if inspection.compliant !== null && inspection.compliant !== undefined}
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full {inspection.compliant ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}">
                    {inspection.compliant ? "Conforme" : "Non conforme"}
                  </span>
                {/if}
                {#if inspection.cost}
                  <span>{new Intl.NumberFormat("fr-BE", { style: "currency", currency: "EUR" }).format(inspection.cost)}</span>
                {/if}
              </div>
              {#if inspection.defects_found}
                <div class="mt-2 bg-yellow-50 border border-yellow-200 rounded px-2 py-1 text-xs text-yellow-800">
                  Défauts: {inspection.defects_found}
                </div>
              {/if}
            </div>
            <button on:click={() => deleteInspection(inspection.id)} class="text-red-400 hover:text-red-600 p-1" title="Supprimer">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
