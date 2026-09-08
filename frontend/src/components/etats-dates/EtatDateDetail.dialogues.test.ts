import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

/**
 * Les quatre actions de l'état daté sont atteignables.
 *
 * ── Pourquoi cet écran plutôt qu'un autre ─────────────────────────────────
 *
 * L'état daté est demandé par un notaire à la vente d'un lot, et l'Art. 3.94
 * donne au syndic **quinze jours ouvrables** pour le transmettre. C'est un
 * chemin critique et daté.
 *
 * Il passait par trois `confirm()` et un `prompt()` NATIFS. Un navigateur
 * piloté les supprime, et l'action prend alors la forme exacte d'une panne :
 * aucun dialogue, aucune requête, aucun message. C'est ce qui a fait déclarer
 * mort le bouton « Reporter » d'une assemblée pendant deux recettes, alors que
 * sa source était correcte (#780, #844).
 *
 * ── Un piège évité, et il valait la peine ─────────────────────────────────
 *
 * Ce composant est en mode LEGACY : il n'appelle pas `$props()`, et ses `let`
 * sont réactifs tels quels. Y introduire un seul `$state` l'aurait basculé en
 * mode runes et aurait rendu **tous les autres `let` non réactifs** — le
 * défaut de #832, où quinze variables de `GdprDataPanel` avaient cessé de
 * redessiner l'écran et où trois modales ne pouvaient plus s'ouvrir.
 *
 * Ces tests montent le composant et cliquent : c'est la seule façon de
 * distinguer « la variable change » de « l'écran change ».
 */

const { etatsDatesApi } = vi.hoisted(() => ({
  etatsDatesApi: {
    getById: vi.fn(),
    markInProgress: vi.fn(),
    markGenerated: vi.fn(),
    markDelivered: vi.fn(),
    delete: vi.fn(),
  },
}));

vi.mock("../../lib/api/etats-dates", async (importer) => {
  const reel = await importer<typeof import("../../lib/api/etats-dates")>();
  return { ...reel, etatsDatesApi };
});

import EtatDateDetail from "./EtatDateDetail.svelte";
import { setupI18n } from "../../lib/i18n";

/**
 * Le gabarit formate onze champs. En omettre un fait échouer le rendu sur un
 * `toFixed` d'`undefined`, et l'écran reste bloqué sur son spinner — ce qui
 * ressemble à s'y méprendre à un composant non réactif.
 *
 * Je l'ai d'abord lu ainsi, et j'ai cru avoir trouvé un second défaut de la
 * famille #832. C'était mon jeu d'essai. Vérifié en montant la version
 * ORIGINALE du composant, puis en confirmant qu'un composant legacy voisin
 * (`FinancialReports`) se redessine bien dans ce même harnais.
 */
/**
 * Les statuts sont en `snake_case` minuscule — `requested`, `in_progress`,
 * `generated` — et chaque bouton n'existe que pour le sien. Un jeu d'essai en
 * `Requested` ne montre aucun bouton, et l'écran ressemble alors à un écran
 * cassé.
 */
function etatDateAuStatut(status: string) {
  return { ...ETAT_DATE, status };
}

const ETAT_DATE = {
  id: "ed-1",
  building_id: "b-1",
  unit_id: "u-1",
  status: "requested",
  reference_number: "ED-2026-001",
  notary_name: "Maître Dupont",
  notary_email: "notaire@example.be",
  reference_date: "2026-09-01T10:00:00Z",
  requested_date: "2026-09-01T10:00:00Z",
  generated_date: null,
  delivered_date: null,
  ordinary_charges_quota: 12.5,
  extraordinary_charges_quota: 8.25,
  owner_balance: 1250.5,
  arrears_amount: 0,
  monthly_provision_amount: 180,
  total_balance: 1250.5,
  approved_works_unpaid: 0,
} as any;

afterEach(cleanup);

beforeEach(() => {
  setupI18n();
  vi.clearAllMocks();
  etatsDatesApi.getById.mockResolvedValue(etatDateAuStatut("requested"));
  etatsDatesApi.markInProgress.mockResolvedValue({
    ...ETAT_DATE,
    status: "InProgress",
  });
  etatsDatesApi.markGenerated.mockResolvedValue({
    ...ETAT_DATE,
    status: "Generated",
  });
  etatsDatesApi.markDelivered.mockResolvedValue({
    ...ETAT_DATE,
    status: "Delivered",
  });
  etatsDatesApi.delete.mockResolvedValue(undefined);
  Object.defineProperty(window, "location", {
    writable: true,
    value: { ...window.location, search: "?id=ed-1", href: "" },
  });
});

describe("l'état daté n'a plus de dialogue natif (#844, Art. 3.94)", () => {
  it("demande confirmation dans la PAGE avant de passer en traitement", async () => {
    render(EtatDateDetail);

    await fireEvent.click(await screen.findByTestId("mark-in-progress-button"));
    expect(etatsDatesApi.markInProgress).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByTestId("confirm-dialog-confirm"));
    await waitFor(() =>
      expect(etatsDatesApi.markInProgress).toHaveBeenCalledWith("ed-1"),
    );
  });

  it("renonce sans rien appeler quand on annule", async () => {
    render(EtatDateDetail);

    await fireEvent.click(await screen.findByTestId("mark-in-progress-button"));
    await fireEvent.click(await screen.findByTestId("confirm-dialog-cancel"));

    await waitFor(() =>
      expect(screen.queryByTestId("confirm-dialog-confirm")).toBeNull(),
    );
    expect(etatsDatesApi.markInProgress).not.toHaveBeenCalled();
  });

  /**
   * LE test du `prompt()`. Le chemin du PDF se saisit dans la page, et la
   * confirmation reste inerte tant qu'il est vide : dire le refus avant la
   * soumission, pas après.
   */
  it("saisit le chemin du PDF dans la page, et le transmet", async () => {
    // Le bouton « marquer généré » n'existe qu'au statut `in_progress`.
    etatsDatesApi.getById.mockResolvedValue(etatDateAuStatut("in_progress"));
    render(EtatDateDetail);

    await fireEvent.click(await screen.findByTestId("mark-generated-button"));

    const champ = await screen.findByTestId("etat-date-pdf-path-input");
    expect(screen.getByTestId("etat-date-pdf-submit")).toBeDisabled();

    await fireEvent.input(champ, { target: { value: "/pdf/ed-2026-001.pdf" } });
    await fireEvent.click(screen.getByTestId("etat-date-pdf-submit"));

    await waitFor(() =>
      expect(etatsDatesApi.markGenerated).toHaveBeenCalledWith(
        "ed-1",
        "/pdf/ed-2026-001.pdf",
      ),
    );
  });

  it("supprime après confirmation, et pas avant", async () => {
    render(EtatDateDetail);

    await fireEvent.click(await screen.findByTestId("delete-etat-date-button"));
    expect(etatsDatesApi.delete).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByTestId("confirm-dialog-confirm"));
    await waitFor(() =>
      expect(etatsDatesApi.delete).toHaveBeenCalledWith("ed-1"),
    );
  });
});
