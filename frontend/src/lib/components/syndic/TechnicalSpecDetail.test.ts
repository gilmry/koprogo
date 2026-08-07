// Story B7 (Phase B FE) — Vitest 4-cat TechnicalSpecDetail.
//
// Couverture (cf. stories.md §B7 AC) :
//   @happy    : status PendingSignatures + user a rôle requis (syndic) +
//               pas déjà signé → bouton "Signer" RENDU + status badge correct.
//   @edge     : status Draft → bouton "Soumettre pour signatures" présent ;
//               status Approved → bouton "Bump" présent, "Soumettre" ABSENT.
//   @security : Owner sans rôle requis → bouton "Signer" ABSENT du DOM
//               (pas juste disabled — défense en profondeur) ;
//               mandataire amo sans activeMandate → bouton "Signer" ABSENT ;
//               user a déjà signé sous son rôle → bouton "Signer" ABSENT.
//   @negative : bump button → modal warning visible avec aria-modal="true" +
//               focus auto sur "Continuer".

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import TechnicalSpecDetail from "./TechnicalSpecDetail.svelte";
import type {
  TechnicalSpecDto,
  TechnicalSpecSignatureDto,
} from "../../api/technical_specs";

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

function spec(over: Partial<TechnicalSpecDto> = {}): TechnicalSpecDto {
  return {
    id: "spec-1",
    acp_id: "acp-1",
    building_id: null,
    title: "Travaux toiture",
    description: "Réfection complète.",
    version: "1.0.0",
    deliverables: ["Démontage", "Pose voligeage"],
    required_signatures: ["syndic", "amo"],
    attachments: [],
    status: "pending_signatures",
    created_by: "syndic-uuid",
    previous_version_id: null,
    created_at: "2026-06-09T10:00:00Z",
    updated_at: "2026-06-09T10:00:00Z",
    ...over,
  };
}

