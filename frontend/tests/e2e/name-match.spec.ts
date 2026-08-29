import { test, expect } from "@playwright/test";
import { normalizeName, nameContains } from "./helpers/name-match";

/**
 * Non-régression du 3e défaut derrière les échecs `[scenarios]` (#696, WP-D1).
 *
 * Sept scénarios cherchaient l'immeuble du monde de scénario avec
 * `"Residence du Parc"` — sans accent — alors que le backend le sème sous
 * `"Résidence du Parc Royal"` (`seed.rs:3133`). La comparaison échouait donc
 * toujours, et EN SILENCE.
 *
 * Ces tests sont des fonctions pures : aucun navigateur, aucun serveur.
 */
test.describe("Correspondance de noms insensible aux accents (#696)", () => {
  test("@negative le code d'origine échouait — c'est la cause du défaut", () => {
    // Reproduit littéralement ce que faisaient les scénarios.
    expect("Résidence du Parc Royal".includes("Residence du Parc")).toBe(false);
  });

  test("@happy la comparaison normalisée trouve l'immeuble semé", () => {
    expect(nameContains("Résidence du Parc Royal", "Residence du Parc")).toBe(
      true,
    );
  });

  test("@edge la normalisation est symétrique et tolère casse et espaces", () => {
    expect(nameContains("Residence du Parc Royal", "Résidence du Parc")).toBe(
      true,
    );
    expect(nameContains("RÉSIDENCE DU PARC ROYAL", "résidence du parc")).toBe(
      true,
    );
    expect(
      nameContains("Résidence  du   Parc Royal", "Residence du Parc"),
    ).toBe(true);
    expect(normalizeName("  Résidence   du Parc  ")).toBe("residence du parc");
  });

  test("@security la normalisation ne sur-matche pas un autre immeuble", () => {
    // Le monde de scénario contient trois immeubles : une comparaison trop
    // permissive sélectionnerait le mauvais et le test passerait pour de
    // mauvaises raisons.
    expect(nameContains("Le Clos des Hirondelles", "Residence du Parc")).toBe(
      false,
    );
    expect(nameContains("Les Terrasses de Flagey", "Residence du Parc")).toBe(
      false,
    );
  });
});
