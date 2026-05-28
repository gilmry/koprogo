<script lang="ts">
  // Story 2.3 — ContextBanner (bannière contextuelle 3 niveaux).
  //
  // ADR-0012 (Navigation contextualisée).
  //
  // Affiche `Cabinet · ACP · Immeuble` sous le header AppLayout dès qu'un
  // building est sélectionné dans le store `scope`. La couleur de l'icône
  // de conformité reflète l'état :
  // - vert : `is_conformant === true`
  // - orange : `is_conformant === false && quota_delta !== 0` (units OK mais
  //   somme quotas hors 1000 — warning)
  // - rouge : `is_conformant === false && (quota_sum === "0" || delta négatif
  //   significatif)` (immeuble non publiable)
  //
  // Cas particuliers (cf. tests Vitest 4-cat) :
  // - @edge ACP auto-gérée (`acp.organization_id === null`) → 2 niveaux
  //   (Cabinet masqué, pas de placeholder vide).
  // - @security organization non résolvable (403 silencieux côté tryGetOrgName)
  //   → masque Cabinet par défense cross-tenant.
  // - @negative aucun building sélectionné → composant retourne null.
  //
  // data-testid (contrat stable, i18n-safe — cf. memory data-testid-systematic) :
  //   context-banner, -cabinet, -acp, -building, -conformity-icon
  //
  // Pourquoi pas i18n hardcodé sur les niveaux : les noms (Cabinet, ACP,
  // Immeuble) sont des données, pas des labels. On affiche les VALEURS
  // (ex: "Cabinet Maury") avec un séparateur visuel `·`, pas des labels.

  import { onMount } from "svelte";
  import { _ } from "../../lib/i18n";
  import { scope } from "../../stores/scope.svelte";
  import type { Building } from "../../lib/types";
  import { getBuilding } from "../../lib/api/buildings";
  import { getAcp, type AcpResponseDto } from "../../lib/api/acps";
  import { tryGetOrganizationName } from "../../lib/api/organizations";

  // -------------------------------------------------------------------------
  // State local — détails enrichis après fetch lazy
  // -------------------------------------------------------------------------

  /** Building détaillé (avec metrics conformité). null = pas encore fetché. */
  let buildingDetail = $state<Building | null>(null);
  /** ACP parente (récupéré via /acps/{acp_id}). */
  let acpDetail = $state<AcpResponseDto | null>(null);
  /** Nom du cabinet syndic. null = soit ACP auto-gérée, soit fetch interdit. */
  let cabinetName = $state<string | null>(null);
  /** ID du building en cours de fetch, pour éviter les race conditions. */
  let inFlightBuildingId = $state<string | null>(null);

  // -------------------------------------------------------------------------
  // Effect — fetch détails quand le building sélectionné change
  // -------------------------------------------------------------------------

  $effect(() => {
    const id = scope.selectedBuildingId;
    if (!id) {
      buildingDetail = null;
      acpDetail = null;
      cabinetName = null;
      inFlightBuildingId = null;
      return;
    }
    // Évite un re-fetch si déjà en cours pour le même id.
    if (inFlightBuildingId === id) return;
    inFlightBuildingId = id;
    void loadDetails(id);
  });

  async function loadDetails(buildingId: string): Promise<void> {
    try {
      const b = await getBuilding(buildingId);
      // Anti-race : si le scope a changé pendant le fetch, on jette.
      if (scope.selectedBuildingId !== buildingId) return;
      buildingDetail = b;

      // Story 1.2 : Building expose `acp_id` (anciennement `organization_id`).
      // Le DTO TypeScript actuel garde `organization_id` legacy le temps du
      // rebranding FE (cf. BuildingSelector lignes 96-100). On lit les deux.
      const acpId =
        (b as Building & { acp_id?: string }).acp_id ??
        b.organization_id ??
        null;

      if (acpId) {
        try {
          const acp = await getAcp(acpId);
          if (scope.selectedBuildingId !== buildingId) return;
          acpDetail = acp;

          // Cabinet : seulement si l'ACP a un organization_id (cf. @edge).
          // tryGetOrganizationName est conçu pour catcher 403 silencieusement
          // (option `silent: true` dans api.get → pas de toast "Accès refusé"
          // pour cet appel optionnel non-admin).
          if (acp.organization_id) {
            const name = await tryGetOrganizationName(acp.organization_id);
            if (scope.selectedBuildingId !== buildingId) return;
            cabinetName = name; // null si non résolvable → masque
          } else {
            cabinetName = null;
          }
        } catch {
          // ACP non résolvable — on n'affiche que le building.
          acpDetail = null;
          cabinetName = null;
        }
      }
    } catch {
      // Fetch building échoué — composant reste vide gracieusement.
      buildingDetail = null;
      acpDetail = null;
      cabinetName = null;
    } finally {
      inFlightBuildingId = null;
    }
  }

  // -------------------------------------------------------------------------
  // Dérivations couleurs conformité (cf. Story 1.4 + ConformityBadge)
  // -------------------------------------------------------------------------

  let iconColorClass = $derived.by(() => {
    const b = buildingDetail;
    if (!b) return "bg-gray-300 text-gray-700"; // chargement
    if (b.is_conformant) return "bg-green-500 text-white";
    // Non conformant : on distingue warning (orange) vs erreur (rouge)
    // par la `quota_sum`. quota_sum === "0" → bâtiment vide → rouge.
    const sum = (b.quota_sum ?? "0").trim();
    if (sum === "0" || sum === "0.0" || sum === "")
      return "bg-red-500 text-white";
    return "bg-orange-500 text-white";
  });

  let iconSymbol = $derived.by(() => {
    const b = buildingDetail;
    if (!b) return "…";
    if (b.is_conformant) return "✓";
    const sum = (b.quota_sum ?? "0").trim();
    if (sum === "0" || sum === "0.0" || sum === "") return "✕";
    return "!";
  });

  let iconAriaLabel = $derived.by(() => {
    const b = buildingDetail;
    if (!b)
      return (
        $_("contextBanner.conformity.loading") || "Conformité : chargement"
      );
    if (b.is_conformant)
      return $_("contextBanner.conformity.green") || "Conformité : conforme";
    const sum = (b.quota_sum ?? "0").trim();
    if (sum === "0" || sum === "0.0" || sum === "")
      return (
        $_("contextBanner.conformity.red") || "Conformité : non conforme (vide)"
      );
    return $_("contextBanner.conformity.orange") || "Conformité : à vérifier";
  });

  // -------------------------------------------------------------------------
  // Visibility — composant null si pas de building sélectionné (@negative)
  // -------------------------------------------------------------------------

  let visible = $derived(scope.selectedBuildingId !== null);
  let buildingNameDisplay = $derived(
    buildingDetail?.name ?? scope.selectedBuilding?.name ?? "",
  );
  let acpNameDisplay = $derived(acpDetail?.name ?? "");
  let showCabinet = $derived(cabinetName !== null && cabinetName !== "");
  let showAcp = $derived(acpNameDisplay !== "");
