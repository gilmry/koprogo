import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen } from "../../../test-helpers";
import DecompteLegal from "../DecompteLegal.svelte";

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
 * Le décompte d'échéance légale.
 *
 * ── Ce que ces tests tiennent ────────────────────────────────────────────
 *
 * Le composant affiche un nombre de jours à un syndic qui va agir dessus. Une
 * erreur d'un jour n'est pas cosmétique : à trente jours d'un procès-verbal
 * (Art. 3.87 § 12), afficher « 1 » quand il reste « 0 » fait manquer
 * l'échéance.
 *
 * Trois propriétés que la lecture du code ne suffit pas à garantir :
 *
 * 1. **L'arrondi va vers le bas.** À 23 h 59 il reste zéro jour, pas un.
 * 2. **Le dépassement se dit, il ne se déduit pas d'une couleur.** Un
 *    manquement constaté et une urgence à venir n'appellent pas la même
 *    action.
 * 3. **La barre ne déborde pas.** Au-delà de l'échéance elle est pleine ; un
 *    dépassement se dit par le mot, pas par une piste qui sort de son cadre.
 */

/** Le temps est figé : un décompte qui dépend de l'heure du test est un piège. */
const MAINTENANT = new Date("2026-09-10T12:00:00Z");

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(MAINTENANT);
});

afterEach(() => {
  vi.useRealTimers();
});

function dansNJours(n: number, heures = 0): string {
  const d = new Date(MAINTENANT);
  d.setUTCDate(d.getUTCDate() + n);
  d.setUTCHours(d.getUTCHours() + heures);
  return d.toISOString();
}

describe("@happy le décompte dit les jours restants et leur dénominateur", () => {
  it("affiche les jours restants et l'article qui les fonde", () => {
    render(DecompteLegal, {
      props: {
        echeance: dansNJours(18),
        article: "Art. 3.87 § 12 CC",
        delaiJours: 30,
      },
    });

    const bloc = screen.getByTestId("decompte-legal");
    expect(bloc).toHaveAttribute("data-jours-restants", "18");
    expect(bloc).toHaveAttribute("data-delai-jours", "30");
    expect(bloc).toHaveAttribute("data-depasse", "false");

    // L'article distingue une échéance imposée par la loi d'un rappel
    // que nous aurions inventé.
    expect(
      screen.getByTestId("decompte-legal-article").textContent?.trim(),
    ).toBe("Art. 3.87 § 12 CC");
  });
});

describe("@edge l'arrondi ne donne jamais un jour de trop", () => {
  it("compte zéro jour quand l'échéance tombe dans quelques heures", () => {
    // À 23 h de l'échéance, il ne reste PAS un jour : il reste aujourd'hui.
    render(DecompteLegal, {
      props: {
        echeance: dansNJours(0, 23),
        article: "Art. 3.87 § 12 CC",
        delaiJours: 30,
      },
    });

    expect(screen.getByTestId("decompte-legal")).toHaveAttribute(
      "data-jours-restants",
      "0",
    );
  });

  it("ne déborde pas la piste quand le délai est largement dépassé", () => {
    render(DecompteLegal, {
      props: {
        echeance: dansNJours(-90),
        article: "Art. 3.87 § 12 CC",
        delaiJours: 30,
      },
    });

    // Trois fois le délai écoulé : la barre est pleine, pas triple.
    const piste = screen
      .getByTestId("decompte-legal")
      .querySelector<HTMLElement>("[style*='width']");
    expect(piste?.style.width).toBe("100%");
  });
});

describe("@negative un dépassement se dit, il ne se devine pas", () => {
  it("marque le dépassement autrement que par une couleur", () => {
    render(DecompteLegal, {
      props: {
        echeance: dansNJours(-12),
        article: "Art. 3.87 § 12 CC",
        delaiJours: 30,
      },
    });

    const bloc = screen.getByTestId("decompte-legal");
    // L'attribut porte l'état : une recette et un lecteur d'écran peuvent le
    // lire sans interpréter une nuance de rouge.
    expect(bloc).toHaveAttribute("data-depasse", "true");
    expect(bloc).toHaveAttribute("data-jours-restants", "-12");
    // Et le nombre affiché porte son signe.
    expect(bloc.textContent).toContain("+12");
  });

  it("ne rend rien quand l'échéance est illisible", () => {
    // Fail-closed : une date que le serveur n'a pas su former ne doit pas
    // produire un décompte inventé. Mieux vaut pas de chiffre qu'un faux.
    render(DecompteLegal, {
      props: {
        echeance: "pas-une-date",
        article: "Art. 3.87 § 12 CC",
        delaiJours: 30,
      },
    });

    expect(screen.queryByTestId("decompte-legal")).toBeNull();
  });
});
