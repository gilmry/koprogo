// Story 2.4 — Navigation Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD : tests rouges AVANT le refacto Navigation.
//
// Couverture :
// - @happy : syndic + building -> 5 menus business visibles ; owner -> communaute + mes-lots
// - @edge  : admin sans building -> menus /admin/* ; admin in-context -> menus business
// - @security : accountant n'a pas le menu communaute (RBAC strict)
// - @negative : user authentifie sans aucun UserRoleAssignment -> empty state
//
// data-testid contractuels (cf. story 2.4 + memory data-testid-systematic) :
//   navigation-menu-gestion, -compta, -gouvernance, -communaute, -ticketing,
//   navigation-menu-mes-lots, navigation-menu-admin,
//   navigation-submenu-{key}, navigation-empty-no-role
//
// Pattern : on stubbe les boundaries (authStore, scope, i18n, NotificationBell).
// La logique permission reste reelle (canSee), la dérivation reste réelle.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { writable, type Writable } from "svelte/store";
import { render, screen } from "../../../test-helpers";
import { resetScope, setBuilding } from "../../../stores/scope.svelte";
import { UserRole, type User, type Building } from "../../../lib/types";

// ---------------------------------------------------------------------------
// Mocks — boundaries uniquement
// ---------------------------------------------------------------------------

// authStore est un writable Svelte ({ subscribe, ... }) + methodes (logout, etc.).
// On reconstruit un mock writable pour piloter user/isAuthenticated dans chaque test.
type AuthState = {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  token: string | null;
};

const mockAuthState: Writable<AuthState> = writable({
  user: null,
  isAuthenticated: false,
  isLoading: false,
  token: null,
});

vi.mock("../../../stores/auth", () => ({
  authStore: {
    subscribe: (...args: any[]) => mockAuthState.subscribe(...(args as [any])),
    init: vi.fn(),
    logout: vi.fn(),
    switchRole: vi.fn().mockResolvedValue(true),
    refreshAccessToken: vi.fn().mockResolvedValue(true),
  },
}));

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("../../notifications/NotificationBell.svelte", () => ({
  default: () => null,
}));

import Navigation from "../Navigation.svelte";

// ---------------------------------------------------------------------------
// Fixtures users + building
// ---------------------------------------------------------------------------

function makeUser(role: UserRole, hasRoleAssignment = true): User {
  return {
    id: `u-${role}`,
    email: `${role}@test.com`,
    first_name: "First",
    last_name: "Last",
    role,
    roles: hasRoleAssignment
      ? [
          {
            id: `r-${role}`,
            role,
            organizationId: "acp-001",
            isPrimary: true,
          },
        ]
      : [],
    activeRole: hasRoleAssignment
      ? {
          id: `r-${role}`,
          role,
          organizationId: "acp-001",
          isPrimary: true,
        }
      : undefined,
  };
}

function makeBuilding(): Building {
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
  };
}

function setAuth(user: User | null) {
  mockAuthState.set({
    user,
    isAuthenticated: user !== null,
    isLoading: false,
    token: user ? "tok" : null,
  });
}

beforeEach(() => {
  resetScope();
  setAuth(null);
});

afterEach(() => {
  resetScope();
  setAuth(null);
});

// ===========================================================================
// @happy — chemin nominal : matrice role × menu
// ===========================================================================

