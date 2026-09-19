/**
 * Le balayage, côté gate — rejoué, jamais jugé.
 *
 * ── Pourquoi cette spec existe ────────────────────────────────────────────
 *
 * `garde-parcours-partage.test.ts` impose que l'union des parcours rejoués
 * par le gate égale celle de la vitrine. C'est l'invariant anti-dette du
 * skill `documentation-vivante` : un parcours filmé que le gate ne rejoue
 * pas est une promesse que rien ne vérifie.
 *
 * L'invariant a mordu à l'instant même où les quatre balayages ont rejoint
 * le tournage — la vitrine en filmait onze, le gate en rejouait sept.
 *
 * ── Ce qu'elle vérifie, et ce qu'elle refuse de vérifier ──────────────────
 *
 * Elle vérifie que le balayage **s'exécute de bout en bout** : que le compte
 * s'amorce, que la connexion aboutit, que les 95 écrans s'ouvrent sans faire
 * tomber le harnais.
 *
 * Elle NE vérifie PAS l'état des écrans. Arbitrage du PO du 2026-09-19 :
 * « décrire, ne rien casser ». Sur les 95 écrans du syndic, le premier
 * balayage a compté 30 rebonds, 2 écrans quasi vides et 14 consoles en
 * erreur — dont un `each_key_duplicate` bien réel sur `/legal-rules`. En
 * faire des assertions dès le premier jour rendrait le gate ingérable, et
 * un gate ingérable finit débranché.
 *
 * Ces constats partent en issues, pas en rouge. C'est une différence de
 * nature : le balayage est un instrument de mesure, pas un juge.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { parcours as balayageSyndic } from "./balayage-syndic.journey";
import { parcours as balayageCoproprietaire } from "./balayage-coproprietaire.journey";
import { parcours as balayageComptable } from "./balayage-comptable.journey";
import { parcours as balayageAdministrateur } from "./balayage-administrateur.journey";

const BALAYAGES = [
  balayageSyndic,
  balayageCoproprietaire,
  balayageComptable,
  balayageAdministrateur,
];

for (const parcours of BALAYAGES) {
  test.describe(`Balayage — ${parcours.titre}`, () => {
    test(`@happy ${parcours.slug} ouvre tous les écrans sans tomber`, async ({
      page,
    }) => {
      // Quinze minutes : 95 écrans à ~2 s dans le gate, qui ne narre pas les
      // descriptions d'étape (seules les anomalies parlent). Les cinq
      // minutes des parcours métier ne suffiraient pas, et un plafond trop
      // juste transformerait la lenteur d'un runner en faux défaut.
      test.setTimeout(900_000);

      const comptes = await test.step("amorçage", () => parcours.amorcer(page));
      const scene = new Scene(page, comptes);

      for (const etape of parcours.etapes) {
        await test.step(`${etape.acteur} · ${etape.id}`, async () => {
          await etape.action(scene);
          if (etape.assertion) await etape.assertion(page);
        });
      }

      // La seule assertion tolérable ici : le balayage a bien produit
      // quelque chose. Un balayage muet n'aurait rien ouvert, et c'est le
      // seul défaut dont CETTE spec a le droit de témoigner.
      expect(
        scene.narration.length,
        "Le balayage n'a produit aucun chapitre : il n'a donc ouvert aucun " +
          "écran, ou sa narration d'anomalie ne fonctionne plus.",
      ).toBeGreaterThan(0);
    });
  });
}
