import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

/**
 * Le bouton « générer le rapport » déclenche vraiment l'appel.
 *
 * ── Ce que ce test rattrape ────────────────────────────────────────────────
 *
 * `AccountantReportsJourney.spec.ts` cherchait le bouton par son libellé
 * FRANÇAIS — `getByRole("button", { name: "Générer le rapport" })` — sur un
 * composant traduit en quatre langues. Dès que la langue résolue n'est pas le
 * français, le bouton est introuvable : le clic ne part jamais, et c'est le
 * `waitForResponse` d'à côté qui expire au bout de dix secondes.
 *
 * Le symptôme — « Timeout while waiting for event "response" » — ne dit rien
 * de la cause, et oriente vers le réseau ou le serveur. C'est la troisième
 * fois de la journée qu'une assertion sur une FORMULATION se déguise en panne
 * d'autre chose.
 *
 * Une assertion peut exiger une ancre, une valeur interpolée, ou une
 * structure. Jamais une formulation.
 */

const { api } = vi.hoisted(() => ({
  api: { get: vi.fn(), post: vi.fn(), put: vi.fn(), delete: vi.fn() },
}));
vi.mock("../lib/api", () => ({ api }));

import FinancialReports from "./FinancialReports.svelte";
import { setupI18n } from "../lib/i18n";

afterEach(cleanup);
beforeEach(() => {
  setupI18n();
  vi.clearAllMocks();
  api.get.mockResolvedValue({ assets: [], liabilities: [], equity: [] });
});

describe("les rapports financiers s'ancrent, ils ne se nomment pas (#832)", () => {
  it("expose une ancre stable sur le bouton de génération", () => {
    render(FinancialReports);
    expect(
      screen.getByTestId("financial-reports-generate"),
    ).toBeInTheDocument();
  });

  /**
   * LE test. Le bilan est le type de rapport par défaut : un clic doit
   * appeler `/reports/balance-sheet`, quelle que soit la langue affichée.
   */
  it("appelle /reports/balance-sheet au clic, indépendamment de la langue", async () => {
    render(FinancialReports);
    await fireEvent.click(screen.getByTestId("financial-reports-generate"));
    await waitFor(() =>
      expect(api.get).toHaveBeenCalledWith("/reports/balance-sheet"),
    );
  });
});
