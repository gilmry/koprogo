import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/svelte";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

/**
 * Les cinq actions de l'écran de convocation sont atteignables.
 *
 * ── Ce que ce test rattrape ────────────────────────────────────────────────
 *
 * Cet écran passait par quatre `confirm()` et un `prompt()` NATIFS. Ce sont
 * des dialogues du navigateur, pas de la page : un navigateur piloté les
 * supprime, et l'action prend alors la forme exacte d'une panne — aucun
 * dialogue, aucune requête, aucun message.
 *
 * C'est le diagnostic qui a coûté le plus cher au produit. Le bouton
 * « Reporter » d'une assemblée a été rapporté mort en recette 3 PUIS en
 * recette 5, alors que sa source était correcte depuis `fbccd7b0` : l'outil de
 * recette avalait le `prompt()`.
 *
 * Cet écran-ci est celui du deuxième verrou de #780 — la convocation qui ne
 * part jamais. Il ne peut pas se permettre d'être intestable, et cinq
 * dialogues natifs en faisaient le fichier le plus chargé du dépôt (#844).
 *
 * ── La forme du test ───────────────────────────────────────────────────────
 *
 * On monte, on clique, et on vérifie que l'appel PART. C'est la seule façon de
 * distinguer « le gestionnaire est câblé » de « l'action aboutit ».
 */

const { convocationsApi } = vi.hoisted(() => ({
  convocationsApi: {
    send: vi.fn(),
    cancel: vi.fn(),
    sendReminders: vi.fn(),
    delete: vi.fn(),
    schedule: vi.fn(),
    getTracking: vi.fn(),
  },
}));

vi.mock("../../lib/api/convocations", async (importer) => {
  const reel = await importer<typeof import("../../lib/api/convocations")>();
  return { ...reel, convocationsApi };
});

// `authStore` n'expose pas `set` — il n'expose que `subscribe` et ses
// méthodes métier. On le remplace donc par un magasin minimal : l'écran
// réserve ses actions au syndic, et c'est cette lecture-là qu'il faut
// satisfaire.
vi.mock("../../stores/auth", () => ({
  authStore: {
    subscribe: (fn: (v: unknown) => void) => {
      fn({
        isAuthenticated: true,
        user: { id: "u-1", email: "s@e.be", role: "syndic" },
        loading: false,
      });
      return () => {};
    },
    init: vi.fn().mockResolvedValue(undefined),
  },
}));

import ConvocationDetailView from "./ConvocationDetailView.svelte";
import { ConvocationStatus, MeetingType } from "../../lib/api/convocations";
import { setupI18n } from "../../lib/i18n";

function convocation(status: string) {
  return {
    id: "c-1",
    meeting_id: "m-1",
    building_id: "b-1",
    organization_id: "o-1",
    meeting_type: MeetingType.Ordinary,
    meeting_date: "2026-11-15T18:00:00Z",
    minimum_send_date: "2026-10-31T18:00:00Z",
    status,
    language: "FR",
    total_recipients: 3,
    opened_count: 0,
    will_attend_count: 0,
    respects_legal_deadline: true,
    created_at: "2026-09-01T10:00:00Z",
    updated_at: "2026-09-01T10:00:00Z",
  } as any;
}

afterEach(cleanup);

beforeEach(() => {
  setupI18n();
  vi.clearAllMocks();
  convocationsApi.getTracking.mockResolvedValue(null);
  convocationsApi.send.mockResolvedValue(convocation(ConvocationStatus.Sent));
  convocationsApi.cancel.mockResolvedValue(convocation("Cancelled"));
  convocationsApi.sendReminders.mockResolvedValue({});
  convocationsApi.delete.mockResolvedValue({});
  convocationsApi.schedule.mockResolvedValue(
    convocation(ConvocationStatus.Scheduled),
  );
});

describe("l'écran de convocation n'a plus de dialogue natif (#844, #780)", () => {
  it("demande confirmation dans la PAGE avant d'envoyer, puis envoie", async () => {
    render(ConvocationDetailView, {
      props: { convocation: convocation(ConvocationStatus.Draft) },
    });

    await fireEvent.click(
      await screen.findByTestId("convocation-detail-btn-send"),
    );

    // Le dialogue est dans la page : un navigateur piloté peut le voir.
    const confirmer = await screen.findByTestId("confirm-dialog-confirm");
    expect(convocationsApi.send).not.toHaveBeenCalled();

    await fireEvent.click(confirmer);
    await waitFor(() =>
      expect(convocationsApi.send).toHaveBeenCalledWith("c-1"),
    );
  });

  it("renonce sans appeler quoi que ce soit quand on annule", async () => {
    render(ConvocationDetailView, {
      props: { convocation: convocation(ConvocationStatus.Draft) },
    });

    await fireEvent.click(
      await screen.findByTestId("convocation-detail-btn-send"),
    );
    await fireEvent.click(await screen.findByTestId("confirm-dialog-cancel"));

    await waitFor(() =>
      expect(screen.queryByTestId("confirm-dialog-confirm")).toBeNull(),
    );
    expect(convocationsApi.send).not.toHaveBeenCalled();
  });

  /**
   * LE test du `prompt()`. Une date saisie dans la page, pas dans une boîte
   * que le navigateur piloté ferait disparaître.
   */
  it("saisit la date de programmation dans la page, et la transmet", async () => {
    render(ConvocationDetailView, {
      props: { convocation: convocation(ConvocationStatus.Draft) },
    });

    await fireEvent.click(
      await screen.findByTestId("convocation-detail-btn-schedule"),
    );

    const champ = await screen.findByTestId("convocation-schedule-date-input");
    // Tant que la date est vide, la confirmation reste inerte : dire le refus
    // avant la soumission, pas après.
    expect(screen.getByTestId("convocation-schedule-submit")).toBeDisabled();

    await fireEvent.input(champ, { target: { value: "2026-10-20T09:00" } });
    await fireEvent.click(screen.getByTestId("convocation-schedule-submit"));

    await waitFor(() =>
      expect(convocationsApi.schedule).toHaveBeenCalledWith(
        "c-1",
        "2026-10-20T09:00",
      ),
    );
  });

  it("supprime après confirmation, et pas avant", async () => {
    render(ConvocationDetailView, {
      props: { convocation: convocation(ConvocationStatus.Draft) },
    });

    await fireEvent.click(
      await screen.findByTestId("convocation-detail-btn-delete"),
    );
    expect(convocationsApi.delete).not.toHaveBeenCalled();

    await fireEvent.click(await screen.findByTestId("confirm-dialog-confirm"));
    await waitFor(() =>
      expect(convocationsApi.delete).toHaveBeenCalledWith("c-1"),
    );
  });
});
