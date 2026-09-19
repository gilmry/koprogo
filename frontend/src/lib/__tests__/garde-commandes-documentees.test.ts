import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Une commande écrite dans la documentation doit exister.
 *
 * ── Le dégât qui a fait naître cette garde ────────────────────────────────
 *
 * #876 a retiré `slow-down-tests.sh` et `restore-test-speed.sh` — ils
 * modifiaient les fichiers du gate pour y insérer des pauses, ce que la
 * Méthode Foyer nomme comme l'anti-patron de la documentation vivante. Les
 * **fichiers** sont partis. Les **instructions** sont restées :
 *
 *   docs/E2E_TESTING_GUIDE.rst      « make test-e2e-slow »
 *   .claude/scripts/README.md       « bash .claude/scripts/restore-test-speed.sh »
 *   docs/governance/AGENT_RECIPES.md « slow-down-tests.sh / restore-test-speed.sh »
 *
 * Le même README annonçait le retrait au paragraphe 84 et documentait
 * l'usage au paragraphe 98 — il se contredisait lui-même, et personne ne
 * pouvait le voir puisque rien ne lisait ces blocs.
 *
 * En mesurant l'ampleur, le compte est monté à **26 couples (fichier, cible
 * fantôme)**. `docs/MAKEFILE_GUIDE.rst`, qui prétendait « lister toutes les
 * commandes make disponibles », en citait **treize** qui n'existaient plus :
 * `docker-up`, `dev-all`, `test-e2e-ui`, `test-e2e-report`… Le `README.md`
 * du dépôt lui-même ouvrait sur `make docker-up`.
 *
 * ── Pourquoi un cliquet à zéro, et pas une dette tolérée ──────────────────
 *
 * Une commande fausse ne dégrade pas progressivement : elle échoue au
 * premier essai, sur `No rule to make target`, et c'est la première chose
 * qu'un nouveau venu tape. Il n'y a pas de dette raisonnable ici, donc pas
 * de cliquet à faire descendre : c'est une interdiction.
 *
 * ── Ce qu'elle lit, et ce qu'elle refuse de lire ──────────────────────────
 *
 * Seulement les **blocs de commandes** — ` ```bash ` en Markdown,
 * `.. code-block:: bash` en reStructuredText. C'est ce qu'un lecteur
 * copie-colle.
 *
 * Elle ignore délibérément les mentions en ligne (`` `make test-e2e-slow` ``
 * dans une phrase) : écrire « la cible `make test-e2e-slow` n'existe plus »
 * est de la documentation utile, pas une instruction. Une garde qui
 * interdirait de **nommer** ce qui a été retiré pousserait à effacer
 * l'histoire plutôt qu'à la raconter.
 *
 * Elle ignore aussi les matières **importées ou historiques** — issues
 * exportées de GitHub, plans, journaux d'agents, archives, CHANGELOG. Elles
 * décrivent un état passé ; les corriger serait les falsifier.
 *
 * Comme toutes les gardes de ce dépôt, elle lit des fichiers, pas du sens :
 * elle vérifie qu'une cible existe, jamais qu'elle fait ce que le texte
 * promet.
 */

const RACINE = join(process.cwd(), "..");

/** Les répertoires dont le contenu décrit un état passé. */
const MATIERE_HISTORIQUE = [
  "archives",
  "archive",
  "github-export",
  "cowork",
  "maury",
  "agent-activity",
  "plans",
  "_build",
  "node_modules",
];

function documents(racine: string): string[] {
  if (!existsSync(racine)) return [];
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    if (MATIERE_HISTORIQUE.includes(entree)) continue;
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouves.push(...documents(chemin));
    else if (/\.(rst|md)$/.test(entree)) trouves.push(chemin);
  }
  return trouves;
}

/** Les cibles réellement déclarées par le Makefile racine. */
function ciblesDuMakefile(): Set<string> {
  const makefile = readFileSync(join(RACINE, "Makefile"), "utf8");
  return new Set(
    [...makefile.matchAll(/^([a-zA-Z][a-zA-Z0-9_-]*):/gm)].map((m) => m[1]),
  );
}

/** Les blocs de commandes d'un document — pas ses mentions en ligne. */
function blocsDeCommandes(texte: string, extension: string): string[] {
  if (extension === "md") {
    return [
      ...texte.matchAll(/```(?:bash|sh|shell|console)\n([\s\S]*?)```/g),
    ].map((m) => m[1]);
  }
  return [
    ...texte.matchAll(
      /\.\. code-block:: (?:bash|sh|shell|console)\n\n((?:(?:[ \t]+.*)?\n)+)/g,
    ),
  ].map((m) => m[1]);
}

interface Citation {
  readonly fichier: string;
  readonly cible: string;
}

function citations(): Citation[] {
  const fichiers = [
    ...documents(join(RACINE, "docs")),
    ...documents(join(RACINE, ".claude")),
    ...readdirSync(RACINE)
      .filter((f) => /\.md$/.test(f) && f !== "CHANGELOG.md")
      .map((f) => join(RACINE, f)),
  ];

  const trouvees: Citation[] = [];
  for (const chemin of fichiers) {
    const texte = readFileSync(chemin, "utf8");
    const extension = chemin.endsWith(".md") ? "md" : "rst";
    for (const bloc of blocsDeCommandes(texte, extension)) {
      for (const m of bloc.matchAll(
        /^\s*(?:\$ )?make\s+([a-z][a-z0-9-]{2,})/gm,
      )) {
        trouvees.push({ fichier: relative(RACINE, chemin), cible: m[1] });
      }
    }
  }
  return trouvees;
}

describe("la documentation ne cite que des commandes qui existent", () => {
  it("@happy aucune cible `make` citée dans un bloc de commandes n'est absente du Makefile", () => {
    const cibles = ciblesDuMakefile();
    const fantomes = citations().filter((c) => !cibles.has(c.cible));

    expect(
      fantomes.map((c) => `${c.fichier} → make ${c.cible}`).sort(),
      "Ces documents demandent au lecteur de lancer une cible qui n'existe " +
        "pas : il recevra `No rule to make target`. Corrigez la commande, ou " +
        "retirez le bloc — `make help` reste la seule liste à jour.",
    ).toEqual([]);
  });

  it("@edge la garde lit encore quelque chose : des citations valides existent", () => {
    // Sans ce contrôle, un motif cassé ou une exclusion de trop rendrait la
    // garde définitivement verte en ne trouvant plus rien à examiner. C'est
    // le même garde-fou que `le_cliquet_examine_encore_du_code_de_production`
    // côté Rust.
    expect(citations().length).toBeGreaterThan(30);
  });

  it("@negative une cible inventée dans un bloc de commandes serait refusée", () => {
    const cibles = ciblesDuMakefile();
    const bloc = blocsDeCommandes(
      "```bash\nmake une-cible-qui-nexiste-pas\n```\n",
      "md",
    );
    const vues = [
      ...bloc[0].matchAll(/^\s*(?:\$ )?make\s+([a-z][a-z0-9-]{2,})/gm),
    ].map((m) => m[1]);

    expect(vues).toEqual(["une-cible-qui-nexiste-pas"]);
    expect(cibles.has("une-cible-qui-nexiste-pas")).toBe(false);
  });

  it("@security nommer une commande retirée hors bloc de commandes reste permis", () => {
    // La garde ne doit pas pousser à effacer l'histoire. Une phrase qui dit
    // « `make test-e2e-slow` a été retiré » est de la documentation ; seule
    // l'INSTRUCTION de la lancer est refusée.
    const prose =
      "La cible ``make test-e2e-slow`` a été retirée le 2026-09-12.";
    expect(blocsDeCommandes(prose, "rst")).toEqual([]);
  });
});
