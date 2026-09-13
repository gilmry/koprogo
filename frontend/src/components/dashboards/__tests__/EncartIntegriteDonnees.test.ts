import { describe, it, expect, vi } from "vitest";
import { render, screen } from "../../../test-helpers";
import EncartIntegriteDonnees from "../EncartIntegriteDonnees.svelte";

vi.mock("../../../lib/i18n", () => ({
  _: {
    subscribe: (fn: (v: unknown) => void) => {
      fn((cle: string, opts?: { values?: Record<string, unknown> }) =>
        opts?.values ? `${cle}:${JSON.stringify(opts.values)}` : cle,
      );
      return () => {};
    },
  },
}));

/**
 * L'encart d'intégrité des lots.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Deux propriétés, et la seconde compte plus que la première.
 *
 * 1. **Le compte est juste.** Neuf lots manquants, ce sont neuf appels de
 *    fonds mal répartis.
 * 2. **L'encart se tait quand il n'y a rien à dire.** Un bandeau d'alerte
 *    permanent cesse d'être lu au bout de trois jours ; son absence devient
 *    alors l'information utile. Un encart qui crie toujours ne prévient
 *    jamais.
 */

describe("@happy l'encart dit l'écart et sa conséquence", () => {
  it("compte les lots manquants et porte les deux nombres", () => {
    render(EncartIntegriteDonnees, { props: { encodes: 23, declares: 32 } });

    const encart = screen.getByTestId("encart-integrite-lots");
    expect(encart).toHaveAttribute("data-lots-manquants", "9");
    expect(encart).toHaveAttribute("data-lots-encodes", "23");
    expect(encart).toHaveAttribute("data-lots-declares", "32");
  });

  it("propose une action, et elle mène quelque part", () => {
    render(EncartIntegriteDonnees, { props: { encodes: 23, declares: 32 } });

    // Annoncer un problème sans dire quoi en faire laisse le syndic devant un
    // constat. `/units` est la page d'encodage, et elle n'est pas gardée
    // contre le syndic — vérifié dans `guards.ts`.
    expect(screen.getByTestId("encart-integrite-action")).toHaveAttribute(
      "href",
      "/units",
    );
  });
});

describe("@negative l'encart se tait quand il n'y a rien à signaler", () => {
  it("ne rend rien quand tous les lots sont encodés", () => {
    render(EncartIntegriteDonnees, { props: { encodes: 32, declares: 32 } });
    expect(screen.queryByTestId("encart-integrite-lots")).toBeNull();
  });

  it("ne rend rien quand il y a PLUS d'encodés que de déclarés", () => {
    // Cas anormal — il désigne une erreur de déclaration à l'acte de base,
    // pas un défaut d'encodage. Afficher « -3 lots non encodés » serait faux
    // et enverrait le syndic corriger la mauvaise chose.
    render(EncartIntegriteDonnees, { props: { encodes: 35, declares: 32 } });
    expect(screen.queryByTestId("encart-integrite-lots")).toBeNull();
  });
});

describe("@edge les cas limites du décompte", () => {
  it("signale un immeuble entièrement vide", () => {
    // Zéro lot encodé sur trente-deux déclarés : la copropriété existe à
    // l'acte et n'existe pas en base. C'est le pire cas, et il doit crier.
    render(EncartIntegriteDonnees, { props: { encodes: 0, declares: 32 } });
    expect(screen.getByTestId("encart-integrite-lots")).toHaveAttribute(
      "data-lots-manquants",
      "32",
    );
  });

  it("ne rend rien quand rien n'est déclaré", () => {
    // Sans acte de base renseigné, il n'y a pas d'écart à mesurer : on ne
    // peut pas reprocher un manque par rapport à un total inconnu.
    render(EncartIntegriteDonnees, { props: { encodes: 0, declares: 0 } });
    expect(screen.queryByTestId("encart-integrite-lots")).toBeNull();
  });
});
