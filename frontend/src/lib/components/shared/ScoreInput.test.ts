// Story B8 (Phase B FE) — Vitest 4-cat ScoreInput atomique.
//
// Couverture (cf. stories.md §B8 ScoreInput data-testid + AC bornes 1-5) :
//   @happy    rendu 5 radios (1..5) avec data-testid `score-input-{n}` ou
//             prefixé `{prefix}-score-input-{n}` ; click → onChange(n).
//   @edge     value=null → aucun radio coché ; value=3 → radio 3 checked ;
//             bornes exhaustives 1, 2, 3, 4, 5.
//   @security INV-FE9 WCAG 2.1 AA — fieldset+legend ; role="radiogroup" ;
//             aria-label + aria-required.
//   @negative pas de radio "0" ni "6" rendus (bornes natives — pas saisissable
//             via UI).

import { describe, it, expect, vi } from "vitest";
import { render } from "../../../test-helpers";
import ScoreInput from "./ScoreInput.svelte";

describe("ScoreInput — Story B8 (4-cat)", () => {
  it("@happy rendu 5 radios (1..5) + data-testid score-input-{n}", () => {
    const onChange = vi.fn();
    const { container } = render(ScoreInput, {
      props: {
        name: "quality",
        label: "Qualité",
        value: null,
        onChange,
      },
    });
    // Bornes 1..5 : 5 radios exactement.
    for (let s = 1; s <= 5; s += 1) {
      const radio = container.querySelector(
        `[data-testid="score-input-${s}"]`,
      ) as HTMLInputElement | null;
      expect(radio, `radio score-input-${s} doit exister`).not.toBeNull();
      expect(radio?.type).toBe("radio");
      expect(radio?.checked).toBe(false);
    }
  });

  it("@happy click radio 4 → onChange(4) appelé", () => {
    const onChange = vi.fn();
    const { container } = render(ScoreInput, {
      props: {
        name: "timeliness",
        label: "Délais",
        value: null,
        onChange,
      },
    });
    const radio4 = container.querySelector(
      '[data-testid="score-input-4"]',
    ) as HTMLInputElement;
    radio4.click();
    radio4.dispatchEvent(new Event("change", { bubbles: true }));
    expect(onChange).toHaveBeenCalledWith(4);
  });

  it("@happy testIdPrefix → data-testid composé (quality-score-input-{n})", () => {
    const { container } = render(ScoreInput, {
      props: {
        name: "quality",
        label: "Qualité",
        value: null,
        onChange: vi.fn(),
        testIdPrefix: "quality",
      },
    });
    for (let s = 1; s <= 5; s += 1) {
      expect(
        container.querySelector(`[data-testid="quality-score-input-${s}"]`),
        `prefix quality-score-input-${s}`,
      ).not.toBeNull();
    }
    // Sans prefix, les ids `score-input-{n}` ne doivent PAS exister (sinon
    // collision multi-instances dans un même DOM).
    expect(container.querySelector('[data-testid="score-input-3"]')).toBeNull();
  });

  it("@edge value=3 → radio 3 checked, autres non", () => {
    const { container } = render(ScoreInput, {
      props: {
        name: "communication",
        label: "Communication",
        value: 3,
        onChange: vi.fn(),
      },
    });
    for (let s = 1; s <= 5; s += 1) {
      const radio = container.querySelector(
        `[data-testid="score-input-${s}"]`,
      ) as HTMLInputElement;
      expect(radio.checked).toBe(s === 3);
    }
  });

  it("@edge value=null → aucun radio coché", () => {
    const { container } = render(ScoreInput, {
      props: {
        name: "overall",
        label: "Note globale",
        value: null,
        onChange: vi.fn(),
      },
    });
    for (let s = 1; s <= 5; s += 1) {
      const radio = container.querySelector(
        `[data-testid="score-input-${s}"]`,
      ) as HTMLInputElement;
      expect(radio.checked).toBe(false);
    }
  });

  it("@security INV-FE9 — fieldset + legend + role=radiogroup + aria-label", () => {
    const { container } = render(ScoreInput, {
      props: {
        name: "quality",
        label: "Qualité technique",
        value: null,
        onChange: vi.fn(),
        required: true,
      },
    });
    // Fieldset wrapper sémantique.
    const fs = container.querySelector("fieldset") as HTMLFieldSetElement;
    expect(fs).not.toBeNull();

    // Legend visible (label).
    const legend = container.querySelector("legend");
    expect(legend?.textContent).toMatch(/Qualité technique/);

    // Radiogroup ARIA + aria-required (porté par le div role=radiogroup, pas
    // le fieldset — éviter clash "aria-required pas supporté par role=group").
    const group = container.querySelector('[role="radiogroup"]');
    expect(group).not.toBeNull();
    expect(group?.getAttribute("aria-label")).toBe("Qualité technique");
    expect(group?.getAttribute("aria-required")).toBe("true");

    // Chaque radio a aria-label complet (lecteur d'écran sait "Qualité : 3 sur 5").
    const radio3 = container.querySelector(
      '[data-testid="score-input-3"]',
    ) as HTMLInputElement;
    expect(radio3.getAttribute("aria-label")).toMatch(/3 sur 5/);
  });

  it("@security disabled=true → tous les radios disabled (pas d'event possible)", () => {
    const onChange = vi.fn();
    const { container } = render(ScoreInput, {
      props: {
        name: "quality",
        label: "Qualité",
        value: 2,
        onChange,
        disabled: true,
      },
    });
    for (let s = 1; s <= 5; s += 1) {
      const radio = container.querySelector(
        `[data-testid="score-input-${s}"]`,
      ) as HTMLInputElement;
      expect(radio.disabled).toBe(true);
    }
    // Click ignoré (disabled — browser ne fire pas change).
    const radio4 = container.querySelector(
      '[data-testid="score-input-4"]',
    ) as HTMLInputElement;
    radio4.click();
    expect(onChange).not.toHaveBeenCalled();
  });

  it("@negative pas de radio score-input-0 ni score-input-6 (bornes natives)", () => {
    const { container } = render(ScoreInput, {
      props: {
        name: "overall",
        label: "Note globale",
        value: null,
        onChange: vi.fn(),
      },
    });
    expect(
      container.querySelector('[data-testid="score-input-0"]'),
    ).toBeNull();
    expect(
      container.querySelector('[data-testid="score-input-6"]'),
    ).toBeNull();
    // Total = 5 radios exactement.
    const radios = container.querySelectorAll('input[type="radio"]');
    expect(radios.length).toBe(5);
  });
});
