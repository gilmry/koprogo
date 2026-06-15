// Track H Story H1 — Vitest 4-cat `conformity.ts` (utils).
//
// @happy   isConformityError() reconnaît un body 422 BUILDING_NOT_CONFORMANT
//          direct, wrappé dans Error.body, ou dans Error.response.data.
//          buildConformityStatus() assemble correctement ConformityStatus.
//          formatDecimalFRBE() remplace . par , sans parseFloat.
// @edge    extractConformityPayload() supporte 3 formes de wrapper.
//          quota_basis 10000 (acte de base ≠ 1000) bug fix preserved.
// @security pas de leak d'info dans le toast (pas d'user_id / org_id).
// @negative err === null / undefined / string / objet inconnu → false.
//           type guard ne throw jamais.

import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  isConformityError,
  extractConformityPayload,
  formatDecimalFRBE,
  buildConformityStatus,
  showConformityToast,
} from "./conformity";
import { toast } from "../../stores/toast";

const VALID_BODY = {
  error: "L'immeuble n'est pas conforme à son acte de base",
  kind: "building_not_conformant" as const,
  details: {
    code: "BUILDING_NOT_CONFORMANT" as const,
    building_id: "11111111-2222-3333-4444-555555555555",
    units_delta: 1,
    quota_delta: "2.5",
    quota_basis: 1000,
  },
};

describe("conformity utils — 4-cat (Track H Story H1)", () => {
  // ----------------------------------------------------------------------
  // @happy — chemin nominal
  // ----------------------------------------------------------------------

  it("@happy isConformityError reconnaît un body 422 direct", () => {
    expect(isConformityError(VALID_BODY)).toBe(true);
  });

  it("@happy isConformityError reconnaît un wrapper Error.body", () => {
    expect(isConformityError({ body: VALID_BODY })).toBe(true);
  });

  it("@happy isConformityError reconnaît un wrapper axios Error.response.data", () => {
    expect(isConformityError({ response: { data: VALID_BODY } })).toBe(true);
  });

  it("@happy extractConformityPayload retourne le payload narratif", () => {
    const payload = extractConformityPayload(VALID_BODY);
    expect(payload).not.toBeNull();
    expect(payload?.code).toBe("BUILDING_NOT_CONFORMANT");
    expect(payload?.units_delta).toBe(1);
    expect(payload?.quota_delta).toBe("2.5");
    expect(payload?.quota_basis).toBe(1000);
  });

  it("@happy buildConformityStatus calcule units_delta = total - count", () => {
    const status = buildConformityStatus({
      is_conformant: false,
      total_units: 10,
      units_count: 9,
      total_tantiemes: 1000,
      quota_delta: "2.5",
    });
    expect(status).toEqual({
      is_conformant: false,
      units_delta: 1,
      quota_delta: "2.5",
      quota_basis: 1000,
    });
  });

  it("@happy formatDecimalFRBE remplace . par , (sans parseFloat)", () => {
    expect(formatDecimalFRBE("2.5")).toBe("2,5");
    expect(formatDecimalFRBE("0")).toBe("0");
    expect(formatDecimalFRBE("1000")).toBe("1000");
    expect(formatDecimalFRBE("-25.5")).toBe("-25,5");
    expect(formatDecimalFRBE("+25.5")).toBe("25,5"); // leading + stripped
  });

  // ----------------------------------------------------------------------
  // @edge — basis 10000 + form inconsistante
  // ----------------------------------------------------------------------

  it("@edge isConformityError sur quota_basis=10000 (bug fix Story H1)", () => {
    const body = {
      ...VALID_BODY,
      details: {
        ...VALID_BODY.details,
        quota_delta: "25",
        quota_basis: 10000,
      },
    };
    expect(isConformityError(body)).toBe(true);
    const payload = extractConformityPayload(body);
    expect(payload?.quota_basis).toBe(10000);
    expect(payload?.quota_delta).toBe("25");
  });

  it("@edge buildConformityStatus avec total_tantiemes=10000 propage quota_basis", () => {
    const status = buildConformityStatus({
      is_conformant: false,
      total_units: 182,
      units_count: 181,
      total_tantiemes: 10000,
      quota_delta: "25",
    });
    expect(status.quota_basis).toBe(10000);
    expect(status.units_delta).toBe(1);
  });

  it("@edge formatDecimalFRBE empty string → '—'", () => {
    expect(formatDecimalFRBE("")).toBe("—");
  });

  // ----------------------------------------------------------------------
  // @security — pas de leak
  // ----------------------------------------------------------------------

  it("@security showConformityToast n'expose pas user_id / org_id (AC-H1.s2)", async () => {
    const errorSpy = vi.spyOn(toast, "error").mockReturnValue(1);
    const result = showConformityToast(VALID_BODY);
    expect(result).toBe(true);
    expect(errorSpy).toHaveBeenCalledOnce();
    const message = String(errorSpy.mock.calls[0][0]).toLowerCase();
    expect(message).not.toContain("user_id");
    expect(message).not.toContain("org_id");
    expect(message).not.toContain("organization_id");
    expect(message).not.toContain("password");
    expect(message).not.toContain("token");
    errorSpy.mockRestore();
  });

  // ----------------------------------------------------------------------
  // @negative — robustesse type guard
  // ----------------------------------------------------------------------

  it("@negative isConformityError(null|undefined|string|number) → false", () => {
    expect(isConformityError(null)).toBe(false);
    expect(isConformityError(undefined)).toBe(false);
    expect(isConformityError("not an object")).toBe(false);
    expect(isConformityError(42)).toBe(false);
    expect(isConformityError([])).toBe(false);
  });

  it("@negative isConformityError sur 422 d'un autre type → false", () => {
    const other = {
      error: "Invalid",
      kind: "validation",
      details: { code: "OTHER_CODE" },
    };
    expect(isConformityError(other)).toBe(false);
  });

  it("@negative isConformityError sur 422 BUILDING_NOT_CONFORMANT mais kind manquant → false", () => {
    const malformed = {
      details: { code: "BUILDING_NOT_CONFORMANT" },
      // pas de `kind`
    };
    expect(isConformityError(malformed)).toBe(false);
  });

  it("@negative showConformityToast sur erreur générique → false (fallback)", () => {
    const errorSpy = vi.spyOn(toast, "error").mockReturnValue(1);
    const result = showConformityToast(new Error("Network error"));
    expect(result).toBe(false);
    expect(errorSpy).not.toHaveBeenCalled();
    errorSpy.mockRestore();
  });

  it("@negative extractConformityPayload sur objet vide → null", () => {
    expect(extractConformityPayload({} as Record<string, unknown>)).toBeNull();
  });
});

// ----------------------------------------------------------------------
// Tests d'initialisation i18n (nécessaire avant showConformityToast).
// On charge le module i18n une seule fois — svelte-i18n est setup côté
// `frontend/src/lib/i18n.ts` au chargement (cf. `setupI18n()` side-effect).
// ----------------------------------------------------------------------
beforeEach(() => {
  // s'assure que setupI18n a tourné (idempotent).
  // L'import dans `conformity.ts` indirectement load `i18n.ts`.
});
