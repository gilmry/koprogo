/**
 * Une annonce se publie, et seul son auteur le peut (#978, #979).
 *
 * ── Les deux défauts figés ici ────────────────────────────────────────────
 *
 * **#978 — le brouillon sans issue.** `Notice::new` crée en `Draft`, et le
 * domaine le commente : « Draft (not visible to others) ». La route de
 * sortie existait côté serveur (`notice_handlers.rs:383`) ; aucun appelant
 * ne l'empruntait. Toute annonce rédigée au formulaire restait donc
 * invisible des copropriétaires, pour toujours.
 *
 * Mesuré contre la recette le 2026-09-20 :
 *
 *     POST /notices                          -> Draft, published_at = None
 *     GET  /buildings/{id}/notices/published -> []
 *     POST /notices/{id}/publish             -> 200, Published
 *     GET  /buildings/{id}/notices/published -> 1 annonce
 *
 * **#979 — l'auteur qu'on ne reconnaît pas.** `isAuthor` comparait
 * `notice.author_id` (un `users.id`) au retour de `getMyOwner()` (un
 * `owners.id`). L'égalité ne pouvait pas se produire, et les trois actions
 * de l'auteur — publier, archiver, supprimer — vivent dans le même bloc.
 *
 * ── Ce que ce test couvre, et ce qu'il ne couvre pas ──────────────────────
 *
 * Il couvre le COMPOSANT : le bouton existe, il est réservé au brouillon et
 * à l'auteur, et il appelle bien la route de publication.
 *
 * Il ne couvre PAS le branchement de `notice-detail.astro`, qui est
 * précisément l'endroit où #979 vivait. Un test qui passe lui-même le
 * `currentUserId` ne peut pas voir qu'on le lit au mauvais endroit — c'est
 * exactement pourquoi le défaut a survécu.
 *
 * Ce branchement-là est tenu par `annonce.journey.ts`, où un vrai navigateur
 * ouvre la page avec une vraie session. Les deux niveaux sont nécessaires,
 * et aucun ne remplace l'autre.
 */
import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "../../test-helpers";

const publish = vi.fn();
const getById = vi.fn();

vi.mock("../../lib/api/notices", async () => {
  const reel = await vi.importActual<any>("../../lib/api/notices");
  return {
    ...reel,
    noticesApi: {
      getById: (...a: unknown[]) => getById(...a),
      publish: (...a: unknown[]) => publish(...a),
      archive: vi.fn(),
      delete: vi.fn(),
      incrementViewCount: vi.fn().mockResolvedValue(undefined),
    },
  };
});

vi.mock("../../stores/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), show: vi.fn() },
}));

const AUTEUR = "11111111-1111-4111-8111-111111111111";
const QUELQU_UN_DAUTRE = "22222222-2222-4222-8222-222222222222";

function uneAnnonce(statut: string) {
  return {
    id: "notice-1",
    building_id: "b-1",
    author_id: AUTEUR,
    author_name: "Sophie Syndic",
    notice_type: "Announcement",
    category: "Maintenance",
    title: "Travaux ascenseur",
    content: "L'ascenseur sera à l'arrêt du 5 au 9 octobre.",
    status: statut,
    is_pinned: false,
    published_at: statut === "Published" ? new Date().toISOString() : null,
    view_count: 0,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

async function monter(statut: string, currentUserId: string) {
  getById.mockResolvedValue(uneAnnonce(statut));
  const { default: NoticeDetail } = await import("./NoticeDetail.svelte");
  return render(NoticeDetail, { noticeId: "notice-1", currentUserId });
}

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("Annonce — publication (#978, #979)", () => {
  it("@happy l'auteur d'un brouillon peut le publier", async () => {
    await monter("Draft", AUTEUR);

    await waitFor(() => {
      expect(
        screen.getByTestId("notice-publish-btn"),
        "Sans ce bouton, une annonce rédigée au formulaire reste un " +
          "brouillon pour toujours et aucun copropriétaire ne la voit.",
      ).toBeTruthy();
    });
  });

  it("@negative une annonce déjà publiée n'offre pas de la republier", async () => {
    await monter("Published", AUTEUR);

    await waitFor(() => {
      expect(screen.getByTestId("notice-archive-btn")).toBeTruthy();
    });
    // Les deux états sont exclusifs. Laisser le bouton inviterait à un geste
    // sans effet, que l'utilisateur interpréterait comme une panne.
    expect(screen.queryByTestId("notice-publish-btn")).toBeNull();
  });

  it("@security quelqu'un d'autre ne publie ni n'archive l'annonce", async () => {
    await monter("Draft", QUELQU_UN_DAUTRE);

    await waitFor(() => {
      expect(screen.getByTestId("notice-detail")).toBeTruthy();
    });
    expect(screen.queryByTestId("notice-publish-btn")).toBeNull();
    expect(screen.queryByTestId("notice-archive-btn")).toBeNull();
    expect(screen.queryByTestId("notice-delete-btn")).toBeNull();
  });

  it("@edge un identifiant vide ne fait de personne un auteur", async () => {
    // Le cas exact de #979 : `getMyOwner()` rendait `null` pour un syndic,
    // donc `currentUserId` valait `""`. Que la chaîne vide n'ouvre aucune
    // action est la bonne moitié du comportement — c'est la comparaison
    // avec la mauvaise table qui était le défaut.
    await monter("Draft", "");

    await waitFor(() => {
      expect(screen.getByTestId("notice-detail")).toBeTruthy();
    });
    expect(screen.queryByTestId("notice-publish-btn")).toBeNull();
  });
});
