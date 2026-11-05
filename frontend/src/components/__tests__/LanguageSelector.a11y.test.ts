// WCAG 2.5.3 « Label in Name » sur le sélecteur de langue.
//
// axe 4.14 (#1090, 2026-11-05) range `label-content-name-mismatch` parmi les
// règles AA : le barrage du VPS a rougi sur l'audit mobile, et rien n'a été
// déployé. Le bouton affiche « FR » mais s'annonçait « Changer de langue » :
// un utilisateur de commande vocale qui dit « FR » ne l'atteint pas.

import { describe, it, expect } from "vitest";
import { render, screen } from "../../test-helpers";
import {
  libelleDansLeNom,
  nomAccessible,
  texteVisible,
} from "../../lib/a11y/libelle-dans-le-nom";
import LanguageSelector from "../LanguageSelector.svelte";

describe("LanguageSelector @security accessibilité", () => {
  it("le nom accessible du bouton contient son texte visible (WCAG 2.5.3)", () => {
    render(LanguageSelector);
    const bouton = screen.getByTestId("language-selector-button");
    expect(texteVisible(bouton)).not.toBe("");
    expect(
      libelleDansLeNom(bouton),
      `« ${nomAccessible(bouton)} » ne contient pas « ${texteVisible(bouton)} »`,
    ).toBe(true);
  });

  it("le nom accessible dit toujours à quoi sert le bouton", () => {
    render(LanguageSelector);
    const bouton = screen.getByTestId("language-selector-button");
    expect(nomAccessible(bouton)).toMatch(/changer de langue/);
  });
});
