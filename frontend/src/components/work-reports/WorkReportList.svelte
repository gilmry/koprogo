<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from "../../lib/i18n";
  import {
    workReportsApi,
    workTypeLabels,
    warrantyTypeLabels,
  } from "../../lib/api/work-reports";
  import type {
    WorkReport,
    CreateWorkReportDto,
  } from "../../lib/api/work-reports";
  import { WorkType, WarrantyType } from "../../lib/api/work-reports";
  import { toast } from "../../stores/toast";
  import WorkReportDetail from "./WorkReportDetail.svelte";
  import { formatDate } from "../../lib/utils/date.utils";
  import { formatCurrency } from "../../lib/utils/finance.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";
  import ConfirmDialog from "../ui/ConfirmDialog.svelte";

  let {
    buildingId,
    organizationId = "",
    showHeader = true,
  }: {
    buildingId: string;
    organizationId?: string;
    // La page dédiée porte déjà un H1 identique ; l'en-tête interne y faisait
    // doublon. Conservé par défaut pour la fiche immeuble, où le composant
    // est une section parmi d'autres et a besoin de son étiquette.
    showHeader?: boolean;
  } = $props();

  let reports: WorkReport[] = $state([]);

  // L'action en attente de confirmation.
  //
  // Ce composant est en mode RUNES : un `let` simple n'y est PAS réactif.
  // `svelte-check --fail-on-warnings` l'a dit — « is updated, but is not
  // declared with $state(...) » — après que je l'avais pris pour du legacy.
  // C'est très exactement le défaut de #832, attrapé cette fois par le
  // barrage plutôt qu'en recette.
  //
  // Le `confirm()` remplacé était un dialogue du NAVIGATEUR : un navigateur
  // piloté le supprime, et l'action prend la forme exacte d'une panne (#844).
  //
  // Un rapport de travaux alimente le carnet d'entretien de l'immeuble.
  let suppressionEnAttente = $state(false);
  let cibleEnAttente = $state<string | null>(null);
  let loading = $state(true);
  let error = $state("");
  let showCreateForm = $state(false);
  let filterType = $state("all");
  let selectedReport = $state<WorkReport | null>(null);
  let detailOpen = $state(false);

  // Create form
  let form: Partial<CreateWorkReportDto> = $state(resetForm());

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
    const result = await withErrorHandling({
      action: () => workReportsApi.listByBuilding(buildingId),
      errorMessage: $_("workReports.loadError"),
    });
    if (result) {
      reports = result;
    } else {
      error = $_("workReports.loadError");
    }
    loading = false;
  }

  async function createReport() {
    if (!form.title || !form.contractor_name) {
      toast.error($_("workReports.titleAndContractorRequired"));
      return;
    }
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
    const result = await withErrorHandling({
      action: () => workReportsApi.create(data),
      successMessage: $_("workReports.createSuccess"),
      errorMessage: $_("common.createError"),
      onSuccess: () => {
        form = resetForm();
        showCreateForm = false;
      },
    });
    if (result) await loadReports();
  }

  function deleteReport(id: string) {
    cibleEnAttente = id;
    suppressionEnAttente = true;
  }

  async function executerSuppression() {
    suppressionEnAttente = false;
    const id = cibleEnAttente;
    cibleEnAttente = null;
    if (!id) return;
    const result = await withErrorHandling({
      action: () => workReportsApi.delete(id),
      successMessage: $_("workReports.deleteSuccess"),
      errorMessage: $_("workReports.deleteError"),
    });
    if (result !== undefined) await loadReports();
  }

  function openDetail(report: WorkReport) {
    selectedReport = report;
    detailOpen = true;
  }

  function handleDetailUpdated(updated: WorkReport) {
    reports = reports.map((r) => (r.id === updated.id ? updated : r));
  }

  function handleDetailDeleted(id: string) {
    reports = reports.filter((r) => r.id !== id);
    detailOpen = false;
  }

  let filteredReports = $derived(
    filterType === "all"
      ? reports
      : reports.filter((r) => r.work_type === filterType),
  );

  let typeCounts = $derived(
    reports.reduce(
      (acc, r) => {
        acc[r.work_type] = (acc[r.work_type] || 0) + 1;
        return acc;
      },
      {} as Record<string, number>,
    ),
  );

  $effect(() => {
    loadReports();
  });
</script>

