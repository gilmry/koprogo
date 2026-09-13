// Story #798 — AcpSelector : le périmètre courant devient l'ACP, l'immeuble
// devient le filtre secondaire (BuildingSelector.svelte, inchangé, continue
// de couvrir ce filtre).
//
// CRITICAL §3 — RED-first TDD : tests rouges AVANT le composant.
//
// Couverture :
// - @happy    : rôle syndic + clic résultat -> currentAcp posé
// - @edge     : relation ACP <-> immeuble n:n (pas 1:1) ; cap 20 résultats
// - @security : rôle owner -> composant null ; ACP hors portefeuille -> 403
// - @negative : aucune ACP ne correspond -> empty state ; erreur réseau
//
// data-testid contractuels — NEUFS (cf. docs/refonte/CONTRAT_DE_TESTS.md §4) :
// ne réemploient JAMAIS `building-selector-*`, dont le contrat doit rester
// intact tant que le chemin par immeuble existe.
//   acp-selector-root, -input, -result-{id}, -favourite-{id}, -clear, -empty,
//   -403, -listbox
//
// Pourquoi pas de debounce réseau (contrairement à BuildingSelector) :
// `GET /acps` (`listAcps()`) n'a pas de paramètre `?search=` côté backend
// (cf. `acp_handlers::list_acps`) — le portefeuille ACP d'un syndic est de
// toute façon une liste courte (quelques ACP, pas des milliers de buildings
// plateforme). On charge une fois, on filtre côté client.
//
// Pattern mocks : on stubbe uniquement la boundary réseau (`api/acps`). La
// logique métier (RBAC, filtre, garde n:n, scope 403) reste réelle.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "../../../test-helpers";
import AcpSelector from "../AcpSelector.svelte";
import {
  resetScope,
  getScope,
  setBuilding,
  demanderAcp,
} from "../../../stores/scope.svelte";
import { UserRole } from "../../../lib/types";
import type { Building } from "../../../lib/types";
import type { AcpResponseDto } from "../../../lib/api/acps";

// ---------------------------------------------------------------------------
// Mocks — boundary réseau uniquement
// ---------------------------------------------------------------------------

vi.mock("../../../lib/api/acps", () => ({
  listAcps: vi.fn(),
}));

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

import { listAcps } from "../../../lib/api/acps";

const mockedListAcps = vi.mocked(listAcps);

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

