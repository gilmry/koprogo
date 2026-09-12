/**
 * Harnais n°1 — le gate E2E (correctness).
 *
 * Rejoue le parcours partagé **à la vitesse** et rend vert ou rouge. C'est la
 * première des deux lectures que `skills/documentation-vivante.md` décrit :
 *
 * > « le test et la preuve de valeur doivent être **la même chose, lue deux
 * > fois**. Un parcours de référence est écrit **une seule fois**, et deux
 * > harnais distincts le rejouent. »
 *
 * Ce fichier ne redit donc rien du parcours : il l'importe. Toute étape
 * ajoutée à `perimetre-multi-role.journey.ts` est jouée ici sans qu'on touche
 * à ce fichier — et c'est le but. Si ces deux-là devaient être maintenus en
 * parallèle, on aurait recréé la double-maintenance que le skill élimine.
 *
 * La cadence n'est PAS appliquée ici : ce harnais court à la vitesse. Le
 * bandeau de narration s'affiche quand même — il ne coûte presque rien et
 * rend les captures d'échec lisibles.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { acteursDe } from "./parcours";
import { perimetreMultiRole } from "./perimetre-multi-role.journey";

const parcours = perimetreMultiRole;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(180_000);
    const scene = new Scene(page);

    for (const etape of parcours.etapes) {
      await test.step(`${etape.acteur} · ${etape.id}`, async () => {
        await etape.action(scene);
        if (etape.assertion) await etape.assertion(page);
      });
    }

    // Le parcours a bien été narré : sans narration, la vitrine n'aurait pas
    // de chapitres, et la preuve de valeur ne serait pas lisible.
    expect(scene.narration.length).toBeGreaterThan(0);
  });

  test(`@security ${parcours.slug} fait bien agir plusieurs acteurs`, async () => {
    // La règle 9 : « pas un seul login pour tout le scénario ». Un parcours
    // qui se réduirait à un acteur unique aurait perdu ce qu'il démontre,
    // sans que rien ne le signale. Cette assertion le signale.
    const acteurs = acteursDe(parcours);
    expect(
      acteurs.length,
      `Le parcours ${parcours.slug} ne fait agir que ${acteurs.join(", ")}. ` +
        "Un parcours multi-rôles qui perd ses rôles ne prouve plus rien du " +
        "cloisonnement.",
    ).toBeGreaterThanOrEqual(2);
  });
});
