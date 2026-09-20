/**
 * Harnais n°1 — le gate E2E du panneau d'affichage.
 *
 *   - @happy    — les six étapes aboutissent : l'annonce est rédigée,
 *                 publiée, retrouvée par une recherche, lue, puis archivée.
 *   - @edge     — deux acteurs : celui qui publie n'est pas celui qui lit.
 *   - @security — une copropriétaire n'archive pas l'annonce du syndic.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { acteursDe } from "./parcours";
import { annonce, mondeDuParcours } from "./annonce.journey";

const parcours = annonce;

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
  });

  test(`@edge ${parcours.slug} — celui qui publie n'est pas celui qui lit`, async () => {
    expect(
      acteursDe(parcours),
      "Un panneau d'affichage rédigé et lu par la même personne ne " +
        "démontre rien de la diffusion, qui est tout son objet.",
    ).toEqual(["syndic", "copropriétaire"]);
  });

  test(`@security ${parcours.slug} — une copropriétaire n'archive pas l'annonce du syndic`, async ({
    page,
  }) => {
    test.setTimeout(240_000);
    const comptes = await parcours.amorcer(page);
    const scene = new Scene(page, comptes);

    // Le syndic rédige, enregistre, relit et publie (les quatre premières
    // étapes). Sans la publication, la copropriétaire ne verrait rien et ce
    // test passerait au vert pour la mauvaise raison (#978).
    await scene.devenir("syndic");
    for (const etape of parcours.etapes.slice(0, 4)) {
      await etape.action(scene);
    }
    const monde = mondeDuParcours()!;

    // La copropriétaire ouvre la même annonce.
    await scene.devenir("copropriétaire");
    await scene.aller("/notices");
    await scene.attendreChargement();
    await scene.saisir("notice-list-search-input", "ascenseur");
    await scene.cliquerLePremier("notice-list-detail-link");
    await expect(page.getByTestId("notice-detail")).toBeVisible({
      timeout: 20000,
    });
    await expect(page.getByTestId("notice-detail")).toContainText(monde.titre);

    // Elle la lit, et n'en dispose pas. `NoticeDetail.svelte:123` réserve
    // archivage et suppression à l'auteur. Une copropriétaire qui pourrait
    // retirer du panneau l'annonce de travaux qui la dérange viderait
    // l'information descendante de tout effet.
    await expect(
      page.getByTestId("notice-archive-btn"),
      "Une copropriétaire peut archiver l'annonce du syndic.",
    ).toHaveCount(0);
    await expect(
      page.getByTestId("notice-delete-btn"),
      "Une copropriétaire peut supprimer l'annonce du syndic.",
    ).toHaveCount(0);
  });
});
