// Story B7 (Phase B FE) — Vitest 4-cat TechnicalSpecSignatureForm.
//
// Couverture (cf. stories.md §B7 AC) :
//   @happy    : rôle direct (syndic) → bandeau "rôle direct" + signature OK.
//   @edge     : rôle mandataire (amo) avec activeMandate → bandeau "via
//               mandat #..." visible + signature OK.
//   @security : rôle mandataire (amo) SANS activeMandate → warning visible
//               + bouton signer disabled (externallyDisabled=true) +
//               onSign PAS appelé.
//   @negative : onSign rejette (409 signature déjà présente) → message
//               inline visible via le SignatureForm enfant.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import TechnicalSpecSignatureForm from "./TechnicalSpecSignatureForm.svelte";

const SPEC_ID = "spec-uuid-1";
const MANDATE_ID = "mandate-uuid-42";

function clickCheckbox(el: HTMLInputElement) {
  el.click();
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("TechnicalSpecSignatureForm — Story B7 (4-cat)", () => {
  it("@happy rôle direct (syndic) → bandeau 'rôle direct' + signature OK", async () => {
    const onSign = vi.fn().mockResolvedValue({
      id: "sig-1",
      technical_spec_id: SPEC_ID,
      signatory_user_id: "user-syndic",
      role: "syndic",
      mandate_id: null,
      signed_at: new Date().toISOString(),
    });

    const { getByTestId } = render(TechnicalSpecSignatureForm, {
      props: {
        specId: SPEC_ID,
        role: "syndic",
        activeMandate: null,
        onSign,
      },
    });

    const info = getByTestId("tech-spec-sign-mandate-info");
    expect(info.textContent).toMatch(/rôle direct/i);
    expect(info.textContent).toMatch(/syndic/);

    // Coche la checkbox RPGD.
    const cb = getByTestId(
      "tech-spec-sign-confirm-checkbox",
    ) as HTMLInputElement;
    clickCheckbox(cb);

    const btn = getByTestId("tech-spec-sign-submit") as HTMLButtonElement;
    await waitFor(() => expect(btn.disabled).toBe(false));
    btn.click();

    await waitFor(() => expect(onSign).toHaveBeenCalledTimes(1));
    expect(onSign).toHaveBeenCalledWith(SPEC_ID, {
      role: "syndic",
      mandate_id: null,
    });
  });

  it("@edge rôle mandataire (amo) avec activeMandate → bandeau visible + signature avec mandate_id", async () => {
    const onSign = vi.fn().mockResolvedValue({
      id: "sig-2",
      technical_spec_id: SPEC_ID,
      signatory_user_id: "user-amo",
      role: "amo",
      mandate_id: MANDATE_ID,
      signed_at: new Date().toISOString(),
    });

    const { getByTestId, queryByTestId } = render(TechnicalSpecSignatureForm, {
      props: {
        specId: SPEC_ID,
        role: "amo",
        activeMandate: { id: MANDATE_ID, validUntil: "2027-06-09T23:59:59Z" },
        onSign,
      },
    });

    const info = getByTestId("tech-spec-sign-mandate-info");
    expect(info.textContent).toMatch(/via mandat/i);
    expect(info.textContent).toMatch(/amo/);

    // Pas de warning.
    expect(queryByTestId("tech-spec-sign-no-mandate-warning")).toBeNull();

    // Signature OK.
    const cb = getByTestId(
      "tech-spec-sign-confirm-checkbox",
    ) as HTMLInputElement;
    clickCheckbox(cb);
    const btn = getByTestId("tech-spec-sign-submit") as HTMLButtonElement;
    await waitFor(() => expect(btn.disabled).toBe(false));
    btn.click();

    await waitFor(() => expect(onSign).toHaveBeenCalledTimes(1));
    expect(onSign).toHaveBeenCalledWith(SPEC_ID, {
      role: "amo",
      mandate_id: MANDATE_ID,
    });
  });

  it("@security rôle mandataire (amo) SANS activeMandate → warning + bouton signer disabled", async () => {
    const onSign = vi.fn();
    const { getByTestId, queryByTestId } = render(
      TechnicalSpecSignatureForm,
      {
        props: {
          specId: SPEC_ID,
          role: "amo",
          activeMandate: null,
          onSign,
        },
      },
    );

    // Warning visible.
    const warn = queryByTestId("tech-spec-sign-no-mandate-warning");
    expect(warn).not.toBeNull();
    expect(warn?.textContent).toMatch(/aucun mandat/i);
    expect(warn?.textContent).toMatch(/amo/);

    // Bandeau "mandate-info" PAS rendu (ni "rôle direct" ni "via mandat") —
    // ce sont les deux cas alternatifs, et ici aucun n'est vrai.
    expect(queryByTestId("tech-spec-sign-mandate-info")).toBeNull();

    // Bouton signer disabled même si on coche la checkbox.
    const cb = getByTestId(
      "tech-spec-sign-confirm-checkbox",
    ) as HTMLInputElement;
    clickCheckbox(cb);
    const btn = getByTestId("tech-spec-sign-submit") as HTMLButtonElement;
    await waitFor(() => expect(cb.checked).toBe(true));
    expect(btn.disabled).toBe(true);
    expect(btn.getAttribute("aria-disabled")).toBe("true");

    btn.click();
    expect(onSign).not.toHaveBeenCalled();
  });

  it("@negative onSign rejette (409) → message inline via SignatureForm enfant", async () => {
    const onSign = vi
      .fn()
      .mockRejectedValue(new Error("Signature déjà présente pour (user, role)"));

    const { getByTestId, queryByTestId } = render(
      TechnicalSpecSignatureForm,
      {
        props: {
          specId: SPEC_ID,
          role: "syndic",
          activeMandate: null,
          onSign,
        },
      },
    );

    const cb = getByTestId(
      "tech-spec-sign-confirm-checkbox",
    ) as HTMLInputElement;
    clickCheckbox(cb);

    const btn = getByTestId("tech-spec-sign-submit") as HTMLButtonElement;
    await waitFor(() => expect(btn.disabled).toBe(false));
    btn.click();

    await waitFor(() => {
      const err = queryByTestId("signature-form-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/déjà présente/i);
    });
  });
});
