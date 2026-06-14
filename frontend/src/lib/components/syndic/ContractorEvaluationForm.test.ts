// Story B8 (Phase B FE) — Vitest 4-cat ContractorEvaluationForm.
//
// Couverture (cf. stories.md §B8 AC) :
//   @happy    : form complet → onSubmit appelé avec payload typé
//               CreateContractorEvaluationRequest (contractor + spec Approved
//               + 2 tickets + 5 scores + comment ≥ 10 chars).
//   @edge     : 0 tickets liés → autorisé (multi-select non requis) ;
//               comment exactement 10 chars → OK ; spec au seuil Approved
//               sélectionnable.
//   @security : INV-22 evaluator=contractor → banner + submit DISABLED.
//               Aucun bouton Edit/Delete sur évaluations existantes (testé
//               via ContractorReputation — pas ici car form de création).
//   @negative : spec Draft non sélectionnable (filtre Approved côté FE) ;
//               comment < 10 chars → counter rouge + submit disabled ;
//               approvedSpecs vide → submit disabled + helper.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import ContractorEvaluationForm from "./ContractorEvaluationForm.svelte";
import type { ContractorEvaluationDto } from "../../api/contractor_evaluations";

vi.mock("../../../stores/toast", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}));

const SYNDIC_ID = "11111111-1111-1111-1111-111111111111";
const CONTRACTOR_ID = "22222222-2222-2222-2222-222222222222";
const OTHER_CONTRACTOR_ID = "33333333-3333-3333-3333-333333333333";
const SPEC_APPROVED_ID = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
const SPEC_DRAFT_ID = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
const TICKET_42 = "ticket-42";
const TICKET_43 = "ticket-43";

