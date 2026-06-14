// Story B6 (Phase B FE) — Vitest 4-cat SlaBadge.
//
// Couverture (cf. mission §SlaBadge + stories.md §B6 AC) :
//   @happy    syndic répond AVANT sla_due_at → level=met + label
//             "Réponse postée à T-3h ✓" (vert)
//             non répondu + >50% temps restant → level=fresh + vert
//             ≤ 50% → warning (orange)
//             ≤ 25% → urgent (rouge)
//   @edge     sla_due_at - 1s → met ; sla_due_at + 1s → breached
//             ratio bornes 0.50 / 0.25 → passage palette exact
//             pas de createdAt → fallback dueAt - 24h
//   @security INV-FE9 daltoniens — texte + icône + couleur ; aria-label
//             complet ; pas de tooltip-only.
//   @negative dueAt invalide → ne crash pas ; respondedAt absent + dueAt
//             dépassé → breached (pas de fallback silencieux).
//
// nowOverride : injection déterministe — vital (sinon flaky).

import { describe, it, expect } from "vitest";
import { render } from "../../../test-helpers";
import SlaBadge from "./SlaBadge.svelte";
import {
  slaStatus,
  formatResponseDelta,
  type SlaStatus,
} from "../../utils/dateBadge";

// Helper : ISO à N heures de NOW_FIXED.
function plusHours(nowFixed: Date, h: number): string {
  return new Date(nowFixed.getTime() + h * 60 * 60 * 1000).toISOString();
}

// Snapshot fixée — mardi 10 juin 2026 12:00 UTC (cohérent avec ExpirationBadge
// tests pour réviseurs humains).
const NOW_FIXED = new Date("2026-06-10T12:00:00Z");

// =============================================================================
// Pure utils — slaStatus + formatResponseDelta
// =============================================================================

