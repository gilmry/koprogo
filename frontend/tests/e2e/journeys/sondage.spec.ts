/**
 * Harnais n°1 — le gate E2E du parcours de consultation.
 *
 *   - @happy    — les huit étapes aboutissent : la consultation est ouverte,
 *                 publiée, votée, clôturée, et le dépouillement la reflète.
 *   - @edge     — le parcours fait agir deux personnes distinctes. Un
 *                 sondage voté par son auteur ne démontre rien.
 *   - @security — un brouillon n'est pas lisible par un copropriétaire.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { acteursDe } from "./parcours";
import { sondage, mondeDuParcours } from "./sondage.journey";

const parcours = sondage;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(300_000);
    const comptes = await test.step("amorçage", () => parcours.amorcer(page));
    const scene = new Scene(page, comptes);

    for (const etape of parcours.etapes) {
      await test.step(`${etape.acteur} · ${etape.id}`, async () => {
        await etape.action(scene);
        if (etape.assertion) await etape.assertion(page);
      });
    }

    expect(scene.narration.length).toBeGreaterThan(0);
    expect(
      mondeDuParcours()?.sondageId,
      "Le parcours s'est déroulé sans qu'aucune consultation ne reçoive " +
        "d'identifiant : rien n'a été créé.",
    ).toBeTruthy();
  });

  test(`@edge ${parcours.slug} — celui qui consulte n'est pas celui qui vote`, async () => {
    expect(
      acteursDe(parcours),
      "Un sondage ouvert et voté par la même personne ne démontre ni la " +
        "publication, ni le dépôt d'une voix par un tiers.",
    ).toEqual(["syndic", "copropriétaire"]);
  });

  test(`@security ${parcours.slug} — un brouillon n'est pas offert au vote`, async ({
    page,
  }) => {
    test.setTimeout(240_000);
    const comptes = await parcours.amorcer(page);
    const scene = new Scene(page, comptes);

    // Le syndic ouvre la consultation et s'arrête AVANT de publier : c'est
    // exactement l'état d'un brouillon en cours de relecture.
    await scene.devenir("syndic");
    for (const etape of parcours.etapes.slice(0, 4)) {
      await etape.action(scene);
    }
    const monde = mondeDuParcours()!;
    expect(monde.sondageId).toBeTruthy();

    // La copropriétaire ouvre le même lien.
    await scene.devenir("copropriétaire");
    await page.goto(`/polls/detail?id=${monde.sondageId}`);

    // Elle ne doit trouver aucune urne. Un brouillon offert au vote
    // recueillerait des voix sur une question que le syndic peut encore
    // réécrire — et personne ne saurait sur quoi les votants se sont
    // prononcés.
    await expect(
      page.getByTestId("poll-voting-section"),
      "Un brouillon est offert au vote : la question peut encore changer " +
        "après que des voix ont été déposées.",
    ).toHaveCount(0, { timeout: 20000 });

    // Et elle n'hérite pas des commandes d'administration.
    await expect(
      page.getByTestId("poll-publish-button"),
      "Une copropriétaire peut publier la consultation d'un syndic.",
    ).toHaveCount(0);
  });
});
