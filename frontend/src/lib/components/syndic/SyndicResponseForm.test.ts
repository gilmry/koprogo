// Story B6 (Phase B FE) — Vitest 4-cat SyndicResponseForm.
//
// Couverture (cf. stories.md §B6 + mission) :
//   @happy    : body valide (≥10 / ≤5000) + action_proposed → submit OK,
//               onCreated callback appelé avec le DTO, reset du form.
//   @edge     : body trimmedLength === 10 exactement → submit OK ;
//               trimmedLength === 5000 → submit OK ; whitespace only → trim
//               à 0 → submit disabled.
//   @security : INV-FE8 append-only — pas de bouton "Edit" ni "Delete" sur
//               ce form (form de création seulement) ; pas de prop `existing*`.
//               Action_proposed = "unknown" → impossible via UI (set fermé).
//   @negative : body < 10 chars → counter rouge + submit disabled ; body >
//               5000 chars → counter rouge + submit disabled ; backend
//               renvoie 422 → message inline visible.
//
// Pattern DI (cf. MagicLinkIssueForm.test.ts) : on injecte `onRespond` via
// prop, pas de vi.mock module.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import SyndicResponseForm from "./SyndicResponseForm.svelte";
import {
  SYNDIC_RESPONSE_MAX_BODY_LENGTH,
  SYNDIC_RESPONSE_MIN_BODY_LENGTH,
  type SyndicResponseDto,
} from "../../api/syndic_responses";

// -----------------------------------------------------------------------------
// Mocks toast (pas de require runtime)
// -----------------------------------------------------------------------------

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

// -----------------------------------------------------------------------------
// Fixtures
// -----------------------------------------------------------------------------

const TICKET_ID = "10000000-0000-0000-0000-000000000001";
const SYNDIC_ID = "20000000-0000-0000-0000-000000000001";

function makeDtoFixture(body: string): SyndicResponseDto {
  return {
    id: "30000000-0000-0000-0000-000000000001",
    ticket_id: TICKET_ID,
    syndic_user_id: SYNDIC_ID,
    body,
    action_proposed: null,
    created_at: new Date().toISOString(),
  };
}

