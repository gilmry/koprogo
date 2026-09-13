import { describe, it, expect } from "vitest";
import { tantiemes, formatTantiemes, valeurQuota } from "./tantiemes";

describe("tantiemes", () => {
  it("lit une quote-part sérialisée en chaîne, comme le backend l'envoie", () => {
    // `Decimal` en JSON, c'est une chaîne (ADR-0008 § A).
    expect(tantiemes("250")).toBe(250);
    expect(tantiemes("250.4")).toBe(250);
    expect(tantiemes("250.5")).toBe(251);
  });

  it("accepte aussi un nombre, pour les appelants déjà convertis", () => {
    expect(tantiemes(250)).toBe(250);
  });

  it("rend null sur une quote-part absente ou illisible, jamais NaN", () => {
    // Le défaut d'origine : `Math.round(undefined)` vaut NaN, et l'écran
    // affichait « NaN/1000èmes » à un copropriétaire.
    expect(tantiemes(undefined)).toBeNull();
    expect(tantiemes(null)).toBeNull();
    expect(tantiemes("")).toBeNull();
    expect(tantiemes("abc")).toBeNull();
  });

  it("distingue une quote-part inconnue d'une quote-part nulle", () => {
    // Zéro voudrait dire « ce lot ne vote pas », ce qui est une tout autre
    // affirmation qu'« on ne sait pas ».
    expect(tantiemes(undefined)).toBeNull();
    expect(tantiemes("0")).toBe(0);
  });
});

describe("formatTantiemes", () => {
  it("affiche la quote-part avec sa base", () => {
    expect(formatTantiemes("250")).toBe("250/1000èmes");
  });

  it("lit la base sur l'acte de base au lieu de la coder en dur", () => {
    // Story H1 : `quota_basis` se lit sur l'immeuble. Toutes les copropriétés
    // ne sont pas en millièmes.
    expect(formatTantiemes("2500", 10000)).toBe("2500/10000èmes");
    expect(formatTantiemes("2500", "10000")).toBe("2500/10000èmes");
  });

  it("affiche un tiret cadratin plutôt que NaN quand la quote-part manque", () => {
    expect(formatTantiemes(undefined)).toBe("—");
  });
});

describe("valeurQuota", () => {
  it("ne rend pas la valeur arrondie, contrairement à tantiemes", () => {
    // La validation d'une quote-part ne doit pas arrondir : 0,4 est une
    // quote-part non positive une fois arrondie à 0, et l'accepter parce que
    // 0,6 monterait à 1 serait arbitraire.
    expect(valeurQuota("250.4")).toBe(250.4);
    expect(tantiemes("250.4")).toBe(250);
  });

  it("rend null sur une quote-part absente ou illisible", () => {
    expect(valeurQuota(undefined)).toBeNull();
    expect(valeurQuota("abc")).toBeNull();
  });

  it("laisse la validation refuser une quote-part non positive", () => {
    // 0,4 ne doit pas passer un contrôle `> 0` après arrondi à 0.
    const v = valeurQuota("0.4");
    expect(v).toBe(0.4);
    expect(v !== null && v > 0).toBe(true);
  });
});
