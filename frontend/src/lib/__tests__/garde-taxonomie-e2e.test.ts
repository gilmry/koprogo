import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : la part des specs Playwright qui ne déclarent pas leur catégorie
 * ne grossit pas.
 *
 * ── Le pendant frontend de `garde_taxonomie_des_tests.rs` ─────────────────
 *
 * #427 demande depuis le 2026-04-29 que le ratio
 * happy/edge/security/negative soit mesuré. Il l'est enfin, et la couche
 * Playwright est la plus muette des trois :
 *
 * ```
 * Rust        2510 tests     1807 sans catégorie   72 %
 * Playwright   420 tests      354 sans étiquette   84 %
 * BDD         1109 scénarios  919 sans étiquette   82 %
 * ```
 *
 * **Ce chiffre ne dit pas que 84 % des specs sont des chemins nominaux.** Il
 * dit qu'on ne peut pas savoir. C'est la maladie que l'issue décrit — « la
 * métrique cache la maladie » — et elle est plus muette qu'annoncé : l'issue
 * SUPPOSE que la majorité sont des variations *happy path*, et la mesure
 * montre qu'on n'est pas en position de le confirmer.
 *
 * ── La convention, relevée ────────────────────────────────────────────────
 *
 * Soixante-six specs la suivent déjà, dans leur titre :
 *
 * ```
 * @happy     30    le chemin nominal
 * @security  15    le refus d'accès, la fuite
 * @negative  11    l'entrée invalide, l'erreur attendue
 * @edge      10    la borne, le cas limite
 * ```
 *
 * ── Les deux contrôles, et pourquoi le second compte ──────────────────────
 *
 * Un cliquet sur le seul nombre de specs sans étiquette **se satisferait
 * d'une suppression de specs**. Le total doit donc rester à son niveau : on ne
 * descend la dette qu'en nommant, jamais en effaçant.
 *
 * Cinq gardes se sont révélées défaillantes le 2026-09-08 — quatre aveugles à
 * la pire variante de ce qu'elles traquaient, une punissant son propre
 * succès. La question posée avant d'écrire celle-ci : qu'est-ce qu'elle ne
 * voit pas, et que se passe-t-il quand elle réussit ?
 *
 * Suivi en #427.
 */

const RACINE = join(process.cwd(), "tests/e2e");

/** Specs Playwright sans étiquette de catégorie. **Ne doit que BAISSER.** */
// 354 → 366 le 2026-09-09. **Le chiffre monte parce que la garde voit plus,
// pas parce que le dépôt s'est dégradé** : son filtre ne retenait que
// `.spec.ts` et ignorait les douze `.scenario.ts` du projet de documentation
// vivante — ceux qu'on filme, donc les plus visibles.
const SANS_ETIQUETTE_AU_2026_09_08 = 366;

/** Total des specs. **Ne doit pas BAISSER.** */
// 420 → 432, même cause : les douze scénarios entrent dans le décompte.
const TOTAL_AU_2026_09_08 = 432;

const CATEGORIES = ["@happy", "@edge", "@security", "@negative"];

/** `test("…")`, `test('…')` ou `test(\`…\`)`, y compris sur plusieurs lignes. */
const DECLARATION = /\btest\(\s*(["`'])([\s\S]*?)\1/g;

function specs(dossier: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...specs(chemin));
      // `.scenario.ts` autant que `.spec.ts`. Le total de 420 excluait les
      // douze scénarios de documentation vivante — ceux qu'on filme.
    } else if (chemin.endsWith(".spec.ts") || chemin.endsWith(".scenario.ts")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

function titres(): { titre: string; fichier: string }[] {
  const sortie: { titre: string; fichier: string }[] = [];
  for (const chemin of specs(RACINE)) {
    const source = readFileSync(chemin, "utf8");
    for (const m of source.matchAll(DECLARATION)) {
      sortie.push({
        titre: m[2],
        fichier: chemin.slice(process.cwd().length + 1),
      });
    }
  }
  return sortie;
}

describe("la taxonomie des specs Playwright (#427)", () => {
  it("n'ajoute pas de spec sans catégorie déclarée", () => {
    const nues = titres().filter(
      ({ titre }) => !CATEGORIES.some((c) => titre.includes(c)),
    );
    expect(
      nues.length,
      `${nues.length} specs ne déclarent pas leur catégorie, contre ` +
        `${SANS_ETIQUETTE_AU_2026_09_08} au 2026-09-08.\n\n` +
        `Un décompte de tests sans taxonomie rassure sans informer : il ne ` +
        `dit pas si les chemins d'erreur, les bornes et les refus d'accès ` +
        `sont éprouvés, ou si tout est nominal.\n\n` +
        `Ajoutez @happy, @edge, @security ou @negative au titre.\n\n` +
        nues
          .slice(-3)
          .map(({ fichier, titre }) => `${fichier} — « ${titre.slice(0, 60)} »`)
          .join("\n"),
    ).toBeLessThanOrEqual(SANS_ETIQUETTE_AU_2026_09_08);
  });

  /**
   * Le contrôle qui empêche de solder la dette en supprimant des specs.
   */
  it("ne laisse pas le total des specs baisser", () => {
    expect(
      titres().length,
      `moins de specs qu'au 2026-09-08 (${TOTAL_AU_2026_09_08}). Le cliquet ` +
        `de taxonomie se satisferait d'une SUPPRESSION : moins de specs, ` +
        `donc moins de specs sans étiquette. Si des specs ont été ` +
        `légitimement retirées, abaissez la constante en disant lesquelles.`,
    ).toBeGreaterThanOrEqual(TOTAL_AU_2026_09_08);
  });
});
