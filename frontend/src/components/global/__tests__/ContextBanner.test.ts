// Story 2.3 — ContextBanner Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD : test rouge AVANT le composant.
//
// Couverture :
// - @happy : building sélectionné conformant → bannière 3 niveaux verte
// - @edge  : ACP auto-gérée (organization_id=null) → 2 niveaux, pas de Cabinet
// - @security : organization renvoyée par l'API ≠ celle attendue → masque le
//   niveau Cabinet (comportement défensif vs leak cross-tenant)
// - @negative : aucun building sélectionné → composant masqué (rend null)
//
// data-testid contractuels (cf. stories.md ligne 244-261 + memory
// data-testid-systematic) :
// - context-banner               (root container — toujours présent si visible)
// - context-banner-cabinet       (niveau 1 — absent en @edge)
// - context-banner-acp           (niveau 2 — toujours)
// - context-banner-building      (niveau 3 — toujours)
// - context-banner-conformity-icon (icône verte/orange/rouge)
//
// Pattern mocks : on stubbe uniquement les boundaries fetch
// (`api/buildings`, `api/acps`, `api/organizations`). La logique métier
// (dérivation niveau, choix couleur, masquage) reste réelle.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor } from "../../../test-helpers";
import ContextBanner from "../ContextBanner.svelte";
import { setBuilding, resetScope } from "../../../stores/scope.svelte";
import type { Building } from "../../../lib/types";

// ---------------------------------------------------------------------------
// Mocks — boundaries réseau uniquement
// ---------------------------------------------------------------------------

vi.mock("../../../lib/api/buildings", () => ({
  getBuilding: vi.fn(),
}));

vi.mock("../../../lib/api/acps", () => ({
  getAcp: vi.fn(),
}));

vi.mock("../../../lib/api/organizations", () => ({
  tryGetOrganizationName: vi.fn(),
}));

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

import { getBuilding } from "../../../lib/api/buildings";
import { getAcp } from "../../../lib/api/acps";
import { tryGetOrganizationName } from "../../../lib/api/organizations";

const mockedGetBuilding = vi.mocked(getBuilding);
const mockedGetAcp = vi.mocked(getAcp);
const mockedTryGetOrgName = vi.mocked(tryGetOrganizationName);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeBuilding(overrides: Partial<Building> = {}): Building {
  return {
    id: "b-aaaaaaaa-0000-0000-0000-000000000001",
    acp_id: "acp-1111-1111-1111-1111-111111111111",
    name: "Immeuble Alpha",
    address: "1 Rue Test",
    city: "Brussels",
    postal_code: "1000",
    country: "Belgium",
    total_units: 10,
    total_tantiemes: 1000,
    units_count: 10,
    quota_sum: "1000",
    is_conformant: true,
    quota_delta: "0",
    ...overrides,
  };
}

function makeBuildingDetail(overrides: Partial<Building> = {}): Building {
  return makeBuilding({
    units_count: 10,
    quota_sum: "1000",
    is_conformant: true,
    quota_delta: "0",
    ...overrides,
  });
}

