// Story B8 (Phase B FE) — Vitest 4-cat ContractorReputation.
//
// Couverture (cf. stories.md §B8 AC) :
//   @happy    : 3 évaluations → moyennes calculées correctement + 3 rows
//               affichées + count = 3.
//   @edge     : 0 évaluations → empty state + moyennes = "—" ; 1 évaluation →
//               moyennes = exactement la valeur unique.
//   @security : INV-24 (append-only) — AUCUN bouton "Modifier" / "Supprimer"
//               sur les lignes. Vérification explicite par sélecteurs négatifs.
//   @negative : evaluations.length === 0 → table absente, empty state visible.
//
// Pas de mock fetch — les évaluations sont passées en prop (DI cohérent
// SlaBadge, ExpirationBadge, etc.).

import { describe, it, expect } from "vitest";
import { render } from "../../../test-helpers";
import ContractorReputation from "./ContractorReputation.svelte";
import type { ContractorEvaluationDto } from "../../api/contractor_evaluations";

function makeEval(
  id: string,
  scores: Partial<ContractorEvaluationDto["scores"]> = {},
): ContractorEvaluationDto {
  return {
    id,
    contractor_user_id: "contractor-1",
    evaluator_user_id: "syndic-1",
    technical_spec_id: "spec-1",
    linked_ticket_ids: [],
    scores: {
      quality: 4,
      timeliness: 4,
      communication: 4,
      cost_compliance: 4,
      overall: 4,
      ...scores,
    },
    average_score: 4,
    comment: `Comment for ${id}`,
    created_at: new Date("2026-06-01T10:00:00Z").toISOString(),
  };
}

