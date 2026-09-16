// Story 5.7 — OnboardingWizard Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD : ce fichier accompagne le composant écrit
// dans la même story.
//
// Couverture (cf. issue #591 AC) :
//   @happy    profil complet → ACP créée → modules recommandés → activation
//             (PUT /acps/{id}/modules/{module}/enable) → démo → confirmation
//             en 4min23 (< 5 min) → analytics enregistrée → onFinish rappelé
//             avec les modules sélectionnés.
//   @edge     l'utilisateur saute la recommandation (étape 2) → modules par
//             défaut activés (community + identity), pas ceux recommandés.
//   @security currentUserRole !== "superadmin" → aucune étape rendue, écran
//             d'accès refusé uniquement (fail-closed, defense-in-depth ;
//             la garde d'URL vit dans `/admin/*` cf. commentaire composant).
//   @negative wizard interrompu après l'étape 1 (démontage brutal) → un
//             nouveau montage restaure l'état depuis IndexedDB et affiche la
//             bannière de reprise, sans repartir de zéro.
//
// Pourquoi un shim IndexedDB inline : jsdom (env de test) n'expose pas
// indexedDB ; le composant scope sa propre DB ("koprogo-onboarding") comme
// MagicLinkContractorPage.svelte (story 3.3) — même pattern de shim minimal,
// pas de dépendance runtime ajoutée pour un seul fichier de test.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor } from "../../../../test-helpers";
import OnboardingWizard from "../OnboardingWizard.svelte";

// ---------------------------------------------------------------------------
// Mocks — createAcp (API ACP, déjà réelle) + api.put (contrat Story 5.1,
// pas encore implémenté côté backend — cf. commentaire d'en-tête du composant).
// ---------------------------------------------------------------------------

const createAcpMock = vi.fn();
const updateAcpMock = vi.fn();
vi.mock("../../../api/acps", () => ({
  createAcp: (...args: unknown[]) => createAcpMock(...args),
  updateAcp: (...args: unknown[]) => updateAcpMock(...args),
}));

const apiPutMock = vi.fn();
vi.mock("../../../api", () => ({
  api: { put: (...args: unknown[]) => apiPutMock(...args) },
  ApiError: class ApiError extends Error {
    constructor(
      message: string,
      readonly status: number,
    ) {
      super(message);
    }
  },
}));

// Toutes les scénarios passent `currentUserRole` explicitement — ce mock
// n'est là que pour fournir un `authStore` valide à la souscription `$authStore`
// que le composant compile (fallback prod quand la prop est omise, cf.
// commentaire du composant), sans dépendre du vrai store (localStorage, sync…).
// `isLoading: false` évite que `authPending` sorte vrai si un test omettait un
// jour la prop `currentUserRole`.
vi.mock("../../../../stores/auth", () => ({
  authStore: {
    subscribe: (
      fn: (v: {
        user: { role: string | null } | null;
        isAuthenticated: boolean;
        isLoading: boolean;
      }) => void,
    ) => {
      fn({ user: null, isAuthenticated: false, isLoading: false });
      return () => {};
    },
  },
}));

vi.mock("../../../i18n", () => {
  const store = {
    subscribe: (fn: (v: (key: string, opts?: unknown) => string) => void) => {
      fn(() => "");
      return () => {};
    },
  };
  return { _: store };
});

// ---------------------------------------------------------------------------
// Shim IndexedDB minimal, scoped à "koprogo-onboarding" / "wizard-drafts".
// ---------------------------------------------------------------------------

type IdbRequestLike<T> = {
  result: T | undefined;
  error: unknown;
  onsuccess: (() => void) | null;
  onerror: (() => void) | null;
};