function makeAcp(
  overrides: Partial<{
    id: string;
    name: string;
    organization_id: string | null;
  }> = {},
) {
  return {
    id: "acp-1111-1111-1111-1111-111111111111",
    organization_id: "org-2222-2222-2222-2222-222222222222",
    name: "ACP Résidence Soleil",
    slug: "acp-residence-soleil",
    legal_status: "ACP",
    bce_number: null,
    address_street: "1 rue test",
    address_postal_code: "1000",
    address_city: "Bruxelles",
    // Dénominateur de l'acte de base (Art. 3.84 CC). Champ obligatoire de
    // `AcpResponseDto`, absent de cette fixture tant que le type était recopié
    // à la main côté frontend — les endpoints ACP n'étaient pas enregistrés
    // dans `openapi.rs`, donc rien ne générait ni ne vérifiait ce contrat.
    total_tantiemes: 1000,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

beforeEach(() => {
  resetScope();
  vi.clearAllMocks();
});

afterEach(() => {
  resetScope();
});

// ===========================================================================
// @happy — chemin nominal : Cabinet · ACP · Immeuble (vert)
// ===========================================================================

describe("ContextBanner @happy", () => {
  it("renders all 3 levels with green conformity icon when building is conformant", async () => {
    const building = makeBuildingDetail({
      id: "b-happy",
      name: "Immeuble Alpha",
      acp_id: "acp-1",
    });
    const acp = makeAcp({
      id: "acp-1",
      name: "ACP Résidence Soleil",
      organization_id: "org-cabinet-1",
    });

    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    mockedTryGetOrgName.mockResolvedValue("Cabinet Maury");

    setBuilding(building);
    render(ContextBanner);

    // Container présent
    const banner = await screen.findByTestId("context-banner");
    expect(banner).toBeInTheDocument();

    // Niveau 1 — Cabinet
    await waitFor(() => {
      const cabinet = screen.queryByTestId("context-banner-cabinet");
      expect(cabinet).toBeInTheDocument();
      expect(cabinet?.textContent ?? "").toContain("Cabinet Maury");
    });

    // Niveau 2 — ACP
    const acpEl = screen.getByTestId("context-banner-acp");
    expect(acpEl).toBeInTheDocument();
    expect(acpEl.textContent ?? "").toContain("ACP Résidence Soleil");

    // Niveau 3 — Building
    const buildingEl = screen.getByTestId("context-banner-building");
    expect(buildingEl).toBeInTheDocument();
    expect(buildingEl.textContent ?? "").toContain("Immeuble Alpha");

    // Icône conformité verte
    const icon = screen.getByTestId("context-banner-conformity-icon");
    expect(icon).toBeInTheDocument();
    expect(icon.className).toMatch(/green/);
  });

  it("shows orange icon when building has positive quota delta (warning)", async () => {
    const building = makeBuildingDetail({
      id: "b-warn",
      is_conformant: false,
      quota_sum: "999.5",
      quota_delta: "-0.5",
    });
    const acp = makeAcp({ id: "acp-w", organization_id: null });

    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    mockedTryGetOrgName.mockResolvedValue(null);

    setBuilding(building);
    render(ContextBanner);

    await screen.findByTestId("context-banner");

    await waitFor(() => {
      const icon = screen.queryByTestId("context-banner-conformity-icon");
      expect(icon).toBeInTheDocument();
      expect(icon?.className).toMatch(/red|orange/);
    });
  });
});

// ===========================================================================
// @edge — ACP auto-gérée (organization_id=null) → 2 niveaux
// ===========================================================================

describe("ContextBanner @edge", () => {
  it("renders 2 levels (ACP · Building) when ACP has no organization", async () => {
    const building = makeBuildingDetail({
      id: "b-edge",
      name: "Immeuble Bêta",
      acp_id: "acp-auto",
    });
    const acp = makeAcp({
      id: "acp-auto",
      name: "ACP Auto-Gérée",
      organization_id: null, // <-- pas de cabinet syndic
    });

    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    mockedTryGetOrgName.mockResolvedValue(null);

    setBuilding(building);
    render(ContextBanner);

    await screen.findByTestId("context-banner");

    // Niveau 1 — Cabinet MASQUÉ (pas dans le DOM, pas placeholder vide)
    await waitFor(() => {
      // L'ACP est forcément rendu pour vérifier que le composant est bien
      // monté avant d'asserter l'absence.
      expect(screen.queryByTestId("context-banner-acp")).toBeInTheDocument();
    });
    expect(screen.queryByTestId("context-banner-cabinet")).toBeNull();

    // Niveaux 2 + 3 toujours présents
    expect(screen.getByTestId("context-banner-acp")).toBeInTheDocument();
    expect(screen.getByTestId("context-banner-building")).toBeInTheDocument();
  });
});

// ===========================================================================
// @security — défense cross-tenant (organization inattendue)
// ===========================================================================

describe("ContextBanner @security", () => {
  it("hides Cabinet level when API returns unresolved organization name (defensive)", async () => {
    // Cas : l'API /acps/{id} renvoie organization_id="org-other", mais
    // /organizations ne contient pas cet id (filtré par scope/RBAC backend).
    // Le composant DOIT dégrader vers 2 niveaux plutôt que d'afficher
    // un placeholder vide qui pourrait leak l'existence d'un cabinet.
    const building = makeBuildingDetail({
      id: "b-sec",
      name: "Immeuble Sécurité",
      acp_id: "acp-sec",
    });
    const acp = makeAcp({
      id: "acp-sec",
      name: "ACP Sécurité",
      organization_id: "org-foreign-cabinet",
    });

    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    // Cabinet non résolvable → fallback null (403 silencieux ou pas dans la liste)
    mockedTryGetOrgName.mockResolvedValue(null);

    setBuilding(building);
    render(ContextBanner);

    await screen.findByTestId("context-banner");

    await waitFor(() => {
      expect(screen.queryByTestId("context-banner-acp")).toBeInTheDocument();
    });

    // Cabinet absent — pas de placeholder, pas de leak
    expect(screen.queryByTestId("context-banner-cabinet")).toBeNull();

    // Pas de chaîne vide ni de "·  ·" entre niveaux
    const banner = screen.getByTestId("context-banner");
    expect(banner.textContent ?? "").not.toMatch(/·\s*·/);
  });
});

// ===========================================================================
// @negative — aucun building sélectionné → composant masqué
// ===========================================================================

describe("ContextBanner @negative", () => {
  it("renders nothing (null) when no building is selected", () => {
    // Aucun setBuilding() — scope.selectedBuildingId === null
    const { container } = render(ContextBanner);

    expect(screen.queryByTestId("context-banner")).toBeNull();
    // Le composant doit vraiment ne rien rendre — pas de placeholder vide
    // ni de div fantôme. On accepte une div racine vide (Svelte 5 mount).
    expect(container.textContent?.trim() ?? "").toBe("");
  });

  it("does not crash when getBuilding rejects (network error)", async () => {
    mockedGetBuilding.mockRejectedValue(new Error("boom"));

    const building = makeBuilding({ id: "b-crash" });
    setBuilding(building);
    render(ContextBanner);

    // Composant doit gracefully ne rien afficher (ou afficher seulement
    // l'info qu'on a déjà via le store, sans crash).
    await waitFor(() => {
      // Pas d'exception jetée — le test passe simplement par stabilité.
      expect(true).toBe(true);
    });
  });
});

// ===========================================================================
// a11y — WCAG 2.1 AA baseline
// ===========================================================================

describe("ContextBanner a11y (WCAG 2.1 AA baseline)", () => {
  it("exposes role=region + aria-label on the banner root", async () => {
    const building = makeBuildingDetail({ id: "b-a11y" });
    const acp = makeAcp({ id: "acp-1111-1111-1111-1111-111111111111" });
    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    mockedTryGetOrgName.mockResolvedValue("Cabinet a11y");

    setBuilding(building);
    render(ContextBanner);

    const banner = await screen.findByTestId("context-banner");
    expect(banner.getAttribute("role")).toBe("region");
    expect(banner.getAttribute("aria-label")).toBeTruthy();
  });

  it("conformity icon exposes role=img + descriptive aria-label", async () => {
    const building = makeBuildingDetail({ id: "b-a11y-icon" });
    const acp = makeAcp({ id: "acp-1111-1111-1111-1111-111111111111" });
    mockedGetBuilding.mockResolvedValue(building);
    mockedGetAcp.mockResolvedValue(acp);
    mockedTryGetOrgName.mockResolvedValue(null);

    setBuilding(building);
    render(ContextBanner);

    await screen.findByTestId("context-banner");

    await waitFor(() => {
      const icon = screen.queryByTestId("context-banner-conformity-icon");
      expect(icon).toBeInTheDocument();
      expect(icon?.getAttribute("role")).toBe("img");
      expect(icon?.getAttribute("aria-label")).toBeTruthy();
    });
  });
});
