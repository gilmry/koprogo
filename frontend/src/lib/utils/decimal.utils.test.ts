import { describe, expect, it } from "vitest";
import { toNumber } from "./decimal.utils";

/**
 * Non-régression — rapport de test « workflows financiers » du 2026-09-01.
 *
 * Le backend sérialise ses `Decimal` en CHAÎNE (ADR-0008). En JavaScript, cela
 * ne casse qu'UN seul opérateur, et c'est ce qui rend le défaut si discret :
 * `*`, `/`, `-` et les comparaisons convertissent implicitement la chaîne,
 * seul `+` concatène. Un même champ se comporte donc correctement dans un
 * calcul et silencieusement faux dans une somme.
 */
describe("toNumber — conversion des Decimal sérialisés en chaîne", () => {
  it("convertit une chaîne décimale", () => {
    expect(toNumber("200.00")).toBe(200);
    expect(toNumber("0")).toBe(0);
    expect(toNumber("1000")).toBe(1000);
  });

  it("laisse un nombre intact", () => {
    expect(toNumber(200)).toBe(200);
    expect(toNumber(0)).toBe(0);
  });

  it("rend 0 sur les valeurs absentes ou illisibles, jamais NaN", () => {
    // Le point clé : `NaN` se propage en silence dans tout calcul en aval, et
    // `Math.abs(NaN - x) > 0.5` vaut FALSE — un indicateur de conformité bâti
    // dessus annonce « tout va bien » quelle que soit la donnée.
    for (const v of [null, undefined, "", "abc", NaN, Infinity]) {
      expect(Number.isFinite(toNumber(v as never))).toBe(true);
      expect(toNumber(v as never)).toBe(0);
    }
  });

  it("répare la somme qui concaténait — défaut F14", () => {
    const lots = [{ quota: "200.00" }, { quota: "200.00" }, { quota: "600.00" }];

    // Le comportement d'origine, reproduit pour mémoire.
    const avant = lots.reduce((s, u) => s + ((u.quota as unknown as number) || 0), 0);
    expect(typeof avant).toBe("string");
    expect(Math.round(avant as unknown as number)).toBeNaN();

    // Et sa correction.
    const apres = lots.reduce((s, u) => s + toNumber(u.quota), 0);
    expect(apres).toBe(1000);
  });

  it("rétablit l'indicateur de conformité des quotités", () => {
    // Un immeuble volontairement NON conforme : 900 millièmes encodés sur 1000.
    const lots = [{ quota: "200.00" }, { quota: "700.00" }];
    const attendu = 1000;

    const totalCasse = lots.reduce((s, u) => s + ((u.quota as unknown as number) || 0), 0);
    // Le cœur du défaut : la comparaison ne dit pas « écart », elle dit « rien
    // à signaler ». L'écran affichait donc « quotités correctes » sur un
    // immeuble en dérive.
    expect(Math.abs((totalCasse as unknown as number) - attendu) > 0.5).toBe(false);

    const totalCorrige = lots.reduce((s, u) => s + toNumber(u.quota), 0);
    expect(Math.abs(totalCorrige - attendu) > 0.5).toBe(true);
  });
});
