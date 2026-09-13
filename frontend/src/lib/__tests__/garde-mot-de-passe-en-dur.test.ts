import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Le mot de passe du superadministrateur n'est écrit qu'à UN endroit.
 *
 * `admin123` l'était à quarante-cinq. Ce mot de passe a été tourné le
 * 2026-09-06 (#763) et le repli du seed ne joue que tant que la CI ne définit
 * pas `KOPROGO_SUPERADMIN_PASSWORD`.
 *
 * L'issue #832 avait nommé le risque et l'avait chiffré : « le jour où elle
 * le fera, TOUTE la suite e2e tombera d'un coup, et le message d'erreur ne
 * dira rien du mot de passe. Le découpler est un travail de cinq minutes qui
 * évitera une demi-journée d'enquête. »
 *
 * C'est le motif de ces deux jours poussé à sa limite : une panne dont la
 * cause n'apparaît nulle part dans le symptôme. Ici on peut l'empêcher avant
 * qu'elle survienne.
 *
 * `helpers/identifiants.ts` est le seul endroit autorisé.
 */

const RACINE = join(process.cwd(), "tests/e2e");
const SEUL_ENDROIT = "identifiants.ts";

function fichiers(racine: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouves.push(...fichiers(chemin));
    else if (entree.endsWith(".ts")) trouves.push(chemin);
  }
  return trouves;
}

function motsDePasseEnDur(): string[] {
  const fautifs: string[] = [];
  for (const fichier of fichiers(RACINE)) {
    if (fichier.endsWith(SEUL_ENDROIT)) continue;
    const code = readFileSync(fichier, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    for (const m of code.matchAll(/["'`]admin123["'`]/g)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      fautifs.push(`  ${relative(process.cwd(), fichier)}:${ligne}`);
    }
  }
  return fautifs;
}

describe("le mot de passe superadmin n'est écrit qu'à un endroit", () => {
  it("n'apparaît pas en dur dans les recettes", () => {
    expect(
      motsDePasseEnDur().join("\n"),
      "Le mot de passe du superadministrateur est écrit en dur ici. Il a été " +
        "tourné le 2026-09-06 : le jour où la CI définira " +
        "`KOPROGO_SUPERADMIN_PASSWORD`, ces tests tomberont sans qu'aucun " +
        "message ne parle du mot de passe.\n\n" +
        "Importez `ADMIN_PASSWORD` depuis `helpers/identifiants`.",
    ).toBe("");
  });

  it("l'endroit unique existe encore et lit l'environnement", () => {
    // Contrôle d'aveuglement : la garde ne vaut que si le point unique
    // subsiste ET reste paramétrable. Vide, elle passerait sans rien garder.
    const source = readFileSync(
      join(RACINE, "helpers/identifiants.ts"),
      "utf-8",
    );
    expect(source).toContain("KOPROGO_SUPERADMIN_PASSWORD");
    expect(source).toContain("ADMIN_PASSWORD");
  });
});
