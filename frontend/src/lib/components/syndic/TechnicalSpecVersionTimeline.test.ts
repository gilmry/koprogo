// Story B7 (Phase B FE) — Vitest 4-cat TechnicalSpecVersionTimeline.
//
// Couverture (cf. stories.md §B7 AC) :
//   @happy    : 2 versions (v1.1.0 Approved courante + v1.0.0 Superseded) →
//               timeline rendue, v1.0.0 grisée + opacity-70.
//   @edge     : versions [] → message "Aucune version".
//   @security : pas de bouton Edit/Delete sur les lignes (read-only).
//   @negative : status inconnu → fallback badge sans crash.

import { describe, it, expect, vi } from "vitest";
import { render } from "../../../test-helpers";
import TechnicalSpecVersionTimeline from "./TechnicalSpecVersionTimeline.svelte";
import type { TechnicalSpecDto } from "../../api/technical_specs";

function spec(over: Partial<TechnicalSpecDto>): TechnicalSpecDto {
  return {
    id: "id-x",
    acp_id: "acp-1",
    building_id: null,
    title: "t",
    description: "d",
    version: "1.0.0",
    deliverables: ["dlv"],
    required_signatures: ["syndic"],
    attachments: [],
    status: "Approved",
    created_by: "u",
    previous_version_id: null,
    created_at: "2026-06-09T10:00:00Z",
    updated_at: "2026-06-09T10:00:00Z",
    ...over,
  };
}

describe("TechnicalSpecVersionTimeline — Story B7 (4-cat)", () => {
  it("@happy 2 versions (Approved courante + Superseded) → timeline rendue + grisage Superseded", () => {
    const v100 = spec({
      id: "v100",
      version: "1.0.0",
      status: "Superseded",
      created_at: "2026-06-09T10:00:00Z",
    });
    const v110 = spec({
      id: "v110",
      version: "1.1.0",
      status: "Approved",
      created_at: "2026-06-10T10:00:00Z",
    });

    const { getByTestId } = render(TechnicalSpecVersionTimeline, {
      props: { versions: [v100, v110], currentVersionId: "v110" },
    });

    expect(getByTestId("tech-spec-timeline")).toBeTruthy();
    const row110 = getByTestId("tech-spec-version-row-1.1.0");
    const row100 = getByTestId("tech-spec-version-row-1.0.0");

    expect(row110.getAttribute("data-status")).toBe("Approved");
    expect(row110.getAttribute("data-current")).toBe("true");
    expect(row110.getAttribute("aria-current")).toBe("true");

    expect(row100.getAttribute("data-status")).toBe("Superseded");
    expect(row100.getAttribute("data-current")).toBe("false");
    // Grisage via class opacity (combiné avec text-gray-500 — INV-FE9 daltoniens).
    expect(row100.className).toMatch(/opacity-70|text-gray-500/);
  });

  it("@happy onSelect callback déclenché au click sur 'Voir'", () => {
    const v = spec({ id: "v100", version: "1.0.0", status: "Approved" });
    const onSelect = vi.fn();
    const { container } = render(TechnicalSpecVersionTimeline, {
      props: { versions: [v], currentVersionId: "v100", onSelect },
    });

    const btn = container.querySelector(
      'button[aria-label="Voir la version 1.0.0"]',
    ) as HTMLButtonElement | null;
    expect(btn).not.toBeNull();
    btn!.click();
    expect(onSelect).toHaveBeenCalledWith(v);
  });

  it("@edge versions vide → message 'Aucune version'", () => {
    const { getByTestId, queryByTestId } = render(
      TechnicalSpecVersionTimeline,
      { props: { versions: [] } },
    );
    expect(getByTestId("tech-spec-timeline-empty")).toBeTruthy();
    expect(queryByTestId("tech-spec-timeline")).toBeNull();
  });

  it("@security pas de bouton Edit/Delete sur les lignes (read-only)", () => {
    const v = spec({ id: "v100", version: "1.0.0", status: "Approved" });
    const { container } = render(TechnicalSpecVersionTimeline, {
      props: { versions: [v], currentVersionId: "v100" },
    });
    const buttons = Array.from(container.querySelectorAll("button"));
    for (const b of buttons) {
      expect(b.textContent ?? "").not.toMatch(
        /modifier|supprimer|delete|edit/i,
      );
    }
  });

  it("@negative status inconnu → fallback badge sans crash", () => {
    // status est typé `string` côté DTO (cf. api.d.ts) — le composant doit
    // gérer un libellé hors enum (défense en profondeur).
    const v = spec({
      id: "weird",
      version: "9.9.9",
      status: "WeirdUnknownStatus",
    });
    const { getByTestId } = render(TechnicalSpecVersionTimeline, {
      props: { versions: [v] },
    });
    const row = getByTestId("tech-spec-version-row-9.9.9");
    expect(row.textContent).toMatch(/WeirdUnknownStatus/);
  });

  it("tri par created_at desc — version la plus récente en tête", () => {
    const old = spec({
      id: "old",
      version: "1.0.0",
      created_at: "2026-01-01T00:00:00Z",
    });
    const recent = spec({
      id: "recent",
      version: "2.0.0",
      created_at: "2026-06-01T00:00:00Z",
    });
    const { getByTestId } = render(TechnicalSpecVersionTimeline, {
      props: { versions: [old, recent] },
    });
    const list = getByTestId("tech-spec-timeline");
    const rows = list.querySelectorAll('[data-testid^="tech-spec-version-row-"]');
    expect(rows[0].getAttribute("data-testid")).toBe(
      "tech-spec-version-row-2.0.0",
    );
    expect(rows[1].getAttribute("data-testid")).toBe(
      "tech-spec-version-row-1.0.0",
    );
  });
});
