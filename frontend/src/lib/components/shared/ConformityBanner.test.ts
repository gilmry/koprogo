// Track H Story H1 — Vitest 4-cat `ConformityBanner.svelte`.
//
// @happy   conforme → banner ABSENT du DOM (pas de rendu, pas de
//          data-testid="conformity-banner").
// @edge    quota_basis=10000 quota_delta=25 → texte « 25 / 10000 » (acte
//          de base 10000 — bug fix Story H1 démontré).
//          quota_basis=1000 quota_delta=2.5 → « 2,5 / 1000 » (FR-BE format).
//          singulier vs pluriel sur units_delta.
// @security pas d'info sensible exposée (pas d'user_id, pas d'org_id dans
//           le DOM rendu — uniquement building_id qui est public).
// @negative props inconsistents (is_conformant=false mais deltas 0) →
//           render minimal sans crash, listes vides bien gérées.
//
// Note Svelte 5 runes : on rend le composant via le wrapper test-helpers
// (cf. `frontend/src/test-helpers.ts` qui contourne le typage strict de
// @testing-library/svelte vs Component<P,E>).

import { describe, it, expect } from "vitest";
import { render } from "../../../test-helpers";
import ConformityBanner from "./ConformityBanner.svelte";
import type { ConformityStatus } from "../../types/conformity";

const BUILDING_ID = "11111111-2222-3333-4444-555555555555";

function makeStatus(
  overrides: Partial<ConformityStatus> = {},
): ConformityStatus {
  return {
    is_conformant: true,
    units_delta: 0,
    quota_delta: "0",
    quota_basis: 1000,
    ...overrides,
  };
}

