import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Garde-fou : aucune classe Tailwind construite par interpolation.
 *
 * ── Le défaut ──────────────────────────────────────────────────────────────
 *
 * Tailwind ne fait pas d'analyse dynamique : il scanne le source à la
 * recherche de **classes entières**. Une classe dont le nom est assemblé à
 * l'exécution n'est jamais reconnue, donc jamais générée, donc absente de la
 * feuille de style livrée.
 *
 *     bg-{getTransactionColor(t)}-50          → bg-green-50 n'existe pas
 *     md:grid-cols-{cond ? '5' : '4'}         → aucune des deux n'existe
 *
 * Deux occurrences relevées par la revue de design du 2026-09-06 (points 0.3
 * et 0.4, issues #788 et #789) : les transactions du tableau de bord comptable
 * s'affichaient sans fond ni couleur, et la grille à cinq colonnes des membres
 * du conseil ne se matérialisait jamais.
 *
 * ── Pourquoi ce défaut échappe à tout ─────────────────────────────────────
 *
 * Le code compile, les types passent, les tests de rendu passent — puisque
 * l'attribut `class` contient bien la chaîne attendue. Seul le RÉSULTAT VISUEL
 * diffère, et il n'est vérifié par rien. C'est un défaut lexical : il se
 * détecte en lisant le source, pas en l'exécutant.
 *
 * ── Ce qui reste autorisé ─────────────────────────────────────────────────
 *
 * Interpoler une classe **complète** est parfait, et le dépôt le fait
 * largement :
 *
 *     class="rounded {actif ? 'bg-green-50' : 'bg-gray-50'}"
 *     class="border {getUrgencyColor(alert.urgency)} rounded-md"
 *
 * Ce qui est refusé, c'est l'interpolation **au milieu** d'un nom de classe.
 */

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".astro"]);

/**
 * Les racines d'utilitaires Tailwind qu'on surveille.
 *
 * La liste est volontairement restreinte aux familles réellement employées
 * dans ce dépôt. Elle évite un faux positif important : une classe CSS
 * **propre au composant**, définie dans son bloc `<style>`, peut légitimement
 * être assemblée — `class="message message-{message.role}"` dans
 * `McpChatbot.svelte` s'appuie sur `.message-user` déclaré plus bas dans le
 * même fichier, et Tailwind n'a rien à voir là-dedans.
 *
 * Un garde-fou qui crie à tort finit désactivé ; mieux vaut le tenir étroit et
 * l'élargir quand une famille manque.
 */
const RACINES_TAILWIND = [
  "bg", "text", "border", "ring", "fill", "stroke", "from", "via", "to",
  "grid-cols", "col-span", "row-span", "gap", "space-x", "space-y",
  "p", "px", "py", "pt", "pb", "pl", "pr",
  "m", "mx", "my", "mt", "mb", "ml", "mr",
  "w", "h", "min-w", "min-h", "max-w", "max-h",
  "rounded", "shadow", "opacity", "z", "order", "flex", "basis",
];

/**
 * Une accolade collée à un fragment de nom d'utilitaire Tailwind, dans les
 * deux sens : `bg-{…}` (racine suivie d'une expression) ou `{…}-50`
 * (expression suivie d'un suffixe numérique ou de nuance). Les deux
 * produisent un nom assemblé à l'exécution, que le scanner ne voit pas.
 */
const INTERPOLATION_PARTIELLE = new RegExp(
  "(?:^|\\s|:)(?:" + RACINES_TAILWIND.join("|") + ")-\\{" +
    "|\\}-(?:\\d|[a-z]+-\\d)",
  "i",
);

function fichiersDeGabarit(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeGabarit(chemin));
    } else if (EXTENSIONS.has(extname(entree)) && !entree.includes(".test.")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/** Les valeurs des attributs `class` d'une ligne. */
function attributsDeClasse(ligne: string): string[] {
  return [...ligne.matchAll(/class="([^"]*)"/g)].map((m) => m[1]);
}

describe("aucune classe Tailwind n'est construite par interpolation (#788, #789)", () => {
  it("ne trouve aucun nom de classe assemblé à l'exécution", () => {
    const fautes: string[] = [];

    for (const chemin of fichiersDeGabarit(RACINE)) {
      const lignes = readFileSync(chemin, "utf8").split("\n");
      lignes.forEach((ligne, i) => {
        const nue = ligne.trim();
        // Les commentaires qui citent le défaut ne sont pas le défaut.
        if (nue.startsWith("//") || nue.startsWith("*") || nue.startsWith("<!--")) {
          return;
        }
        for (const valeur of attributsDeClasse(ligne)) {
          if (INTERPOLATION_PARTIELLE.test(valeur)) {
            fautes.push(
              `${chemin.replace(process.cwd() + "/", "")}:${i + 1} — ${valeur.slice(0, 90)}`,
            );
          }
        }
      });
    }

    expect(
      fautes,
      `Une classe Tailwind est assemblée à l'exécution et ne sera donc jamais ` +
        `générée : le style ne partira pas, sans que rien ne le signale.\n\n` +
        `Employez des chaînes COMPLÈTES choisies par une condition ou par une ` +
        `table, par exemple :\n` +
        `  class="rounded {actif ? 'bg-green-50' : 'bg-gray-50'}"\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });
});
