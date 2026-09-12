import { test, expect } from "@playwright/test";
import { ACCUEIL, DEUX_IMMEUBLES, ouvreEnTantQue, repond } from "./socle";

/**
 * Le sélecteur d'immeuble sous le pouce.
 *
 * ── Ce que la mesure a donné, et ce qu'elle a corrigé ──────────────────────
 *
 * J'avais conclu, en lisant les classes, que les lignes de résultat faisaient
 * 36 px. Le navigateur a répondu **52** : le contenu sur deux lignes les
 * étirait. La lecture de classes est une hypothèse, pas une mesure.
 *
 * Ce qu'elle a trouvé en revanche, et que la lecture n'avait pas vu :
 *
 *   champ de recherche       38 px de haut
 *   étoile « favori »        14 × 20 px
 *
 * L'étoile est le vrai défaut. Elle vit DANS une ligne de 52 px qui fait autre
 * chose — choisir l'immeuble — et elle arrête la propagation du clic. Viser à
 * côté n'ouvrait donc pas la copropriété : cela ajoutait un favori, sans rien
 * dire. Deux cibles imbriquées dont la petite renverse l'action de la grande.
 *
 * ── Ce que ce fichier ne dit pas ───────────────────────────────────────────
 *
 * Rien sur la PLACE de la liste. Mesurée, elle s'ouvre à 106 px et descend à
 * 264 px dans une fenêtre de 727 : elle tient largement au-dessus du clavier.
 * Je pensais devoir la déplacer dans une feuille du bas ; la mesure dit que
 * non, et c'est la mesure qui tranche.
 */

const CIBLE_TACTILE_MIN = 44;

async function ouvreLaListe(page: import("@playwright/test").Page) {
  await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);
  await repond(page, /\/api\/v1\/buildings\?/, DEUX_IMMEUBLES);
  await page.getByTestId("building-selector-input").click();
  await page.waitForSelector("[data-testid^='building-selector-result-']");
}

test.describe("@edge les cibles du sélecteur d'immeuble", () => {
  test("@edge le champ de recherche fait au moins 44 px", async ({ page }) => {
    await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);

    const boite = await page
      .getByTestId("building-selector-input")
      .boundingBox();
    expect(boite).not.toBeNull();
    expect(
      boite!.height,
      `Le champ mesure ${boite!.height} px de haut. C'est le premier geste de ` +
        `chaque écran : le manquer met le focus sur la barre, pas dans le champ.`,
    ).toBeGreaterThanOrEqual(CIBLE_TACTILE_MIN);
  });

  test("@edge l'étoile « favori » ne se vise plus par accident", async ({
    page,
  }) => {
    await ouvreLaListe(page);

    const etoiles = page.locator(
      "[data-testid^='building-selector-favorite-']",
    );
    const nombre = await etoiles.count();
    expect(nombre).toBeGreaterThan(0);

    for (let i = 0; i < nombre; i++) {
      const boite = await etoiles.nth(i).boundingBox();
      expect(boite).not.toBeNull();
      expect(
        Math.min(boite!.width, boite!.height),
        `L'étoile mesure ${boite!.width}×${boite!.height} px dans une ligne ` +
          `qui en fait bien plus. Elle arrête la propagation du clic : la ` +
          `manquer AJOUTE UN FAVORI au lieu d'ouvrir la copropriété.`,
      ).toBeGreaterThanOrEqual(CIBLE_TACTILE_MIN);
    }
  });

  test("@edge l'étoile reste dans sa ligne", async ({ page }) => {
    await ouvreLaListe(page);

    // Agrandir une cible peut la faire déborder de son conteneur, et un
    // débordement à droite est invisible sur un écran étroit tant qu'on ne
    // mesure pas. La cible serait alors grande et partiellement hors d'atteinte.
    //
    // Le témoin a demandé deux essais, et le premier est instructif : élargir
    // l'étoile à 96 px ne la fait PAS déborder, parce que la ligne est une
    // boîte flexible et que c'est la colonne de texte qui se comprime. Il
    // faut la pousser dehors (`-right-8`) pour que la règle morde — 24 px de
    // débordement, dit par le message.
    const ligne = await page
      .locator("[data-testid^='building-selector-result-']")
      .first()
      .boundingBox();
    const etoile = await page
      .locator("[data-testid^='building-selector-favorite-']")
      .first()
      .boundingBox();

    expect(ligne).not.toBeNull();
    expect(etoile).not.toBeNull();
    expect(
      etoile!.x + etoile!.width,
      `L'étoile déborde de ${etoile!.x + etoile!.width - (ligne!.x + ligne!.width)} px ` +
        `à droite de sa ligne.`,
    ).toBeLessThanOrEqual(ligne!.x + ligne!.width + 1);
  });
});
