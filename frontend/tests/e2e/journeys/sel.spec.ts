/**
 * Harnais n°1 — le gate E2E du système d'échange local.
 *
 *   - @happy    — les six étapes aboutissent : l'offre est publiée,
 *                 demandée, rendue, et les crédits ont bougé.
 *   - @edge     — deux acteurs : l'offrant et le demandeur sont distincts.
 *   - @security — un compte ne peut pas se créditer tout seul.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { acteursDe } from "./parcours";
import { sel, mondeDuParcours } from "./sel.journey";

const parcours = sel;

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

  test(`@edge ${parcours.slug} — l'offrant et le demandeur sont deux personnes`, async () => {
    // Un SEL joué par un seul compte ne démontre rien : c'est le
    // déplacement d'un crédit d'une personne vers une autre qui fait la
    // monnaie. Un solde qui tourne en rond n'en est pas une.
    expect(acteursDe(parcours).length).toBeGreaterThanOrEqual(2);
  });

  test(`@security ${parcours.slug} — un compte ne se crédite pas lui-même`, async ({
    page,
  }) => {
    test.setTimeout(240_000);
    const comptes = await parcours.amorcer(page);
    const scene = new Scene(page, comptes);

    // L'offrante publie (les trois premières étapes).
    await scene.devenir("copropriétaire");
    for (const etape of parcours.etapes.slice(0, 3)) {
      await etape.action(scene);
    }
    const monde = mondeDuParcours()!;
    expect(monde.titreDeLOffre).toBeTruthy();

    // Sur SON propre échange, elle n'a aucune commande de demande. C'est la
    // seule barrière entre un système de crédits et une machine à en
    // fabriquer : `ExchangeDetail.svelte:400` exige `!isProvider`.
    await expect(page.getByTestId("exchange-detail")).toBeVisible({
      timeout: 20000,
    });
    await expect(
      page.getByTestId("exchange-request-btn"),
      "L'offrante peut demander son propre service : elle enchaînerait " +
        "demande, acceptation et clôture sans que personne ne rende rien, " +
        "et se créditerait autant qu'elle le souhaite.",
    ).toHaveCount(0);

    // Elle ne peut pas non plus court-circuiter l'étape de demande : tant
    // que personne n'a demandé, il n'y a rien à démarrer.
    await expect(
      page.getByTestId("exchange-start-btn"),
      "Un échange que personne n'a demandé peut être démarré.",
    ).toHaveCount(0);
  });
});
