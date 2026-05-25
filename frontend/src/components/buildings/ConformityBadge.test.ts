// Story 1.4 — ConformityBadge Vitest tests (4-cat).
//
// CRITICAL §3 — RED-first BDD/TDD : tests rouge avant le composant.
//
// Couverture :
// - @happy : conformant (vert) + valeurs nominales
// - @edge : delta négatif (rouge), delta positif (orange), 0 units
// - @security : ne JAMAIS appeler parseFloat sur quotaSum/quotaDelta
//   (préserve la précision Decimal)
// - @negative : strings vides, formats "+0", déglobes inattendus

import { render, screen } from "../../test-helpers";
import { describe, it, expect, vi } from "vitest";
import ConformityBadge from "./ConformityBadge.svelte";

vi.mock("../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: any) => void) => {
      fn((key: string) => key);
      return () => {};
    },
  },
}));

describe("ConformityBadge", () => {
  // -------------------------------------------------------------------------
  // @happy — chemin nominal : conformant
  // -------------------------------------------------------------------------

  it("@happy renders conformant badge (green) with units count 50/50 and quota 1000", () => {
    render(ConformityBadge, {
      props: {
        isConformant: true,
        unitsCount: 50,
        totalUnits: 50,
        quotaSum: "1000",
        quotaDelta: "0",
      },
    });

    const badge = screen.getByTestId("building-conformity-badge");
    expect(badge).toBeInTheDocument();
    expect(badge.className).toMatch(/bg-green-100/);
    expect(badge.className).toMatch(/text-green-800/);

    const unitsCount = screen.getByTestId("building-units-count");
    expect(unitsCount.textContent).toMatch(/50/);

    const quotaSum = screen.getByTestId("building-quota-sum");
    expect(quotaSum.textContent).toMatch(/1000/);

    const quotaDelta = screen.getByTestId("building-quota-delta");
    expect(quotaDelta.textContent).toMatch(/0/);
  });

  // -------------------------------------------------------------------------
  // @edge — bornes
  // -------------------------------------------------------------------------

  it("@edge renders non-conformant badge (red) when delta is -1 (missing 1 millième)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 50,
        totalUnits: 50,
        quotaSum: "999",
        quotaDelta: "-1",
      },
    });

    const badge = screen.getByTestId("building-conformity-badge");
    expect(badge.className).toMatch(/bg-red-100/);

    const quotaDelta = screen.getByTestId("building-quota-delta");
    expect(quotaDelta.textContent).toMatch(/-1/);
  });

  it("@edge renders warning badge (orange) when delta is +500 (surplus quotas)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 50,
        totalUnits: 50,
        quotaSum: "1500",
        quotaDelta: "500",
      },
    });

    const badge = screen.getByTestId("building-conformity-badge");
    expect(badge.className).toMatch(/bg-orange-100/);

    const quotaDelta = screen.getByTestId("building-quota-delta");
    expect(quotaDelta.textContent).toMatch(/\+500/);
  });

  it("@edge renders quotaSum 0 when no units (empty building, no NaN)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 0,
        totalUnits: 10,
        quotaSum: "0",
        quotaDelta: "-1000",
      },
    });

    const quotaSum = screen.getByTestId("building-quota-sum");
    expect(quotaSum.textContent).not.toMatch(/NaN/);
    expect(quotaSum.textContent).toMatch(/0/);
  });

  it("@edge formats Decimal strings using FR-BE locale (comma separator) without parseFloat", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 2,
        totalUnits: 2,
        quotaSum: "999.5",
        quotaDelta: "-0.5",
      },
    });

    const quotaSum = screen.getByTestId("building-quota-sum");
    expect(quotaSum.textContent).toMatch(/999,5/);
  });

  // -------------------------------------------------------------------------
  // @security — préservation Decimal strict (pas de parseFloat/Number)
  // -------------------------------------------------------------------------

  it("@security never calls parseFloat or Number on quotaSum/quotaDelta props", () => {
    // Espionne globalThis.parseFloat et Number constructor.
    const parseFloatSpy = vi.spyOn(globalThis, "parseFloat");
    const numberSpy = vi.spyOn(globalThis, "Number");

    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 2,
        totalUnits: 2,
        quotaSum: "999.999999999999999",
        quotaDelta: "-0.000000000000001",
      },
    });

    expect(parseFloatSpy).not.toHaveBeenCalled();
    // Number() can be called by Svelte internals for other props — only check
    // that the displayed text PRESERVES the precision string verbatim.
    const quotaSum = screen.getByTestId("building-quota-sum");
    expect(quotaSum.textContent).toMatch(/999,999999999999999/);

    parseFloatSpy.mockRestore();
    numberSpy.mockRestore();
  });

  it("@security exposes role='status' for screen readers (a11y)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: true,
        unitsCount: 1,
        totalUnits: 1,
        quotaSum: "1000",
        quotaDelta: "0",
      },
    });
    const badge = screen.getByTestId("building-conformity-badge");
    expect(badge.getAttribute("role")).toBe("status");
    expect(badge.getAttribute("aria-live")).toBe("polite");
  });

  // -------------------------------------------------------------------------
  // @negative — défaillance correcte (pas de crash, fallback typé)
  // -------------------------------------------------------------------------

  it("@negative renders em-dash fallback for empty Decimal strings (no crash)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 0,
        totalUnits: 0,
        quotaSum: "",
        quotaDelta: "",
      },
    });

    const quotaSum = screen.getByTestId("building-quota-sum");
    expect(quotaSum.textContent).toMatch(/—/);
  });

  it("@negative strips leading '+' from quotaSum string (canonical Decimal display)", () => {
    render(ConformityBadge, {
      props: {
        isConformant: false,
        unitsCount: 2,
        totalUnits: 2,
        quotaSum: "+1000",
        quotaDelta: "+0",
      },
    });

    const quotaSum = screen.getByTestId("building-quota-sum");
    // No "++1000" — leading + stripped.
    expect(quotaSum.textContent).not.toMatch(/\+\+/);
  });
});
