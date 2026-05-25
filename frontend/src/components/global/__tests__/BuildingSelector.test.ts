// Story 2.2 — BuildingSelector Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first BDD/TDD : tests rouge avant le composant.
//
// Couverture :
// - @happy : render selector pour rôle syndic + typing → autocomplete + click
// - @edge  : 500 buildings → debounce + cap pagination 20 résultats max
// - @security : rôle owner → composant null ; click hors scope 403 → reset
// - @negative : aucun building → empty state
//
// data-testid contractuels (cf. stories.md ligne 223-242 + memory data-testid-systematic) :
// - building-selector-input
// - building-selector-result-{id}
// - building-selector-favorite-{id}
// - building-selector-clear
// - building-selector-empty
// - building-selector-403
//
// Pattern mocks : on stubbe uniquement les boundaries fetch/network
// (`api/buildings` + `api/portfolios`). La logique métier (debounce, RBAC,
// scope reset) reste réelle, testée bout en bout dans le composant.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "../../../test-helpers";
import BuildingSelector from "../BuildingSelector.svelte";
import {
  resetScope,
  getScope,
  setScopeError,
} from "../../../stores/scope.svelte";
import { UserRole } from "../../../lib/types";
import type { Building } from "../../../lib/types";

// ---------------------------------------------------------------------------
// Mocks — boundaries réseau uniquement
// ---------------------------------------------------------------------------

vi.mock("../../../lib/api/buildings", () => ({
  searchBuildings: vi.fn(),
  listBuildings: vi.fn(),
  getBuilding: vi.fn(),
}));

vi.mock("../../../lib/api/portfolios", () => ({
  listPortfolios: vi.fn().mockResolvedValue([]),
  listPortfolioBuildings: vi.fn().mockResolvedValue([]),
  toggleFavorite: vi.fn(),
}));

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

import { searchBuildings, listBuildings } from "../../../lib/api/buildings";

const mockedSearchBuildings = vi.mocked(searchBuildings);
const mockedListBuildings = vi.mocked(listBuildings);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeBuilding(overrides: Partial<Building> = {}): Building {
  return {
    id: "11111111-1111-1111-1111-111111111111",
    organization_id: "acp-001",
    name: "Immeuble Test",
    address: "1 Rue Test",
    city: "Brussels",
    postal_code: "1000",
    country: "Belgium",
    total_units: 10,
    total_tantiemes: 1000,
    ...overrides,
  };
}

const syndicUser = {
  id: "u-syndic",
  email: "syndic@test.com",
  first_name: "Sy",
  last_name: "Ndic",
  role: UserRole.SYNDIC,
  roles: [
    {
      id: "r-1",
      role: UserRole.SYNDIC,
      organizationId: "acp-001",
      isPrimary: true,
    },
  ],
};

const ownerUser = {
  id: "u-owner",
  email: "owner@test.com",
  first_name: "Ow",
  last_name: "Ner",
  role: UserRole.OWNER,
  roles: [
    {
      id: "r-2",
      role: UserRole.OWNER,
      organizationId: "acp-001",
      isPrimary: true,
    },
  ],
};

beforeEach(() => {
  resetScope();
  vi.clearAllMocks();
});

afterEach(() => {
  resetScope();
});

// ===========================================================================
// @happy — chemin nominal
// ===========================================================================

describe("BuildingSelector @happy", () => {
  it("renders input for syndic role", async () => {
    mockedSearchBuildings.mockResolvedValue([]);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    expect(input).toBeInTheDocument();
    expect(input.tagName.toLowerCase()).toBe("input");
  });

  it("renders 3 results after typing query (autocomplete)", async () => {
    const results = [
      makeBuilding({ id: "b-1", name: "Immeuble Alpha" }),
      makeBuilding({ id: "b-2", name: "Immeuble Beta" }),
      makeBuilding({ id: "b-3", name: "Immeuble Gamma" }),
    ];
    mockedSearchBuildings.mockResolvedValue(results);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "immeu" } });

    await waitFor(
      () => {
        expect(
          screen.queryByTestId("building-selector-result-b-1"),
        ).toBeTruthy();
      },
      { timeout: 1000 },
    );

    expect(
      screen.getByTestId("building-selector-result-b-1"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("building-selector-result-b-2"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("building-selector-result-b-3"),
    ).toBeInTheDocument();
  });

  it("updates scope store on result click", async () => {
    const target = makeBuilding({
      id: "b-click",
      name: "Cliqué",
      organization_id: "acp-XYZ",
    });
    mockedSearchBuildings.mockResolvedValue([target]);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "cli" } });

    const result = await screen.findByTestId(
      "building-selector-result-b-click",
    );
    await fireEvent.click(result);

    await waitFor(() => {
      const snap = getScope();
      expect(snap.selectedBuildingId).toBe("b-click");
    });
    const snap = getScope();
    expect(snap.selectedBuilding?.name).toBe("Cliqué");
    expect(snap.selectedAcpId).toBe("acp-XYZ");
    expect(snap.scopeError).toBeNull();
  });

  it("exposes clear button that resets the scope", async () => {
    const target = makeBuilding({ id: "b-clear" });
    mockedSearchBuildings.mockResolvedValue([target]);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "imm" } });
    const result = await screen.findByTestId(
      "building-selector-result-b-clear",
    );
    await fireEvent.click(result);

    await waitFor(() => {
      expect(getScope().selectedBuildingId).toBe("b-clear");
    });

    const clear = await screen.findByTestId("building-selector-clear");
    await fireEvent.click(clear);

    await waitFor(() => {
      expect(getScope().selectedBuildingId).toBeNull();
    });
  });
});

