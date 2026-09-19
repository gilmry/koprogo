// Story 2.4 — Navigation Vitest tests (4-cat).
// Story #798 — la règle "admin en contexte" bascule de l'immeuble vers l'ACP.
//
// CRITICAL §3 — RED-first TDD : tests rouges AVANT le refacto Navigation.
//
// Couverture :
// - @happy : syndic + building -> 5 menus business visibles ; owner -> communaute + mes-lots
// - @edge  : admin sans building -> menus /admin/* ; admin in-context -> menus business
//            (in-context se lit maintenant building OU ACP — cf. #798 ci-dessous)
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
import { resetScope, setBuilding, setAcp } from "../../../stores/scope.svelte";
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
    acp_id: "acp-001",
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

  /**
   * Story #798 — LA NOUVELLE RÈGLE : le périmètre qui déclenche le mode
   * "in-context" n'est plus l'immeuble, c'est l'ACP.
   *
   * L'ACP est la personne morale (numéro BCE, compte bancaire, AG, quotités
   * totalisant 1000 PAR ACP) — pas l'immeuble, qui n'est qu'un actif qu'elle
   * détient. Une ACP peut couvrir plusieurs blocs (ACP principale +
   * secondaires, droit belge) : un syndic peut donc être "en contexte" sur
   * une ACP entière SANS avoir choisi un bloc précis.
   *
   * Ces deux tests sont l'équivalent ACP-only des deux tests `@edge`
   * précédents (déjà couverts côté `canSee` par `permissions.test.ts @edge`,
   * bloc "admin AVEC ACP seule"). Rien n'est retiré : les deux tests pilotés
   * par `setBuilding()` restent ci-dessus, intacts — l'immeuble reste un
   * périmètre "in-context" valide, il devient seulement le FILTRE SECONDAIRE
   * à l'intérieur de l'ACP (`setBuilding()` continue de dériver
   * `selectedAcpId`, donc les deux tests précédents restent vrais pour la
   * même raison qu'avant). « Only remove the building-scoped code path once
   * every spec has an ACP equivalent, in a separate commit. » — remise de
   * design, §6.4.
   */
  it("admin AVEC ACP seule (sans immeuble) -> mode in-context : menus business + masque admin", async () => {
    setAuth(makeUser(UserRole.SUPERADMIN));
    setAcp("acp-001"); // pas de setBuilding — l'ACP seule porte le contexte

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
    // Mode in-context : le menu admin est masqué, exactement comme pour le
    // cas "building sélectionné" ci-dessus.
    expect(screen.queryByTestId("navigation-menu-admin")).toBeNull();
  });

  it("admin SANS ACP ni immeuble -> menu admin visible, menus business masqués (inchangé)", async () => {
    setAuth(makeUser(UserRole.SUPERADMIN));
    // Ni setBuilding, ni setAcp -> scope vide, comme le premier test @edge.

    render(Navigation);

    expect(
      await screen.findByTestId("navigation-menu-admin"),
    ).toBeInTheDocument();
    expect(screen.queryByTestId("navigation-menu-gestion")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-compta")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-gouvernance")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-communaute")).toBeNull();
    expect(screen.queryByTestId("navigation-menu-ticketing")).toBeNull();
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

// ===========================================================================
// Icônes (#797) — remise « Design tokens / Icons », section Navigation.
//
// Gherkin de la story : « Étant donné la barre de navigation d'un syndic /
// Quand je relève les icônes de ses entrées de menu / Alors aucune icône
// n'apparaît deux fois / Et aucune n'est un émoji. »
//
// `garde-icones-uniques.test.ts` vérifie que le JEU d'icônes (lib/icones.ts)
// ne définit pas deux noms au même tracé. Il ne peut pas voir si DEUX
// ENTRÉES DE MENU DIFFÉRENTES réclament, par erreur de copier-coller, le
// même nom d'icône — c'est exactement la forme du défaut d'origine (📊 pour
// tableau de bord ET budgets ET sondages). D'où un rendu réel, ici.
//
// Le Gherkin ne cite que le syndic, mais un rendu réel ne voit QUE les
// entrées visibles pour le rôle qu'il joue. Deux collisions concrètes se
// cachaient dans des combinaisons de menus que le syndic ne peut pas
// atteindre — `/admin/gdpr` + `/settings/gdpr` (superadmin sans immeuble),
// `/documents` + `/owner/documents` (membre du conseil, seul rôle à voir
// `gouvernance` ET `mes-lots`). D'où les deux rendus supplémentaires
// ci-dessous, en plus de celui du syndic.
//
// @edge (#a8a29e/#c4c0bb jamais en texte, chevron compris) n'est pas
// dupliqué ici : `garde-neutres-de-trait.test.ts` le couvre déjà pour
// TOUTE la source, Navigation.svelte inclus — le refaire localement
// n'ajouterait aucun signal.
// ===========================================================================

/** Émojis pictographiques et symboles divers — même motif que les gardes globales. */
const EMOJI = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}\u{2B00}-\u{2BFF}]/gu;

