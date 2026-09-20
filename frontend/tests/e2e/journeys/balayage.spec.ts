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

/**
 * Pas de trace pour un balayage — et ce n'est pas une économie de confort.
 *
 * ── Ce que la trace faisait ───────────────────────────────────────────────
 *
 * Un balayage ouvre 93 écrans. La trace accumule captures, requêtes et
 * instantanés DOM pour chacun, et atteint **148 Mo**. L'écrire dépasse le
 * budget de démontage, et Playwright rapporte cela comme un dépassement de
 * TEST :
 *
 *     ✘ @happy balayage-owner ouvre tous les écrans sans tomber (8.2m)
 *       Test timeout of 30000ms exceeded.
 *
 * Aucune pile, aucun emplacement, et un plafond annoncé de 30 s là où
 * `test.setTimeout(900_000)` en pose 900. Le corps du test avait réussi.
 *
 * Mesuré le 2026-09-20, trois exécutions sur la même recette (#982) :
 *
 *     balayage-owner  (trace: "on")  -> ✘  8.2 min
 *     balayage-owner  (--trace=off)  -> ✓  6.6 min
 *     balayage-syndic (trace: "on")  -> ✓  7.6 min
 *
 * `balayage-syndic` passait de justesse : ce n'est pas une différence de
 * nature, c'est une marge — et elle se referme dès que le runner ralentit.
 *
 * ── Pourquoi on ne perd rien ──────────────────────────────────────────────
 *
 * Une trace de 148 Mo sur 93 écrans est illisible en pratique. Le balayage
 * produit déjà l'artefact fait pour être lu : le tableau route par route
 * (arrivée, taille rendue, erreurs console) et le `.json` de la vitrine avec
 * ses chapitres. C'est lui qui a servi à écrire #968 et #969, jamais une
 * trace Playwright.
 *
 * La vidéo reste : elle est bornée, et c'est elle qui alimente la galerie.
 *
 * ── Pourquoi un rouge faux coûte plus qu'un rouge absent ──────────────────
 *
 * Le Gantt le dit pour un motif voisin : « Un gate jamais lancé ne dit rien,
 * et son silence se lit comme un accord. » Ici c'est la variante chère — le
 * gate parle, il a tort, et la leçon qu'on en tire est d'arrêter de
 * l'écouter. Le jour où le balayage trouvera une vraie régression, son rouge
 * ressemblera aux précédents.
 */
test.use({ trace: "off" });

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
