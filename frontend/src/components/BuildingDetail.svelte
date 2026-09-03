<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "../lib/i18n";
  import { api } from "../lib/api";
  import { toast } from "../stores/toast";
  import type { Building } from "../lib/types";
  import { getAcp } from "../lib/api/acps";
  import { tryGetOrganizationName } from "../lib/api/organizations";
  import BuildingForm from "./admin/BuildingForm.svelte";
  import Button from "./ui/Button.svelte";
  import UnitList from "./UnitList.svelte";
  import ExpenseList from "./ExpenseList.svelte";
  import MeetingList from "./MeetingList.svelte";
  import DocumentList from "./DocumentList.svelte";
  import BuildingFinancialReports from "./BuildingFinancialReports.svelte";
  import WorkReportList from "./work-reports/WorkReportList.svelte";
  import InspectionList from "./inspections/InspectionList.svelte";
  // Story 1.4 — badge conformité (#553 Bugs 1/3/4 + FR11)
  import ConformityBadge from "./buildings/ConformityBadge.svelte";
  // Track H Story H1 — banner narratif conformité (FR-H1 + INV-H1)
  import ConformityBanner from "../lib/components/shared/ConformityBanner.svelte";
  import { buildConformityStatus } from "../lib/utils/conformity";

  let building: Building | null = null;
  let loading = true;
  let error = "";
  let showEditModal = false;
  let buildingId: string = "";
  let organizationName: string = "";
  let organizationId: string = "";

  // Track H Story H1 — Statut conformité dérivé du DTO BE.
  // `conformityStatus = null` tant que `building` n'est pas chargé ou que
  // le DTO n'expose pas encore les champs metrics (Story 1.4).
  // `canCompute` propage l'autorisation de déclencher des calculs (charges,
  // appels de fonds, etc.) — bouton « Modifier » reste actif (corrige drift).
  $: conformityStatus =
    building && building.is_conformant !== undefined
      ? buildConformityStatus({
          is_conformant: !!building.is_conformant,
          total_units: building.total_units,
          units_count: building.units_count ?? 0,
          total_tantiemes: building.total_tantiemes,
          quota_delta: building.quota_delta ?? "0",
        })
      : null;
  $: canCompute = conformityStatus ? conformityStatus.is_conformant : true;

  onMount(() => {
    // Get building ID from URL query params
    const urlParams = new URLSearchParams(window.location.search);
    buildingId = urlParams.get("id") || "";

    if (!buildingId) {
      error = $_("buildings.idMissing");
      loading = false;
      return;
    }

    loadBuilding();
  });

  async function loadBuilding() {
    try {
      loading = true;
      error = "";
      building = await api.get<Building>(`/buildings/${buildingId}`);

      // Résout le nom du cabinet syndic via l'ACP (building.acp_id ->
      // acp.organization_id -> nom). Dégrade silencieusement (403 syndic/
      // owner, ACP auto-gérée sans organisation) — cf. tryGetOrganizationName.
      if (building && building.acp_id) {
        try {
          const acp = await getAcp(building.acp_id);
          organizationId = acp.organization_id ?? "";
          organizationName = acp.organization_id
            ? (await tryGetOrganizationName(acp.organization_id)) ?? ""
            : "";
        } catch (e) {
          console.error("Error loading ACP/organization:", e);
          organizationName = "";
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : $_("buildings.errorLoading");
      console.error("Error loading building:", e);
    } finally {
      loading = false;
    }
  }

  const handleEdit = () => {
    showEditModal = true;
  };

  const handleEditSuccess = async () => {
    showEditModal = false;
    await loadBuilding();
  };

  const handleGoBack = () => {
    window.history.back();
  };
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"
        ></div>
        <p class="mt-4 text-gray-600">{$_("common.loading")}</p>
      </div>
    </div>
  {:else if error}
    <div
      class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg"
    >
      ⚠️ {error}
    </div>
    <div class="mt-4">
      <Button variant="outline" onclick={handleGoBack}>
        ← {$_("common.back")}
      </Button>
    </div>
  {:else if building}
    <!-- Track H Story H1 — Banner narratif conformité (au-dessus du titre).
         Rendu UNIQUEMENT si non-conforme — sinon DOM-absent (cf. Vitest @happy). -->
    {#if conformityStatus}
      <ConformityBanner
        status={conformityStatus}
        buildingId={building.id}
        buildingName={building.name}
      />
    {/if}

    <!-- Header -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <button
            onclick={handleGoBack}
            class="text-gray-600 hover:text-gray-900"
          >
            ← {$_("common.back")}
          </button>
          <h1
            class="text-3xl font-bold text-gray-900"
            data-testid="building-detail-name"
            data-can-compute={canCompute}
          >
            {building.name}
          </h1>
        </div>
        <Button
          variant="primary"
          onclick={handleEdit}
          data-testid="building-edit-submit"
        >
          ✏️ {$_("common.edit")}
        </Button>
      </div>
    </div>

    <!-- Story 1.4 — Badge conformité immeuble (#553 Bugs 1/3/4 + FR11/FR12/FR23) -->
    {#if building.is_conformant !== undefined}
      <div class="mb-6">
        <ConformityBadge
          isConformant={building.is_conformant}
          unitsCount={building.units_count ?? 0}
          totalUnits={building.total_units}
          quotaSum={building.quota_sum ?? "0"}
          quotaDelta={building.quota_delta ?? "0"}
        />
      </div>
    {/if}

    <!-- Building Info Card -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <h2 class="text-xl font-semibold text-white">{$_("buildings.info")}</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h3
              class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("buildings.address")}
            </h3>
            <p class="text-lg text-gray-900">{building.address}</p>
            <p class="text-gray-600">{building.postal_code} {building.city}</p>
            <p class="text-gray-600">
              {building.country || $_("buildings.defaultCountry")}
            </p>
          </div>
          <div>
            <h3
              class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2"
            >
              {$_("buildings.details")}
            </h3>
            <div class="space-y-2">
              {#if organizationName}
                <div class="flex items-center">
                  <span class="text-gray-600"
                    >🏛️ {$_("buildings.organization")}:</span
                  >
                  <span class="ml-2 font-semibold text-gray-900"
                    >{organizationName}</span
                  >
                </div>
              {/if}
              <div class="flex items-center">
                <span class="text-gray-600"
                  >🏢 {$_("buildings.unitCount")}:</span
                >
                <!-- Story 1.4 — count RÉEL depuis API (units_count), fallback total_units si pas encore peuplé -->
                <span
                  class="ml-2 font-semibold text-gray-900"
                  data-testid="building-units-count-detail"
                >
                  {#if building.units_count !== undefined}
                    {building.units_count} / {building.total_units}
                  {:else}
                    {building.total_units}
                  {/if}
                </span>
              </div>
              <div class="flex items-center">
                <span class="text-gray-600"
                  >📊 {$_("buildings.totalTantiemes")}:</span
                >
                <!-- Story 1.4 — somme RÉELLE (Decimal-as-string), jamais parseFloat. Fallback total_tantiemes legacy. -->
                <span
                  class="ml-2 font-semibold text-gray-900"
                  data-testid="building-quota-sum-detail"
                >
                  {#if building.quota_sum !== undefined && building.quota_sum !== ""}
                    {building.quota_sum.replace(".", ",")}
                  {:else}
                    {building.total_tantiemes}
                  {/if}
                  {$_("buildings.millioths")}
                </span>
              </div>
              {#if building.construction_year}
                <div class="flex items-center">
                  <span class="text-gray-600"
                    >🏗️ {$_("buildings.constructionYear")}:</span
                  >
                  <span class="ml-2 font-semibold text-gray-900"
                    >{building.construction_year}</span
                  >
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Related Data Sections -->
    <div class="space-y-8">
      <!-- Units Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.units")}
        </h3>
        <UnitList {buildingId} />
      </div>

      <!-- Expenses Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.expenses")}
        </h3>
        <ExpenseList {buildingId} />
      </div>

      <!-- Meetings Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.meetings")}
        </h3>
        <MeetingList {buildingId} />
      </div>

      <!-- Documents Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.documents")}
        </h3>
        <DocumentList {buildingId} />
      </div>

      <!-- Work Reports Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.workReports")}
        </h3>
        <WorkReportList {buildingId} {organizationId} />
      </div>

      <!-- Technical Inspections Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {$_("buildings.technicalInspections")}
        </h3>
        <InspectionList {buildingId} />
      </div>

      <!-- Financial Reports Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          📊 {$_("buildings.financialReports")}
        </h3>
        <p class="text-sm text-gray-600 mb-4">
          {$_("buildings.financialReportsDesc")}
        </p>
        <BuildingFinancialReports {buildingId} buildingName={building.name} />
      </div>
    </div>
  {/if}
</div>

<!-- Edit Modal -->
{#if building}
  <BuildingForm
    isOpen={showEditModal}
    {building}
    mode="edit"
    onsuccess={handleEditSuccess}
    onclose={() => (showEditModal = false)}
  />
{/if}
