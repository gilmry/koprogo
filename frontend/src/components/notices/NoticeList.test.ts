import { render, screen, waitFor } from "../../test-helpers";
import { describe, it, expect, vi, beforeEach } from "vitest";
import NoticeList from "./NoticeList.svelte";

// Les fabriques de `vi.mock` sont hissées : chacune doit être autonome et ne
// fermer sur aucune déclaration du fichier.
vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

// `NoticeTypeBadge`, rendu par la liste, importe `svelte-i18n` DIRECTEMENT et
// non le module maison. Sans ce second doublon, il lève « Cannot format a
// message without first setting the initial locale », ce qui masquerait le
// défaut que ce test surveille.
vi.mock("svelte-i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
  // Exporté par le vrai module et consommé ailleurs dans l'arbre rendu.
  locale: {
    subscribe: (fn: (v: any) => void) => {
      fn("fr");
      return () => {};
    },
    set: () => {},
  },
  init: () => {},
  waitLocale: async () => {},
  getLocaleFromNavigator: () => "fr",
}));

const listByBuilding = vi.fn();
const listActive = vi.fn();

vi.mock("../../lib/api/notices", async () => {
  const reel = await vi.importActual<any>("../../lib/api/notices");
  return {
    ...reel,
    noticesApi: {
      listByBuilding: (...a: any[]) => listByBuilding(...a),
      listActive: (...a: any[]) => listActive(...a),
    },
  };
});

/// Ce que rend RÉELLEMENT `/buildings/{id}/notices` : un `NoticeSummaryDto`,
/// qui n'a PAS de champ `content`. Le type TypeScript généré déclare pourtant
/// `content: string`, parce qu'il décrit `NoticeResponseDto`, servi par une
/// autre route de la même famille.
const resumeSansContenu = {
  id: "de1ecd3c-2ad0-45a2-9197-0d5738b99ab0",
  building_id: "914f219e-057a-4cb2-8619-ddf8b3ce2dae",
  author_name: "RECETTE-Jean Syndic",
  notice_type: "Announcement",
  category: "General",
  title: "RECETTE4-Entretien des communs",
  status: "Draft",
  is_pinned: false,
  published_at: null,
  event_date: null,
  created_at: "2026-09-06T10:00:00Z",
  is_expired: false,
} as any;

describe("NoticeList — tolérance à une annonce sans contenu", () => {
  beforeEach(() => {
    listByBuilding.mockReset();
    listActive.mockReset();
  });

  /// Régression RN-7, constatée en recette les 2026-09-06 (recettes 3 et 4).
  ///
  /// `truncate(notice.content, 150)` faisait `undefined.length` PENDANT LE
  /// RENDU, donc hors du `try` de `withLoadingState`. Le `finally` qui remet
  /// `loading` à faux n'était jamais atteint : l'écran restait bloqué sur
  /// « Chargement des annonces… », indéfiniment et sans message.
  ///
  /// L'effet de bord était plus grave que le symptôme. La correction de RN-6
  /// fait atterrir l'utilisateur sur le filtre « Brouillon » juste après une
  /// création — c'est-à-dire précisément dans la vue qui ne chargeait jamais.
  /// Du point de vue du syndic, créer une annonce échouait toujours, alors que
  /// le serveur rendait 201 et que la donnée existait bien en base.
  it("affiche la liste au lieu de rester bloquée sur le chargement", async () => {
    listByBuilding.mockResolvedValue([resumeSansContenu]);
    listActive.mockResolvedValue([]);

    render(NoticeList, {
      props: {
        buildingId: "914f219e-057a-4cb2-8619-ddf8b3ce2dae",
        initialStatus: "Draft",
      },
    });

    await waitFor(
      () => {
        expect(screen.getByText(/RECETTE4-Entretien des communs/)).toBeInTheDocument();
      },
      { timeout: 3000 },
    );

    // Le témoin de chargement doit avoir disparu : c'est lui qui restait à
    // l'écran quand le rendu levait une exception.
    expect(
      document.querySelector('[data-testid="notice-list-loading"]'),
    ).toBeNull();
  });

});
