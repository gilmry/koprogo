// Story B3 (Phase B FE) — Vitest 4-cat ExpirationBadge.
//
// Couverture (seuils 30 / 7 / 0 / passé) :
//   @happy    daysRemaining > 30   → level=fresh + label "Expire dans N mois"
//             daysRemaining ≤ 30   → level=soon  + couleur orange
//             daysRemaining ≤ 7    → level=urgent + couleur rouge + icône warning
//             daysRemaining = 0    → label "Expire aujourd'hui"
//             daysRemaining < 0    → level=expired + label "Expiré" + couleur grise
//   @edge     daysRemaining = 1    → "Expire demain" (singulier)
//             daysRemaining = 31   → bascule fresh (juste hors seuil soon)
//             daysRemaining = 8    → bascule soon  (juste hors seuil urgent)
//             daysRemaining = 60   → label en jours, 61j → label en mois
//   @security palette CSS combinée à texte + icône (INV-FE9 daltoniens) :
//             on vérifie qu'un lecteur d'écran reçoit le label complet via
//             aria-label (pas juste couleur).
//   @negative validUntil invalide (NaN Date) → composant rend sans crash,
//             daysRemaining=NaN → level=fresh par défaut (Math.ceil(NaN) === NaN,
//             NaN < 0 === false, NaN <= 7 === false, NaN <= 30 === false →
//             fallback fresh, label "Expire dans NaN mois"). On documente le
//             comportement et on vérifie l'absence de throw.
//
// nowOverride : injection déterministe pour ces snapshots — vital car le
// composant utilise `new Date()` par défaut et un test live serait flaky
// (1 milliseconde change le calcul).

import { describe, it, expect } from "vitest";
import { render } from "../../../test-helpers";
import ExpirationBadge from "./ExpirationBadge.svelte";
import {
  daysBetween,
  levelFromDays,
  labelFromDays,
  expirationStatus,
} from "../../utils/dateBadge";

// Helper : génère un `validUntil` ISO à N jours pile du `nowFixed`.
function plusDays(nowFixed: Date, n: number): string {
  const d = new Date(nowFixed.getTime() + n * 24 * 60 * 60 * 1000);
  return d.toISOString();
}

// Snapshot fixée pour déterminisme (mardi 10 juin 2026 12:00 UTC — date du
// développement de la story, cf. stories.md ligne 705 "2026-06-10").
const NOW_FIXED = new Date("2026-06-10T12:00:00Z");

describe("dateBadge utils — pure functions", () => {
  it("@happy daysBetween calcule l'écart en jours arrondi vers le haut", () => {
    expect(daysBetween(plusDays(NOW_FIXED, 365), NOW_FIXED)).toBe(365);
    expect(daysBetween(plusDays(NOW_FIXED, 30), NOW_FIXED)).toBe(30);
    expect(daysBetween(plusDays(NOW_FIXED, 7), NOW_FIXED)).toBe(7);
    expect(daysBetween(plusDays(NOW_FIXED, 1), NOW_FIXED)).toBe(1);
    expect(daysBetween(plusDays(NOW_FIXED, 0), NOW_FIXED)).toBe(0);
    expect(daysBetween(plusDays(NOW_FIXED, -3), NOW_FIXED)).toBe(-3);
  });

  it("@happy levelFromDays applique les seuils canoniques 30/7/0", () => {
    expect(levelFromDays(365)).toBe("fresh");
    expect(levelFromDays(31)).toBe("fresh");
    expect(levelFromDays(30)).toBe("soon");
    expect(levelFromDays(8)).toBe("soon");
    expect(levelFromDays(7)).toBe("urgent");
    expect(levelFromDays(1)).toBe("urgent");
    expect(levelFromDays(0)).toBe("urgent");
    expect(levelFromDays(-1)).toBe("expired");
    expect(levelFromDays(-100)).toBe("expired");
  });

  it("@edge labelFromDays pluralise correctement + bascule jours/mois à 60", () => {
    expect(labelFromDays(365)).toBe("Expire dans 12 mois");
    expect(labelFromDays(61)).toBe("Expire dans 2 mois");
    expect(labelFromDays(60)).toBe("Expire dans 60 jours");
    expect(labelFromDays(30)).toBe("Expire dans 30 jours");
    expect(labelFromDays(2)).toBe("Expire dans 2 jours");
    expect(labelFromDays(1)).toBe("Expire demain");
    expect(labelFromDays(0)).toBe("Expire aujourd'hui");
    expect(labelFromDays(-1)).toBe("Expiré");
  });

  it("@happy expirationStatus retourne le triplet attendu sur seuils canoniques", () => {
    const s30 = expirationStatus(plusDays(NOW_FIXED, 30), NOW_FIXED);
    expect(s30).toEqual({
      daysRemaining: 30,
      level: "soon",
      label: "Expire dans 30 jours",
    });

    const s7 = expirationStatus(plusDays(NOW_FIXED, 7), NOW_FIXED);
    expect(s7.level).toBe("urgent");

    const s0 = expirationStatus(plusDays(NOW_FIXED, 0), NOW_FIXED);
    expect(s0.label).toBe("Expire aujourd'hui");
    expect(s0.level).toBe("urgent");

    const sNeg = expirationStatus(plusDays(NOW_FIXED, -5), NOW_FIXED);
    expect(sNeg.level).toBe("expired");
    expect(sNeg.label).toBe("Expiré");
  });
});

