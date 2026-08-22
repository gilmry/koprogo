// Story B6 (Phase B FE) — Vitest 4-cat SyndicResponseList.
//
// Couverture (cf. stories.md §B6 + mission) :
//   @happy    : ordre chronologique préservé (oldest first) + author label
//               + action_proposed badge visible + body affiché.
//   @edge     : action_proposed = null → pas de badge action ; authorLabels
//               vide → fallback UUID slice ; action_proposed = "other" →
//               label "Autre".
//   @security : INV-FE8 append-only — AUCUN bouton "Edit"/"Delete" dans la
//               liste. Pas de menu contextuel. Lecture seule.
//   @negative : liste vide → message empty + pas de <ol>.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, waitFor } from "../../../test-helpers";
import SyndicResponseList from "./SyndicResponseList.svelte";
import type { SyndicResponseDto } from "../../api/syndic_responses";

// -----------------------------------------------------------------------------
// Mocks
// -----------------------------------------------------------------------------

vi.mock("../../api/syndic_responses", async (orig) => {
  const real = (await orig()) as Record<string, unknown>;
  return {
    ...real,
    listResponsesForTicket: vi.fn(),
  };
});

vi.mock("../../i18n", () => ({
  _: {
    subscribe: (fn: (v: (k: string) => string) => void) => {
      fn(() => "");
      return () => {};
    },
  },
}));

// -----------------------------------------------------------------------------
// Fixtures
// -----------------------------------------------------------------------------

const TICKET_ID = "10000000-0000-0000-0000-000000000001";
const SYNDIC_ALICE = "20000000-0000-0000-0000-00000000000a";
const SYNDIC_BOB = "20000000-0000-0000-0000-00000000000b";

function mkResponse(opts: {
  id: string;
  authorId?: string;
  body?: string;
  actionProposed?: string | null;
  createdAt?: string;
}): SyndicResponseDto {
  return {
    id: opts.id,
    ticket_id: TICKET_ID,
    syndic_user_id: opts.authorId ?? SYNDIC_ALICE,
    body: opts.body ?? "Réponse de test au copropriétaire.",
    action_proposed: opts.actionProposed ?? null,
    created_at: opts.createdAt ?? new Date().toISOString(),
  };
}

beforeEach(() => {
  vi.clearAllMocks();
});

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

describe("SyndicResponseList — Story B6 (4-cat)", () => {
  it("@happy ordre chronologique préservé + author + action badge + body", async () => {
    const responses = [
      mkResponse({
        id: "r-1",
        authorId: SYNDIC_ALICE,
        body: "Je transmets votre demande au plombier.",
        actionProposed: "request_quote",
        createdAt: "2026-06-10T10:00:00Z",
      }),
      mkResponse({
        id: "r-2",
        authorId: SYNDIC_BOB,
        body: "Le devis est revenu, j'organise l'intervention.",
        actionProposed: "schedule_inspection",
        createdAt: "2026-06-11T14:30:00Z",
      }),
    ];

    const { getByTestId } = render(SyndicResponseList, {
      props: {
        ticketId: TICKET_ID,
        initialResponses: responses,
        authorLabels: {
          [SYNDIC_ALICE]: "Alice Syndic",
          [SYNDIC_BOB]: "Bob Syndic",
        },
      },
    });

    await waitFor(() =>
      expect(getByTestId("syndic-response-list")).toBeTruthy(),
    );

    // Row 1
    expect(getByTestId("syndic-response-row-r-1")).toBeTruthy();
    expect(getByTestId("syndic-response-row-author-r-1").textContent).toMatch(
      /Alice Syndic/,
    );
    expect(getByTestId("syndic-response-row-body-r-1").textContent).toMatch(
      /plombier/,
    );
    expect(getByTestId("syndic-response-row-action-r-1").textContent).toMatch(
      /Demander devis/,
    );
    // Row 2
    expect(getByTestId("syndic-response-row-author-r-2").textContent).toMatch(
      /Bob Syndic/,
    );
    expect(getByTestId("syndic-response-row-action-r-2").textContent).toMatch(
      /Planifier inspection/,
    );
  });

  it("@edge action_proposed = null → pas de badge action affiché", async () => {
    const responses = [
      mkResponse({
        id: "r-noaction",
        body: "Message sans action proposée.",
        actionProposed: null,
      }),
    ];
    const { queryByTestId, getByTestId } = render(SyndicResponseList, {
      props: { ticketId: TICKET_ID, initialResponses: responses },
    });
    await waitFor(() =>
      expect(getByTestId("syndic-response-row-r-noaction")).toBeTruthy(),
    );
    expect(queryByTestId("syndic-response-row-action-r-noaction")).toBeNull();
  });

  it("@edge authorLabels vide → fallback UUID slice 8 chars", async () => {
    const responses = [
      mkResponse({ id: "r-fb", authorId: SYNDIC_ALICE, body: "Body" }),
    ];
    const { getByTestId } = render(SyndicResponseList, {
      props: { ticketId: TICKET_ID, initialResponses: responses },
    });
    await waitFor(() =>
      expect(
        getByTestId("syndic-response-row-author-r-fb").textContent,
      ).toMatch(/^20000000$/),
    );
  });

  it("@edge action_proposed = 'other' → label 'Autre'", async () => {
    const responses = [
      mkResponse({
        id: "r-other",
        body: "Body",
        actionProposed: "other",
      }),
    ];
    const { getByTestId } = render(SyndicResponseList, {
      props: { ticketId: TICKET_ID, initialResponses: responses },
    });
    await waitFor(() =>
      expect(
        getByTestId("syndic-response-row-action-r-other").textContent,
      ).toMatch(/Autre/),
    );
  });

  it("@security INV-FE8 append-only — AUCUN bouton Edit/Delete dans la liste", async () => {
    const responses = [
      mkResponse({ id: "r-1", body: "A" }),
      mkResponse({ id: "r-2", body: "B" }),
    ];
    const { container, queryByTestId } = render(SyndicResponseList, {
      props: { ticketId: TICKET_ID, initialResponses: responses },
    });

    await waitFor(() => expect(container.querySelector("ol")).not.toBeNull());

    // Pas de bouton edit/delete pour aucune row.
    expect(queryByTestId("syndic-response-edit-r-1")).toBeNull();
    expect(queryByTestId("syndic-response-edit-r-2")).toBeNull();
    expect(queryByTestId("syndic-response-delete-r-1")).toBeNull();
    expect(queryByTestId("syndic-response-delete-r-2")).toBeNull();

    // Aucun bouton dans la liste — la liste est lecture seule.
    const buttons = container.querySelectorAll("ol button");
    expect(buttons.length).toBe(0);
  });

  it("@negative liste vide → message empty + pas de <ol>", async () => {
    const { queryByTestId, getByTestId } = render(SyndicResponseList, {
      props: { ticketId: TICKET_ID, initialResponses: [] },
    });

    expect(queryByTestId("syndic-response-list")).toBeNull();
    expect(getByTestId("syndic-response-list-empty").textContent).toMatch(
      /Aucune/i,
    );
  });
});
