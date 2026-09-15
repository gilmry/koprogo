// #868 — Deux BuildingSelector coexistent, et partageaient un ancrage.
//
// CRITICAL §3 — RED-first : ce fichier reproduit le doublon réel plutôt que
// de le supposer. Avant le renommage de `components/BuildingSelector.svelte`
// (`building-selector-empty` → `page-building-selector-empty`), le test
// `@negative` ci-dessous plantait avec une "strict mode violation" :
// `screen.getByTestId("building-selector-empty")` trouvait DEUX noeuds, l'un
// dans ce composant (l'ancien, monté par page), l'autre dans
// `global/BuildingSelector.svelte` (la barre de contexte) — exactement le
// scénario des 14 pages (skills, polls, sharing, ...) où syndic/accountant/
// superadmin voient les deux à la fois.
//
// Reproduction : les deux composants sont montés dans le MÊME document (pas
// de `cleanup()` entre les deux `render()`), comme ils le sont réellement
// sur une page Astro qui monte l'ancien localement pendant que
// `BarreDeContexte` monte le global.

import { describe, it, expect, vi, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "../test-helpers";
import PageBuildingSelector from "./BuildingSelector.svelte";
import GlobalBuildingSelector from "./global/BuildingSelector.svelte";
import { resetScope } from "../stores/scope.svelte";
import { UserRole } from "../lib/types";
import type { User } from "../lib/types";

vi.mock("../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: (key: string) => string) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("../lib/api", () => ({
  api: {
    get: vi.fn((url: string) => {
      if (url.startsWith("/acps")) return Promise.resolve({ acps: [] });
      return Promise.resolve({ buildings: [] });
    }),
  },
}));

vi.mock("../lib/api/buildings", () => ({
  searchBuildings: vi.fn().mockResolvedValue([]),
  listBuildings: vi.fn().mockResolvedValue({
    data: [],
    pagination: { page: 1, per_page: 20, total_items: 0, total_pages: 0 },
  }),
  getBuilding: vi.fn(),
}));

vi.mock("../lib/api/portfolios", () => ({
  listPortfolios: vi.fn().mockResolvedValue([]),
  listPortfolioBuildings: vi.fn().mockResolvedValue([]),
  toggleFavorite: vi.fn(),
}));

import { api } from "../lib/api";
const mockedApiGet = vi.mocked(api.get);

const syndicUser: User = {
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
} as User;

/**
 * Monte les deux composants tels qu'ils le sont réellement sur une des 14
 * pages (ex. skills.astro monte l'ancien ; `BarreDeContexte` monte le
 * global) pour un syndic sans immeuble. Ouvre le global au focus pour
 * atteindre son état vide, comme le ferait un utilisateur qui clique dans
 * le champ.
 */
async function monteLesDeuxEnEtatVide() {
  render(PageBuildingSelector, { props: {} });
  render(GlobalBuildingSelector, { props: { user: syndicUser } });

  const input = await screen.findByTestId("building-selector-input");
  await fireEvent.focus(input);

  await waitFor(() => {
    expect(screen.queryByTestId("building-selector-empty")).toBeTruthy();
  });
}

function defaultApiGet(url: string) {
  if (url.startsWith("/acps")) return Promise.resolve({ acps: [] });
  return Promise.resolve({ buildings: [] });
}

afterEach(() => {
  cleanup();
  resetScope();
  vi.clearAllMocks();
  // `mockImplementation` (utilisé par le test @security) survit à
  // `clearAllMocks` — le remettre explicitement évite qu'un test ultérieur
  // hérite silencieusement d'un /buildings qui échoue.
  mockedApiGet.mockImplementation(defaultApiGet);
});

describe("#868 — building-selector-empty ne désigne plus deux composants @negative", () => {
  it("getByTestId (strict) résout sans lever de strict-mode violation", async () => {
    await monteLesDeuxEnEtatVide();

    // Avant le fix, les deux composants posaient `building-selector-empty` :
    // `getByTestId` (mode strict, lève si >1 noeud) plantait ici.
    expect(() => screen.getByTestId("building-selector-empty")).not.toThrow();
  });

  it("exactement un noeud répond à building-selector-empty", async () => {
    await monteLesDeuxEnEtatVide();

    expect(screen.queryAllByTestId("building-selector-empty")).toHaveLength(
      1,
    );
  });
});

describe("#868 — les deux ancres restent atteignables sans ambiguïté @happy", () => {
  it("building-selector-empty désigne la barre de contexte (global), pas la page", async () => {
    await monteLesDeuxEnEtatVide();

    const empty = screen.getByTestId("building-selector-empty");
    // Le noeud vide du global vit dans la listbox du dropdown, jamais dans
    // l'ancien composant (qui n'a pas de role="listbox").
    expect(empty.closest("#building-selector-listbox")).not.toBeNull();
  });

  it("page-building-selector-empty désigne le sélecteur local de la page", async () => {
    await monteLesDeuxEnEtatVide();

    const pageEmpty = screen.getByTestId("page-building-selector-empty");
    expect(pageEmpty.closest("#building-selector-listbox")).toBeNull();
  });
});

describe("#868 — les deux ancres cohabitent sans collision @edge", () => {
  it("les deux data-testid distincts résolvent chacun un seul noeud simultanément", async () => {
    await monteLesDeuxEnEtatVide();

    expect(screen.queryAllByTestId("page-building-selector-empty")).toHaveLength(
      1,
    );
    expect(screen.queryAllByTestId("building-selector-empty")).toHaveLength(1);
  });
});

describe("#868 — un refus de périmètre n'est jamais confondu avec une absence légitime @security", () => {
  /**
   * `building-selector-403` n'existe que dans le composant global — l'ancien
   * ne l'implémente pas. Ça ne suffit pas à dire que l'ancien est sûr : un
   * ancrage recouvert peut masquer qu'un refus a cessé de s'afficher, donc
   * l'assertion doit porter sur le COMPORTEMENT face à un refus, pas sur la
   * simple absence du noeud `-403` (qui passerait aussi si l'ancien
   * plantait, ou masquait silencieusement l'erreur).
   *
   * Le comportement attendu : quand `/buildings` échoue (403 backend inclus
   * — le composant ne distingue pas les codes HTTP, `withLoadingState` les
   * traite tous comme un échec), l'ancien affiche SON état d'erreur
   * (`building-selector-error` + `building-selector-retry`), jamais l'état
   * vide (`page-building-selector-empty`). Un refus de périmètre ne doit
   * jamais se lire comme « vous n'avez aucun immeuble » : l'utilisateur
   * perdrait le seul indice qu'il s'agit d'un refus et non d'un fait.
   */
  it("un échec de /buildings (403 inclus) rend l'état d'erreur, jamais l'état vide", async () => {
    mockedApiGet.mockImplementation((url: string) => {
      if (url.startsWith("/acps")) return Promise.resolve({ acps: [] });
      const err = new Error("Forbidden") as Error & { status?: number };
      err.status = 403;
      return Promise.reject(err);
    });

    render(PageBuildingSelector, { props: {} });

    await waitFor(() => {
      expect(screen.queryByTestId("building-selector-error")).toBeTruthy();
    });

    expect(screen.queryByTestId("page-building-selector-empty")).toBeNull();
    expect(screen.getByTestId("building-selector-retry")).toBeInTheDocument();
    // Et il n'a toujours pas de -403 à lui : ce n'est pas son contrat.
    expect(screen.queryByTestId("building-selector-403")).toBeNull();
  });
});