function makeAcp(overrides: Partial<AcpResponseDto> = {}): AcpResponseDto {
  return {
    id: "acp-001",
    name: "ACP Résidence Soleil",
    slug: "acp-residence-soleil",
    legal_status: "copropriete_belge",
    bce_number: null,
    organization_id: "org-1",
    address_street: "1 rue test",
    address_postal_code: "1000",
    address_city: "Bruxelles",
    total_tantiemes: 1000,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function makeBuilding(overrides: Partial<Building> = {}): Building {
  return {
    id: "b-1",
    acp_id: "acp-001",
    name: "Bloc A",
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
    { id: "r-1", role: UserRole.SYNDIC, organizationId: "org-1", isPrimary: true },
  ],
};

const ownerUser = {
  id: "u-owner",
  email: "owner@test.com",
  first_name: "Ow",
  last_name: "Ner",
  role: UserRole.OWNER,
  roles: [
    { id: "r-2", role: UserRole.OWNER, organizationId: "org-1", isPrimary: true },
  ],
};

beforeEach(() => {
  resetScope();
  vi.clearAllMocks();
});
afterEach(() => resetScope());

// ===========================================================================
// @happy — chemin nominal
// ===========================================================================

describe("AcpSelector @happy", () => {
  it("renders input for syndic role", async () => {
    mockedListAcps.mockResolvedValue([]);

    render(AcpSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("acp-selector-input");
    expect(input).toBeInTheDocument();
    expect(input.tagName.toLowerCase()).toBe("input");
  });

  it("ouvre la liste au focus, sans avoir tapé (portefeuille précharge)", async () => {
    const deux = [
      makeAcp({ id: "acp-a", name: "ACP Les Érables" }),
      makeAcp({ id: "acp-b", name: "ACP Les Glycines" }),
    ];
    mockedListAcps.mockResolvedValue(deux);

    render(AcpSelector, { props: { user: syndicUser } });

    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-selector-result-acp-a")).toBeTruthy();
    });
    expect(screen.getByTestId("acp-selector-result-acp-b")).toBeInTheDocument();
    expect(input).toHaveAttribute("aria-expanded", "true");
  });

  it("filtre côté client en tapant (pas de round-trip réseau par frappe)", async () => {
    mockedListAcps.mockResolvedValue([
      makeAcp({ id: "acp-a", name: "ACP Alpha" }),
      makeAcp({ id: "acp-b", name: "ACP Beta" }),
    ]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);
    await waitFor(() => expect(mockedListAcps).toHaveBeenCalledTimes(1));

    await fireEvent.input(input, { target: { value: "alph" } });

    await waitFor(() => {
      expect(screen.queryByTestId("acp-selector-result-acp-a")).toBeTruthy();
      expect(screen.queryByTestId("acp-selector-result-acp-b")).toBeNull();
    });
    // Un seul appel réseau, au montage/focus — pas un par frappe.
    expect(mockedListAcps).toHaveBeenCalledTimes(1);
  });

  it("le clic sur un résultat pose currentAcp et les écrans se rendent sur ce périmètre", async () => {
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-click", name: "Cliquée" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);

    const result = await screen.findByTestId("acp-selector-result-acp-click");
    await fireEvent.click(result);

    await waitFor(() => {
      expect(getScope().selectedAcpId).toBe("acp-click");
    });
    expect(getScope().scopeError).toBeNull();
  });

  it("exposes clear button that resets selectedAcpId", async () => {
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-clear" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);
    const result = await screen.findByTestId("acp-selector-result-acp-clear");
    await fireEvent.click(result);

    await waitFor(() => expect(getScope().selectedAcpId).toBe("acp-clear"));

    const clear = await screen.findByTestId("acp-selector-clear");
    await fireEvent.click(clear);

    await waitFor(() => expect(getScope().selectedAcpId).toBeNull());
  });
});

// ===========================================================================
// @edge — la relation ACP <-> immeuble n'est pas 1:1 (n:n via les lots)
// ===========================================================================

describe("AcpSelector @edge — n:n via les lots, pas 1:1", () => {
  it("choisir une ACP couvrant plusieurs blocs ne force AUCUN immeuble unique", async () => {
    // ACP principale qui couvre deux blocs (Art. 3.87 — ACP principale et
    // secondaires). Un champ unique fusionnant building+acp serait un modèle
    // faux : la story #798 l'affirme explicitement.
    mockedListAcps.mockResolvedValue([
      makeAcp({ id: "acp-principale", name: "ACP Principale (Blocs A+B)" }),
    ]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);
    const result = await screen.findByTestId("acp-selector-result-acp-principale");
    await fireEvent.click(result);

    await waitFor(() => expect(getScope().selectedAcpId).toBe("acp-principale"));
    // Aucun immeuble n'est déduit : le filtre reste à "tous les blocs".
    expect(getScope().selectedBuildingId).toBeNull();
  });

  it("un immeuble déjà choisi sous la MÊME ACP survit à la (re)confirmation de l'ACP", async () => {
    setBuilding(makeBuilding({ id: "b-survit", acp_id: "acp-001" }));
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-001" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);
    const result = await screen.findByTestId("acp-selector-result-acp-001");
    await fireEvent.click(result);

    await waitFor(() => expect(getScope().selectedAcpId).toBe("acp-001"));
    expect(getScope().selectedBuildingId).toBe("b-survit");
  });

  it("choisir une AUTRE ACP efface le filtre immeuble devenu incohérent", async () => {
    setBuilding(makeBuilding({ id: "b-ancien", acp_id: "acp-ancienne" }));
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-nouvelle" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);
    const result = await screen.findByTestId("acp-selector-result-acp-nouvelle");
    await fireEvent.click(result);

    await waitFor(() => expect(getScope().selectedAcpId).toBe("acp-nouvelle"));
    // L'immeuble de l'ancienne ACP n'a plus de sens comme filtre : il est
    // effacé plutôt que de laisser un filtre qui pointe hors du périmètre.
    expect(getScope().selectedBuildingId).toBeNull();
  });

  it("caps rendered results at 20 even when the portfolio has more", async () => {
    const beaucoup = Array.from({ length: 50 }, (_, i) =>
      makeAcp({ id: `acp-${i}`, name: `ACP ${i}` }),
    );
    mockedListAcps.mockResolvedValue(beaucoup);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-selector-result-acp-0")).toBeTruthy();
    });
    expect(screen.queryByTestId("acp-selector-result-acp-19")).toBeInTheDocument();
    expect(screen.queryByTestId("acp-selector-result-acp-20")).toBeNull();
  });
});