function fillTextarea(el: HTMLTextAreaElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function selectOption(el: HTMLSelectElement, value: string) {
  el.value = value;
  el.dispatchEvent(new Event("input", { bubbles: true }));
  el.dispatchEvent(new Event("change", { bubbles: true }));
}

function clickScore(
  container: HTMLElement,
  dim: "quality" | "timeliness" | "communication" | "cost" | "overall",
  score: number,
): void {
  const radio = container.querySelector(
    `[data-testid="contractor-eval-scores-${dim}-score-input-${score}"]`,
  ) as HTMLInputElement;
  if (!radio) {
    throw new Error(
      `Radio not found: contractor-eval-scores-${dim}-score-input-${score}`,
    );
  }
  radio.click();
  radio.dispatchEvent(new Event("change", { bubbles: true }));
}

function makeFixture(
  over: Partial<ContractorEvaluationDto> = {},
): ContractorEvaluationDto {
  return {
    id: "eval-uuid-new",
    contractor_user_id: CONTRACTOR_ID,
    evaluator_user_id: SYNDIC_ID,
    technical_spec_id: SPEC_APPROVED_ID,
    linked_ticket_ids: [TICKET_42, TICKET_43],
    scores: {
      quality: 4,
      timeliness: 5,
      communication: 3,
      cost_compliance: 5,
      overall: 4,
    },
    average_score: 4.2,
    comment: "Très professionnel, travail soigné conforme au cahier des charges.",
    created_at: new Date().toISOString(),
    ...over,
  };
}

const baseProps = () => ({
  currentUserId: SYNDIC_ID,
  contractors: [
    { id: CONTRACTOR_ID, label: "Entreprise Toiture SA" },
    { id: OTHER_CONTRACTOR_ID, label: "Plomberie Express SPRL" },
  ],
  specs: [
    {
      id: SPEC_APPROVED_ID,
      title: "Travaux toiture",
      version: "1.0.0",
      status: "Approved",
    },
    {
      id: SPEC_DRAFT_ID,
      title: "Spec brouillon",
      version: "0.1.0",
      status: "Draft",
    },
  ],
  tickets: [
    { id: TICKET_42, title: "Ticket #42 — fuite couverture" },
    { id: TICKET_43, title: "Ticket #43 — gouttière endommagée" },
  ],
});

beforeEach(() => {
  vi.clearAllMocks();
});

describe("ContractorEvaluationForm — Story B8 (4-cat)", () => {
  it("@happy form complet → onSubmit appelé avec CreateContractorEvaluationRequest typé", async () => {
    const created = makeFixture();
    const onSubmit = vi.fn().mockResolvedValue(created);
    const { getByTestId, container } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit },
    });

    // Sélectionne contractor
    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    // Sélectionne spec Approved
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    // Lie 2 tickets
    (
      getByTestId(`contractor-eval-ticket-option-${TICKET_42}`) as HTMLInputElement
    ).click();
    (
      getByTestId(`contractor-eval-ticket-option-${TICKET_43}`) as HTMLInputElement
    ).click();
    // 5 scores
    clickScore(container as HTMLElement, "quality", 4);
    clickScore(container as HTMLElement, "timeliness", 5);
    clickScore(container as HTMLElement, "communication", 3);
    clickScore(container as HTMLElement, "cost", 5);
    clickScore(container as HTMLElement, "overall", 4);
    // Comment ≥ 10 chars
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "Très professionnel, travail soigné conforme au cahier des charges.",
    );

    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const [req] = onSubmit.mock.calls[0];
    expect(req).toMatchObject({
      contractor_user_id: CONTRACTOR_ID,
      technical_spec_id: SPEC_APPROVED_ID,
      linked_ticket_ids: expect.arrayContaining([TICKET_42, TICKET_43]),
      scores: {
        quality: 4,
        timeliness: 5,
        communication: 3,
        cost_compliance: 5,
        overall: 4,
      },
    });
    expect(req.comment.length).toBeGreaterThanOrEqual(10);
  });

  it("@edge 0 tickets liés → autorisé (multi-select non requis)", async () => {
    const onSubmit = vi.fn().mockResolvedValue(makeFixture({ linked_ticket_ids: [] }));
    const { getByTestId, container } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit },
    });

    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    clickScore(container as HTMLElement, "quality", 4);
    clickScore(container as HTMLElement, "timeliness", 4);
    clickScore(container as HTMLElement, "communication", 4);
    clickScore(container as HTMLElement, "cost", 4);
    clickScore(container as HTMLElement, "overall", 4);
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "Comment minimum 10 chars OK.",
    );

    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
    submit.click();

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const [req] = onSubmit.mock.calls[0];
    expect(req.linked_ticket_ids).toEqual([]);
  });

  it("@edge comment exactement 10 chars → submit autorisé", async () => {
    const onSubmit = vi.fn().mockResolvedValue(makeFixture());
    const { getByTestId, container } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit },
    });

    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    clickScore(container as HTMLElement, "quality", 3);
    clickScore(container as HTMLElement, "timeliness", 3);
    clickScore(container as HTMLElement, "communication", 3);
    clickScore(container as HTMLElement, "cost", 3);
    clickScore(container as HTMLElement, "overall", 3);
    // Exactement 10 chars (sans espaces périphériques que trim removerait).
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "0123456789",
    );

    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(false));
  });

  it("@edge spec_select ne montre QUE les Approved (filtre INV-21 côté FE)", () => {
    const { getByTestId, queryByTestId } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit: vi.fn() },
    });
    // L'option SPEC_APPROVED_ID DOIT être présente
    expect(
      queryByTestId(`contractor-eval-spec-option-${SPEC_APPROVED_ID}`),
    ).not.toBeNull();
    // L'option SPEC_DRAFT_ID NE DOIT PAS être présente (filtre Approved).
    expect(
      queryByTestId(`contractor-eval-spec-option-${SPEC_DRAFT_ID}`),
    ).toBeNull();

    // Sanity : le <select> existe et est non-disabled (specs Approved > 0).
    const select = getByTestId(
      "contractor-eval-spec-select",
    ) as HTMLSelectElement;
    expect(select.disabled).toBe(false);
  });

  it("@security INV-22 — self-evaluation → banner visible + submit DISABLED", async () => {
    const onSubmit = vi.fn();
    const { getByTestId, queryByTestId, container } = render(
      ContractorEvaluationForm,
      {
        props: {
          ...baseProps(),
          // current user = un des contractors → self-evaluation forbidden.
          currentUserId: CONTRACTOR_ID,
          onSubmit,
        },
      },
    );

    // Sélectionne lui-même comme contractor.
    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    // Complète tout le reste (sinon submit serait disabled pour autre raison).
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    clickScore(container as HTMLElement, "quality", 4);
    clickScore(container as HTMLElement, "timeliness", 4);
    clickScore(container as HTMLElement, "communication", 4);
    clickScore(container as HTMLElement, "cost", 4);
    clickScore(container as HTMLElement, "overall", 4);
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "Comment suffisamment long pour passer le check.",
    );

    // Banner visible
    await waitFor(() => {
      const banner = queryByTestId("contractor-eval-self-eval-warning");
      expect(banner).not.toBeNull();
      expect(banner?.textContent).toMatch(/ne peut pas s'évaluer/i);
    });

    // Submit DISABLED même si tout le reste est ok.
    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    expect(submit.disabled).toBe(true);

    // Click forcé → onSubmit pas appelé (defense en profondeur).
    submit.click();
    expect(onSubmit).not.toHaveBeenCalled();

    // Switching à un autre contractor → banner disparaît + submit autorisé.
    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      OTHER_CONTRACTOR_ID,
    );
    await waitFor(() => {
      expect(queryByTestId("contractor-eval-self-eval-warning")).toBeNull();
    });
    await waitFor(() => expect(submit.disabled).toBe(false));
  });

  it("@negative comment < 10 chars → counter rouge + submit disabled", async () => {
    const { getByTestId, container } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit: vi.fn() },
    });

    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    clickScore(container as HTMLElement, "quality", 3);
    clickScore(container as HTMLElement, "timeliness", 3);
    clickScore(container as HTMLElement, "communication", 3);
    clickScore(container as HTMLElement, "cost", 3);
    clickScore(container as HTMLElement, "overall", 3);
    // 9 chars (< 10 minimum).
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "trop court",  // 10 chars (exact) — on force 9.
    );
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "123456789",
    );

    const counter = getByTestId("contractor-eval-comment-counter");
    expect(counter.className).toMatch(/text-red-600/);

    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
  });

  it("@negative scores incomplets → submit disabled", async () => {
    const { getByTestId, container } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit: vi.fn() },
    });

    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    selectOption(
      getByTestId("contractor-eval-spec-select") as HTMLSelectElement,
      SPEC_APPROVED_ID,
    );
    // Seulement 3 scores sur 5 → form incomplet.
    clickScore(container as HTMLElement, "quality", 4);
    clickScore(container as HTMLElement, "timeliness", 4);
    clickScore(container as HTMLElement, "communication", 4);
    fillTextarea(
      getByTestId("contractor-eval-comment-textarea") as HTMLTextAreaElement,
      "Comment suffisamment long pour passer le check.",
    );

    const submit = getByTestId(
      "contractor-eval-submit",
    ) as HTMLButtonElement;
    await waitFor(() => expect(submit.disabled).toBe(true));
  });

  it("@negative aucune spec Approved disponible → select disabled + helper", async () => {
    const { getByTestId, queryByTestId } = render(ContractorEvaluationForm, {
      props: {
        ...baseProps(),
        specs: [
          {
            id: SPEC_DRAFT_ID,
            title: "Brouillon seul",
            version: "0.1.0",
            status: "Draft",
          },
        ],
        onSubmit: vi.fn(),
      },
    });

    const select = getByTestId(
      "contractor-eval-spec-select",
    ) as HTMLSelectElement;
    expect(select.disabled).toBe(true);

    // Sélectionne contractor → error spec quand même puisque aucune Approved.
    selectOption(
      getByTestId("contractor-eval-contractor-select") as HTMLSelectElement,
      CONTRACTOR_ID,
    );
    await waitFor(() => {
      const err = queryByTestId("contractor-eval-error-spec");
      expect(err).not.toBeNull();
      expect(err?.textContent).toMatch(/aucune fiche technique approuvée/i);
    });
  });

  it("rendu data-testid scores-{dim} pour chaque dimension (5 critères)", () => {
    const { queryByTestId } = render(ContractorEvaluationForm, {
      props: { ...baseProps(), onSubmit: vi.fn() },
    });
    expect(queryByTestId("contractor-eval-scores-quality")).not.toBeNull();
    expect(queryByTestId("contractor-eval-scores-timeliness")).not.toBeNull();
    expect(
      queryByTestId("contractor-eval-scores-communication"),
    ).not.toBeNull();
    expect(queryByTestId("contractor-eval-scores-cost")).not.toBeNull();
    expect(queryByTestId("contractor-eval-scores-overall")).not.toBeNull();
  });
});
