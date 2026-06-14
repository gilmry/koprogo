// Story B7 (Phase B FE) — Vitest 4-cat SignatureForm atomique.
//
// Couverture (cf. stories.md §B7 + Gotcha #2 RPGD checkbox) :
//   @happy    : checkbox cochée → bouton enabled → click → onSign appelé
//               + checkbox reset après succès.
//   @edge     : externallyDisabled=true → bouton disabled même si checkbox
//               cochée (mandate inactif côté parent).
//   @security : checkbox PAS cochée → bouton disabled → click → onSign PAS
//               appelé (gating RPGD obligatoire). Pattern juridique : pas
//               de signature sans consentement éclairé.
//   @negative : onSign rejette → message d'erreur inline visible + checkbox
//               reste cochée pour permettre retry sans re-cliquer.
//
// Pattern DI : on injecte `onSign` via prop (pas de vi.mock module) — cohérent
// avec SyndicResponseForm.test.ts B6 pattern.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import SignatureForm from "./SignatureForm.svelte";

function clickCheckbox(el: HTMLInputElement) {
  el.click();
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("SignatureForm — Story B7 (4-cat atomique)", () => {
  it("@happy checkbox cochée → bouton enabled → click → onSign appelé + reset", async () => {
    const onSign = vi.fn().mockResolvedValue(undefined);
    const { getByTestId } = render(SignatureForm, { props: { onSign } });

    const cb = getByTestId("signature-confirm-checkbox") as HTMLInputElement;
    const btn = getByTestId("signature-sign-button") as HTMLButtonElement;

    // État initial : bouton disabled (checkbox vide).
    expect(btn.disabled).toBe(true);
    expect(cb.checked).toBe(false);

    // Coche la checkbox → bouton enabled.
    clickCheckbox(cb);
    await waitFor(() => expect(btn.disabled).toBe(false));

    // Click signature → onSign appelé.
    btn.click();
    await waitFor(() => expect(onSign).toHaveBeenCalledTimes(1));

    // Reset checkbox après succès.
    await waitFor(() => {
      const cb2 = getByTestId(
        "signature-confirm-checkbox",
      ) as HTMLInputElement;
      expect(cb2.checked).toBe(false);
    });
  });

  it("@edge externallyDisabled=true → bouton disabled même si checkbox cochée", async () => {
    const onSign = vi.fn().mockResolvedValue(undefined);
    const { getByTestId } = render(SignatureForm, {
      props: { onSign, externallyDisabled: true },
    });

    const cb = getByTestId("signature-confirm-checkbox") as HTMLInputElement;
    const btn = getByTestId("signature-sign-button") as HTMLButtonElement;

    // Cocher la checkbox ne suffit pas — externallyDisabled gate.
    clickCheckbox(cb);
    await waitFor(() => expect(cb.checked).toBe(true));
    expect(btn.disabled).toBe(true);
    expect(btn.getAttribute("aria-disabled")).toBe("true");

    btn.click();
    expect(onSign).not.toHaveBeenCalled();
  });

  it("@security checkbox NON cochée → bouton disabled → click ignoré (gating RPGD)", () => {
    const onSign = vi.fn().mockResolvedValue(undefined);
    const { getByTestId } = render(SignatureForm, { props: { onSign } });

    const btn = getByTestId("signature-sign-button") as HTMLButtonElement;
    expect(btn.disabled).toBe(true);
    expect(btn.getAttribute("aria-disabled")).toBe("true");

    btn.click();
    expect(onSign).not.toHaveBeenCalled();
  });

  it("@negative onSign rejette → message inline visible + checkbox reste cochée", async () => {
    const onSign = vi
      .fn()
      .mockRejectedValue(new Error("Signature déjà présente pour (user, role)"));
    const { getByTestId, queryByTestId } = render(SignatureForm, {
      props: { onSign },
    });

    const cb = getByTestId("signature-confirm-checkbox") as HTMLInputElement;
    const btn = getByTestId("signature-sign-button") as HTMLButtonElement;

    clickCheckbox(cb);
    await waitFor(() => expect(btn.disabled).toBe(false));
    btn.click();

    // Message erreur visible.
    await waitFor(() => {
      const err = queryByTestId("signature-form-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/déjà présente/i);
    });

    // Checkbox PAS reset en cas d'erreur — permet retry direct.
    const cbAfter = getByTestId(
      "signature-confirm-checkbox",
    ) as HTMLInputElement;
    expect(cbAfter.checked).toBe(true);
  });

  it("idSuffix → testids composables (réutilisable plusieurs fois)", () => {
    const onSign = vi.fn();
    const { getByTestId } = render(SignatureForm, {
      props: { onSign, idSuffix: "spec-uuid-42" },
    });

    // Les testids sont suffixés avec idSuffix.
    expect(
      getByTestId("signature-confirm-checkbox-spec-uuid-42"),
    ).toBeTruthy();
    expect(getByTestId("signature-sign-button-spec-uuid-42")).toBeTruthy();
  });
});
