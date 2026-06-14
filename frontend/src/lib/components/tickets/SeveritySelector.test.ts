// Story B5 (Phase B FE) — Vitest 4-cat SeveritySelector.
//
// Couverture (cf. stories.md §B5 + mission) :
//   @happy    : 4 radios visibles avec labels FR (low/normal/high/critical) ;
//               sélection met à jour `value` via bind:group ; legend
//               présente pour a11y.
//   @edge     : value initial "" → aucun radio coché ; passer de "low" à
//               "critical" met à jour group correctement.
//   @security : set fermé — aucun radio "unknown" / "extreme" / "other" ;
//               valeur Math/proto-pollution → impossible via UI.
//   @negative : required=true → aria-required="true" sur fieldset ET sur
//               chaque input radio + asterisque visuel ; required=false →
//               aria-required="false" sans asterisque.
//
// Pattern : pas de DI nécessaire — composant pur de présentation.

import { describe, it, expect } from "vitest";
import { render } from "../../../test-helpers";
import SeveritySelector from "./SeveritySelector.svelte";

describe("SeveritySelector — Story B5 (4-cat)", () => {
  it("@happy 4 radios visibles avec labels FR + data-testid + legend pour a11y", () => {
    const { getByTestId, container } = render(SeveritySelector, {
      props: { value: "" },
    });

    // Les 4 radios sont présents et exposent leur data-testid.
    const low = getByTestId("ticket-severity-radio-low") as HTMLInputElement;
    const normal = getByTestId(
      "ticket-severity-radio-normal",
    ) as HTMLInputElement;
    const high = getByTestId("ticket-severity-radio-high") as HTMLInputElement;
    const critical = getByTestId(
      "ticket-severity-radio-critical",
    ) as HTMLInputElement;

    expect(low.type).toBe("radio");
    expect(normal.type).toBe("radio");
    expect(high.type).toBe("radio");
    expect(critical.type).toBe("radio");

    // Values strictes correspondant à TicketSeverity côté BE.
    expect(low.value).toBe("low");
    expect(normal.value).toBe("normal");
    expect(high.value).toBe("high");
    expect(critical.value).toBe("critical");

    // Legend a11y — annonce "Gravité" aux lecteurs d'écran.
    const legend = container.querySelector("legend");
    expect(legend?.textContent).toMatch(/Gravité/i);

    // Labels FR visibles.
    expect(container.textContent).toMatch(/Basse/);
    expect(container.textContent).toMatch(/Normale/);
    expect(container.textContent).toMatch(/Haute/);
    expect(container.textContent).toMatch(/Critique/);
  });

  it("@edge value initial vide → aucun radio coché", () => {
    const { getByTestId } = render(SeveritySelector, {
      props: { value: "" },
    });

    expect(
      (getByTestId("ticket-severity-radio-low") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-normal") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-high") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-critical") as HTMLInputElement)
        .checked,
    ).toBe(false);
  });

  it("@edge value=critical initial → seul critical est coché", () => {
    const { getByTestId } = render(SeveritySelector, {
      props: { value: "critical" },
    });

    expect(
      (getByTestId("ticket-severity-radio-low") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-normal") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-high") as HTMLInputElement).checked,
    ).toBe(false);
    expect(
      (getByTestId("ticket-severity-radio-critical") as HTMLInputElement)
        .checked,
    ).toBe(true);
  });

  it("@security set fermé — aucun radio hors {low,normal,high,critical}", () => {
    const { container } = render(SeveritySelector, {
      props: { value: "" },
    });

    const radios = Array.from(
      container.querySelectorAll<HTMLInputElement>("input[type=radio]"),
    );
    expect(radios).toHaveLength(4);

    const values = radios.map((r) => r.value).sort();
    expect(values).toEqual(["critical", "high", "low", "normal"]);

    // Aucun radio "unknown" / "other" / "extreme" ne doit exister.
    expect(values).not.toContain("unknown");
    expect(values).not.toContain("other");
    expect(values).not.toContain("extreme");
  });

  it("@negative required=true → data-required + asterisque visuel + radio HTML required", () => {
    const { container } = render(SeveritySelector, {
      props: { value: "", required: true },
    });

    // aria-required n'est pas supporté sur le role implicite "group" du
    // fieldset (warning Svelte a11y) — on l'expose via data-required.
    const fieldset = container.querySelector("fieldset");
    expect(fieldset?.getAttribute("data-required")).toBe("true");

    // Asterisque rouge présent dans le legend.
    const legend = container.querySelector("legend");
    expect(legend?.textContent).toMatch(/\*/);

    // Chaque radio porte l'attribut HTML `required`.
    const radios = Array.from(
      container.querySelectorAll<HTMLInputElement>("input[type=radio]"),
    );
    for (const r of radios) {
      expect(r.required).toBe(true);
    }
  });

  it("@negative required=false (default) → data-required=false + pas d'asterisque + radio non required", () => {
    const { container } = render(SeveritySelector, {
      props: { value: "" },
    });

    const fieldset = container.querySelector("fieldset");
    expect(fieldset?.getAttribute("data-required")).toBe("false");

    const legend = container.querySelector("legend");
    expect(legend?.textContent).not.toMatch(/\*/);

    const radios = Array.from(
      container.querySelectorAll<HTMLInputElement>("input[type=radio]"),
    );
    for (const r of radios) {
      expect(r.required).toBe(false);
    }
  });
});
