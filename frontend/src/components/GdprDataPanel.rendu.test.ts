import {
  render,
  screen,
  waitFor,
  fireEvent,
  cleanup,
} from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

/**
 * Le modal d'export RGPD s'ouvre vraiment.
 *
 * ── Ce que ce test rattrape ────────────────────────────────────────────────
 *
 * Le composant est en mode runes : il appelle `$props()`. Dans ce mode, un
 * `let showExportModal = false` **n'est pas réactif**. Le handler d'export le
 * passait bien à `true`, et l'interface ne se redessinait jamais. Le modal ne
 * pouvait donc pas s'ouvrir — jamais, pour personne.
 *
 * Quinze variables du composant étaient dans ce cas, dont les trois qui
 * commandent les trois modals et celle qui pilote l'indicateur de chargement.
 *
 * ── Pourquoi rien ne l'a vu ────────────────────────────────────────────────
 *
 * Le code compile. Les ancres `data-testid` sont toutes présentes dans la
 * source — d'où le diagnostic de #832 : « les ancres existent, les éléments ne
 * sont pas rendus ». Le test d'i18n du même composant lisait le fichier
 * comme du texte, sans jamais le monter.
 *
 * `svelte-check` le signalait pourtant, quinze fois. Mais la CI l'appelle avec
 * `--threshold warning`, qui **filtre l'affichage** et n'a jamais fait échouer
 * quoi que ce soit : c'est `--fail-on-warnings` qui bloque, et il est absent.
 * Un avertissement que rien ne lit n'est pas un avertissement.
 *
 * ── La forme du test ───────────────────────────────────────────────────────
 *
 * On monte le composant et on clique. C'est la seule façon de distinguer
 * « la variable change » de « l'écran change » — et c'est exactement la
 * distinction qui manquait.
 */

// `vi.mock` est hissé en tête de fichier : sa fabrique ne peut pas lire une
// variable déclarée ici. `vi.hoisted` monte la déclaration avec elle.
const { api } = vi.hoisted(() => ({
  api: { get: vi.fn(), post: vi.fn(), put: vi.fn(), delete: vi.fn() },
}));

vi.mock("../lib/api", () => ({ api }));
vi.mock("../stores/auth", () => ({
  authStore: {
    init: vi.fn().mockResolvedValue(undefined),
    subscribe: () => () => {},
  },
}));

import GdprDataPanel from "./GdprDataPanel.svelte";
import { setupI18n } from "../lib/i18n";

/** Un export conforme à `GdprExport` : le modal parcourt chacune des listes. */
const EXPORT_FICTIF = {
  export_date: "2026-09-07T10:00:00Z",
  user: {
    id: "u-1",
    email: "test@example.be",
    first_name: "Marcel",
    last_name: "Devos",
    is_active: true,
    is_anonymized: false,
    created_at: "2025-01-01T00:00:00Z",
  },
  owners: [],
  units: [],
  expenses: [],
  documents: [],
  meetings: [],
  total_items: 0,
};

// Sans ce démontage, les composants s'empilent et `getByTestId` voit deux
// boutons identiques : le test devient ininterprétable.
afterEach(cleanup);

beforeEach(() => {
  setupI18n();
  vi.clearAllMocks();
  api.get.mockImplementation((chemin: string) => {
    if (chemin === "/auth/me")
      return Promise.resolve({ email: "test@example.be" });
    if (chemin === "/gdpr/can-erase")
      return Promise.resolve({ can_erase: true, user_id: "u-1" });
    if (chemin === "/gdpr/export") return Promise.resolve(EXPORT_FICTIF);
    return Promise.resolve({});
  });
});

describe("le panneau RGPD réagit vraiment aux clics (#832)", () => {
  it("n'affiche pas le modal d'export avant qu'on le demande", async () => {
    render(GdprDataPanel);
    await waitFor(() =>
      expect(screen.getByTestId("gdpr-export-button")).toBeInTheDocument(),
    );
    expect(screen.queryByTestId("gdpr-export-modal")).not.toBeInTheDocument();
  });

  /**
   * LE test. Avec `let showExportModal = false` au lieu de `$state(false)`,
   * il échoue : l'appel réseau part, la variable passe à true, et le modal
   * n'apparaît pas.
   */
  it("ouvre le modal d'export après un clic sur le bouton", async () => {
    render(GdprDataPanel);
    const bouton = await screen.findByTestId("gdpr-export-button");

    await fireEvent.click(bouton);

    await waitFor(() => expect(api.get).toHaveBeenCalledWith("/gdpr/export"));
    await waitFor(
      () => expect(screen.getByTestId("gdpr-export-modal")).toBeInTheDocument(),
      {
        timeout: 2000,
      },
    );
    expect(screen.getByTestId("gdpr-export-modal-content")).toBeInTheDocument();
    expect(
      screen.getByTestId("gdpr-download-export-button"),
    ).toBeInTheDocument();
  });

  it("referme le modal d'export sur le bouton de fermeture", async () => {
    render(GdprDataPanel);
    await fireEvent.click(await screen.findByTestId("gdpr-export-button"));
    const fermer = await screen.findByTestId("gdpr-export-modal-close");

    await fireEvent.click(fermer);

    await waitFor(() =>
      expect(screen.queryByTestId("gdpr-export-modal")).not.toBeInTheDocument(),
    );
  });

  /**
   * Le modal d'effacement est commandé par une autre variable, qui souffrait
   * du même défaut. Deux modals, deux variables : en vérifier un seul aurait
   * laissé l'autre mort.
   */
  it("ouvre la confirmation d'effacement après un clic", async () => {
    render(GdprDataPanel);
    // Ce bouton vit dans un `{#if}` piloté par `checkingErasure` et `canErase`.
    // Les deux appels de `onMount` le font remplacer ; saisir le nœud trop tôt
    // revient à cliquer sur un élément déjà détaché du document.
    await waitFor(() => {
      expect(api.get).toHaveBeenCalledWith("/auth/me");
      expect(api.get).toHaveBeenCalledWith("/gdpr/can-erase");
    });
    const bouton = await screen.findByTestId("gdpr-erase-button");

    await fireEvent.click(bouton);

    await waitFor(() =>
      expect(
        screen.getByTestId("gdpr-erase-confirm-modal"),
      ).toBeInTheDocument(),
    );
  });
});