<div class="space-y-4" data-testid="work-report-list">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <!-- Le div vide conserve l'alignement : la rangée est en
         justify-between, sans lui le bouton d'action remonterait à gauche. -->
    {#if showHeader}
      <h2 class="text-lg font-semibold text-gray-800">
        {$_("workReports.title")}
      </h2>
    {:else}
      <div></div>
    {/if}
    <button
      data-testid="work-reports-create-toggle-button"
      onclick={() => (showCreateForm = !showCreateForm)}
      class="px-3 py-1.5 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700"
    >
      {showCreateForm
        ? $_("common.cancel")
        : "+ " + $_("workReports.newReport")}
    </button>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class="bg-white shadow rounded-lg p-4 border border-blue-200">
      <h3 class="font-medium text-gray-800 mb-3">
        {$_("workReports.newReport")}
      </h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label for="wr-new-title" class="block text-sm text-gray-600 mb-1"
            >{$_("common.title")} *</label
          >
          <input
            data-testid="wr-new-title"
            id="wr-new-title"
            bind:value={form.title}
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("workReports.titlePlaceholder")}
          />
        </div>
        <div>
          <label
            for="wr-new-contractor"
            class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.contractor")} *</label
          >
          <input
            data-testid="wr-new-contractor"
            id="wr-new-contractor"
            bind:value={form.contractor_name}
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("workReports.contractorPlaceholder")}
          />
        </div>
        <div>
          <label for="wr-new-type" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.workType")}</label
          >
          <select
            data-testid="wr-new-type"
            id="wr-new-type"
            bind:value={form.work_type}
            class="w-full border rounded px-3 py-1.5 text-sm"
          >
            {#each Object.entries(workTypeLabels) as [val, label]}
              <option value={val}>{label}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="wr-new-date" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.workDate")}</label
          >
          <input
            data-testid="wr-new-date"
            id="wr-new-date"
            type="date"
            bind:value={form.work_date}
            class="w-full border rounded px-3 py-1.5 text-sm"
          />
        </div>
        <div>
          <label for="wr-new-cost" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.cost")}</label
          >
          <input
            data-testid="wr-new-cost"
            id="wr-new-cost"
            type="number"
            bind:value={form.cost}
            min="0"
            step="0.01"
            class="w-full border rounded px-3 py-1.5 text-sm"
          />
        </div>
        <div>
          <label for="wr-new-warranty" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.warranty")}</label
          >
          <select
            data-testid="wr-new-warranty"
            id="wr-new-warranty"
            bind:value={form.warranty_type}
            class="w-full border rounded px-3 py-1.5 text-sm"
          >
            {#each Object.entries(warrantyTypeLabels) as [val, label]}
              <option value={val}>{label}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="wr-new-invoice" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.invoiceNumber")}</label
          >
          <input
            data-testid="wr-new-invoice"
            id="wr-new-invoice"
            bind:value={form.invoice_number}
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("common.optional")}
          />
        </div>
        <div>
          <label for="wr-new-contact" class="block text-sm text-gray-600 mb-1"
            >{$_("workReports.contactContractor")}</label
          >
          <input
            data-testid="wr-new-contact"
            id="wr-new-contact"
            bind:value={form.contractor_contact}
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("workReports.phoneOrEmail")}
          />
        </div>
        <div class="md:col-span-2">
          <label for="wr-new-desc" class="block text-sm text-gray-600 mb-1"
            >{$_("common.description")}</label
          >
          <textarea
            data-testid="wr-new-desc"
            id="wr-new-desc"
            bind:value={form.description}
            rows="2"
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("workReports.descriptionPlaceholder")}></textarea>
        </div>
        <div class="md:col-span-2">
          <label for="wr-new-notes" class="block text-sm text-gray-600 mb-1"
            >{$_("common.notes")}</label
          >
          <textarea
            data-testid="wr-new-notes"
            id="wr-new-notes"
            bind:value={form.notes}
            rows="2"
            class="w-full border rounded px-3 py-1.5 text-sm"
            placeholder={$_("workReports.notesPlaceholder")}></textarea>
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button
          data-testid="work-reports-create-submit-button"
          onclick={createReport}
          class="px-4 py-1.5 bg-green-600 text-white text-sm rounded hover:bg-green-700"
          >{$_("common.create")}</button
        >
        <button
          data-testid="work-reports-create-cancel-button"
          onclick={() => (showCreateForm = false)}
          class="px-4 py-1.5 bg-gray-200 text-gray-700 text-sm rounded hover:bg-gray-300"
          >{$_("common.cancel")}</button
        >
      </div>
    </div>
  {/if}

  <!-- Filters -->
  <div class="flex flex-wrap gap-2">
    <button
      data-testid="work-reports-filter-all-button"
      onclick={() => (filterType = "all")}
      class="px-3 py-1 text-xs rounded-full {filterType === 'all'
        ? 'bg-blue-100 text-blue-800 font-medium'
        : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
    >
      {$_("common.all")} ({reports.length})
    </button>
    {#each Object.entries(workTypeLabels) as [val, label]}
      {#if typeCounts[val]}
        <button
          data-testid="work-reports-filter-button"
          onclick={() => (filterType = val)}
          class="px-3 py-1 text-xs rounded-full {filterType === val
            ? 'bg-blue-100 text-blue-800 font-medium'
            : 'bg-gray-100 text-gray-600 hover:bg-gray-200'}"
        >
          {label} ({typeCounts[val]})
        </button>
      {/if}
    {/each}
  </div>

  <!-- Loading / Error / Empty -->
  {#if loading}
    <div class="text-center py-8 text-gray-500">
      <div
        class="animate-spin inline-block w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full"
        data-testid="work-report-list-spinner"
      ></div>
      <p class="mt-2 text-sm">{$_("common.loading")}</p>
    </div>
  {:else if error}
    <div
      class="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700"
    >
      {error}
      <button
        data-testid="work-reports-retry-button"
        onclick={loadReports}
        class="ml-2 underline">{$_("common.retry")}</button
      >
    </div>
  {:else if filteredReports.length === 0}
    <div class="text-center py-8 text-gray-400 text-sm">
      {$_("workReports.none")}
    </div>
  {:else}
    <!-- Reports list -->
    <div class="space-y-3">
      {#each filteredReports as report}
        <div
          class="bg-white shadow-sm rounded-lg p-4 border border-gray-200 hover:border-blue-300 transition-colors cursor-pointer"
          data-testid="work-report-row"
          onclick={() => openDetail(report)}
          onkeydown={(e) => e.key === "Enter" && openDetail(report)}
          role="button"
          tabindex="0"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-1">
                <h3 class="font-medium text-gray-800">{report.title}</h3>
                <span
                  class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium
                  {report.work_type === 'emergency'
                    ? 'bg-red-100 text-red-800'
                    : report.work_type === 'repair'
                      ? 'bg-orange-100 text-orange-800'
                      : report.work_type === 'renovation'
                        ? 'bg-purple-100 text-purple-800'
                        : report.work_type === 'installation'
                          ? 'bg-blue-100 text-blue-800'
                          : 'bg-gray-100 text-gray-800'}"
                >
                  {workTypeLabels[report.work_type] || report.work_type}
                </span>
              </div>
              <p class="text-sm text-gray-600">{report.contractor_name}</p>
              {#if report.description}
                <p class="text-sm text-gray-500 mt-1 line-clamp-2">
                  {report.description}
                </p>
              {/if}
              <div
                class="flex flex-wrap items-center gap-4 mt-2 text-xs text-gray-500"
              >
                <span
                  >{$_("workReports.workDate")}: {formatDate(
                    report.work_date,
                  )}</span
                >
                <span
                  >{$_("workReports.cost")}: {formatCurrency(report.cost)}</span
                >
                {#if report.invoice_number}
                  <span
                    >{$_("workReports.invoiceNumber")}: {report.invoice_number}</span
                  >
                {/if}
                {#if report.warranty_type !== "none"}
                  <span
                    class="inline-flex items-center px-2 py-0.5 rounded-full {report.is_warranty_valid
                      ? 'bg-green-100 text-green-800'
                      : 'bg-red-100 text-red-800'}"
                  >
                    {report.is_warranty_valid
                      ? $_("workReports.warrantyActive", {
                          values: { days: report.warranty_days_remaining },
                        })
                      : $_("workReports.warrantyExpired")}
                  </span>
                {/if}
                {#if report.photos.length > 0}
                  <span>{report.photos.length} {$_("common.photos")}</span>
                {/if}
              </div>
            </div>
            <button
              data-testid="work-reports-delete-button"
              onclick={(e) => {
                e.stopPropagation();
                deleteReport(report.id);
              }}
              class="text-red-400 hover:text-red-600 p-1"
              aria-label={$_("common.delete")}
              title={$_("common.delete")}
            >
              <svg
                class="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                ><path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                /></svg
              >
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
    onclose={() => (detailOpen = false)}
    onupdated={(updated) => handleDetailUpdated(updated)}
    ondeleted={(id) => handleDetailDeleted(id)}
  />
{/if}

<!-- Le dialogue qui remplace un `confirm()` natif (#844). -->
<ConfirmDialog
  isOpen={suppressionEnAttente}
  title={$_("common.confirm")}
  message={$_("workReports.deleteConfirm")}
  variant="danger"
  onconfirm={executerSuppression}
  oncancel={() => {
    suppressionEnAttente = false;
    cibleEnAttente = null;
  }}
/>
