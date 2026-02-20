<script lang="ts">
  import { onMount } from "svelte";
  import { workReportsApi, workTypeLabels, warrantyTypeLabels } from "../../lib/api/work-reports";
  import type { WorkReport, CreateWorkReportDto } from "../../lib/api/work-reports";
  import { WorkType, WarrantyType } from "../../lib/api/work-reports";
  import { toast } from "../../stores/toast";
  import WorkReportDetail from "./WorkReportDetail.svelte";

  export let buildingId: string;
  export let organizationId: string = "";

  let reports: WorkReport[] = [];
  let loading = true;
  let error = "";
  let showCreateForm = false;
  let filterType = "all";
  let selectedReport: WorkReport | null = null;
  let detailOpen = false;

  // Create form
  let form: Partial<CreateWorkReportDto> = resetForm();

  function resetForm(): Partial<CreateWorkReportDto> {
    return {
      title: "",
      description: "",
      work_type: WorkType.Maintenance,
      contractor_name: "",
      contractor_contact: "",
      work_date: new Date().toISOString().split("T")[0],
      cost: 0,
      invoice_number: "",
      notes: "",
      warranty_type: WarrantyType.Standard,
    };
  }

  async function loadReports() {
    loading = true;
    error = "";
    try {
      reports = await workReportsApi.listByBuilding(buildingId);
    } catch (e: any) {
      error = e.message || "Erreur lors du chargement des rapports";
    } finally {
      loading = false;
    }
  }

  async function createReport() {
    if (!form.title || !form.contractor_name) {
      toast.error("Le titre et le nom de l'entrepreneur sont requis");
      return;
    }
    try {
      const data: CreateWorkReportDto = {
        organization_id: organizationId,
        building_id: buildingId,
        title: form.title!,
        description: form.description || "",
        work_type: form.work_type || WorkType.Maintenance,
        contractor_name: form.contractor_name!,
        contractor_contact: form.contractor_contact || undefined,
        work_date: new Date(form.work_date!).toISOString(),
        cost: form.cost || 0,
        invoice_number: form.invoice_number || undefined,
        notes: form.notes || undefined,
        warranty_type: form.warranty_type || WarrantyType.Standard,
      };
      await workReportsApi.create(data);
      toast.success("Rapport de travaux créé");
      form = resetForm();
      showCreateForm = false;
      await loadReports();
    } catch (e: any) {
      toast.error(e.message || "Erreur lors de la création");
    }
  }

  async function deleteReport(id: string) {
    if (!confirm("Supprimer ce rapport de travaux ?")) return;
    try {
      await workReportsApi.delete(id);
      toast.success("Rapport supprimé");
      await loadReports();
    } catch (e: any) {
      toast.error(e.message || "Erreur lors de la suppression");
    }
  }

  function openDetail(report: WorkReport) {
    selectedReport = report;
    detailOpen = true;
  }

  function handleDetailUpdated(event: CustomEvent<WorkReport>) {
    const updated = event.detail;
    reports = reports.map((r) => (r.id === updated.id ? updated : r));
  }

  function handleDetailDeleted(event: CustomEvent<string>) {
    reports = reports.filter((r) => r.id !== event.detail);
    detailOpen = false;
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE", {
      day: "numeric",
      month: "short",
      year: "numeric",
    });
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat("fr-BE", { style: "currency", currency: "EUR" }).format(amount);
  }

  $: filteredReports = filterType === "all"
    ? reports
    : reports.filter((r) => r.work_type === filterType);

  $: typeCounts = reports.reduce((acc, r) => {
    acc[r.work_type] = (acc[r.work_type] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  onMount(loadReports);
</script>

<div class="space-y-4">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <h2 class="text-lg font-semibold text-gray-800">Rapports de travaux</h2>
    <button
      on:click={() => (showCreateForm = !showCreateForm)}
      class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700"
    >
      {showCreateForm ? "Annuler" : "+ Nouveau rapport"}
    </button>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class="bg-white shadow rounded-lg p-4 border border-blue-200">
      <h3 class="font-medium text-gray-800 mb-3">Nouveau rapport de travaux</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label for="wr-new-title" class="block text-sm text-gray-600 mb-1">Titre *</label>
          <input id="wr-new-title" bind:value={form.title} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Ex: Réparation ascenseur" />
        </div>
        <div>
          <label for="wr-new-contractor" class="block text-sm text-gray-600 mb-1">Entrepreneur *</label>
          <input id="wr-new-contractor" bind:value={form.contractor_name} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Nom de l'entreprise" />
        </div>
        <div>
          <label for="wr-new-type" class="block text-sm text-gray-600 mb-1">Type de travaux</label>
          <select id="wr-new-type" bind:value={form.work_type} class="w-full border rounded px-3 py-1.5 text-sm">
            {#each Object.entries(workTypeLabels) as [val, label]}
              <option value={val}>{label}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="wr-new-date" class="block text-sm text-gray-600 mb-1">Date des travaux</label>
          <input id="wr-new-date" type="date" bind:value={form.work_date} class="w-full border rounded px-3 py-1.5 text-sm" />
        </div>
        <div>
          <label for="wr-new-cost" class="block text-sm text-gray-600 mb-1">Coût (EUR)</label>
          <input id="wr-new-cost" type="number" bind:value={form.cost} min="0" step="0.01" class="w-full border rounded px-3 py-1.5 text-sm" />
        </div>
        <div>
          <label for="wr-new-warranty" class="block text-sm text-gray-600 mb-1">Garantie</label>
          <select id="wr-new-warranty" bind:value={form.warranty_type} class="w-full border rounded px-3 py-1.5 text-sm">
            {#each Object.entries(warrantyTypeLabels) as [val, label]}
              <option value={val}>{label}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="wr-new-invoice" class="block text-sm text-gray-600 mb-1">N° Facture</label>
          <input id="wr-new-invoice" bind:value={form.invoice_number} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Optionnel" />
        </div>
        <div>
          <label for="wr-new-contact" class="block text-sm text-gray-600 mb-1">Contact entrepreneur</label>
          <input id="wr-new-contact" bind:value={form.contractor_contact} class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Téléphone ou email" />
        </div>
        <div class="md:col-span-2">
          <label for="wr-new-desc" class="block text-sm text-gray-600 mb-1">Description</label>
          <textarea id="wr-new-desc" bind:value={form.description} rows="2" class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Détails des travaux effectués"></textarea>
        </div>
        <div class="md:col-span-2">
          <label for="wr-new-notes" class="block text-sm text-gray-600 mb-1">Notes</label>
          <textarea id="wr-new-notes" bind:value={form.notes} rows="2" class="w-full border rounded px-3 py-1.5 text-sm" placeholder="Notes additionnelles"></textarea>
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button on:click={createReport} class="px-4 py-1.5 bg-green-600 text-white text-sm rounded hover:bg-green-700">Créer</button>
        <button on:click={() => (showCreateForm = false)} class="px-4 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300">Annuler</button>
      </div>
    </div>
  {/if}

  <!-- Filters -->
  <div class="flex flex-wrap gap-2">
    <button on:click={() => (filterType = "all")} class="px-3 py-1 text-xs rounded-full {filterType === 'all' ? 'bg-blue-100 text-blue-800 font-medium' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}">
      Tous ({reports.length})
    </button>
    {#each Object.entries(workTypeLabels) as [val, label]}
      {#if typeCounts[val]}
        <button on:click={() => (filterType = val)} class="px-3 py-1 text-xs rounded-full {filterType === val ? 'bg-blue-100 text-blue-800 font-medium' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}">
          {label} ({typeCounts[val]})
        </button>
      {/if}
    {/each}
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
      <button on:click={loadReports} class="ml-2 underline">Réessayer</button>
    </div>
  {:else if filteredReports.length === 0}
    <div class="text-center py-8 text-gray-400 text-sm">Aucun rapport de travaux</div>
  {:else}
    <!-- Reports list -->
    <div class="space-y-3">
      {#each filteredReports as report}
        <div
          class="bg-white shadow-sm rounded-lg p-4 border border-gray-200 hover:border-blue-300 transition-colors cursor-pointer"
          on:click={() => openDetail(report)}
          on:keydown={(e) => e.key === "Enter" && openDetail(report)}
          role="button"
          tabindex="0"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="font-medium text-gray-800">{report.title}</h3>
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium
                  {report.work_type === 'emergency' ? 'bg-red-100 text-red-800' :
                   report.work_type === 'repair' ? 'bg-orange-100 text-orange-800' :
                   report.work_type === 'renovation' ? 'bg-purple-100 text-purple-800' :
                   report.work_type === 'installation' ? 'bg-blue-100 text-blue-800' :
                   'bg-gray-100 text-gray-800'}">
                  {workTypeLabels[report.work_type] || report.work_type}
                </span>
              </div>
              <p class="text-sm text-gray-600">{report.contractor_name}</p>
              {#if report.description}
                <p class="text-sm text-gray-500 mt-1 line-clamp-2">{report.description}</p>
              {/if}
              <div class="flex flex-wrap items-center gap-4 mt-2 text-xs text-gray-500">
                <span>Date: {formatDate(report.work_date)}</span>
                <span>Coût: {formatCurrency(report.cost)}</span>
                {#if report.invoice_number}
                  <span>Facture: {report.invoice_number}</span>
                {/if}
                {#if report.warranty_type !== "none"}
                  <span class="inline-flex items-center px-2 py-0.5 rounded-full {report.is_warranty_valid ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}">
                    {report.is_warranty_valid ? `Garantie: ${report.warranty_days_remaining}j restants` : "Garantie expirée"}
                  </span>
                {/if}
                {#if report.photos.length > 0}
                  <span>{report.photos.length} photo(s)</span>
                {/if}
              </div>
            </div>
            <button
              on:click|stopPropagation={() => deleteReport(report.id)}
              class="text-red-400 hover:text-red-600 p-1"
              title="Supprimer"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if selectedReport}
  <WorkReportDetail
    isOpen={detailOpen}
    report={selectedReport}
    on:close={() => (detailOpen = false)}
    on:updated={handleDetailUpdated}
    on:deleted={handleDetailDeleted}
  />
{/if}