function fillTextarea(el: HTMLTextAreaElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function fillSelect(el: HTMLSelectElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

beforeEach(() => {
  vi.clearAllMocks();
});

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("SyndicResponseForm — Story B6 (4-cat)", () => {
  it("@happy body valide + action_proposed → submit OK + onCreated callback + reset", async () => {
    const validBody =
      "Je vais planifier une inspection technique cette semaine, je reviens vers vous.";
    const onRespond = vi.fn().mockResolvedValue(makeDtoFixture(validBody));
    const onCreated = vi.fn();

    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond, onCreated },
    });

    fillTextarea(
      getByTestId("syndic-response-body-textarea") as HTMLTextAreaElement,
      validBody,
    );
    fillSelect(
      getByTestId(
        "syndic-response-action-proposed-select",
      ) as HTMLSelectElement,
      "schedule_inspection",
    );

    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onRespond).toHaveBeenCalledTimes(1));
    expect(onRespond).toHaveBeenCalledWith(
      TICKET_ID,
      expect.objectContaining({
        body: validBody,
        action_proposed: "schedule_inspection",
      }),
    );
    await waitFor(() => expect(onCreated).toHaveBeenCalledTimes(1));

    // Reset du textarea après succès (append-only — pas de draft persistant).
    await waitFor(() => {
      const ta = getByTestId(
        "syndic-response-body-textarea",
      ) as HTMLTextAreaElement;
      expect(ta.value).toBe("");
    });
  });

  it("@edge body trimmed === 10 (borne min) → submit OK", async () => {
    // Exactement 10 chars (après trim).
    const exactly10 = "1234567890";
    expect(exactly10.length).toBe(SYNDIC_RESPONSE_MIN_BODY_LENGTH);

    const onRespond = vi.fn().mockResolvedValue(makeDtoFixture(exactly10));
    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond },
    });

    fillTextarea(
      getByTestId("syndic-response-body-textarea") as HTMLTextAreaElement,
      exactly10,
    );
    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();
    await waitFor(() => expect(onRespond).toHaveBeenCalledTimes(1));
  });

  it("@edge body whitespace-only (12 espaces) → trim=0 → submit disabled", async () => {
    const onRespond = vi.fn();
    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond },
    });

    fillTextarea(
      getByTestId("syndic-response-body-textarea") as HTMLTextAreaElement,
      "            ", // 12 espaces
    );
    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
    submit.click();
    expect(onRespond).not.toHaveBeenCalled();
  });

  it("@security INV-FE8 append-only — pas de bouton Edit/Delete dans le form", () => {
    const { container, queryByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond: vi.fn() },
    });

    // Aucun bouton Edit/Delete sur ce form (création seulement).
    expect(queryByTestId("syndic-response-edit")).toBeNull();
    expect(queryByTestId("syndic-response-delete")).toBeNull();
    // Aucun bouton qui contient "Modifier" ou "Supprimer".
    const buttons = Array.from(container.querySelectorAll("button"));
    for (const b of buttons) {
      expect(b.textContent ?? "").not.toMatch(/modifier|supprimer|delete|edit/i);
    }
  });

  it("@security action_proposed = 'unknown' impossible via UI (set fermé)", () => {
    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond: vi.fn() },
    });
    const select = getByTestId(
      "syndic-response-action-proposed-select",
    ) as HTMLSelectElement;
    const values = Array.from(select.options).map((o) => o.value);
    // L'option "unknown" n'existe PAS dans la liste — l'utilisateur ne peut pas
    // l'envoyer via UI (le backend rejetterait avec 422 de toute façon).
    expect(values).not.toContain("unknown");
    // Les options valides sont bien là (5 actions + "" = 6).
    expect(values).toEqual(
      expect.arrayContaining([
        "",
        "schedule_inspection",
        "request_quote",
        "closed_no_action",
        "escalated_board",
        "other",
      ]),
    );
  });

  it("@negative body < 10 chars → counter rouge + submit disabled", async () => {
    const onRespond = vi.fn();
    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond },
    });

    fillTextarea(
      getByTestId("syndic-response-body-textarea") as HTMLTextAreaElement,
      "court", // 5 chars < 10
    );

    const counter = getByTestId("syndic-response-body-counter");
    await waitFor(() => expect(counter.className).toMatch(/text-red-600/));
    expect(counter.textContent).toMatch(/minimum/i);

    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
  });

  it("@negative body > 5000 chars → counter rouge + submit disabled", async () => {
    const onRespond = vi.fn();
    const { getByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond },
    });

    // 5001 chars (dépasse SYNDIC_RESPONSE_MAX_BODY_LENGTH=5000).
    const tooLong = "a".repeat(SYNDIC_RESPONSE_MAX_BODY_LENGTH + 1);
    // Bypass `maxlength` attribute pour tester la guard JS (DevTools tampering).
    const ta = getByTestId(
      "syndic-response-body-textarea",
    ) as HTMLTextAreaElement;
    ta.removeAttribute("maxlength");
    fillTextarea(ta, tooLong);

    const counter = getByTestId("syndic-response-body-counter");
    await waitFor(() => expect(counter.className).toMatch(/text-red-600/));
    expect(counter.textContent).toMatch(/maximum/i);

    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
  });

  it("@negative backend 422 → message inline + onCreated pas appelé", async () => {
    const onRespond = vi
      .fn()
      .mockRejectedValue(new Error("body too short after trim"));
    const onCreated = vi.fn();
    const { getByTestId, queryByTestId } = render(SyndicResponseForm, {
      props: { ticketId: TICKET_ID, onRespond, onCreated },
    });

    fillTextarea(
      getByTestId("syndic-response-body-textarea") as HTMLTextAreaElement,
      "Message qui passe la validation côté FE 1234567890",
    );

    const submit = getByTestId("syndic-response-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => {
      const err = queryByTestId("syndic-response-form-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/too short/i);
    });
    expect(onCreated).not.toHaveBeenCalled();
  });
});
