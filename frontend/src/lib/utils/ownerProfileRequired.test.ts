// Issue #781 — Vitest 4-cat `ownerProfileRequired.ts`.
//
// @happy    Body 403 kind=owner_profile_required → détecté + toast affiché.
// @edge     Wrapper `Error.body` (forme réelle d'une `ApiError`) détecté.
// @security Erreur arbitraire (string / null / kind différent) → pas de
//           toast, pas de fuite, return false.
// @negative Objet sans `kind` du tout → null safe, pas de crash.

import { describe, it, expect, vi, beforeEach } from "vitest";

const { toastErrorMock } = vi.hoisted(() => ({
  toastErrorMock: vi.fn(),
}));

vi.mock("../i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

vi.mock("../../stores/toast", () => ({
  toast: {
    error: toastErrorMock,
    warning: vi.fn(),
    success: vi.fn(),
    show: vi.fn(),
  },
}));

import {
  isOwnerProfileRequiredError,
  showOwnerProfileRequiredToast,
} from "./ownerProfileRequired";

const SAMPLE_BODY = {
  error:
    "Cette action est réservée aux copropriétaires : elle engage une personne, pas la copropriété.",
  kind: "owner_profile_required",
};

beforeEach(() => {
  toastErrorMock.mockClear();
});

describe("ownerProfileRequired — 4-cat (#781)", () => {
  // ----------------------------------------------------------------------
  // @happy
  // ----------------------------------------------------------------------

  it("@happy body direct 403 → isOwnerProfileRequiredError true", () => {
    expect(isOwnerProfileRequiredError(SAMPLE_BODY)).toBe(true);
  });

  it("@happy showOwnerProfileRequiredToast appelle toast.error et retourne true", () => {
    const handled = showOwnerProfileRequiredToast(SAMPLE_BODY);
    expect(handled).toBe(true);
    expect(toastErrorMock).toHaveBeenCalledTimes(1);
    const [msg, duration] = toastErrorMock.mock.calls[0];
    expect(msg).toContain("skills.createModal.ownerProfileRequiredTitle");
    expect(msg).toContain("skills.createModal.ownerProfileRequiredMessage");
    expect(duration).toBe(8000);
  });

  // ----------------------------------------------------------------------
  // @edge
  // ----------------------------------------------------------------------

  it("@edge wrapper Error.body 403 → détecté (forme réelle d'ApiError)", () => {
    const wrapped = { message: "HTTP 403", body: SAMPLE_BODY };
    expect(isOwnerProfileRequiredError(wrapped)).toBe(true);
    expect(showOwnerProfileRequiredToast(wrapped)).toBe(true);
  });

  it("@edge wrapper Error.response.data (axios-style) → détecté", () => {
    const wrapped = { message: "HTTP 403", response: { data: SAMPLE_BODY } };
    expect(isOwnerProfileRequiredError(wrapped)).toBe(true);
  });

  // ----------------------------------------------------------------------
  // @security
  // ----------------------------------------------------------------------

  it("@security erreur arbitraire (Error string) → pas de toast, return false", () => {
    const err = new Error("Network refused");
    expect(isOwnerProfileRequiredError(err)).toBe(false);
    expect(showOwnerProfileRequiredToast(err)).toBe(false);
    expect(toastErrorMock).not.toHaveBeenCalled();
  });

  it("@security null / undefined → return false sans crash", () => {
    expect(isOwnerProfileRequiredError(null)).toBe(false);
    expect(isOwnerProfileRequiredError(undefined)).toBe(false);
    expect(showOwnerProfileRequiredToast(null)).toBe(false);
  });

  it("@security kind différent (autre 403) → ignoré, la règle métier n'est pas assouplie", () => {
    const autreRefus = {
      error: "Unauthorized: only owner can update skill",
      kind: "forbidden",
    };
    expect(isOwnerProfileRequiredError(autreRefus)).toBe(false);
    expect(showOwnerProfileRequiredToast(autreRefus)).toBe(false);
  });

  // ----------------------------------------------------------------------
  // @negative
  // ----------------------------------------------------------------------

  it("@negative objet sans kind du tout → null safe", () => {
    const sansKind = { error: "X" };
    expect(isOwnerProfileRequiredError(sansKind)).toBe(false);
    expect(showOwnerProfileRequiredToast(sansKind)).toBe(false);
  });
});

/**
 * Régression #782 — même trou que pour la conformité et la clôture d'AG :
 * une `ApiError` réelle levée par `apiFetch` porte le `kind` dans `.body`,
 * pas à la racine de l'objet. Un test qui ne fabrique que des objets plain
 * passerait pendant que l'intégration réelle est cassée.
 */
describe("le refus owner_profile_required avec une ApiError réelle (#782)", () => {
  it("reconnaît le 403 tel que apiFetch le lève", async () => {
    const { ApiError } = await import("../api");
    const corps = {
      error:
        "Cette action est réservée aux copropriétaires : elle engage une personne, pas la copropriété.",
      kind: "owner_profile_required",
    };
    const erreur = new ApiError(corps.error, 403, undefined, corps);

    expect(isOwnerProfileRequiredError(erreur)).toBe(true);
    expect(showOwnerProfileRequiredToast(erreur)).toBe(true);
  });
});
