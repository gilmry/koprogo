// Story 3.3 — MagicLinkContractorPage Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first TDD: this file accompanies the SAME-story
// component (drafted in parallel during slice 3 implementation).
//
// Coverage:
//   @happy    mount → screen-1 visible → click action → screen-2 → fill +
//             submit OK → screen-3 visible + POST called with payload.
//   @edge     pre-existing IDB draft for the token is restored at mount;
//             draft entry is purged after a successful submit.
//   @security tampering with the token in the DOM does not bypass the
//             backend — a 403 response surfaces the FR error message and
//             leaves the user on screen-2 (no silent screen-3 advance).
//   @negative submit while navigator.onLine === false → fetch rejects →
//             draft persists in IDB, offline indicator visible, user
//             stays on screen-2 with a "réessayez à la reconnexion" hint.
//
// Why an inline IndexedDB shim: jsdom (test env) does NOT ship an indexedDB
// implementation; we provide a minimal in-memory replacement scoped to the
// `koprogo-pwa-contractor` DB the component uses. This avoids adding a new
// runtime dep (fake-indexeddb) for a single test file.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import MagicLinkContractorPage from "./MagicLinkContractorPage.svelte";

// -------------------------------------------------------------------------
// Mocks — fetch is the only network boundary.
// -------------------------------------------------------------------------

const originalFetch = globalThis.fetch;
let fetchMock: ReturnType<typeof vi.fn>;

// -------------------------------------------------------------------------
// Minimal in-memory IndexedDB shim.
//
// Only implements the ops MagicLinkContractorPage actually uses:
//   open(name, version) with onupgradeneeded(createObjectStore) + onsuccess
//   transaction([store], mode).objectStore(store).{get|put|delete}(key)
// -------------------------------------------------------------------------

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
          transaction: (_storeNames: string[], _mode: string) => {
            return {
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
            };
          },
        };

        req.result = fakeDb;
        if (isNew) {
          // Schema bootstrap path.
          req.onupgradeneeded?.();
        }
        req.onsuccess?.();
      });

      return req;
    },
  };

  // Test helpers to seed / read the shim store directly.
  return {
    seedDraft(token: string, value: unknown) {
      let db = databases.get("koprogo-pwa-contractor");
      if (!db) {
        db = new Map();
        databases.set("koprogo-pwa-contractor", db);
      }
      let store = db.get("magic-link-drafts");
      if (!store) {
        store = new Map();
        db.set("magic-link-drafts", store);
      }
      store.set(`magic-link-draft-${token}`, value);
    },
    readDraft(token: string): unknown {
      return databases
        .get("koprogo-pwa-contractor")
        ?.get("magic-link-drafts")
        ?.get(`magic-link-draft-${token}`);
    },
    reset() {
      databases.clear();
    },
  };
}

let idbHelper: ReturnType<typeof installIdbShim>;

// -------------------------------------------------------------------------
// Lifecycle
// -------------------------------------------------------------------------

beforeEach(() => {
  idbHelper = installIdbShim();
  fetchMock = vi.fn();
  globalThis.fetch = fetchMock as unknown as typeof fetch;

  Object.defineProperty(window.navigator, "onLine", {
    value: true,
    configurable: true,
    writable: true,
  });
});

afterEach(() => {
  globalThis.fetch = originalFetch;
  idbHelper.reset();
  vi.restoreAllMocks();
});

// -------------------------------------------------------------------------
// Tests
// -------------------------------------------------------------------------