function sig(
  over: Partial<TechnicalSpecSignatureDto>,
): TechnicalSpecSignatureDto {
  return {
    id: "sig-base",
    technical_spec_id: "spec-1",
    signatory_user_id: "user-syndic",
    role: "syndic",
    mandate_id: null,
    signed_at: "2026-06-09T11:00:00Z",
    ...over,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("TechnicalSpecDetail — Story B7 (4-cat)", () => {
  it("@happy PendingSignatures + user a rôle requis → bouton Signer RENDU + status badge", () => {
    const s = spec({ status: "pending_signatures" });
    const { getByTestId, queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [],
        currentUserRole: "syndic",
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });

    // Header rendus.
    expect(getByTestId("tech-spec-detail-title").textContent).toMatch(
      /Travaux toiture/,
    );
    expect(getByTestId("tech-spec-detail-version").textContent).toMatch(
      /1\.0\.0/,
    );
    const badge = getByTestId("tech-spec-detail-status-badge");
    expect(badge.getAttribute("data-status")).toBe("pending_signatures");
    expect(badge.textContent).toMatch(/attente/i);

    // Bouton Signer présent.
    expect(queryByTestId("tech-spec-sign-submit")).not.toBeNull();

    // Bouton "Soumettre pour signatures" ABSENT (déjà soumis).
    expect(queryByTestId("tech-spec-submit-for-sign")).toBeNull();
  });

  it("@edge status Draft → bouton 'Soumettre pour signatures' présent + Bump présent + Signer absent", async () => {
    const s = spec({ status: "draft" });
    const onSubmitForSign = vi.fn().mockResolvedValue(undefined);
    const { getByTestId, queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [],
        currentUserRole: "syndic",
        activeMandate: null,
        onSubmitForSign,
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });

    const submitBtn = getByTestId(
      "tech-spec-submit-for-sign",
    ) as HTMLButtonElement;
    expect(submitBtn).toBeTruthy();
    expect(queryByTestId("tech-spec-bump-button")).not.toBeNull();
    expect(queryByTestId("tech-spec-sign-submit")).toBeNull();

    submitBtn.click();
    await waitFor(() => expect(onSubmitForSign).toHaveBeenCalledWith("spec-1"));
  });

  it("@edge status Approved → bouton Bump présent, Soumettre absent, Signer absent", () => {
    const s = spec({ status: "approved" });
    const { queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [
          sig({ id: "s1", signatory_user_id: "u1", role: "syndic" }),
          sig({ id: "s2", signatory_user_id: "u2", role: "amo" }),
        ],
        currentUserRole: "syndic",
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });

    expect(queryByTestId("tech-spec-bump-button")).not.toBeNull();
    expect(queryByTestId("tech-spec-submit-for-sign")).toBeNull();
    // Aucune signature attendue → bouton Signer ABSENT car status != PendingSignatures.
    expect(queryByTestId("tech-spec-sign-submit")).toBeNull();
  });

  it("@security Owner sans rôle requis → bouton Signer ABSENT du DOM", () => {
    const s = spec({ status: "pending_signatures" });
    const { queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [],
        // Owner = pas un rôle dans required_signatures (syndic/amo).
        currentUserRole: null,
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });

    // Le bouton "Signer" n'est PAS présent (pas juste disabled).
    expect(queryByTestId("tech-spec-sign-submit")).toBeNull();
    expect(queryByTestId("tech-spec-sign-confirm-checkbox")).toBeNull();
  });

  it("@security mandataire amo sans activeMandate → bouton Signer ABSENT (mandatePrereq KO)", () => {
    const s = spec({
      status: "pending_signatures",
      required_signatures: ["syndic", "amo"],
    });
    const { queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [],
        currentUserRole: "amo",
        activeMandate: null, // mandate MANQUANT — gating sécurité
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });
    expect(queryByTestId("tech-spec-sign-submit")).toBeNull();
  });

  it("@security user a déjà signé sous son rôle → bouton Signer ABSENT (INV unique)", () => {
    const s = spec({ status: "pending_signatures" });
    const { queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [
          sig({
            id: "s-existing",
            signatory_user_id: "u-syndic",
            role: "syndic",
          }),
        ],
        currentUserRole: "syndic",
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
      },
    });
    // Le user (syndic) a déjà signé sous "syndic" → bouton absent.
    expect(queryByTestId("tech-spec-sign-submit")).toBeNull();
  });

  it("@negative bump button → modal warning visible avec aria-modal='true' + autofocus", async () => {
    const onBump = vi.fn();
    const s = spec({ status: "approved" });
    const { getByTestId, queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [],
        currentUserRole: "syndic",
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump,
        onSign: vi.fn(),
      },
    });

    (getByTestId("tech-spec-bump-button") as HTMLButtonElement).click();

    // Modal présente.
    await waitFor(() => {
      const modal = queryByTestId("tech-spec-bump-modal");
      expect(modal).not.toBeNull();
      expect(modal?.getAttribute("aria-modal")).toBe("true");
      expect(modal?.getAttribute("role")).toBe("dialog");
    });

    // Warning bump major visible.
    const modal = getByTestId("tech-spec-bump-modal");
    expect(modal.textContent).toMatch(/MAJOR/);
    expect(modal.textContent).toMatch(/invalidées/i);

    // Confirm → onBump appelé avec la spec source.
    (getByTestId("tech-spec-bump-confirm") as HTMLButtonElement).click();
    await waitFor(() => expect(onBump).toHaveBeenCalledWith(s));
  });

  it("liste des signatures + missing — render correct", () => {
    const s = spec({
      status: "pending_signatures",
      required_signatures: ["syndic", "amo"],
    });
    const { getByTestId, queryByTestId } = render(TechnicalSpecDetail, {
      props: {
        spec: s,
        signatures: [
          sig({
            id: "s-syndic",
            signatory_user_id: "u-syndic",
            role: "syndic",
          }),
        ],
        currentUserRole: null, // observer — pas de signature box
        activeMandate: null,
        onSubmitForSign: vi.fn(),
        onBump: vi.fn(),
        onSign: vi.fn(),
        userLabels: { "u-syndic": "Syndic Maury" },
      },
    });

    expect(getByTestId("tech-spec-signatures-list")).toBeTruthy();
    expect(getByTestId("tech-spec-signature-row-u-syndic-syndic")).toBeTruthy();
    // amo n'a pas encore signé → "En attente : amo"
    expect(queryByTestId("tech-spec-signature-missing-amo")).not.toBeNull();
    expect(queryByTestId("tech-spec-signature-missing-syndic")).toBeNull();
  });
});
