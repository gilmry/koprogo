import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Une ancre citée par un persona doit exister dans le code.
 *
 * ── Le dégât qui a fait naître cette garde ────────────────────────────────
 *
 * `docs/personas/accountant.md` annonçait, pour l'étape 5 du comptable :
 *
 *     /journal-entries : `journal-entry-form`, …
 *
 * Cette ancre n'existait pas. `JournalEntryForm.svelte:181` expose
 * `journal-entry-panel`, et son propre commentaire dit pourquoi : l'ancienne
 * « résolvait deux éléments ». Le composant a été renommé ; le persona ne
 * l'a pas suivi, et rien ne l'y obligeait.
 *
 * Découvert le 2026-09-18 en écrivant le parcours filmé du comptable (#808),
 * qui a échoué dessus. Le même jour, une seconde ancre m'a trompé sur le
 * conseil — `board-member-list` au lieu de `board-management`. Deux fois en
 * une journée, sur deux documents différents.
 *
 * ── Pourquoi ce n'est pas une coquille ────────────────────────────────────
 *
 * Depuis #805, les personas ne sont pas de la prose d'accompagnement : ils
 * sont la SOURCE du parcours filmé. Le plan de revue humaine y renvoie pour
 * « ce qu'il faut faire », la vitrine les transpose en étapes, et le
 * relecteur G1 les suit.
 *
 * Une ancre fausse s'y propage donc en cascade : un parcours qui échoue, un
 * relecteur qui cherche un élément absent, une story de documentation
 * vivante qui part sur un inventaire faux.
 *
 * ── Pourquoi un cliquet à ZÉRO ────────────────────────────────────────────
 *
 * Mesuré avant de poser la garde, comme #938 l'a appris à ses dépens : sur
 * **85 ancres citées** par les six personas, **une seule** ne résolvait pas,
 * et c'était `data-testid` lui-même — les documents parlent du « contrat
 * `data-testid` gelé ». Un faux positif, pas une dette.
 *
 * Il n'y a donc rien à tolérer : c'est une interdiction.
 */

const RACINE = join(process.cwd(), "..");
const PERSONAS = join(RACINE, "docs", "personas");
const SOURCES = join(process.cwd(), "src");

/**
 * Ce que les personas écrivent entre accents graves sans que ce soit une
 * ancre. La liste est courte à dessein : chaque entrée est une exception, et
 * une exception de trop rendrait la garde muette.
 */
const PAS_DES_ANCRES = new Set([
  // Les personas parlent du « contrat `data-testid` gelé » (#802/#803).
  "data-testid",
]);

/** Toutes les ancres que le code connaît, déclarées ou interrogées. */
function ancresDuCode(): Set<string> {
  const trouvees = new Set<string>();
  const parcourir = (dossier: string): void => {
    for (const entree of readdirSync(dossier)) {
      const chemin = join(dossier, entree);
      if (statSync(chemin).isDirectory()) {
        if (entree === "node_modules") continue;
        parcourir(chemin);
        continue;
      }
      if (!/\.(svelte|astro|ts)$/.test(entree)) continue;
      const contenu = readFileSync(chemin, "utf8");
      for (const m of contenu.matchAll(/data-testid="([^"]+)"/g))
        trouvees.add(m[1]);
      for (const m of contenu.matchAll(/getByTestId\("([^"]+)"\)/g))
        trouvees.add(m[1]);
    }
  };
  parcourir(SOURCES);
  return trouvees;
}

interface Citation {
  readonly persona: string;
  readonly ancre: string;
}

/**
 * Les ancres citées par les personas.
 *
 * Le motif est volontairement étroit : du kebab-case entre accents graves,
 * au moins deux segments. Un mot seul (`compta`, `owner`) n'est pas une
 * ancre, et une chaîne à points (`accountant.encodeur`) est un rôle.
 */
function citationsDesPersonas(): Citation[] {
  const trouvees: Citation[] = [];
  for (const fichier of readdirSync(PERSONAS)) {
    if (!fichier.endsWith(".md")) continue;
    if (fichier === "GABARIT.md" || fichier === "README.md") continue;
    const contenu = readFileSync(join(PERSONAS, fichier), "utf8");
    for (const m of contenu.matchAll(
      /`([a-z][a-z0-9]*(?:-[a-z0-9]+){1,6})`/g,
    )) {
      if (PAS_DES_ANCRES.has(m[1])) continue;
      trouvees.push({ persona: fichier, ancre: m[1] });
    }
  }
  return trouvees;
}

describe("les personas ne citent que des ancres qui existent", () => {
  it("@happy toute ancre citée par un persona existe dans le code", () => {
    const connues = ancresDuCode();
    const fantomes = citationsDesPersonas()
      .filter((c) => !connues.has(c.ancre))
      .map((c) => `${c.persona} → ${c.ancre}`)
      .sort();

    expect(
      [...new Set(fantomes)],
      "Ces personas citent des ancres que le code ne connaît pas. Un persona " +
        "est la SOURCE du parcours filmé et du plan de revue G1 : une ancre " +
        "fausse s'y propage en cascade. Vérifiez si l'ancre a été renommée, " +
        "et suivez-la.",
    ).toEqual([]);
  });

  it("@edge la garde lit encore les deux côtés", () => {
    // Sans ce contrôle, un chemin qui change rendrait la règle vraie sur
    // deux ensembles vides — le piège que `garde_harnais_executes` et
    // `garde_paniques_en_production` portent déjà côté Rust.
    expect(ancresDuCode().size).toBeGreaterThan(500);
    expect(citationsDesPersonas().length).toBeGreaterThan(50);
  });

  it("@negative une ancre inventée serait refusée", () => {
    const connues = ancresDuCode();
    expect(connues.has("ancre-qui-nexiste-pas-du-tout")).toBe(false);
  });

  it("@security la liste d'exceptions reste courte et justifiée", () => {
    // Une exception de trop rend la garde muette. Chaque entrée porte sa
    // raison en commentaire ; ce test empêche la liste de gonfler sans
    // qu'on s'en aperçoive.
    expect(
      PAS_DES_ANCRES.size,
      "au-delà de trois exceptions, c'est le détecteur qu'il faut revoir, " +
        "pas la liste qu'il faut allonger",
    ).toBeLessThanOrEqual(3);
  });
});