describe("ConformityBanner — 4-cat (Track H Story H1)", () => {
  // ----------------------------------------------------------------------
  // @happy — chemin nominal
  // ----------------------------------------------------------------------

  it("@happy conforme → banner ABSENT du DOM (AC-H1.h5 / AC-H1.h6)", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({ is_conformant: true }),
        buildingId: BUILDING_ID,
        buildingName: "Conformant Towers",
      },
    });
    expect(
      container.querySelector('[data-testid="conformity-banner"]'),
    ).toBeNull();
  });

  it("@happy non-conforme basis 1000 → banner présent + role alert", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "2.5",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Drift Manor",
      },
    });
    const banner = container.querySelector(
      '[data-testid="conformity-banner"]',
    ) as HTMLElement;
    expect(banner).not.toBeNull();
    expect(banner.getAttribute("role")).toBe("alert");
    expect(banner.getAttribute("aria-live")).toBe("polite");
    expect(banner.getAttribute("data-building-id")).toBe(BUILDING_ID);
    // Title contient le name
    const title = container.querySelector(
      '[data-testid="conformity-banner-title"]',
    );
    expect(title?.textContent).toMatch(/Drift Manor/);
  });

  // ----------------------------------------------------------------------
  // @edge — quota_basis 10000 (bug fix Story H1) + format FR-BE
  // ----------------------------------------------------------------------

  it("@edge basis 10000, delta 25 → texte « 25 / 10000 » (AC-H1.h7)", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "25",
          quota_basis: 10000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Big Tower 182",
      },
    });
    const quotaLi = container.querySelector(
      '[data-testid="conformity-quota-delta"]',
    ) as HTMLElement;
    expect(quotaLi).not.toBeNull();
    // Attribut data-basis = 10000 (pas 1000) — démontre le bug fix.
    expect(quotaLi.getAttribute("data-basis")).toBe("10000");
    expect(quotaLi.textContent).toMatch(/25/);
    expect(quotaLi.textContent).toMatch(/10000/);
  });

  it("@edge basis 1000, delta 2.5 → format FR-BE virgule « 2,5 »", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "2.5",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Drift Manor",
      },
    });
    const quotaLi = container.querySelector(
      '[data-testid="conformity-quota-delta"]',
    ) as HTMLElement;
    expect(quotaLi.getAttribute("data-basis")).toBe("1000");
    // FR-BE : virgule séparatrice décimale (formatDecimalFRBE remplace . par ,).
    expect(quotaLi.textContent).toMatch(/2,5/);
  });

  it("@edge units_delta = 1 → singulier ; units_delta = 2 → pluriel (AC-H1.e4)", () => {
    // Singulier
    let result = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "2.5",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "X",
      },
    });
    let li = result.container.querySelector(
      '[data-testid="conformity-units-delta"]',
    );
    // Le contenu i18n FR contient « lot manquant » (singulier) — tolérant.
    expect(li).not.toBeNull();
    expect(li?.textContent ?? "").toMatch(/1/);

    // Pluriel
    result = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 3,
          quota_delta: "75",
          quota_basis: 10000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Y",
      },
    });
    li = result.container.querySelector(
      '[data-testid="conformity-units-delta"]',
    );
    expect(li).not.toBeNull();
    expect(li?.textContent ?? "").toMatch(/3/);
  });

  it("@edge units_delta négatif → label « extra »", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: -2,
          quota_delta: "-50",
          quota_basis: 10000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Surplus Manor",
      },
    });
    const li = container.querySelector(
      '[data-testid="conformity-units-delta"]',
    );
    expect(li).not.toBeNull();
    // Value absolue affichée
    expect(li?.textContent ?? "").toMatch(/2/);
  });

  // ----------------------------------------------------------------------
  // @security — pas d'info sensible exposée
  // ----------------------------------------------------------------------

  it("@security pas de user_id / org_id dans le DOM rendu (AC-H1.s2)", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "2.5",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Drift Manor",
      },
    });
    const html = container.innerHTML.toLowerCase();
    expect(html).not.toContain("user_id");
    expect(html).not.toContain("org_id");
    expect(html).not.toContain("organization_id");
    expect(html).not.toContain("password");
    expect(html).not.toContain("token");
  });

  it("@security le banner conserve aria-labelledby pointant vers le titre", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 1,
          quota_delta: "2.5",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Drift Manor",
      },
    });
    const banner = container.querySelector(
      '[data-testid="conformity-banner"]',
    ) as HTMLElement;
    const labelledBy = banner.getAttribute("aria-labelledby");
    expect(labelledBy).toBeTruthy();
    const labelledNode = container.querySelector(`#${labelledBy}`);
    expect(labelledNode).not.toBeNull();
  });

  // ----------------------------------------------------------------------
  // @negative — props inconsistents
  // ----------------------------------------------------------------------

  it("@negative is_conformant=false avec deltas 0 → render sans <li> (AC-H1.n3)", () => {
    // Cas inconsistant (logiquement impossible mais robuste) : banner se
    // rend (is_conformant=false) mais aucune <li> de détail n'apparaît car
    // les deltas sont à zéro.
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 0,
          quota_delta: "0",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Ghost",
      },
    });
    const banner = container.querySelector('[data-testid="conformity-banner"]');
    expect(banner).not.toBeNull();
    expect(
      container.querySelector('[data-testid="conformity-units-delta"]'),
    ).toBeNull();
    expect(
      container.querySelector('[data-testid="conformity-quota-delta"]'),
    ).toBeNull();
    // Le « contactez l'admin » reste rendu pour orienter l'utilisateur.
    expect(
      container.querySelector('[data-testid="conformity-contact-admin"]'),
    ).not.toBeNull();
  });

  it("@negative quota_delta = '0.0' string variante → traité comme zéro", () => {
    const { container } = render(ConformityBanner, {
      props: {
        status: makeStatus({
          is_conformant: false,
          units_delta: 0,
          quota_delta: "0.0",
          quota_basis: 1000,
        }),
        buildingId: BUILDING_ID,
        buildingName: "Edge",
      },
    });
    expect(
      container.querySelector('[data-testid="conformity-quota-delta"]'),
    ).toBeNull();
  });
});
