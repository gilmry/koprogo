import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : une même `data-testid` ne se répand pas sur de nouveaux composants.
 *
 * ── Le constat ────────────────────────────────────────────────────────────
 *
 * Le portique de caractérisation a échoué quarante fois d'affilée sur un seul
 * test, « owner units page renders (his lots) », faute de trouver
 * `[data-testid='owner-units']`.
 *
 * L'ancre existait pourtant. Elle était dans `OwnerUnits.svelte` — mais ce
 * composant est la liste repliée dans le tableau des copropriétaires, côté
 * syndic. La page `/owner/units` monte `OwnerUnitList.svelte`, qui n'en
 * portait aucune. **Le nom correspondait, l'écran non.**
 *
 * Le même jour, deux autres cas de la même famille :
 *
 * - `documents-list` vivait sur `ExpenseDocuments.svelte`, les pièces jointes
 *   d'une dépense, tandis que `/documents` monte `DocumentList.svelte`, qui
 *   portait `document-list` au singulier. Trois composants, deux noms
 *   voisins, aucun sur l'écran visé par le test.
 * - `document-list` était sur DEUX composants à la fois.
 *
 * ── Pourquoi c'est plus vicieux qu'une ancre absente ──────────────────────
 *
 * Une ancre absente se voit : le détecteur de couverture (#803) la compte
 * manquante. Une ancre présente sur le mauvais composant fait paraître
 * l'écran couvert. Le décompte est bon, le test échoue, et on cherche la
 * panne du côté du rendu ou du réseau plutôt que du côté du nom.
 *
 * ── Ce que ce cliquet garde ───────────────────────────────────────────────
 *
 * Il n'interdit pas les doublons — quinze existent, et certains sont
 * légitimes : `loading-spinner` et `cancel-button` désignent la même chose
 * partout, et les specs les visent toujours dans une portée. Il empêche le
 * seizième, c'est-à-dire la prochaine ambiguïté introduite sans y penser.
 *
 * Suivi en #832.
 */

const RACINE = join(process.cwd(), "src");

/** Nombre d'ancres portées par plus d'un composant. **Ne doit que BAISSER.** */
const DETTE_AU_2026_09_08 = 15;

const ANCRE = /data-testid=["']([a-zA-Z0-9_-]+)["']/g;

function fichiersDeGabarit(dossier: string): string[] {
  const sortie: string[] = [];
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      sortie.push(...fichiersDeGabarit(chemin));
    } else if (chemin.endsWith(".svelte") || chemin.endsWith(".astro")) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

function ancresParComposant(): Map<string, Set<string>> {
  const index = new Map<string, Set<string>>();
  for (const chemin of fichiersDeGabarit(RACINE)) {
    const source = readFileSync(chemin, "utf8");
    for (const m of source.matchAll(ANCRE)) {
      const nom = m[1];
      if (!index.has(nom)) index.set(nom, new Set());
      index.get(nom)!.add(chemin.slice(RACINE.length + 1));
    }
  }
  return index;
}

describe("les ancres ne deviennent pas ambiguës (#832)", () => {
  it("ne répand pas une même data-testid sur de nouveaux composants", () => {
    const index = ancresParComposant();
    const ambigues = [...index.entries()]
      .filter(([, fichiers]) => fichiers.size > 1)
      .map(([nom, fichiers]) => `${nom} → ${[...fichiers].sort().join(", ")}`)
      .sort();

    // `toBeLessThanOrEqual`, jamais `toHaveLength` : un cliquet borne une
    // dette, il ne doit pas échouer quand elle baisse.
    expect(
      ambigues.length,
      `${ambigues.length} ancres sont portées par plusieurs composants, ` +
        `contre ${DETTE_AU_2026_09_08} au 2026-09-08.\n\n` +
        `Une ancre présente sur le mauvais composant fait paraître l'écran ` +
        `couvert : le décompte est bon, le test échoue, et on cherche la ` +
        `panne ailleurs. C'est ce qui a fait échouer le portique de ` +
        `caractérisation quarante fois de suite.\n\n` +
        `Donnez à chaque écran son propre nom d'ancre.\n\n` +
        ambigues.join("\n"),
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_08);
  });

  /**
   * Sans ce contrôle, une expression régulière cassée rendrait le cliquet
   * vert en ne trouvant plus aucune ancre à comparer.
   */
  it("voit encore les ancres du dépôt", () => {
    const index = ancresParComposant();
    expect(
      index.size,
      "moins de cinq cents ancres trouvées : le détecteur ne lit plus les " +
        "gabarits. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(500);
  });
});