describe("dateBadge.slaStatus — pure helper SLA", () => {
  it("@happy répondu 3h AVANT dueAt → level=met + delta=-3h + label ✓", () => {
    // dueAt = now + 10h, createdAt = now - 14h (fenêtre totale 24h),
    // respondedAt = now + 7h → 3h avant dueAt.
    const dueAt = plusHours(NOW_FIXED, 10);
    const createdAt = plusHours(NOW_FIXED, -14);
    const respondedAt = plusHours(NOW_FIXED, 7);
    const s = slaStatus(dueAt, respondedAt, createdAt, NOW_FIXED);
    expect(s.level).toBe("met");
    expect(s.responseDeltaHours).toBe(-3);
    expect(s.label).toMatch(/T-3h ✓/);
  });

  it("@happy non répondu + 80% temps restant → fresh", () => {
    // createdAt = now - 2h, dueAt = now + 8h → fenêtre 10h, 8h restantes = 80%.
    const createdAt = plusHours(NOW_FIXED, -2);
    const dueAt = plusHours(NOW_FIXED, 8);
    const s = slaStatus(dueAt, null, createdAt, NOW_FIXED);
    expect(s.level).toBe("fresh");
    expect(s.label).toMatch(/Sous SLA/);
    expect(s.remainingRatio).toBeCloseTo(0.8, 1);
  });

  it("@happy non répondu + 40% temps restant → warning", () => {
    // createdAt = now - 6h, dueAt = now + 4h → fenêtre 10h, 4h restantes = 40%.
    const createdAt = plusHours(NOW_FIXED, -6);
    const dueAt = plusHours(NOW_FIXED, 4);
    const s = slaStatus(dueAt, null, createdAt, NOW_FIXED);
    expect(s.level).toBe("warning");
    expect(s.label).toMatch(/Échéance dans/);
  });

  it("@happy non répondu + 10% temps restant → urgent", () => {
    // createdAt = now - 9h, dueAt = now + 1h → fenêtre 10h, 1h restante = 10%.
    const createdAt = plusHours(NOW_FIXED, -9);
    const dueAt = plusHours(NOW_FIXED, 1);
    const s = slaStatus(dueAt, null, createdAt, NOW_FIXED);
    expect(s.level).toBe("urgent");
    expect(s.label).toMatch(/⚠/);
  });

  it("@happy dueAt dépassé sans réponse → breached", () => {
    const dueAt = plusHours(NOW_FIXED, -2);
    const createdAt = plusHours(NOW_FIXED, -10);
    const s = slaStatus(dueAt, null, createdAt, NOW_FIXED);
    expect(s.level).toBe("breached");
    expect(s.label).toMatch(/Hors SLA/);
  });

  it("@edge sla_due_at - 1s (répondu 1s avant) → met", () => {
    // sla_due_at = now + 10h, respondedAt = now + 10h - 1s = avant.
    const dueAt = plusHours(NOW_FIXED, 10);
    const createdAt = plusHours(NOW_FIXED, -14);
    const respondedAt = new Date(
      new Date(dueAt).getTime() - 1000,
    ).toISOString();
    const s = slaStatus(dueAt, respondedAt, createdAt, NOW_FIXED);
    expect(s.level).toBe("met");
  });

  it("@edge sla_due_at + 1s (répondu 1s après) → breached", () => {
    const dueAt = plusHours(NOW_FIXED, 10);
    const createdAt = plusHours(NOW_FIXED, -14);
    const respondedAt = new Date(
      new Date(dueAt).getTime() + 1000,
    ).toISOString();
    const s = slaStatus(dueAt, respondedAt, createdAt, NOW_FIXED);
    expect(s.level).toBe("breached");
  });

  it("@edge createdAt absent → fallback dueAt - 24h", () => {
    // dueAt = now + 6h → fallback createdAt = now - 18h, fenêtre 24h, 6h
    // restantes = 25% → urgent borne sup.
    const dueAt = plusHours(NOW_FIXED, 6);
    const s = slaStatus(dueAt, null, undefined, NOW_FIXED);
    expect(s.remainingRatio).toBeCloseTo(0.25, 2);
    // 25% est dans la borne urgent (≤ 25%).
    expect(s.level).toBe("urgent");
  });

  it("@edge ratio = 0.5 exactement → warning (borne supérieure incluse)", () => {
    // createdAt = now - 5h, dueAt = now + 5h → 50% remaining exact.
    const createdAt = plusHours(NOW_FIXED, -5);
    const dueAt = plusHours(NOW_FIXED, 5);
    const s = slaStatus(dueAt, null, createdAt, NOW_FIXED);
    expect(s.remainingRatio).toBeCloseTo(0.5, 2);
    expect(s.level).toBe("warning");
  });

  it("@edge formatResponseDelta < 1h → T-<1h / T+<1h", () => {
    expect(formatResponseDelta(0)).toBe("T-<1h");
    expect(formatResponseDelta(-3)).toBe("T-3h");
    expect(formatResponseDelta(2)).toBe("T+2h");
  });
});

// =============================================================================
// Composant SlaBadge — rendu Svelte 5
// =============================================================================