</script>

{#if visible}
  <div
    class="context-banner flex items-center gap-2 px-4 py-2 border-b border-gray-200 bg-white text-sm"
    data-testid="context-banner"
    role="region"
    aria-label={$_("contextBanner.label") || "Contexte courant"}
  >
    <!-- Icône conformité (toujours présente quand banner visible) -->
    <span
      data-testid="context-banner-conformity-icon"
      class="inline-flex h-6 w-6 items-center justify-center rounded-full text-xs font-bold {iconColorClass}"
      role="img"
      aria-label={iconAriaLabel}
    >
      <span aria-hidden="true">{iconSymbol}</span>
    </span>

    <!-- Niveau 1 — Cabinet (peut être absent si ACP auto-gérée ou cross-tenant) -->
    {#if showCabinet}
      <span
        data-testid="context-banner-cabinet"
        class="font-medium text-gray-700"
      >
        {cabinetName}
      </span>
      {#if showAcp}
        <span aria-hidden="true" class="text-gray-400">·</span>
      {/if}
    {/if}

    <!-- Niveau 2 — ACP -->
    {#if showAcp}
      <span data-testid="context-banner-acp" class="font-medium text-gray-800">
        {acpNameDisplay}
      </span>
      <span aria-hidden="true" class="text-gray-400">·</span>
    {/if}

    <!-- Niveau 3 — Immeuble (toujours présent quand banner visible) -->
    <span
      data-testid="context-banner-building"
      class="font-semibold text-gray-900"
    >
      {buildingNameDisplay}
    </span>
  </div>
{/if}

<style>
  .context-banner {
    /* Bandeau plein largeur sous le header, scrollable horizontalement si
       les noms cumulés dépassent en mobile. */
    width: 100%;
    overflow-x: auto;
    white-space: nowrap;
  }
</style>
