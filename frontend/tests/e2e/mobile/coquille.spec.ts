import { test, expect } from "@playwright/test";
import { ACCUEIL, ouvreEnTantQue, ROLES } from "./socle";

/**
 * La coquille mobile, mesurée par un navigateur.
 *
 * ── Ce que ce fichier apporte, et que 621 tests Vitest ne peuvent pas ──────
 *
 * Chaque assertion ici est une question de MISE EN PAGE : ce qui se place, ce
 * qui se recouvre, ce qui se mesure. jsdom n'a pas de moteur de rendu — tout
 * `getBoundingClientRect()` y rend zéro — donc aucun test unitaire ne peut
 * répondre à l'une d'elles, quelle que soit sa qualité.
 *
 * C'est le motif dominant du dépôt vu d'un cran plus haut : non pas une
 * capacité inatteignable, mais une capacité **vérifiée dans une forme qui
 * n'est pas la sienne**. Un composant mobile validé dans un DOM sans mise en
 * page est un composant non validé.
 */

const CIBLE_TACTILE_MIN = 44; // px CSS — le minimum sous le pouce.

test.describe("@happy la barre d'onglets se place et se touche", () => {
  for (const role of ROLES) {
    test(`@happy ${role} — cinq onglets, chacun d'au moins 44 px`, async ({
      page,
    }) => {
      await ouvreEnTantQue(page, role, ACCUEIL[role]);

      const onglets = page.locator(
        "[data-testid^='tabbar-']:not([data-testid^='tabbar-badge-'])",
      );
      await expect(onglets).toHaveCount(5);

      // La mesure que jsdom ne peut pas rendre : `h-11 w-11` dans une chaîne
      // de classes ne prouve pas 44 px à l'écran. Une classe peut être
      // écrasée, non générée par le scanner Tailwind, ou annulée par un
      // parent en `flex`.
      const nombre = await onglets.count();
      for (let i = 0; i < nombre; i++) {
        const boite = await onglets.nth(i).boundingBox();
        const ancre = await onglets.nth(i).getAttribute("data-testid");
        expect(boite, `${ancre} n'a pas de boîte`).not.toBeNull();
        expect(
          Math.min(boite!.width, boite!.height),
          `${ancre} mesure ${boite!.width}×${boite!.height} px. Sous 44 px, ` +
            `on vise à côté, et sur une barre d'onglets viser à côté ouvre ` +
            `un autre écran.`,
        ).toBeGreaterThanOrEqual(CIBLE_TACTILE_MIN);
      }
    });
  }
});

test.describe("@edge la coquille ne recouvre pas le contenu", () => {
  test("@edge `main` réserve exactement la hauteur de la barre d'onglets", async ({
    page,
  }) => {
    await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);

    // La barre est en `fixed bottom-0` : elle est HORS DU FLUX, donc elle se
    // pose sur le dernier élément de la page si rien ne lui réserve sa place.
    // La réserve vit sur `<main>`, et les deux doivent mesurer la même chose.
    //
    // Comparer les deux valeurs calculées est ce qu'un test unitaire ne peut
    // pas faire : `pb-barre-onglets` et `h-barre-onglets` ne sont que des noms
    // tant qu'un moteur n'a pas résolu le jeton, et un jeton absent se résout
    // en zéro sans rien dire.
    const mesures = await page.evaluate(() => {
      const principal = document.getElementById("main-content");
      const barre = document.querySelector("[data-testid='tabbar']");
      return {
        reserve: principal
          ? parseFloat(getComputedStyle(principal).paddingBottom)
          : Number.NaN,
        hauteurBarre: barre ? barre.getBoundingClientRect().height : Number.NaN,
      };
    });

    expect(mesures.reserve).not.toBeNaN();
    expect(
      mesures.reserve,
      `\`main\` réserve ${mesures.reserve} px pour une barre de ` +
        `${mesures.hauteurBarre} px. Le dernier élément de chaque page vit ` +
        `derrière la barre, et sur une liste c'est la dernière ligne qu'on ` +
        `ne peut plus toucher.`,
    ).toBeGreaterThanOrEqual(mesures.hauteurBarre);
  });

  test("@edge l'en-tête ne recouvre pas la barre de contexte", async ({
    page,
  }) => {
    await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);

    // L'en-tête est en `fixed top-0` et `Layout.astro` lui réserve une cale
    // dans le flux. La barre de contexte est le premier élément VISIBLE qui la
    // suit : si elle remonte sous l'en-tête, la cale ne mesure plus la même
    // chose que lui.
    //
    // Viser un élément nommé plutôt que « le premier enfant » : les enrobages
    // `<astro-island>` sont en `display: contents` et rendent une boîte de
    // hauteur zéro à l'origine. La première version de ce test lisait 0 px et
    // accusait la mise en page.
    const entete = await page.getByTestId("mobile-header").boundingBox();
    const contexte = await page.getByTestId("barre-de-contexte").boundingBox();

    expect(entete).not.toBeNull();
    expect(contexte).not.toBeNull();
    expect(
      contexte!.y,
      `La barre de contexte commence à ${contexte!.y} px, sous un en-tête qui ` +
        `descend à ${entete!.y + entete!.height} px.`,
    ).toBeGreaterThanOrEqual(entete!.y + entete!.height - 1);
  });
});

test.describe("@edge rien ne déborde à 393 px", () => {
  test("@edge le corps ne défile pas horizontalement", async ({ page }) => {
    await ouvreEnTantQue(page, "syndic", ACCUEIL.syndic);

    // Un débordement horizontal du CORPS est toujours un défaut : il fait
    // glisser toute la page de côté au moindre geste, et la barre d'onglets
    // fixe reste, elle, en place. Les tableaux qui débordent le font dans
    // leur propre conteneur (#866) — c'est voulu, et invisible ici.
    const debordement = await page.evaluate(() => ({
      largeurDocument: document.documentElement.scrollWidth,
      largeurFenetre: window.innerWidth,
    }));

    expect(
      debordement.largeurDocument,
      `Le document mesure ${debordement.largeurDocument} px pour une fenêtre ` +
        `de ${debordement.largeurFenetre} px. Quelque chose déborde, et toute ` +
        `la page glissera de côté.`,
    ).toBeLessThanOrEqual(debordement.largeurFenetre + 1);
  });
});
