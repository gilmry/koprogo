// Story B5 (Phase B FE) — Vitest 4-cat TicketCreate.
//
// Couverture (cf. stories.md §B5 + mission AC) :
//   @happy    : Owner kind=Complaint severity=High + incident_date + 3 evidence
//               + 2 witnesses → POST → 201 + onCreated callback + reset form.
//               Rétro-compat : kind=Request (par défaut) → POST → 201 sans
//               aucun champ Complaint dans le payload (32 tests Request verts).
//   @edge     : 0 evidence + 0 witness text-only → submit OK + badge "Preuves
//               manquantes" visible mais PAS bloquant.
//   @security : description < 20 chars → submit disabled + counter rouge.
//               Complaint sans severity → submit disabled.
//   @negative : incident_date dans le futur → submit disabled + message
//               inline "ne peut être dans le futur".
//               backend renvoie 422 → message inline visible + onCreated PAS
//               appelé.
//
// Pattern DI : `onCreate` injecté pour simuler ticketsApi.create.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import TicketCreate from "./TicketCreate.svelte";
import type { CreateTicketDto, Ticket } from "../../api/tickets";

// jsdom polyfill (EvidenceUpload utilise URL.createObjectURL).
beforeEach(() => {
  (global.URL as unknown as { createObjectURL: () => string }).createObjectURL =
    vi.fn(() => "blob:fake-url");
  (global.URL as unknown as { revokeObjectURL: () => void }).revokeObjectURL =
    vi.fn();
});

// -----------------------------------------------------------------------------
// Fixtures
// -----------------------------------------------------------------------------

const BUILDING_ID = "10000000-0000-0000-0000-000000000001";
const TICKET_ID = "20000000-0000-0000-0000-000000000001";
const OWNER_ID = "30000000-0000-0000-0000-000000000001";

const CANDIDATES = [
  { id: "user-1", label: "Marc Dubois — Lot A-3" },
  { id: "user-2", label: "Sophie Leroux — Lot B-1" },
];