// ===========================================================================
// @security — RBAC + scope violations
// ===========================================================================

describe("AcpSelector @security", () => {
  it("renders NOTHING (null) for owner role — RBAC role-based render", () => {
    const { container } = render(AcpSelector, { props: { user: ownerUser } });

    expect(screen.queryByTestId("acp-selector-input")).toBeNull();
    expect(container.textContent ?? "").not.toContain("acp-selector");
  });

  it("renders nothing when user is null (logged-out fallback)", () => {
    render(AcpSelector, { props: { user: null } });
    expect(screen.queryByTestId("acp-selector-input")).toBeNull();
  });

  it("ACP hors du portefeuille de l'utilisateur -> 403, périmètre inchangé", async () => {
    mockedListAcps.mockResolvedValue([]);
    setBuilding(makeBuilding({ id: "b-legitime", acp_id: "acp-legitime" }));

    render(AcpSelector, { props: { user: syndicUser } });
    await screen.findByTestId("acp-selector-input");

    // Reproduit la branche "demande validée côté serveur, refusée" — le
    // composant réagit au même `scope.scopeError` que `demanderAcp` pose
    // (store test dédié : `stores/__tests__/scope-acp.test.ts`).
    const charger = vi.fn().mockRejectedValue({ status: 403 });
    await demanderAcp("acp-hors-portefeuille", charger);

    await waitFor(() => {
      expect(screen.queryByTestId("acp-selector-403")).toBeTruthy();
    });
    // Le périmètre qui fonctionnait déjà n'a pas bougé.
    expect(getScope().selectedBuildingId).toBe("b-legitime");
  });
});

// ===========================================================================
// @negative — défaillances correctes
// ===========================================================================

describe("AcpSelector @negative", () => {
  it("shows empty state when no ACP matches the query", async () => {
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-a", name: "ACP Alpha" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.input(input, { target: { value: "zzzzz" } });

    await waitFor(() => {
      expect(screen.queryByTestId("acp-selector-empty")).toBeTruthy();
    });
  });

  it("does not crash when listAcps rejects (network error)", async () => {
    mockedListAcps.mockRejectedValue(new Error("boom"));

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);

    await waitFor(() => {
      expect(input).toBeInTheDocument();
    });
  });
});

// ===========================================================================
// a11y — WCAG 2.1 AA baseline
// ===========================================================================

describe("AcpSelector a11y (WCAG 2.1 AA baseline)", () => {
  it("input exposes role=combobox + aria-expanded/aria-controls (WCAG 4.1.2)", async () => {
    mockedListAcps.mockResolvedValue([]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    expect(input.getAttribute("role")).toBe("combobox");
    expect(input.getAttribute("aria-expanded")).not.toBeNull();
  });

  it("results listbox has role=listbox + each result role=option", async () => {
    mockedListAcps.mockResolvedValue([makeAcp({ id: "acp-a11y", name: "A11y" })]);

    render(AcpSelector, { props: { user: syndicUser } });
    const input = await screen.findByTestId("acp-selector-input");
    await fireEvent.focus(input);

    const result = await screen.findByTestId("acp-selector-result-acp-a11y");
    expect(result.getAttribute("role")).toBe("option");
  });
});
