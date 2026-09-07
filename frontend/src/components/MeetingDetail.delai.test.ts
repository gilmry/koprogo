import { render, screen, waitFor, cleanup } from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

/**
 * L'écran d'assemblée dit jusqu'à quand la convocation peut partir.
 *
 * ── Ce que ce test rattrape ────────────────────────────────────────────────
 *
 * Le serveur calcule `date_limite_envoi_convocation`,
 * `convocation_encore_possible` et `jours_manquants_convocation` depuis
 * `delai_de_convocation.rs` (Art. 3.87 § 3). Le frontend les déclarait dans
 * son type `Meeting`.
 *
 * **Aucun écran ne les affichait.**
 *
 * Le syndic découvrait donc la règle des quinze jours au moment de cliquer sur
 * « Créer une convocation », dans un refus en anglais, sans autre issue que de
 * supprimer l'assemblée. C'est le verrou 1 de #780, constaté au navigateur en
 * recette 4.
 *
 * Un refus qui arrive quand il ne reste plus qu'à le subir n'est pas une
 * garde, c'est une sanction.
 *
 * ── Pourquoi un test de rendu ──────────────────────────────────────────────
 *
 * Le champ existait dans `types.ts`, et un test de type l'aurait trouvé
 * conforme. Seul le montage du composant distingue « la donnée est servie »
 * de « l'écran la montre » — c'est la même distinction qui manquait pour les
 * modals RGPD (#832).
 */

const { api } = vi.hoisted(() => ({
  api: { get: vi.fn(), post: vi.fn(), put: vi.fn(), delete: vi.fn() },
}));

vi.mock("../lib/api", () => ({ api }));
vi.mock("../stores/auth", () => ({
  authStore: {
    subscribe: (fn: (v: unknown) => void) => {
      fn({ user: { role: "syndic", activeRole: { role: "syndic" } } });
      return () => {};
    },
  },
}));

import MeetingDetail from "./MeetingDetail.svelte";
import { setupI18n } from "../lib/i18n";

const ASSEMBLEE_ID = "11111111-2222-4333-8444-555555555555";

function assemblee(surcharge: Record<string, unknown>) {
  return {
    id: ASSEMBLEE_ID,
    building_id: "b-1",
    title: "Assemblée générale ordinaire 2026",
    meeting_type: "Ordinary",
    status: "Scheduled",
    scheduled_date: "2026-10-15T18:00:00Z",
    location: "Salle communale",
    ...surcharge,
  };
}

afterEach(cleanup);

beforeEach(() => {
  setupI18n();
  vi.clearAllMocks();
  window.history.replaceState({}, "", `/?id=${ASSEMBLEE_ID}`);
});

function servir(m: Record<string, unknown>) {
  api.get.mockImplementation((chemin: string) => {
    if (chemin === `/meetings/${ASSEMBLEE_ID}`) return Promise.resolve(m);
    if (chemin.startsWith("/buildings/")) return Promise.resolve(null);
    return Promise.resolve({ missing: [] });
  });
}

describe("l'écran d'assemblée annonce le délai de convocation (#780)", () => {
  it("affiche la date limite d'envoi quand le délai est encore tenable", async () => {
    servir(
      assemblee({
        convocation_encore_possible: true,
        date_limite_envoi_convocation: "2026-09-30T18:00:00Z",
        jours_manquants_convocation: null,
      }),
    );

    render(MeetingDetail);

    const bloc = await screen.findByTestId(
      "meeting-convocation-deadline",
      {},
      { timeout: 3000 },
    );
    expect(bloc).toBeInTheDocument();
    expect(
      screen.queryByTestId("meeting-convocation-too-late"),
    ).not.toBeInTheDocument();
  });

  /**
   * LE cas de #780 : l'assemblée est trop proche. L'écran doit le dire, et
   * dire de combien — sans quoi le syndic ne sait pas de quel montant reculer.
   */
  it("avertit, en nommant les jours manquants, quand le délai ne peut plus être tenu", async () => {
    servir(
      assemblee({
        convocation_encore_possible: false,
        date_limite_envoi_convocation: null,
        jours_manquants_convocation: 6,
      }),
    );

    render(MeetingDetail);

    const alerte = await screen.findByTestId(
      "meeting-convocation-too-late",
      {},
      { timeout: 3000 },
    );
    expect(alerte).toHaveTextContent("6");
    // L'article qui fonde la règle est cité : le syndic doit pouvoir vérifier.
    expect(alerte).toHaveTextContent("3.87");
  });

  /**
   * L'avertissement ne doit pas devenir du bruit sur une assemblée passée ou
   * annulée, où il n'y a plus rien à décider.
   */
  it("ne dit rien sur une assemblée déjà terminée", async () => {
    servir(
      assemblee({
        status: "Completed",
        convocation_encore_possible: false,
        jours_manquants_convocation: 6,
      }),
    );

    render(MeetingDetail);

    await waitFor(() => expect(api.get).toHaveBeenCalled());
    await waitFor(() =>
      expect(
        screen.queryByTestId("meeting-convocation-too-late"),
      ).not.toBeInTheDocument(),
    );
  });
});
