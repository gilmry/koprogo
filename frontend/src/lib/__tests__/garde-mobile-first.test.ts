import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : la coquille applicative ne repositionne plus le mobile par
 * exception.
 *
 * ── Ce que la remise de design demande ─────────────────────────────────────
 *
 * Correctif 0.10 : « **Invert the breakpoints.** The app is written
 * desktop-first (`hidden lg:flex` on the sidebar, `md:` prefixes layered over
 * a wide grid). Mobile-first means the base styles are mobile and `md:`/`lg:`
 * *add* the desktop. **Delete the `fixed top-2 right-14` overlay positioning
 * of the building selector — it only exists because mobile was handled
 * last.** »
 *
 * ── Ce que l'overlay coûtait ───────────────────────────────────────────────
 *
 * `BuildingSelectorBar` flottait en `fixed top-2 right-14` sous `lg`, entre le
 * logo et la cloche du header mobile. Son commentaire décrivait avec soin
 * comment il évitait de chevaucher trois éléments — et un composant qui doit
 * connaître la largeur du hamburger pour se placer est un composant mal placé.
 *
 * `Layout.astro` réserve déjà une gouttière de 56 px pour ce header et rend la
 * barre juste après. En flux normal elle tombe dessous, sans coordonnées et
 * sans `z-index`.
 *
 * ── Ce que ce test refuse ──────────────────────────────────────────────────
 *
 * Le retour d'un positionnement absolu sur les barres de contexte. Il ne juge
 * pas les overlays en général — un tiroir, une modale et un toast en ont
 * légitimement besoin — mais ceux qui remplacent une mise en page.
 *
 * Voir #825 pour le reste de l'inversion, qui accompagne la refonte.
 */

const CIBLES = [
  "src/components/global/BuildingSelectorBar.svelte",
  "src/components/global/ContextBanner.svelte",
];

/** Un positionnement qui sort un élément du flux pour le replacer à la main. */
const HORS_FLUX = /class="[^"]*\b(fixed|absolute)\s+(top|bottom|left|right)-/;

describe("les barres de contexte restent en flux normal (#825)", () => {
  it("ne repositionne aucune barre par coordonnées", () => {
    const fautes: string[] = [];

    for (const chemin of CIBLES) {
      let source: string;
      try {
        source = readFileSync(join(process.cwd(), chemin), "utf8");
      } catch {
        continue; // Le composant peut disparaître dans la refonte.
      }
      source.split("\n").forEach((ligne, i) => {
        const nue = ligne.trim();
        // Le commentaire qui raconte l'ancien overlay n'est pas l'overlay.
        if (
          nue.startsWith("//") ||
          nue.startsWith("*") ||
          nue.startsWith("<!--")
        ) {
          return;
        }
        if (HORS_FLUX.test(ligne)) {
          fautes.push(`${chemin}:${i + 1} — ${nue.slice(0, 80)}`);
        }
      });
    }

    expect(
      fautes,
      `Une barre de contexte est de nouveau positionnée par coordonnées.\n\n` +
        `C'est le motif que la remise de design demande de supprimer ` +
        `(correctif 0.10) : un overlay qui « n'existe que parce que le mobile ` +
        `a été traité en dernier ». Un composant qui doit connaître la largeur ` +
        `du hamburger pour se placer est un composant mal placé.\n\n` +
        `Le gabarit réserve déjà sa gouttière : laissez la barre en flux.\n\n` +
        fautes.join("\n"),
    ).toEqual([]);
  });

  /// Sans ce contrôle, renommer les fichiers rendrait le cliquet
  /// silencieusement vert — le piège déjà posé dans les onze autres.
  it("trouve encore les composants qu'il surveille", () => {
    const presents = CIBLES.filter((c) => {
      try {
        return statSync(join(process.cwd(), c)).isFile();
      } catch {
        return false;
      }
    });
    expect(
      presents.length,
      `Aucune des barres surveillées n'existe plus : ${CIBLES.join(", ")}. ` +
        `Si la refonte les a remplacées, pointez ce cliquet sur leurs ` +
        `successeurs plutôt que de le laisser vert sur rien.`,
    ).toBeGreaterThan(0);
  });
});
