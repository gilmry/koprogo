import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Garde-fou : aucun secret n'atteint la console du navigateur.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * `AdminDashboard.svelte` écrivait le jeton d'accès d'un compte
 * ADMINISTRATEUR en clair dans la console :
 *
 *     console.log("Token:", $authStore.token);
 *     console.log("LocalStorage User:", localStorage.getItem("koprogo_user"));
 *
 * Un jeton en console est lisible par toute personne devant l'écran, toute
 * extension de navigateur, toute capture d'écran et tout outil de collecte de
 * journaux. Mettre le jeton en mémoire et le rafraîchissement dans un cookie
 * `HttpOnly` (WP-FE1) ne sert à rien si on le recopie ensuite.
 *
 * Relevé par la revue de design frontend du 2026-09-06, point 0.5, issue #787.
 *
 * ── Pourquoi un test textuel ───────────────────────────────────────────────
 *
 * Le défaut est de nature lexicale : il s'agit d'un appel écrit dans le
 * source. Un test de rendu ne le verrait pas, puisque le composant ne
 * l'expose pas — il l'écrit dans une console que personne n'inspecte.
 *
 * Le dépôt emploie déjà ce motif pour les directives d'événements
 * (`directives-evenements.test.ts`), qui a rattrapé sept boutons inertes.
 */

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".ts", ".astro"]);

/**
 * Ce qu'on refuse de voir passer dans une console.
 *
 * Les motifs s'appliquent à la ligne **privée de ses chaînes littérales** :
 * `console.error("Token refresh error:", error)` journalise l'ERREUR, et le
 * mot « Token » n'est que dans son libellé. Ne pas faire cette distinction
 * produirait un faux positif sur `stores/auth.ts`, et un garde-fou qui crie à
 * tort finit désactivé.
 */
const MOTIFS_INTERDITS: Array<{ motif: RegExp; quoi: string }> = [
  { motif: /console\.\w+\([^)]*\btoken\b/i, quoi: "un jeton" },
  { motif: /console\.\w+\([^)]*localStorage/i, quoi: "le localStorage" },
  { motif: /console\.\w+\([^)]*\bpassword\b/i, quoi: "un mot de passe" },
  {
    motif: /console\.\w+\([^)]*Authorization/i,
    quoi: "un en-tête Authorization",
  },
  {
    motif: /console\.\w+\([^)]*authStore\b/i,
    quoi: "le store d'authentification",
  },
];

/** Remplace le contenu des chaînes par des blancs, en gardant les guillemets. */
function sansChaines(ligne: string): string {
  return ligne.replace(
    /(['"`])(?:\\.|(?!\1)[^\\])*\1/g,
    (m) => m[0] + " ".repeat(Math.max(0, m.length - 2)) + m[0],
  );
}

function fichiersSources(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersSources(chemin));
    } else if (EXTENSIONS.has(extname(entree)) && !entree.includes(".test.")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

describe("aucun secret n'est écrit dans la console (#787)", () => {
  it("ne trouve aucune trace de jeton, de mot de passe ou de localStorage", () => {
    const fautes: string[] = [];

    for (const chemin of fichiersSources(RACINE)) {
      const lignes = readFileSync(chemin, "utf8").split("\n");
      lignes.forEach((ligne, i) => {
        // Les commentaires expliquant le défaut ne sont pas le défaut.
        const nue = ligne.trim();
        if (
          nue.startsWith("//") ||
          nue.startsWith("*") ||
          nue.startsWith("/*")
        ) {
          return;
        }
        const sansTexte = sansChaines(ligne);
        for (const { motif, quoi } of MOTIFS_INTERDITS) {
          if (motif.test(sansTexte)) {
            fautes.push(
              `${chemin.replace(process.cwd() + "/", "")}:${i + 1} — écrit ${quoi} dans la console`,
            );
          }
        }
      });
    }

    expect(
      fautes,
      `Un secret atteindrait la console du navigateur.\n\n` +
        `Un jeton ainsi écrit est lisible par toute personne devant l'écran, ` +
        `toute extension de navigateur et tout outil de collecte de journaux. ` +
        `Une action d'administration se trace au journal d'audit, côté serveur.\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });
});