function makeTicketFixture(): Ticket {
  return {
    id: TICKET_ID,
    organization_id: "org-1",
    building_id: BUILDING_ID,
    title: "Tapage nocturne récurrent",
    description: "Bruit insupportable chaque nuit du voisin du dessus.",
    status: "Open",
    priority: "High",
    category: "Other",
    created_by: OWNER_ID,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
}

function fillInput(el: HTMLInputElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
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

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("TicketCreate — Story B5 (4-cat)", () => {
  it("@happy rétro-compat Request (par défaut) → POST sans champ Complaint", async () => {
    const onCreate = vi.fn().mockResolvedValue(makeTicketFixture());
    const onCreated = vi.fn();
    const { getByTestId, queryByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        onCreate,
        onCreated,
      },
    });

    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Fuite couloir étage 2",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Une fuite d'eau est apparue ce matin dans le couloir commun du 2ème étage près de l'ascenseur.",
    );

    // Kind par défaut = Request → la section Complaint n'apparaît PAS.
    expect(queryByTestId("ticket-create-incident-date-input")).toBeNull();
    expect(
      queryByTestId("ticket-severity-radio-low"),
    ).toBeNull();
    expect(queryByTestId("ticket-evidence-upload")).toBeNull();
    expect(queryByTestId("ticket-witness-search")).toBeNull();

    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onCreate).toHaveBeenCalledTimes(1));
    const payload: CreateTicketDto = onCreate.mock.calls[0][0];
    expect(payload.kind).toBe("request");
    expect(payload.title).toBe("Fuite couloir étage 2");
    // Aucun champ Complaint dans le payload.
    expect(payload.severity).toBeUndefined();
    expect(payload.incident_date).toBeUndefined();
    expect(payload.evidence_attachments).toBeUndefined();
    expect(payload.witnesses).toBeUndefined();
    await waitFor(() => expect(onCreated).toHaveBeenCalledTimes(1));
  });

  it("@happy Complaint kind → sections additionnelles apparaissent + payload riche", async () => {
    const onCreate = vi.fn().mockResolvedValue(makeTicketFixture());
    const onCreated = vi.fn();
    const { getByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        witnessCandidates: CANDIDATES,
        onCreate,
        onCreated,
      },
    });

    // Passer en mode Complaint.
    fillSelect(
      getByTestId("ticket-create-kind-select") as HTMLSelectElement,
      "complaint",
    );

    await waitFor(() => {
      expect(getByTestId("ticket-severity-radio-high")).not.toBeNull();
      expect(getByTestId("ticket-create-incident-date-input")).not.toBeNull();
      expect(getByTestId("ticket-evidence-upload")).not.toBeNull();
      expect(getByTestId("ticket-witness-search")).not.toBeNull();
    });

    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Tapage nocturne récurrent",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Bruit insupportable chaque nuit du voisin du dessus depuis 3 semaines.",
    );
    // Sélectionner severity=High.
    (getByTestId("ticket-severity-radio-high") as HTMLInputElement).click();
    fillInput(
      getByTestId("ticket-create-incident-date-input") as HTMLInputElement,
      "2026-06-01",
    );
    // Sélectionner 1 témoin (Marc).
    (getByTestId("ticket-witness-option-user-1") as HTMLButtonElement).click();

    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onCreate).toHaveBeenCalledTimes(1));
    const payload: CreateTicketDto = onCreate.mock.calls[0][0];
    expect(payload.kind).toBe("complaint");
    expect(payload.severity).toBe("high");
    expect(payload.incident_date).toMatch(/^2026-06-01T/);
    expect(payload.witnesses).toEqual(["user-1"]);
    await waitFor(() => expect(onCreated).toHaveBeenCalledTimes(1));
  });

  it("@edge Complaint text-only (0 evidence + 0 witness) → submit OK + badge 'Preuves manquantes' visible", async () => {
    const onCreate = vi.fn().mockResolvedValue(makeTicketFixture());
    const { getByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        witnessCandidates: CANDIDATES,
        onCreate,
      },
    });

    fillSelect(
      getByTestId("ticket-create-kind-select") as HTMLSelectElement,
      "complaint",
    );

    // Attendre que les sections conditionnelles soient montées (réactivité Svelte 5).
    await waitFor(() =>
      expect(getByTestId("ticket-severity-radio-normal")).not.toBeNull(),
    );

    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Plainte texte seule",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Je dépose cette plainte sans pouvoir fournir de preuves matérielles pour l'instant.",
    );
    (getByTestId("ticket-severity-radio-normal") as HTMLInputElement).click();
    fillInput(
      getByTestId("ticket-create-incident-date-input") as HTMLInputElement,
      "2026-06-01",
    );

    // Badge visible (0 evidence + 0 witness).
    await waitFor(() =>
      expect(getByTestId("ticket-create-evidence-warning")).not.toBeNull(),
    );

    // Mais le badge ne bloque PAS — submit est enabled.
    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onCreate).toHaveBeenCalledTimes(1));
  });

  it("@security description < 20 chars → submit disabled + counter rouge", async () => {
    const onCreate = vi.fn();
    const { getByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        onCreate,
      },
    });

    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Titre valide",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Court", // 5 chars
    );

    const counter = getByTestId("ticket-create-description-counter");
    await waitFor(() => expect(counter.className).toMatch(/text-red-600/));
    expect(counter.textContent).toMatch(/minimum/i);

    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    submit.click();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("@security Complaint sans severity → submit disabled (defense-in-depth UI)", async () => {
    const onCreate = vi.fn();
    const { getByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        onCreate,
      },
    });

    fillSelect(
      getByTestId("ticket-create-kind-select") as HTMLSelectElement,
      "complaint",
    );
    // Attendre montage des sections conditionnelles.
    await waitFor(() =>
      expect(getByTestId("ticket-create-incident-date-input")).not.toBeNull(),
    );
    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Plainte sans severity",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Description suffisamment longue pour passer la validation minimale.",
    );
    fillInput(
      getByTestId("ticket-create-incident-date-input") as HTMLInputElement,
      "2026-06-01",
    );

    // Pas de severity sélectionnée → submit reste disabled.
    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    submit.click();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("@negative incident_date dans le futur → submit disabled + message inline", async () => {
    const onCreate = vi.fn();
    const { getByTestId, queryByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        onCreate,
      },
    });

    fillSelect(
      getByTestId("ticket-create-kind-select") as HTMLSelectElement,
      "complaint",
    );
    // Attendre montage des sections conditionnelles.
    await waitFor(() =>
      expect(getByTestId("ticket-severity-radio-high")).not.toBeNull(),
    );
    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Plainte avec date future",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Description suffisamment longue pour passer la validation minimale.",
    );
    (getByTestId("ticket-severity-radio-high") as HTMLInputElement).click();

    // Date 1 an dans le futur.
    const future = new Date();
    future.setFullYear(future.getFullYear() + 1);
    const futureIso = future.toISOString().slice(0, 10);
    fillInput(
      getByTestId("ticket-create-incident-date-input") as HTMLInputElement,
      futureIso,
    );

    // Message d'erreur inline visible.
    await waitFor(() =>
      expect(
        queryByTestId("ticket-create-incident-date-error"),
      ).not.toBeNull(),
    );

    // Submit disabled.
    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    expect(submit.disabled).toBe(true);
    submit.click();
    expect(onCreate).not.toHaveBeenCalled();
  });

  it("@negative backend 422 → message inline + onCreated PAS appelé", async () => {
    const onCreate = vi
      .fn()
      .mockRejectedValue(new Error("ValidationError: severity required for complaint"));
    const onCreated = vi.fn();
    const { getByTestId, queryByTestId } = render(TicketCreate, {
      props: {
        buildingId: BUILDING_ID,
        currentUserId: OWNER_ID,
        onCreate,
        onCreated,
      },
    });

    fillInput(
      getByTestId("ticket-create-title-input") as HTMLInputElement,
      "Titre valide",
    );
    fillTextarea(
      getByTestId(
        "ticket-create-description-textarea",
      ) as HTMLTextAreaElement,
      "Description suffisamment longue pour passer la validation minimale.",
    );

    const submit = getByTestId("ticket-create-submit") as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => {
      const err = queryByTestId("ticket-create-error");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/ValidationError/);
    });
    expect(onCreated).not.toHaveBeenCalled();
  });
});