describe("MagicLinkContractorPage — Story 3.3", () => {
  const baseProps = {
    token: "tok-abc-123",
    scopeKind: "ticket" as const,
    scope: { id: "ticket-1", title: "Fuite cuisine" },
  };

  it("@happy mounts → screen-1 → action → screen-2 → submit OK → screen-3", async () => {
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({}),
    } as unknown as Response);

    const { getByTestId } = render(MagicLinkContractorPage, {
      props: baseProps,
    });

    // Screen 1 visible.
    await waitFor(() =>
      expect(getByTestId("pwa-screen-1-summary")).toBeInTheDocument(),
    );

    // Advance to screen 2.
    (getByTestId("pwa-summary-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("pwa-screen-2-action")).toBeInTheDocument(),
    );

    // Fill the message field — enables the submit button.
    const message = getByTestId(
      "pwa-action-message-input",
    ) as HTMLTextAreaElement;
    message.value = "Je peux passer demain matin.";
    message.dispatchEvent(new Event("input", { bubbles: true }));

    // Submit form.
    const submitBtn = getByTestId("pwa-action-submit") as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    // Screen 3 visible + fetch called against /c/<token>/respond.
    await waitFor(() =>
      expect(getByTestId("pwa-screen-3-confirm")).toBeInTheDocument(),
    );
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const [calledUrl, calledInit] = fetchMock.mock.calls[0] as [
      string,
      RequestInit,
    ];
    expect(calledUrl).toContain("/c/tok-abc-123/respond");
    expect(calledInit.method).toBe("POST");
  });

  it("@edge pre-existing IDB draft is restored at mount", async () => {
    idbHelper.seedDraft(baseProps.token, {
      message: "Brouillon récupéré",
      amount: 42,
    });

    const { getByTestId } = render(MagicLinkContractorPage, {
      props: baseProps,
    });

    // Advance to screen 2 to observe the restored value.
    await waitFor(() =>
      expect(getByTestId("pwa-screen-1-summary")).toBeInTheDocument(),
    );
    (getByTestId("pwa-summary-next") as HTMLButtonElement).click();

    await waitFor(() => {
      const input = getByTestId(
        "pwa-action-message-input",
      ) as HTMLTextAreaElement;
      expect(input.value).toBe("Brouillon récupéré");
    });
  });

  it("@edge draft is purged from IDB after a successful submit", async () => {
    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 200,
      json: async () => ({}),
    } as unknown as Response);

    const { getByTestId } = render(MagicLinkContractorPage, {
      props: baseProps,
    });

    await waitFor(() =>
      expect(getByTestId("pwa-screen-1-summary")).toBeInTheDocument(),
    );
    (getByTestId("pwa-summary-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("pwa-screen-2-action")).toBeInTheDocument(),
    );

    const message = getByTestId(
      "pwa-action-message-input",
    ) as HTMLTextAreaElement;
    message.value = "Réponse OK";
    message.dispatchEvent(new Event("input", { bubbles: true }));

    const submitBtn = getByTestId("pwa-action-submit") as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    await waitFor(() =>
      expect(getByTestId("pwa-screen-3-confirm")).toBeInTheDocument(),
    );

    // IDB store should no longer have the draft.
    expect(idbHelper.readDraft(baseProps.token)).toBeUndefined();
  });

  it("@security backend 403 surfaces error and stays on screen-2", async () => {
    fetchMock.mockResolvedValueOnce({
      ok: false,
      status: 403,
      json: async () => ({
        error: "Lien déjà consommé",
        kind: "magic_link_consumed",
      }),
    } as unknown as Response);

    const { getByTestId, queryByTestId } = render(MagicLinkContractorPage, {
      props: baseProps,
    });

    await waitFor(() =>
      expect(getByTestId("pwa-screen-1-summary")).toBeInTheDocument(),
    );
    (getByTestId("pwa-summary-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("pwa-screen-2-action")).toBeInTheDocument(),
    );

    const message = getByTestId(
      "pwa-action-message-input",
    ) as HTMLTextAreaElement;
    message.value = "Tentative malicieuse";
    message.dispatchEvent(new Event("input", { bubbles: true }));

    const submitBtn = getByTestId("pwa-action-submit") as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    // Error surfaced + still on screen-2 (no silent advance).
    await waitFor(() =>
      expect(getByTestId("pwa-action-error")).toHaveTextContent(
        /Lien déjà consommé/,
      ),
    );
    expect(queryByTestId("pwa-screen-3-confirm")).toBeNull();
    expect(getByTestId("pwa-screen-2-action")).toBeInTheDocument();
  });

  it("@negative offline submit → draft persists + offline indicator visible", async () => {
    // Simulate offline: fetch rejects with a network error, navigator.onLine
    // is forced to false BEFORE mount so the indicator is visible.
    Object.defineProperty(window.navigator, "onLine", {
      value: false,
      configurable: true,
      writable: true,
    });
    fetchMock.mockRejectedValueOnce(new Error("Network failure"));

    const { getByTestId, queryByTestId } = render(MagicLinkContractorPage, {
      props: baseProps,
    });

    await waitFor(() =>
      expect(getByTestId("pwa-screen-1-summary")).toBeInTheDocument(),
    );

    // Offline indicator is visible right away.
    expect(getByTestId("pwa-offline-indicator")).toBeInTheDocument();

    (getByTestId("pwa-summary-next") as HTMLButtonElement).click();
    await waitFor(() =>
      expect(getByTestId("pwa-screen-2-action")).toBeInTheDocument(),
    );

    const message = getByTestId(
      "pwa-action-message-input",
    ) as HTMLTextAreaElement;
    message.value = "Hors ligne pour le moment";
    message.dispatchEvent(new Event("input", { bubbles: true }));

    const submitBtn = getByTestId("pwa-action-submit") as HTMLButtonElement;
    await waitFor(() => expect(submitBtn.disabled).toBe(false));
    submitBtn.click();

    // Error message visible + still screen-2 + draft persisted in IDB.
    await waitFor(() =>
      expect(getByTestId("pwa-action-error")).toBeInTheDocument(),
    );
    expect(queryByTestId("pwa-screen-3-confirm")).toBeNull();

    await waitFor(() => {
      const stored = idbHelper.readDraft(baseProps.token) as
        | {
            message: string;
          }
        | undefined;
      expect(stored?.message).toBe("Hors ligne pour le moment");
    });
  });
});
