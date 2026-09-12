import { render, screen, waitFor, fireEvent } from "../../test-helpers";
import { describe, it, expect, vi, beforeEach } from "vitest";
import BuildingForm from "./BuildingForm.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("svelte-i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
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

const acpsServies = vi.fn();
vi.mock("../../lib/api/acps", () => ({
  listAcps: () => acpsServies(),
}));

const postEnvoye = vi.fn();
vi.mock("../../lib/api", () => ({
  api: {
    post: (chemin: string, corps: unknown) => postEnvoye(chemin, corps),
    put: (chemin: string, corps: unknown) => postEnvoye(chemin, corps),
    get: async () => ({}),
  },
}));

/** Le rôle connecté, que le composant lit dans le store d'authentification. */
let roleCourant = "syndic";
vi.mock("../../stores/auth", () => ({
  authStore: {
    subscribe: (fn: (v: any) => void) => {
      fn({ user: { id: "u-1", role: roleCourant }, isAuthenticated: true });
      return () => {};
    },
  },
}));

const ACP = {
  id: "4f5ddb45-0fea-42cd-b868-278a4adb5dfb",
  name: "RECETTE-ACP Résidence des Érables",
  address_street: "Rue des Érables 42",
  address_postal_code: "1000",
  address_city: "Bruxelles",
};

describe("BuildingForm — un syndic doit pouvoir désigner l'ACP (#783)", () => {
  beforeEach(() => {
    roleCourant = "syndic";
    acpsServies.mockReset();
    postEnvoye.mockReset();
    postEnvoye.mockResolvedValue({ id: "b-1" });
  });

  /// Régression RN-17, constatée en recette le 2026-09-06.
  ///
  /// Le serveur a ouvert `POST /buildings` au syndic le 2026-09-04, et le
  /// BOUTON a suivi le 2026-09-05 (#778). Le CHAMP ACP, lui, est resté
  /// derrière `isSuperAdmin` : aucune ACP chargée, aucun select affiché,
  /// `acp_id` absent du corps, et la validation qui aurait pu l'arrêter
  /// gardée pareillement. D'où un `400 missing field `acp_id`` illisible.
  ///
  /// Aucun test n'exerçait ce chemin — ni unitaire, ni end-to-end : les e2e de
  /// création d'immeuble passent tous par un compte admin, et le test backend
  /// fournit `acp_id`. C'est le trou par lequel le défaut est passé.
  it("charge les ACP et affiche le select pour un syndic", async () => {
    acpsServies.mockResolvedValue([
      ACP,
      { ...ACP, id: "autre", name: "Seconde ACP" },
    ]);

    render(BuildingForm, { props: { isOpen: true, mode: "create" } });

    await waitFor(() => expect(acpsServies).toHaveBeenCalled(), {
      timeout: 3000,
    });
    await waitFor(
      () =>
        expect(
          document.querySelector('[data-testid="building-acp-select"]'),
        ).not.toBeNull(),
      { timeout: 3000 },
    );
  });

  /// Un cabinet à ACP unique n'a rien à décider : on choisit pour lui.
  it("présélectionne l'ACP quand il n'y en a qu'une", async () => {
    acpsServies.mockResolvedValue([ACP]);

    render(BuildingForm, { props: { isOpen: true, mode: "create" } });

    await waitFor(
      () => {
        const select = document.querySelector(
          '[data-testid="building-acp-select"] select, select#building-acp',
        ) as HTMLSelectElement | null;
        expect(select?.value).toBe(ACP.id);
      },
      { timeout: 3000 },
    );
  });

  /// Le syndic ne peut pas créer d'ACP lui-même : le message doit nommer le
  /// recours plutôt que de le laisser devant une liste vide.
  it("indique quoi faire quand le cabinet n'a aucune ACP", async () => {
    acpsServies.mockResolvedValue([]);

    render(BuildingForm, { props: { isOpen: true, mode: "create" } });

    await waitFor(
      () =>
        expect(
          screen.getByText(/admin\.building\.noAcpAskAdmin/),
        ).toBeInTheDocument(),
      { timeout: 3000 },
    );
  });
});
