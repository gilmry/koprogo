import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Garde-fou : l'écran de vote lit les pourcentages du serveur, il ne les
 * recalcule pas.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Recette 3 du 2026-09-06 (RN-3), constaté au navigateur. AG « RECETTE3-AG
 * Plafonnement », trois votes saisis :
 *
 *     Alice   550 ‰  pour
 *     Bob     250 ‰  contre
 *     Claire  200 ‰  contre
 *
 * L'écran affichait :
 *
 *     Pour   : 1 vote  (33,3 %)   barre verte COURTE
 *     Contre : 2 votes (66,7 %)   barre rouge LONGUE
 *
 * L'API renvoyait au même instant `pour_percentage = 55` et
 * `contre_percentage = 45`. **La résolution était adoptée et l'écran la
 * montrait rejetée.**
 *
 * ── Pourquoi c'est un défaut juridique, pas d'affichage ────────────────────
 *
 * Art. 3.87 § 6 : « chaque copropriétaire dispose d'un nombre de voix
 * correspondant à sa quote-part dans les parties communes ». Compter des
 * bulletins au lieu de compter des voix inverse le sens d'un vote dès que les
 * quotes-parts sont inégales — c'est-à-dire toujours.
 *
 * Le backend avait été corrigé le 2026-09-04 (`597c728c`). Le calcul existait
 * **en double**, et corriger l'un laissait l'autre faux sans qu'aucun test ne
 * le voie.
 *
 * ── Ce que ce test vérifie ─────────────────────────────────────────────────
 *
 * Que le gabarit ne réintroduise pas un dénominateur fondé sur le NOMBRE de
 * bulletins. C'est une vérification lexicale, et elle est adaptée : le défaut
 * était lexical. Un test de rendu passerait tout aussi bien avec le mauvais
 * calcul, puisqu'il rend fidèlement ce qu'on lui donne.
 *
 * Voir #773.
 */

const PANNEAU = join(
  process.cwd(),
  "src/components/resolutions/ResolutionVotePanel.svelte",
);

/**
 * Les motifs qui trahissent un décompte par tête.
 *
 * `votes.length` employé comme diviseur, ou un filtre sur les bulletins suivi
 * d'une division : dans les deux cas on ramène des voix à des têtes.
 */
const DECOMPTE_PAR_TETE = [
  /votes\.length\s*\)?\s*\*\s*100/,
  /\/\s*votes\.length/,
  /votes\.filter\([^)]*\)\.length\s*\/\s*votes\.length/,
];

describe("les majorités se comptent en voix, pas en bulletins (#773)", () => {
  const source = readFileSync(PANNEAU, "utf8");

  it("lit les pourcentages servis par l'API", () => {
    for (const champ of [
      "pour_percentage",
      "contre_percentage",
      "abstention_percentage",
    ]) {
      expect(
        source,
        `Le panneau de vote ne lit plus \`${champ}\`.\n\n` +
          `S'il recalcule à partir des bulletins, il compte des TÊTES là où ` +
          `l'Art. 3.87 § 6 impose des VOIX proportionnelles aux quotes-parts. ` +
          `Une résolution adoptée s'affiche alors rejetée (#773).`,
      ).toContain(champ);
    }
  });

  it("ne divise jamais par le nombre de bulletins", () => {
    const fautes = DECOMPTE_PAR_TETE.filter((m) => m.test(source));

    expect(
      fautes.length,
      `Le panneau de vote divise par le nombre de bulletins.\n\n` +
        `Alice 550 ‰ pour, Bob 250 ‰ contre, Claire 200 ‰ contre donne alors ` +
        `« 1 pour / 2 contre », soit 33 % contre 67 % — quand les voix disent ` +
        `55 % contre 45 %. Le sens du vote s'inverse (#773).`,
    ).toBe(0);
  });

  /// Les deux dénominateurs diffèrent À DESSEIN et ne somment pas à 100.
  ///
  /// « Pour » et « contre » se rapportent aux voix EXPRIMÉES, les abstentions
  /// en étant exclues par l'Art. 3.87 § 8 ; l'abstention se rapporte à TOUTES
  /// les voix présentes. Un relecteur pressé par le total qui ne fait pas 100
  /// serait tenté de les ramener à une base commune — le commentaire qui
  /// l'explique doit survivre.
  it("conserve l'explication des deux dénominateurs", () => {
    expect(
      source,
      "L'explication des deux dénominateurs a disparu du panneau de vote. " +
        "Sans elle, le prochain lecteur ramènera les trois parts à une base " +
        "commune parce que leur somme ne fait pas 100, et fera entrer les " +
        "abstentions dans le calcul de la majorité — contre l'Art. 3.87 § 8.",
    ).toMatch(/3\.87\s*§\s*8/);
  });
});