function installIdbShim() {
  const databases = new Map<string, Map<string, Map<unknown, unknown>>>();

  function fireAsync<T>(req: IdbRequestLike<T>, result: T) {
    queueMicrotask(() => {
      req.result = result;
      req.onsuccess?.();
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (globalThis as any).indexedDB = {
    open(name: string, _version: number) {
      const req: IdbRequestLike<unknown> & {
        onupgradeneeded: (() => void) | null;
      } = {
        result: undefined,
        error: null,
        onsuccess: null,
        onerror: null,
        onupgradeneeded: null,
      };

      queueMicrotask(() => {
        let db = databases.get(name);
        const isNew = !db;
        if (!db) {
          db = new Map();
          databases.set(name, db);
        }

        const fakeDb = {
          objectStoreNames: {
            contains: (storeName: string) => db!.has(storeName),
          },
          createObjectStore: (storeName: string) => {
            db!.set(storeName, new Map());
            return {};
          },
          transaction: (_storeNames: string[], _mode: string) => ({
            objectStore: (storeName: string) => {
              const store = db!.get(storeName) ?? new Map();
              db!.set(storeName, store);
              return {
                get: (key: unknown) => {
                  const r: IdbRequestLike<unknown> = {
                    result: undefined,
                    error: null,
                    onsuccess: null,
                    onerror: null,
                  };
                  fireAsync(r, store.get(key));
                  return r;
                },
                put: (value: unknown, key: unknown) => {
                  const r: IdbRequestLike<unknown> = {
                    result: undefined,
                    error: null,
                    onsuccess: null,
                    onerror: null,
                  };
                  store.set(key, value);
                  fireAsync(r, undefined);
                  return r;
                },
                delete: (key: unknown) => {
                  const r: IdbRequestLike<unknown> = {
                    result: undefined,
                    error: null,
                    onsuccess: null,
                    onerror: null,
                  };
                  store.delete(key);
                  fireAsync(r, undefined);
                  return r;
                },
              };
            },
          }),
        };

        req.result = fakeDb;
        if (isNew) req.onupgradeneeded?.();
        req.onsuccess?.();
      });

      return req;
    },
  };

  return {
    reset() {
      databases.clear();
    },
  };
}

let idbHelper: ReturnType<typeof installIdbShim>;

beforeEach(() => {
  idbHelper = installIdbShim();
  createAcpMock.mockReset();
  updateAcpMock.mockReset();
  apiPutMock.mockReset();
});

afterEach(() => {
  idbHelper.reset();
  vi.restoreAllMocks();
  vi.useRealTimers();
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function setInput(el: HTMLElement, value: string) {
  (el as HTMLInputElement).value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
}

async function fillProfileStep(getByTestId: (id: string) => HTMLElement) {
  setInput(getByTestId("onboarding-name-input"), "ACP Tilleuls");
  setInput(getByTestId("onboarding-street-input"), "Rue des Tilleuls 12");
  setInput(getByTestId("onboarding-postal-code-input"), "1000");
  setInput(getByTestId("onboarding-city-input"), "Bruxelles");
  setInput(getByTestId("onboarding-units-count-input"), "3");
}

describe("OnboardingWizard — Story 5.7", () => {
  it("@happy naive user completes the wizard in 4min23 → analytics recorded → selected modules activated", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-16T09:00:00Z"));

    createAcpMock.mockResolvedValueOnce({ id: "acp-1", name: "ACP Tilleuls" });
    apiPutMock.mockResolvedValue(undefined);

    const onFinish = vi.fn();
    const onAnalytics = vi.fn();

    const { getByTestId } = render(OnboardingWizard, {
      props: { currentUserRole: "superadmin", onFinish, onAnalytics },
    });

    await vi.waitFor(() =>
      expect(getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );

    await fillProfileStep(getByTestId);
    // has_shared_spaces coché → "community" recommandé en plus du socle.
    (
      getByTestId("onboarding-shared-spaces-checkbox") as HTMLInputElement
    ).click();

    (getByTestId("onboarding-next") as HTMLButtonElement).click();
    await vi.waitFor(() =>
      expect(getByTestId("onboarding-step-2")).toBeInTheDocument(),
    );
    expect(createAcpMock).toHaveBeenCalledTimes(1);

    // Modules recommandés : identity (verrouillé) + community (car espaces
    // communs). Accounting/governance ne le sont pas pour une petite ACP de
    // 3 lots (seuils 5 / 10) — l'utilisateur pourrait les ajouter à la main.
    expect(
      (getByTestId("onboarding-module-toggle-identity") as HTMLInputElement)
        .checked,
    ).toBe(true);
    expect(
      (getByTestId("onboarding-module-toggle-community") as HTMLInputElement)
        .checked,
    ).toBe(true);

    (getByTestId("onboarding-next") as HTMLButtonElement).click();
    await vi.waitFor(() =>
      expect(getByTestId("onboarding-step-3")).toBeInTheDocument(),
    );

    (getByTestId("onboarding-activate-submit") as HTMLButtonElement).click();
    await vi.waitFor(() =>
      expect(getByTestId("onboarding-step-4")).toBeInTheDocument(),
    );
    expect(apiPutMock).toHaveBeenCalledWith(
      "/acps/acp-1/modules/community/enable",
      {},
    );
    // "identity" est déjà actif par défaut — jamais réactivé explicitement.
    expect(apiPutMock).not.toHaveBeenCalledWith(
      expect.stringContaining("/modules/identity/"),
      expect.anything(),
    );

    // Le testeur naïf avance encore 4min23 avant de conclure l'assistant.
    vi.advanceTimersByTime(4 * 60 * 1000 + 23 * 1000);

    (getByTestId("onboarding-demo-next") as HTMLButtonElement).click();
    await vi.waitFor(() =>
      expect(getByTestId("onboarding-step-5")).toBeInTheDocument(),
    );
    expect(getByTestId("onboarding-kpi-status")).toHaveTextContent(
      /Objectif atteint/,
    );

    (getByTestId("onboarding-finish-submit") as HTMLButtonElement).click();

    expect(onAnalytics).toHaveBeenCalledTimes(1);
    const analyticsEvent = onAnalytics.mock.calls[0][0];
    expect(analyticsEvent.acpId).toBe("acp-1");
    // Une BORNE, pas une milliseconde exacte.
    //
    // L'assertion était `toBe(263_000)` et rendait 263_350 : les `waitFor`
    // intercalés avancent l'horloge simulée de quelques centaines de
    // millisecondes. Elle mesurait donc la plomberie du test, pas le produit.
    //
    // Ce que la story affirme est « un utilisateur naïf termine en 4min23,
    // SOUS les cinq minutes » (#5.7). C'est cela qu'on vérifie : au moins le
    // temps qu'on a fait passer, et strictement moins que le seuil qui
    // donnerait tort à la story. Aucune assertion n'est retirée — celle-ci
    // dit désormais ce qu'elle prétendait dire.
    expect(analyticsEvent.elapsedMs).toBeGreaterThanOrEqual(
      4 * 60 * 1000 + 23 * 1000,
    );
    expect(analyticsEvent.elapsedMs).toBeLessThan(5 * 60 * 1000);
    expect(analyticsEvent.modules).toEqual(
      expect.arrayContaining(["identity", "community"]),
    );

    // `finish()` attend `purgeDraft()` AVANT d'appeler `onFinish` — c'est
    // délibéré et commenté dans le composant : si l'appelant navigue, le
    // brouillon doit déjà être purgé. `onFinish` arrive donc un tick plus
    // tard, et l'asserter sans attendre testait l'ordonnancement, pas le
    // contrat. Cette assertion était masquée par l'assertion de durée qui
    // échouait juste avant elle.
    await vi.waitFor(() => expect(onFinish).toHaveBeenCalledTimes(1));
    expect(onFinish.mock.calls[0][0].acpId).toBe("acp-1");
  });

  it("@edge user skips recommendation → default modules (community + identity) are activated, not the recommended set", async () => {
    createAcpMock.mockResolvedValueOnce({ id: "acp-2", name: "ACP Erables" });
    apiPutMock.mockResolvedValue(undefined);

    const onFinish = vi.fn();

    const { getByTestId } = render(OnboardingWizard, {
      props: { currentUserRole: "superadmin", onFinish },
    });

    await waitFor(() =>
      expect(getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );

    // Grande ACP (12 lots) → le moteur recommanderait accounting +
    // governance en plus. On saute quand même la recommandation.
    setInput(getByTestId("onboarding-name-input"), "ACP Erables");
    setInput(getByTestId("onboarding-street-input"), "Avenue des Erables 3");
    setInput(getByTestId("onboarding-postal-code-input"), "1050");
    setInput(getByTestId("onboarding-city-input"), "Ixelles");
    setInput(getByTestId("onboarding-units-count-input"), "12");

    (getByTestId("onboarding-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-2")).toBeInTheDocument(),
    );

    (
      getByTestId("onboarding-skip-recommendation") as HTMLButtonElement
    ).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-3")).toBeInTheDocument(),
    );

    (getByTestId("onboarding-activate-submit") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-4")).toBeInTheDocument(),
    );

    // Seul "community" appelle l'API (identity toujours actif) — ni
    // accounting ni governance, qu'un moteur de recommandation aurait
    // pourtant suggérés pour 12 lots.
    expect(apiPutMock).toHaveBeenCalledTimes(1);
    expect(apiPutMock).toHaveBeenCalledWith(
      "/acps/acp-2/modules/community/enable",
      {},
    );

    (getByTestId("onboarding-demo-next") as HTMLButtonElement).click();
    // Svelte 5 rend au prochain microtask : sans cette attente, le clic sur
    // "Terminer" tombe avant que l'étape 5 n'existe dans le DOM.
    await waitFor(() =>
      expect(getByTestId("onboarding-step-5")).toBeInTheDocument(),
    );
    (getByTestId("onboarding-finish-submit") as HTMLButtonElement).click();

    await waitFor(() => expect(onFinish).toHaveBeenCalledTimes(1));
    const result = onFinish.mock.calls[0][0];
    expect(result.recommendationSkipped).toBe(true);
    expect(result.modules.sort()).toEqual(["community", "identity"]);
  });

  it("@edge returning to step 1 and resubmitting updates the existing ACP instead of creating a duplicate", async () => {
    createAcpMock.mockResolvedValueOnce({ id: "acp-4", name: "ACP Chênes" });
    updateAcpMock.mockResolvedValueOnce({ id: "acp-4", name: "ACP Chênes v2" });

    const { getByTestId } = render(OnboardingWizard, {
      props: { currentUserRole: "superadmin" },
    });

    await waitFor(() =>
      expect(getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );
    await fillProfileStep(getByTestId);
    (getByTestId("onboarding-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-2")).toBeInTheDocument(),
    );
    expect(createAcpMock).toHaveBeenCalledTimes(1);

    // Retour à l'étape 1, modification, puis nouvelle soumission — l'ACP a
    // déjà un id : c'est le chemin qui, sans le contrôle `acpId ? update :
    // create`, recréerait une seconde ACP en doublon.
    (getByTestId("onboarding-back") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );
    setInput(getByTestId("onboarding-name-input"), "ACP Chênes (corrigé)");

    (getByTestId("onboarding-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("onboarding-step-2")).toBeInTheDocument(),
    );

    expect(createAcpMock).toHaveBeenCalledTimes(1); // toujours une seule création.
    expect(updateAcpMock).toHaveBeenCalledTimes(1);
    expect(updateAcpMock).toHaveBeenCalledWith(
      "acp-4",
      expect.objectContaining({ name: "ACP Chênes (corrigé)" }),
    );
  });

  it("@security a non-superadmin role never sees the wizard steps, only an access-denied screen", async () => {
    const { getByTestId, queryByTestId } = render(OnboardingWizard, {
      props: { currentUserRole: "syndic" },
    });

    await waitFor(() =>
      expect(getByTestId("onboarding-access-denied")).toBeInTheDocument(),
    );

    expect(queryByTestId("onboarding-step-1")).toBeNull();
    expect(queryByTestId("onboarding-step-2")).toBeNull();
    expect(queryByTestId("onboarding-name-input")).toBeNull();
    expect(createAcpMock).not.toHaveBeenCalled();
  });

  it("@negative wizard interrupted mid-flow (unmounted after step 1) → resumed from IndexedDB on next mount", async () => {
    createAcpMock.mockResolvedValueOnce({ id: "acp-3", name: "ACP Peupliers" });

    const first = render(OnboardingWizard, {
      props: { currentUserRole: "superadmin" },
    });

    await waitFor(() =>
      expect(first.getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );

    setInput(first.getByTestId("onboarding-name-input"), "ACP Peupliers");
    setInput(
      first.getByTestId("onboarding-street-input"),
      "Rue des Peupliers 5",
    );
    setInput(first.getByTestId("onboarding-postal-code-input"), "4000");
    setInput(first.getByTestId("onboarding-city-input"), "Liège");
    setInput(first.getByTestId("onboarding-units-count-input"), "3");

    (first.getByTestId("onboarding-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(first.getByTestId("onboarding-step-2")).toBeInTheDocument(),
    );

    // Interruption brutale — l'onglet se ferme avant l'étape suivante.
    first.unmount();

    const second = render(OnboardingWizard, {
      props: { currentUserRole: "superadmin" },
    });

    await waitFor(() =>
      expect(
        second.getByTestId("onboarding-resumed-banner"),
      ).toBeInTheDocument(),
    );
    // La reprise ramène directement à l'étape où l'utilisateur s'était arrêté
    // (étape 2 — pas de retour forcé à l'étape 1).
    expect(second.getByTestId("onboarding-step-2")).toBeInTheDocument();
    expect(createAcpMock).toHaveBeenCalledTimes(1); // pas de recréation d'ACP.

    // La reprise offre aussi une sortie explicite plutôt qu'un piège : on
    // peut abandonner le brouillon et repartir de zéro.
    (
      second.getByTestId("onboarding-discard-draft") as HTMLButtonElement
    ).click();
    await waitFor(() =>
      expect(second.getByTestId("onboarding-step-1")).toBeInTheDocument(),
    );
    expect(second.queryByTestId("onboarding-resumed-banner")).toBeNull();
  });
});