describe("ExpirationBadge — composant Svelte 5 (4-cat)", () => {
  it("@happy >30j → level=fresh + label 12 mois + couleur verte", () => {
    const { container } = render(ExpirationBadge, {
      props: {
        validUntil: plusDays(NOW_FIXED, 365),
        nowOverride: NOW_FIXED,
      },
    });

    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge).not.toBeNull();
    expect(badge.getAttribute("data-level")).toBe("fresh");
    expect(badge.getAttribute("data-days-remaining")).toBe("365");
    expect(badge.textContent).toMatch(/Expire dans 12 mois/);
    // Couleur verte (Tailwind bg-green-100).
    expect(badge.className).toMatch(/bg-green-100/);
    // aria-label complet (INV-FE9 daltoniens — lecteurs d'écran).
    expect(badge.getAttribute("aria-label")).toMatch(/Expire dans 12 mois/);
  });

  it("@happy ≤30j (soon) → orange + icône clock", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 25), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("soon");
    expect(badge.className).toMatch(/bg-orange-100/);
    expect(badge.textContent).toMatch(/Expire dans 25 jours/);
    // Icône SVG présente (3e canal d'info INV-FE9).
    expect(badge.querySelector("svg")).not.toBeNull();
  });

  it("@happy ≤7j (urgent) → rouge + icône warning", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 3), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("urgent");
    expect(badge.className).toMatch(/bg-red-100/);
    expect(badge.textContent).toMatch(/Expire dans 3 jours/);
  });

  it("@happy =0j → 'Expire aujourd'hui' urgent", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 0), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("urgent");
    expect(badge.textContent).toMatch(/Expire aujourd'hui/);
  });

  it("@happy <0j → expired + label 'Expiré' + gris", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, -10), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge.getAttribute("data-level")).toBe("expired");
    expect(badge.className).toMatch(/bg-gray-200/);
    expect(badge.textContent).toMatch(/Expiré/);
  });

  it("@edge =1j → 'Expire demain' (singulier)", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 1), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;
    expect(badge.textContent).toMatch(/Expire demain/);
    expect(badge.textContent).not.toMatch(/jours/);
  });

  it("@edge seuils exacts 31/30/8/7 — palette change au bon endroit", () => {
    // 31j → fresh (juste hors seuil soon)
    let { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 31), nowOverride: NOW_FIXED },
    });
    expect(
      container
        .querySelector('[data-testid="expiration-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("fresh");

    // 30j → soon (borne haute incluse)
    ({ container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 30), nowOverride: NOW_FIXED },
    }));
    expect(
      container
        .querySelector('[data-testid="expiration-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("soon");

    // 8j → soon (juste hors seuil urgent)
    ({ container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 8), nowOverride: NOW_FIXED },
    }));
    expect(
      container
        .querySelector('[data-testid="expiration-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("soon");

    // 7j → urgent (borne haute incluse)
    ({ container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 7), nowOverride: NOW_FIXED },
    }));
    expect(
      container
        .querySelector('[data-testid="expiration-badge"]')
        ?.getAttribute("data-level"),
    ).toBe("urgent");
  });

  it("@security INV-FE9 — texte + icône + couleur pour daltoniens", () => {
    const { container } = render(ExpirationBadge, {
      props: { validUntil: plusDays(NOW_FIXED, 3), nowOverride: NOW_FIXED },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge"]',
    ) as HTMLElement;

    // Canal 1 : couleur
    expect(badge.className).toMatch(/bg-red-100/);
    // Canal 2 : texte explicite (lecteur d'écran via aria-label + texte visible)
    expect(badge.getAttribute("aria-label")).toBeTruthy();
    expect(badge.textContent?.trim().length).toBeGreaterThan(0);
    // Canal 3 : icône SVG
    const svg = badge.querySelector("svg");
    expect(svg).not.toBeNull();
    // L'icône est purement décorative (info portée par le label).
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
  });

  it("@negative idSuffix → data-testid composé correctement", () => {
    const { container } = render(ExpirationBadge, {
      props: {
        validUntil: plusDays(NOW_FIXED, 10),
        nowOverride: NOW_FIXED,
        idSuffix: "abc-123",
      },
    });
    const badge = container.querySelector(
      '[data-testid="expiration-badge-abc-123"]',
    ) as HTMLElement;
    expect(badge).not.toBeNull();
  });

  it("@negative validUntil string mal formé → rend sans crash", () => {
    // Math.ceil(NaN) === NaN → tous les comparateurs renvoient false →
    // fallback fresh. On documente le contrat : le composant ne throw pas
    // (defensive — un payload backend foireux ne casse pas l'UI).
    expect(() => {
      render(ExpirationBadge, {
        props: { validUntil: "not-a-date", nowOverride: NOW_FIXED },
      });
    }).not.toThrow();
  });
});