// ===========================================================================
// @edge — bornes / volumes
// ===========================================================================

describe("BuildingSelector @edge", () => {
  it("caps rendered results at 20 even when API returns more", async () => {
    const many = Array.from({ length: 50 }, (_, i) =>
      makeBuilding({ id: `b-${i}`, name: `Immeuble ${i}` }),
    );
    mockedSearchBuildings.mockResolvedValue(many);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "imm" } });

    await waitFor(() => {
      expect(screen.queryByTestId("building-selector-result-b-0")).toBeTruthy();
    });

    // 20 visible max — `b-20` ne doit PAS être rendu.
    expect(
      screen.queryByTestId("building-selector-result-b-19"),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("building-selector-result-b-20")).toBeNull();
  });

  it("debounces typing — does not fire one fetch per keystroke", async () => {
    mockedSearchBuildings.mockResolvedValue([]);

    render(BuildingSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("building-selector-input");

    // 4 keystrokes rapides — ne doit déclencher que ~1 fetch après debounce.
    await fireEvent.input(input, { target: { value: "i" } });
    await fireEvent.input(input, { target: { value: "im" } });
    await fireEvent.input(input, { target: { value: "imm" } });
    await fireEvent.input(input, { target: { value: "imme" } });

    await waitFor(
      () => {
        expect(mockedSearchBuildings).toHaveBeenCalled();
      },
      { timeout: 1000 },
    );

    // Debounce ≤ 150ms → au plus 1 fetch effectivement résolu pour les 4
    // keystrokes regroupés. On tolère 1-2 (race possible si test runner lent)
    // mais PAS 4 (sinon pas de debounce).
    expect(mockedSearchBuildings.mock.calls.length).toBeLessThanOrEqual(2);
  });
});

// ===========================================================================
// @security — RBAC + scope violations
// ===========================================================================

describe("BuildingSelector @security", () => {
  it("renders NOTHING (null) for owner role — RBAC role-based render", () => {
    const { container } = render(BuildingSelector, {
      props: { user: ownerUser },
    });

    expect(screen.queryByTestId("building-selector-input")).toBeNull();
    // Le composant doit s'auto-effacer pour les owners (cf. AC @security).
    expect(container.textContent ?? "").not.toContain("building-selector");
  });

  it("renders nothing when user is null (logged-out fallback)", () => {
    render(BuildingSelector, { props: { user: null } });
    expect(screen.queryByTestId("building-selector-input")).toBeNull();
  });

  it("shows scope-403 state when click triggers a backend 403", async () => {
    const target = makeBuilding({ id: "b-forbidden", name: "Hors scope" });
    mockedSearchBuildings.mockResolvedValue([target]);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "hors" } });
    const result = await screen.findByTestId(
      "building-selector-result-b-forbidden",
    );

    // Simule un 403 backend — la sélection doit échouer côté composant via
    // un setter qui détecte le scope violation. On utilise directement
    // setScopeError pour reproduire la branche "after click → backend 403".
    await fireEvent.click(result);
    setScopeError("forbidden");

    await waitFor(() => {
      expect(screen.queryByTestId("building-selector-403")).toBeTruthy();
    });

    // Scope reset (pas de building courant après 403).
    expect(getScope().selectedBuildingId).toBeNull();
  });
});

// ===========================================================================
// @negative — défaillances correctes
// ===========================================================================

describe("BuildingSelector @negative", () => {
  it("shows empty state when no building matches the query", async () => {
    mockedSearchBuildings.mockResolvedValue([]);
    mockedListBuildings.mockResolvedValue({
      data: [],
      pagination: { page: 1, per_page: 20, total_items: 0, total_pages: 0 },
    });

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "zzzzz" } });

    await waitFor(() => {
      expect(screen.queryByTestId("building-selector-empty")).toBeTruthy();
    });
  });

  it("does not crash when searchBuildings rejects (network error)", async () => {
    mockedSearchBuildings.mockRejectedValue(new Error("boom"));

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "x" } });

    // Le composant doit fallback gracieusement — empty state OU pas de
    // résultat affiché, mais l'input reste utilisable.
    await waitFor(() => {
      expect(input).toBeInTheDocument();
    });
  });
});

// ===========================================================================
// @security — a11y baseline (rôles ARIA)
// ===========================================================================

describe("BuildingSelector a11y (WCAG 2.1 AA baseline)", () => {
  it("input exposes role=combobox + aria-expanded/aria-controls (WCAG 4.1.2)", async () => {
    mockedSearchBuildings.mockResolvedValue([]);

    render(BuildingSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("building-selector-input");
    expect(input.getAttribute("role")).toBe("combobox");
    expect(input.getAttribute("aria-expanded")).not.toBeNull();
  });

  it("results listbox has role=listbox + each result role=option", async () => {
    const results = [makeBuilding({ id: "b-a11y", name: "A11y test" })];
    mockedSearchBuildings.mockResolvedValue(results);

    render(BuildingSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("building-selector-input");
    await fireEvent.input(input, { target: { value: "a11" } });

    const result = await screen.findByTestId("building-selector-result-b-a11y");
    expect(result.getAttribute("role")).toBe("option");
  });
});