async function rendreSidebarSyndic() {
  setAuth(makeUser(UserRole.SYNDIC));
  setBuilding(makeBuilding());
  const { container } = render(Navigation);
  await screen.findByTestId("navigation-menu-gestion");
  const sidebar = container.querySelector('[data-testid="sidebar-desktop"]');
  if (!sidebar) throw new Error("sidebar-desktop introuvable");
  return sidebar;
}

/**
 * Superadmin SANS immeuble sélectionné : la sidebar affiche le menu admin
 * (`navigation-menu-admin`) ET le pied de sidebar (profil, réglages, RGPD,
 * déconnexion), qui sont rendus dans le même `<aside data-testid="sidebar-
 * desktop">` — cf. Navigation.svelte, où le bloc `<nav aria-label="Menus
 * principaux">` et la `<div data-testid="sidebar-user-section">` sont
 * frères dans la même aside. Une icône réemployée entre les deux ne se
 * verrait donc PAS dans `rendreSidebarSyndic`, qui ne rend jamais le menu
 * admin.
 */
async function rendreSidebarAdminSansImmeuble() {
  setAuth(makeUser(UserRole.SUPERADMIN));
  const { container } = render(Navigation);
  await screen.findByTestId("navigation-menu-admin");
  const sidebar = container.querySelector('[data-testid="sidebar-desktop"]');
  if (!sidebar) throw new Error("sidebar-desktop introuvable");
  return sidebar;
}

/**
 * Membre du conseil : le seul rôle qui voit `gouvernance` ET `mes-lots` à la
 * fois (`BOARD_MENUS` dans `permissions.ts` — la charge de conseil s'ajoute à
 * la qualité de copropriétaire, elle ne la remplace pas). `/documents`
 * (gouvernance) et `/owner/documents` (mes-lots) sont deux destinations
 * distinctes que ni `rendreSidebarSyndic` (jamais mes-lots) ni
 * `rendreSidebarAdminSansImmeuble` (jamais gouvernance/mes-lots) ne peuvent
 * voir apparaître ensemble.
 */
async function rendreSidebarMembreDuConseil() {
  setAuth(makeUser(UserRole.BOARD_MEMBER));
  setBuilding(makeBuilding());
  const { container } = render(Navigation);
  await screen.findByTestId("navigation-menu-gouvernance");
  const sidebar = container.querySelector('[data-testid="sidebar-desktop"]');
  if (!sidebar) throw new Error("sidebar-desktop introuvable");
  return sidebar;
}

/**
 * Aucune icône réemployée entre deux liens de destinations différentes,
 * dans le sous-arbre donné.
 */
function collisionsIcones(racine: Element): string[] {
  const liens = Array.from(racine.querySelectorAll("a[href]"));
  const parEmpreinte = new Map<string, string[]>();
  for (const lien of liens) {
    const svg = lien.querySelector("svg");
    if (!svg) continue;
    const empreinte = Array.from(svg.querySelectorAll("path"))
      .map((p) => p.getAttribute("d"))
      .join("|");
    const href = lien.getAttribute("href") ?? "(sans href)";
    parEmpreinte.set(empreinte, [...(parEmpreinte.get(empreinte) ?? []), href]);
  }

  return [...parEmpreinte.values()]
    .filter((hrefs) => hrefs.length > 1)
    .map((hrefs) => `  ${hrefs.join(" = ")}`);
}

describe("Navigation icônes (#797) @happy", () => {
  it("chaque icône rendue est un <svg> 24x24, stroke=currentColor, aria-hidden", async () => {
    const sidebar = await rendreSidebarSyndic();
    const icones = Array.from(sidebar.querySelectorAll("svg"));

    // Vérification d'aveuglement : sans elle, un sélecteur cassé rendrait
    // la liste vide et chaque assertion suivante passerait par vacuité.
    expect(icones.length).toBeGreaterThan(10);

    for (const svg of icones) {
      expect(svg.getAttribute("viewBox")).toBe("0 0 24 24");
      expect(svg.getAttribute("fill")).toBe("none");
      expect(svg.getAttribute("stroke")).toBe("currentColor");
      expect(svg.getAttribute("aria-hidden")).toBe("true");
    }
  });
});

