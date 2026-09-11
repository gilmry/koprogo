import { describe, it, expect } from "vitest";
import {
  formatCurrency,
  percentageToDecimal,
  decimalToPercentage,
} from "../finance.utils";

/**
 * `formatCurrency` a **105 appelants** et n'avait aucun test.
 *
 * C'est la fonction par laquelle passe chaque montant affiché du produit :
 * budgets, charges, appels de fonds, arriérés. Elle tient dans cinq lignes, ce
 * qui explique probablement qu'on ne l'ait jamais éprouvée — et c'est
 * exactement pour cela qu'un défaut y est resté.
 */

describe("@happy un montant s'affiche en euros belges", () => {
  it("place le séparateur de milliers et deux décimales", () => {
    // Séparateur de milliers ESPACE, pas point : j'avais écrit
    // `1.234,50 €` de mémoire, l'ICU de Node rend une espace insécable
    // étroite pour `fr-BE`. Les espaces sont normalisées avant comparaison
    // pour que le test porte sur le FORMAT et non sur la variété d'espace
    // qu'une version d'ICU choisit.
    expect(formatCurrency(1234.5).replace(/\s/g, " ")).toBe("1 234,50 €");
  });

  it("distingue zéro d'une absence", () => {
    // Zéro est une information : le budget vaut zéro. Il doit donc s'écrire,
    // et pas se confondre avec « on ne sait pas ».
    expect(formatCurrency(0).replace(/\s/g, " ")).toBe("0,00 €");
  });

  it("garde le signe des montants négatifs", () => {
    // Un solde débiteur est une information juridique : c'est ce qu'un
    // copropriétaire doit à l'association.
    expect(formatCurrency(-42).replace(/\s/g, " ")).toBe("-42,00 €");
  });
});

describe("@edge un montant absent n'est pas zéro", () => {
  // Le défaut mesuré au banc mobile le 2026-09-11 : `NaN €` s'affichait sur
  // l'écran des budgets, au comptable. Le type dit `number`, mais la valeur
  // vient d'une réponse d'API désérialisée, où TypeScript ne vérifie rien.
  it.each([
    ["undefined", undefined],
    ["null", null],
    ["NaN", Number.NaN],
    ["Infinity", Number.POSITIVE_INFINITY],
  ])("rend un tiret pour %s, jamais « NaN € »", (_nom, valeur) => {
    const rendu = formatCurrency(valeur as unknown as number);
    expect(rendu).not.toContain("NaN");
    expect(rendu).toBe("—");
  });

  it("n'écrit pas « 0,00 € » à la place d'une valeur manquante", () => {
    // Le repli tentant, et faux : afficher zéro AFFIRME que le budget est nul.
    // Un budget non renseigné n'est pas un budget à zéro, et un comptable ne
    // peut pas distinguer les deux si on les écrit pareil.
    expect(formatCurrency(undefined as unknown as number)).not.toContain("0");
  });
});

describe("@happy les conversions de pourcentage", () => {
  it("fait l'aller-retour sans dériver", () => {
    // Les quotités sont stockées en décimal et saisies en pourcentage : un
    // aller-retour qui dérive fausserait une majorité en assemblée.
    for (const quotite of [0, 1, 33.33, 50, 66.67, 100]) {
      expect(decimalToPercentage(percentageToDecimal(quotite))).toBeCloseTo(
        quotite,
        10,
      );
    }
  });
});
