import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : aucune valeur d'énumération ne s'affiche telle quelle.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Recette du 2026-09-06. Les statuts de tickets s'affichaient en valeurs
 * internes : `Open`, `InProgress`, `Resolved`, `Closed`. Et les catégories
 * de même : `Plumbing` là où l'utilisateur attend « Plomberie ».
 *
 * Le défaut se logeait précisément là où on ne l'attendait pas : le composant
 * `TicketStatusBadge` fait la traduction correctement depuis toujours, et
 * `OwnerDashboard` **recopiait sa cascade de couleurs** sans recopier sa
 * traduction. Deux défauts d'un coup — de la duplication, et une valeur brute
 * — sur l'écran du rôle que six recettes n'ont jamais éprouvé.
 *
 * ── Pourquoi aucun test ne le voyait ───────────────────────────────────────
 *
 * Un composant rend fidèlement la chaîne qu'on lui donne. Un test de rendu
 * passe aussi bien avec « InProgress » qu'avec « En cours » : il compare ce
 * qui est affiché à ce qu'on lui a dit d'attendre, et les deux viennent de la
 * même source.
 *
 * C'est un défaut **lexical**, il se détecte en lisant le gabarit.
 *
 * ── Ce que ce test refuse ──────────────────────────────────────────────────
 *
 * Une interpolation qui rend directement un champ dont le nom trahit une
 * énumération — `status`, `category`, `type`, `priority` — sans passer par une
 * traduction ni par un composant dédié.
 *
 * Voir #792 et #786, où la même forme avait produit `notices.draft` en clair.
 */

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".astro"]);

/**
 * Les champs dont la valeur vient d'une énumération du serveur.
 *
 * La liste est étroite à dessein : un garde-fou qui crie à tort finit
 * désactivé. On vise les quatre noms qui ont réellement produit un défaut.
 */
const CHAMPS_ENUMERES =
  /\{\s*[\w.]*\.(status|category|priority|notice_type|vote_choice)\s*\}/;

/** Ce qui atteste qu'une traduction ou un composant s'en occupe. */
const TRADUIT = /\$_\(|Badge|badge|libelle|Libelle/;

/**
 * Ce qui n'affiche rien, malgré l'apparence.
 *
 * `meetingStatus={meeting.status}` PASSE la valeur à un composant, qui la
 * traduira ou non — c'est son affaire, pas celle de l'appelant. `bind:value`
 * de même : un `<select>` porte des valeurs internes dans ses `<option>`, et
 * c'est correct.
 *
 * Un garde-fou qui les compterait crierait à tort, et un garde-fou qui crie à
 * tort finit désactivé.
 */
const NAFFICHE_RIEN = /\w+=\{|bind:value=|:value=/;

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

describe("aucune énumération ne s'affiche en valeur interne (#792)", () => {
  it("ne rend jamais un statut, une catégorie ou une priorité sans traduction", () => {
    const fautes: string[] = [];

    for (const chemin of fichiersDeGabarit(RACINE)) {
      const lignes = readFileSync(chemin, "utf8").split("\n");
      lignes.forEach((ligne, i) => {
        const nue = ligne.trim();
        // Un commentaire qui cite le défaut n'est pas le défaut.
        if (
          nue.startsWith("//") ||
          nue.startsWith("*") ||
          nue.startsWith("<!--")
        ) {
          return;
        }
        // Hors du gabarit, une interpolation n'affiche rien.
        if (!ligne.includes("<") && !ligne.includes(">")) return;

        if (
          CHAMPS_ENUMERES.test(ligne) &&
          !TRADUIT.test(ligne) &&
          !NAFFICHE_RIEN.test(ligne)
        ) {
          fautes.push(
            `${chemin.replace(process.cwd() + "/", "")}:${i + 1} — ${nue.slice(0, 80)}`,
          );
        }
      });
    }

    expect(
      fautes,
      `${fautes.length} valeur(s) d'énumération affichée(s) telle(s) quelle(s).\n\n` +
        `L'utilisateur lit « InProgress » ou « Plumbing » au lieu de « En ` +
        `cours » ou « Plomberie ». Aucun test de rendu ne l'attrape : le ` +
        `composant rend fidèlement ce qu'on lui donne (#792).\n\n` +
        `Passez par \`$_('…')\` ou par le composant de badge dédié, qui porte ` +
        `les couleurs ET la traduction — recopier l'un sans l'autre est ` +
        `exactement ce qui a produit le défaut.\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });
});