describe("Navigation icônes (#797) @negative", () => {
  it("aucune icône n'est réemployée entre deux entrées, et aucune n'est un émoji", async () => {
    const sidebar = await rendreSidebarSyndic();

    // Aucun émoji dans le texte rendu de la sidebar (⚙️ notamment).
    const emojisTrouves = sidebar.textContent?.match(EMOJI) ?? [];
    expect(
      emojisTrouves.join(", "),
      `${emojisTrouves.length} émoji(s) dans la navigation du syndic : ` +
        `${emojisTrouves.join(", ")}. Un émoji n'est pas un système ` +
        "d'icônes : il est annoncé par les lecteurs d'écran et ne rend pas " +
        'pareil selon le système. Employez `<Icone nom="…" />`.',
    ).toBe("");

    // Aucune icône réemployée entre deux liens de destinations différentes.
    const collisions = collisionsIcones(sidebar);

    expect(
      collisions.join("\n"),
      "Ces entrées de la navigation du syndic partagent la même icône :\n" +
        collisions.join("\n") +
        "\n\nDeux destinations qui portent le même signe ne se distinguent " +
        "plus que par leur libellé — l'icône cesse d'informer.",
    ).toBe("");
  });

  it("aucune icône n'est réemployée entre le menu admin et le pied de sidebar (superadmin sans immeuble)", async () => {
    // Défaut réel trouvé en relisant le composant : `/admin/gdpr` (menu admin)
    // et `/settings/gdpr` (pied de sidebar, TOUJOURS rendu) portaient toutes
    // les deux l'icône `gdpr`. `rendreSidebarSyndic` ne peut pas voir cette
    // collision : le syndic n'a jamais le menu admin.
    const sidebar = await rendreSidebarAdminSansImmeuble();
    const collisions = collisionsIcones(sidebar);

    expect(
      collisions.join("\n"),
      "Ces entrées de la navigation admin partagent la même icône :\n" +
        collisions.join("\n") +
        "\n\nDeux destinations qui portent le même signe ne se distinguent " +
        "plus que par leur libellé — l'icône cesse d'informer.",
    ).toBe("");
  });

  it("aucune icône n'est réemployée entre gouvernance et mes-lots (membre du conseil)", async () => {
    // Second défaut réel, même forme : `/documents` (gouvernance) et
    // `/owner/documents` (mes-lots) portaient toutes les deux l'icône
    // `documents`. Seul un `board_member` voit les deux menus ensemble.
    const sidebar = await rendreSidebarMembreDuConseil();

    // Vérification d'aveuglement : sans elle, une régression de `canSee` qui
    // masquerait `mes-lots` pour le conseil ferait disparaître le défaut EN
    // MÊME TEMPS que le menu — la garde passerait au vert sans avoir rien vu.
    expect(
      sidebar.querySelector('[data-testid="navigation-menu-mes-lots"]'),
    ).not.toBeNull();

    const collisions = collisionsIcones(sidebar);

    expect(
      collisions.join("\n"),
      "Ces entrées de la navigation du conseil partagent la même icône :\n" +
        collisions.join("\n") +
        "\n\nDeux destinations qui portent le même signe ne se distinguent " +
        "plus que par leur libellé — l'icône cesse d'informer.",
    ).toBe("");
  });
});

describe("Navigation icônes (#797) @security", () => {
  it("aucune icône n'est annoncée : le nom accessible tient au lien qui l'entoure", async () => {
    const sidebar = await rendreSidebarSyndic();
    const svgs = Array.from(sidebar.querySelectorAll("svg"));
    expect(svgs.length).toBeGreaterThan(10);

    for (const svg of svgs) {
      // Décorative : aria-hidden retire l'icône de l'arbre d'accessibilité,
      // focusable="false" l'exclut du parcours clavier (IE/Edge historique
      // rendait les <svg> focusables par défaut).
      expect(svg.getAttribute("aria-hidden")).toBe("true");
      expect(svg.getAttribute("focusable")).toBe("false");
      // Une icône bavarde masquerait le libellé réel du lien : elle ne doit
      // porter ni nom accessible, ni rôle d'image.
      expect(svg.hasAttribute("aria-label")).toBe(false);
      expect(svg.hasAttribute("aria-labelledby")).toBe(false);
      expect(svg.getAttribute("role")).not.toBe("img");
    }

    // Le nom accessible réel vit sur le lien englobant.
    const lienBudgets = sidebar.querySelector('a[href="/budgets"]');
    expect(lienBudgets?.textContent?.trim().length).toBeGreaterThan(0);
  });
});