describe("Navigation @happy", () => {
  it("syndic + building selectionne -> 5 menus business visibles", async () => {
    setAuth(makeUser(UserRole.SYNDIC));
    setBuilding(makeBuilding());

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-gestion"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("navigation-menu-compta")).toBeInTheDocument();
    expect(
      screen.getByTestId("navigation-menu-gouvernance"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("navigation-menu-communaute"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("navigation-menu-ticketing")).toBeInTheDocument();
    // Le menu admin ne doit pas apparaitre pour le syndic
    expect(screen.queryByTestId("navigation-menu-admin")).toBeNull();
  });

  it("owner -> Communaute + Mes lots visibles seulement", async () => {
    setAuth(makeUser(UserRole.OWNER));
    setBuilding(makeBuilding());

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-communaute"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("navigation-menu-mes-lots")).toBeInTheDocument();
    // Aucun menu business pro pour l'owner
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-compta")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-gouvernance")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-ticketing")).toBeNull();
  });
});

// ===========================================================================
// @edge — admin/superadmin mode in-context
// ===========================================================================

describe("Navigation @edge", () => {
  it("admin SANS building selectionne -> menu admin visible, menus business masques", async () => {
    setAuth(makeUser(UserRole.SUPERADMIN));
    // pas de setBuilding -> scope vide

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-admin"),
    ).toBeInTheDocument();
    // Pas de menus business sans selection
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-compta")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-gouvernance")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-communaute")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-ticketing")).toBeNull();
  });

  it("admin AVEC building selectionne -> mode in-context : menus business + masque admin", async () => {
    setAuth(makeUser(UserRole.SUPERADMIN));
    setBuilding(makeBuilding());

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-gestion"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("navigation-menu-compta")).toBeInTheDocument();
    expect(
      screen.getByTestId("navigation-menu-gouvernance"),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId("navigation-menu-communaute"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("navigation-menu-ticketing")).toBeInTheDocument();
    // Mode in-context : le menu admin est masqué
    expect(screen.queryByTestId("navigation-menu-admin")).toBeNull();
  });
});

// ===========================================================================
// @security — RBAC strict
// ===========================================================================

describe("Navigation @security", () => {
  it("accountant n'a pas le menu Communaute (RBAC produit)", async () => {
    setAuth(makeUser(UserRole.ACCOUNTANT));
    setBuilding(makeBuilding());

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-compta"),
    ).toBeInTheDocument();
    // Accountant n'a JAMAIS communaute
    expect(screen.queryByTestId("navigation-menu-communaute")).toBeNull();
    // Ni les autres menus
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-gouvernance")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-ticketing")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-mes-lots")).toBeNull();
  });

  it("owner n'a pas les menus pro (gestion/compta/gouvernance/ticketing)", async () => {
    setAuth(makeUser(UserRole.OWNER));
    setBuilding(makeBuilding());

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-communaute"),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-compta")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-gouvernance")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-ticketing")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-admin")).toBeNull();
  });
});

// ===========================================================================
// @negative — utilisateur sans assignment de role
// ===========================================================================

describe("Navigation @negative", () => {
  it("user authentifie sans UserRoleAssignment -> ecran empty-no-role", async () => {
    setAuth(makeUser(UserRole.OWNER, /* hasRoleAssignment */ false));
    // Override : on retire activeRole + roles pour simuler un user sans assignment
    mockAuthState.update((s) => ({
      ...s,
      user: s.user
        ? {
            ...s.user,
            role: undefined as any, // pas de role actif
            activeRole: undefined,
            roles: [],
          }
        : null,
    }));

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-empty-no-role"),
    ).toBeInTheDocument();
    // Aucun menu rendu en parallele
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-communaute")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-admin")).toBeNull();
  });

  it("user non authentifie -> aucun menu ni empty-no-role (UI gerée par Layout)", () => {
    setAuth(null);

    const { container } = render(Navigation);

    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-empty-no-role")).toBeNull();
    // Le composant rend (potentiellement) une coquille vide ou des elements
    // d'auth (login button), mais aucun menu role-conditioned.
    expect(container).toBeTruthy();
  });
});

// ===========================================================================
// A11y baseline (WCAG 2.1 AA — cf. memory a11y-wcag-aa-baseline)
// ===========================================================================

describe("Navigation a11y", () => {
  it("nav element expose role=navigation + aria-label", async () => {
    setAuth(makeUser(UserRole.SYNDIC));
    setBuilding(makeBuilding());

    const { container } = render(Navigation);

    // Au moins un element avec role navigation et aria-label
    const navs = container.querySelectorAll('[role="navigation"], nav');
    expect(navs.length).toBeGreaterThan(0);
    // Au moins un avec aria-label non-vide
    const labelled = Array.from(navs).some(
      (n) => (n.getAttribute("aria-label") ?? "").length > 0,
    );
    expect(labelled).toBe(true);
  });
});