describe("SlaBadge — composant Svelte 5 (4-cat)", () => {
  it("@happy répondu AVANT dueAt → badge vert 'Réponse postée à T-3h ✓'", () => {
    const dueAt = plusHours(NOW_FIXED, 10);
    const createdAt = plusHours(NOW_FIXED, -14);
    const respondedAt = plusHours(NOW_FIXED, 7);
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;
    expect(badge).not.toBeNull();
    expect(badge.getAttribute("data-level")).toBe("met");
    expect(badge.className).toMatch(/bg-green-100/);
    expect(badge.textContent).toMatch(/Réponse postée à T-3h/);
    expect(badge.textContent).toMatch(/✓/);
    expect(badge.getAttribute("aria-label")).toMatch(/Réponse postée à T-3h/);
  });

  it("@happy non répondu + >50% temps restant → fresh + couleur verte", () => {
    const createdAt = plusHours(NOW_FIXED, -2);
    const dueAt = plusHours(NOW_FIXED, 8); // 80% remaining
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt: null, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("fresh");
    expect(badge.className).toMatch(/bg-green-100/);
    expect(badge.textContent).toMatch(/Sous SLA/);
  });

  it("@happy <50% → warning orange", () => {
    const createdAt = plusHours(NOW_FIXED, -6);
    const dueAt = plusHours(NOW_FIXED, 4); // 40% remaining
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt: null, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("warning");
    expect(badge.className).toMatch(/bg-orange-100/);
  });

  it("@happy <25% → urgent rouge", () => {
    const createdAt = plusHours(NOW_FIXED, -9);
    const dueAt = plusHours(NOW_FIXED, 1); // 10% remaining
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt: null, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("urgent");
    expect(badge.className).toMatch(/bg-red-100/);
    expect(badge.textContent).toMatch(/⚠/);
  });

  it("@happy dueAt dépassé sans réponse → breached + cross icon", () => {
    const dueAt = plusHours(NOW_FIXED, -2);
    const createdAt = plusHours(NOW_FIXED, -10);
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt: null, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("breached");
    expect(badge.className).toMatch(/bg-red-100/);
    expect(badge.textContent).toMatch(/Hors SLA/);
  });

  it("@edge sla_due_at - 1s → met ; sla_due_at + 1s → breached", () => {
    const dueAt = plusHours(NOW_FIXED, 10);
    const createdAt = plusHours(NOW_FIXED, -14);

    // - 1s
    let respondedAt = new Date(
      new Date(dueAt).getTime() - 1000,
    ).toISOString();
    let { container } = render(SlaBadge, {
      props: { dueAt, respondedAt, createdAt, nowOverride: NOW_FIXED },
    });
    expect(
      container
        .querySelector('[data-testid="ticket-sla-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("met");

    // + 1s
    respondedAt = new Date(new Date(dueAt).getTime() + 1000).toISOString();
    ({ container } = render(SlaBadge, {
      props: { dueAt, respondedAt, createdAt, nowOverride: NOW_FIXED },
    }));
    expect(
      container
        .querySelector('[data-testid="ticket-sla-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("breached");
  });

  it("@security INV-FE9 — texte + icône + couleur pour daltoniens", () => {
    const createdAt = plusHours(NOW_FIXED, -9);
    const dueAt = plusHours(NOW_FIXED, 1);
    const { container } = render(SlaBadge, {
      props: { dueAt, respondedAt: null, createdAt, nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="ticket-sla-badge"]',
    ) as HTMLElement;

    // Canal 1 : couleur
    expect(badge.className).toMatch(/bg-red-100/);
    // Canal 2 : texte explicite (lecteur d'écran via aria-label + texte visible)
    expect(badge.getAttribute("aria-label")).toBeTruthy();
    expect(badge.textContent?.trim().length).toBeGreaterThan(0);
    // Canal 3 : icône SVG
    const svg = badge.querySelector("svg");
    expect(svg).not.toBeNull();
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
  });

  it("@security idSuffix → data-testid composé (ticket-sla-badge-{id})", () => {
    const { container } = render(SlaBadge, {
      props: {
        dueAt: plusHours(NOW_FIXED, 10),
        respondedAt: null,
        createdAt: plusHours(NOW_FIXED, -14),
        nowOverride: NOW_FIXED,
        idSuffix: "ticket-abc-123",
      },
    });
    expect(
      container.querySelector(
        '[data-testid="ticket-sla-badge-ticket-abc-123"]',
      ),
    ).not.toBeNull();
  });

  it("@security tooltip dueTooltip — sr-only span exposé pour debug", () => {
    const { container } = render(SlaBadge, {
      props: {
        dueAt: plusHours(NOW_FIXED, 10),
        respondedAt: null,
        createdAt: plusHours(NOW_FIXED, -14),
        nowOverride: NOW_FIXED,
        idSuffix: "t-42",
        dueTooltip: "Due le 10 juin 2026 à 22h00",
      },
    });
    const tooltip = container.querySelector(
      '[data-testid="ticket-sla-due-tooltip-t-42"]',
    ) as HTMLElement;
    expect(tooltip).not.toBeNull();
    expect(tooltip.textContent).toMatch(/Due le/);
    expect(tooltip.className).toMatch(/sr-only/);
  });

  it("@negative dueAt invalide (string mal formé) → ne crash pas", () => {
    expect(() => {
      render(SlaBadge, {
        props: {
          dueAt: "not-a-date",
          respondedAt: null,
          nowOverride: NOW_FIXED,
        },
      });
    }).not.toThrow();
  });
});
