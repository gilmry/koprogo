import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : tout `test.skip(...)` / `test.fixme(...)` Playwright porte sa
 * raison. Zéro toléré — pas de ratchet à la baisse.
 *
 * ── Le critère `@edge` de la story C7.2 (#427) ────────────────────────────
 *
 * « Étant donné les scénarios marqués `@skip` / `@wip`, quand on les
 * inventorie, alors chacun porte la raison de son skip. Douze l'étaient sans
 * justification : un test désactivé sans motif est un test perdu, pas un
 * test reporté. »
 *
 * Ce dépôt n'a pas de tag Gherkin `@skip`/`@wip` côté Playwright (voir le
 * pendant BDD : `backend/tests/garde_skips_justifies.rs`) — l'équivalent
 * runtime est `test.skip()` / `test.fixme()`. Douze existaient sans
 * justification au moment de l'audit du 2026-04-29 ; ce cliquet inventorie
 * ce qu'il en reste aujourd'hui et empêche d'en ajouter un nouveau non
 * justifié.
 *
 * ── La convention, relevée dans le dépôt ──────────────────────────────────
 *
 * Certains fichiers documentent déjà pourquoi ils skippent, à deux niveaux :
 * - un commentaire juste au-dessus de l'appel (`ContractorReport.spec.ts`) ;
 * - un commentaire juste après l'ouverture du `describe(...)` englobant,
 *   qui vaut pour tous les `test.skip` du bloc (`ApiKeys.spec.ts`,
 *   `Consent.spec.ts`, `SecurityIncidents.spec.ts`).
 *
 * Une troisième forme, native à Playwright, passe la raison en argument :
 * `test.skip(condition, "raison")` (`role-delegation.spec.ts`).
 *
 * Ce que ce cliquet NE reconnaît PAS comme justification : un commentaire
 * qui ne mentionne ni "skip" ni "fixme" (ex. `// TODO plus tard`), et un
 * commentaire placé APRÈS l'appel. Un skip sans motif reste un skip sans
 * motif tant que le motif n'est pas écrit avant lui.
 *
 * Suivi en #427.
 */

const RACINE = join(process.cwd(), "tests/e2e");

