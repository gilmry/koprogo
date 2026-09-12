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

/**
 * L'effacement RGPD envoie vraiment le mot de passe que le serveur exige.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * `GdprEraseRequestDto.password` est un `String`, pas un `Option` : le serveur
 * REFUSE tout effacement sans mot de passe, et le vérifie au bcrypt. Son
 * propre commentaire l'explique — « deux `confirm()` côté navigateur ne
 * prouvent rien, et un appel direct à l'API les ignorait tout à fait ».
 *
 * L'interface appelait `api.delete("/gdpr/erase")` **sans aucun corps**, et la
 * modale de confirmation ne demandait pas de mot de passe. Le droit à
 * l'effacement de l'Article 17 était donc inatteignable pour tout le monde :
 * la modale s'ouvrait, le bouton cliquait, et l'appel repartait en erreur.
 *
 * ── Pourquoi rien ne l'a vu ────────────────────────────────────────────────
 *
 * Deux specs Playwright couvraient ce parcours et échouaient toutes les deux
 * (#832). Leur échec a été classé « l'élément n'est pas rendu » : on cherchait
 * la panne du côté de l'affichage, alors qu'elle était dans le contrat entre
 * les deux moitiés du produit. Le champ obligatoire a été ajouté au serveur
 * sans être ajouté à l'écran — la même forme que RN-17, où le bouton avait été
 * ouvert au syndic sans que le champ le soit.
 */
describe("l'effacement RGPD (Art. 17) envoie le mot de passe exigé (#832)", () => {
  async function ouvrirLaModaleDeffacement() {
    render(GdprDataPanel);
    // `onMount` appelle `checkCanErase`, qui bascule `checkingErasure` puis
    // le rebascule. La section d'effacement est donc rendue DEUX fois, et le
    // second rendu remplace le nœud du bouton. Une référence capturée trop
    // tôt pointe sur un nœud détaché : le clic part dans le vide, sans
    // erreur. On attend que le va-et-vient soit fini, puis on requête.
    await waitFor(() =>
      expect(api.get).toHaveBeenCalledWith("/gdpr/can-erase"),
    );
    await waitFor(() =>
      expect(screen.getByTestId("gdpr-erase-button")).toBeEnabled(),
    );
    await fireEvent.click(screen.getByTestId("gdpr-erase-button"));
    return waitFor(() =>
      expect(
        screen.getByTestId("gdpr-erase-confirm-modal"),
      ).toBeInTheDocument(),
    );
  }

  it("laisse le bouton de confirmation inerte tant que le mot de passe est vide", async () => {
    await ouvrirLaModaleDeffacement();
    const confirmer = screen.getByTestId("gdpr-erase-confirm-button");
    expect(confirmer).toBeDisabled();
    expect(api.delete).not.toHaveBeenCalled();
  });

  /**
   * LE test. Avec `api.delete("/gdpr/erase")` sans corps, il échoue : l'appel
   * part bien, mais sans le mot de passe que le serveur exige.
   */
  it("transmet le mot de passe saisi dans le corps de la requête", async () => {
    api.delete.mockResolvedValue({
      owners_anonymized: 2,
      anonymized_at: "2026-09-08T06:00:00Z",
    });

    await ouvrirLaModaleDeffacement();
    const champ = screen.getByTestId("gdpr-panel-erase-password");
    await fireEvent.input(champ, { target: { value: "MotDePasse!42" } });

    await fireEvent.click(screen.getByTestId("gdpr-erase-confirm-button"));

    await waitFor(() => expect(api.delete).toHaveBeenCalledTimes(1));
    const [chemin, options] = api.delete.mock.calls[0];
    expect(chemin).toBe("/gdpr/erase");
    // On assert la VALEUR transmise, pas la forme du corps : c'est le contrat
    // avec le serveur, et il ne doit pas dépendre de la sérialisation choisie.
    expect(JSON.parse(options.body)).toEqual({ password: "MotDePasse!42" });
  });
});