describe("ContractorReputation — Story B8 (4-cat)", () => {
  it("@happy 3 évaluations → moyennes calculées + 3 rows + count=3", () => {
    const evaluations: ContractorEvaluationDto[] = [
      makeEval("e1", {
        quality: 5,
        timeliness: 4,
        communication: 3,
        cost_compliance: 5,
        overall: 4,
      }),
      makeEval("e2", {
        quality: 4,
        timeliness: 5,
        communication: 4,
        cost_compliance: 5,
        overall: 4,
      }),
      makeEval("e3", {
        quality: 3,
        timeliness: 3,
        communication: 5,
        cost_compliance: 5,
        overall: 4,
      }),
    ];

    const { getByTestId } = render(ContractorReputation, {
      props: { contractorName: "Entreprise Toiture SA", evaluations },
    });

    expect(getByTestId("contractor-reputation-name").textContent).toMatch(
      /Entreprise Toiture SA/,
    );
    expect(getByTestId("contractor-reputation-count").textContent).toMatch(
      /3/,
    );

    // Moyennes : quality = (5+4+3)/3 = 4.0 → "4.0/5"
    expect(
      getByTestId("contractor-reputation-avg-quality").textContent,
    ).toMatch(/4\.0\/5/);
    // timeliness = (4+5+3)/3 = 4.0
    expect(
      getByTestId("contractor-reputation-avg-timeliness").textContent,
    ).toMatch(/4\.0\/5/);
    // communication = (3+4+5)/3 = 4.0
    expect(
      getByTestId("contractor-reputation-avg-communication").textContent,
    ).toMatch(/4\.0\/5/);
    // cost = (5+5+5)/3 = 5.0
    expect(
      getByTestId("contractor-reputation-avg-cost").textContent,
    ).toMatch(/5\.0\/5/);
    // overall = (4+4+4)/3 = 4.0
    expect(
      getByTestId("contractor-reputation-avg-overall").textContent,
    ).toMatch(/4\.0\/5/);

    // 3 rows présentes
    expect(getByTestId("contractor-reputation-eval-row-e1")).not.toBeNull();
    expect(getByTestId("contractor-reputation-eval-row-e2")).not.toBeNull();
    expect(getByTestId("contractor-reputation-eval-row-e3")).not.toBeNull();
  });

  it("@edge 0 évaluations → empty state + moyennes '—'", () => {
    const { getByTestId, queryByTestId } = render(ContractorReputation, {
      props: { contractorName: "Plomberie X", evaluations: [] },
    });

    expect(getByTestId("contractor-reputation-count").textContent).toMatch(
      /0/,
    );
    expect(getByTestId("contractor-reputation-empty")).not.toBeNull();
    expect(
      getByTestId("contractor-reputation-empty").textContent,
    ).toMatch(/aucune évaluation/i);

    // Table absente.
    expect(queryByTestId("contractor-reputation-list")).toBeNull();

    // Moyennes affichent "—".
    expect(
      getByTestId("contractor-reputation-avg-quality").textContent,
    ).toMatch(/—/);
    expect(
      getByTestId("contractor-reputation-avg-overall").textContent,
    ).toMatch(/—/);
  });

  it("@edge 1 évaluation → moyenne = valeur unique (pas de division zero)", () => {
    const evaluations: ContractorEvaluationDto[] = [
      makeEval("e1", {
        quality: 5,
        timeliness: 5,
        communication: 5,
        cost_compliance: 5,
        overall: 5,
      }),
    ];
    const { getByTestId } = render(ContractorReputation, {
      props: { contractorName: "Solo Inc", evaluations },
    });
    expect(getByTestId("contractor-reputation-count").textContent).toMatch(
      /1/,
    );
    expect(
      getByTestId("contractor-reputation-avg-quality").textContent,
    ).toMatch(/5\.0\/5/);
    expect(
      getByTestId("contractor-reputation-avg-overall").textContent,
    ).toMatch(/5\.0\/5/);
  });

  it("@security INV-24 — AUCUN bouton Edit/Delete sur les rows (append-only)", () => {
    const evaluations: ContractorEvaluationDto[] = [
      makeEval("e1"),
      makeEval("e2"),
    ];
    const { container, queryByTestId } = render(ContractorReputation, {
      props: { contractorName: "Test", evaluations },
    });

    // Aucun bouton Edit / Delete / Modifier / Supprimer.
    expect(
      queryByTestId("contractor-reputation-eval-row-e1-edit"),
    ).toBeNull();
    expect(
      queryByTestId("contractor-reputation-eval-row-e1-delete"),
    ).toBeNull();

    // Search exhaustive : aucun <button> dans la table.
    const table = container.querySelector(
      '[data-testid="contractor-reputation-list"]',
    );
    expect(table).not.toBeNull();
    const buttons = table?.querySelectorAll("button");
    expect(buttons?.length ?? 0).toBe(0);

    // Pas non plus de <a> "Modifier" / "Supprimer" (defensive coverage).
    const links = table?.querySelectorAll("a");
    if (links && links.length > 0) {
      for (const link of Array.from(links)) {
        const t = link.textContent?.toLowerCase() ?? "";
        expect(t).not.toMatch(/modifier|supprimer|edit|delete/);
      }
    }
  });

  it("@security INV-FE9 — aria-label complet sur chaque moyenne", () => {
    const evaluations: ContractorEvaluationDto[] = [
      makeEval("e1", { quality: 4, overall: 5 }),
    ];
    const { getByTestId } = render(ContractorReputation, {
      props: { contractorName: "Test", evaluations },
    });
    expect(
      getByTestId("contractor-reputation-avg-quality").getAttribute(
        "aria-label",
      ),
    ).toMatch(/qualité technique\s*:\s*4\.0 sur 5/i);
    expect(
      getByTestId("contractor-reputation-avg-overall").getAttribute(
        "aria-label",
      ),
    ).toMatch(/note globale\s*:\s*5\.0 sur 5/i);
  });

  it("@negative caption sr-only mentionne 'lecture seule' / 'append-only'", () => {
    const evaluations: ContractorEvaluationDto[] = [makeEval("e1")];
    const { container } = render(ContractorReputation, {
      props: { contractorName: "Caption Test", evaluations },
    });
    const caption = container.querySelector("caption");
    expect(caption).not.toBeNull();
    expect(caption?.className).toMatch(/sr-only/);
    expect(caption?.textContent).toMatch(/lecture seule|append-only/i);
  });

  it("idSuffix → data-testid composé (contractor-reputation-name-{suffix})", () => {
    const { getByTestId } = render(ContractorReputation, {
      props: {
        contractorName: "Suffix Test",
        evaluations: [],
        idSuffix: "widget-1",
      },
    });
    expect(getByTestId("contractor-reputation-name-widget-1")).not.toBeNull();
    expect(
      getByTestId("contractor-reputation-count-widget-1"),
    ).not.toBeNull();
    expect(
      getByTestId("contractor-reputation-avg-quality-widget-1"),
    ).not.toBeNull();
  });
});