const APPEL = /\btest\.(skip|fixme)\(/;
const DESCRIBE = /\bdescribe\(/;

function estLigneDeCommentaire(ligne: string): boolean {
  const nu = ligne.trim();
  return nu.startsWith("//") || nu.startsWith("*") || nu.startsWith("/*");
}

function mentionneSkipOuFixme(ligne: string): boolean {
  // Préfixe, pas mot entier : "skipped"/"skips"/"skipping" doivent compter
  // autant que "skip" — c'est la forme réellement utilisée dans le dépôt
  // (`// All tests skipped: ...`), et un \b final l'aurait ratée.
  return /\bskip/i.test(ligne) || /\bfixme/i.test(ligne);
}

/**
 * Vrai si, après la première virgule de niveau 0 qui suit `depart` dans
 * `texte`, le premier caractère non-blanc est un guillemet — la forme
 * `test.skip(condition, "raison")`.
 *
 * Une simple regex se ferait piéger par un titre contenant une virgule
 * (`"should create, edit, and delete organization using test IDs"`) : on
 * suit donc l'état "dans une chaîne" caractère par caractère plutôt que de
 * chercher la virgule à l'aveugle.
 */
function raisonPasseeEnArgument(texte: string, depart: number): boolean {
  let i = depart;
  let profondeur = 0;
  let dansChaine: string | null = null;

  while (i < texte.length) {
    const c = texte[i];
    if (dansChaine) {
      if (c === "\\") {
        i += 2;
        continue;
      }
      if (c === dansChaine) dansChaine = null;
      i++;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      dansChaine = c;
      i++;
      continue;
    }
    if (c === "(") {
      profondeur++;
      i++;
      continue;
    }
    if (c === ")") {
      if (profondeur === 0) return false;
      profondeur--;
      i++;
      continue;
    }
    if (c === "," && profondeur === 0) {
      let j = i + 1;
      while (j < texte.length && /\s/.test(texte[j])) j++;
      return texte[j] === '"' || texte[j] === "'" || texte[j] === "`";
    }
    i++;
  }
  return false;
}

function extraitTitre(ligne: string): string {
  const m = ligne.match(/test\.(?:skip|fixme)\(\s*(["'`])([\s\S]*?)\1/);
  return m ? m[2] : "(skip conditionnel, sans titre)";
}

interface Violation {
  ligne: number;
  titre: string;
}

/** Les `test.skip`/`test.fixme` de `lignes` qui ne portent pas leur raison. */
function violationsSkip(lignes: string[]): Violation[] {
  const violations: Violation[] = [];
  let debutBloc = 0;

  lignes.forEach((ligne, i) => {
    if (DESCRIBE.test(ligne)) {
      debutBloc = i;
    }
    const appel = APPEL.exec(ligne);
    if (!appel) return;

    const fenetreAppel = lignes.slice(i, i + 6).join("\n");
    if (raisonPasseeEnArgument(fenetreAppel, appel.index + appel[0].length)) {
      return;
    }

    const fenetrePrecedente = lignes.slice(debutBloc, i);
    const justifieParCommentaire = fenetrePrecedente.some(
      (l) => estLigneDeCommentaire(l) && mentionneSkipOuFixme(l),
    );
    if (justifieParCommentaire) return;

    violations.push({ ligne: i + 1, titre: extraitTitre(ligne) });
  });

  return violations;
}

describe("détection des skips non justifiés (unité, sans I/O)", () => {
  it("signale un test.skip sans commentaire ni raison en argument", () => {
    const lignes = ['test.skip("un truc cassé", async () => {', "});"];
    expect(violationsSkip(lignes)).toEqual([
      { ligne: 1, titre: "un truc cassé" },
    ]);
  });

  it("ne signale rien pour un test.skip dont un commentaire immédiat justifie le skip", () => {
    const lignes = [
      "// Skip: endpoint pas encore câblé",
      'test.skip("un truc cassé", async () => {',
      "});",
    ];
    expect(violationsSkip(lignes)).toEqual([]);
  });

  it("propage la justification à tous les test.skip d'un même describe", () => {
    const lignes = [
      'describe("un module", () => {',
      "  // All tests skipped: backend bug #999",
      '  test.skip("premier", async () => {});',
      '  test.skip("second", async () => {});',
      "});",
    ];
    expect(violationsSkip(lignes)).toEqual([]);
  });

  it("ne laisse pas un titre contenant une virgule tromper le détecteur de raison en argument", () => {
    const lignes = [
      'test.fixme("should create, edit, and delete organization using test IDs", async ({',
      "  page,",
      "}) => {",
      "});",
    ];
    expect(violationsSkip(lignes)).toEqual([
      {
        ligne: 1,
        titre: "should create, edit, and delete organization using test IDs",
      },
    ]);
  });

  it("reconnaît la raison passée en argument, même sur plusieurs lignes", () => {
    const lignes = [
      "test.skip(",
      "  true,",
      "  `Backend not available — invariant tested in Vitest`,",
      ");",
    ];
    expect(violationsSkip(lignes)).toEqual([]);
  });

  it("ignore un commentaire placé APRÈS l'appel", () => {
    const lignes = [
      'test.skip("un truc cassé", async () => {',
      "});",
      "// Skip: raison arrivée trop tard pour compter",
    ];
    expect(violationsSkip(lignes)).toEqual([
      { ligne: 1, titre: "un truc cassé" },
    ]);
  });

  it("ignore un commentaire qui ne mentionne ni skip ni fixme", () => {
    const lignes = [
      "// TODO plus tard",
      'test.skip("un truc cassé", async () => {',
      "});",
    ];
    expect(violationsSkip(lignes)).toEqual([
      { ligne: 2, titre: "un truc cassé" },
    ]);
  });

  it("reconnaît « skipped » (participe passé), pas seulement « skip »", () => {
    // La convention réelle du dépôt (ApiKeys.spec.ts, Consent.spec.ts,
    // SecurityIncidents.spec.ts) écrit "skipped", pas "skip" : un \b final
    // sur le mot "skip" raterait ce cas, puisque "skipped" est un seul mot.
    const lignes = [
      'describe("un module", () => {',
      "  // All tests skipped due to backend bug #999",
      '  test.skip("premier", async () => {});',
      "});",
    ];
    expect(violationsSkip(lignes)).toEqual([]);
  });

  it("ne signale jamais un test() normal", () => {
    const lignes = ['test("un chemin nominal", async () => {', "});"];
    expect(violationsSkip(lignes)).toEqual([]);
  });
});

function specs(dossier: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...specs(chemin));
    } else if (chemin.endsWith(".spec.ts") || chemin.endsWith(".scenario.ts")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

describe("inventaire des skips Playwright réels (#427 C7.2)", () => {
  it("ne laisse aucun test.skip/test.fixme sans raison déclarée", () => {
    const violations: string[] = [];
    for (const chemin of specs(RACINE)) {
      const lignes = readFileSync(chemin, "utf8").split("\n");
      const fichier = chemin.slice(process.cwd().length + 1);
      for (const v of violationsSkip(lignes)) {
        violations.push(`${fichier}:${v.ligne} — « ${v.titre.slice(0, 70)} »`);
      }
    }

    expect(
      violations.length,
      `${violations.length} test.skip/test.fixme sans raison déclarée.\n\n` +
        `Un skip sans motif est un test perdu, pas un test reporté (#427 C7.2).\n` +
        `Ajoutez un commentaire "// Skip: ..." / "// Fixme: ..." juste avant ` +
        `l'appel (ou juste après le describe englobant), ou passez la raison ` +
        `en second argument : test.skip(condition, "raison").\n\n` +
        violations.join("\n"),
    ).toBe(0);
  });
});
