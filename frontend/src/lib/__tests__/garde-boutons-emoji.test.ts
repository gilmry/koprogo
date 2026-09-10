import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Les actions de ligne passent par un bouton, pas par un caractère.
 *
 * ── Le défaut, et lequel est universel ─────────────────────────────────
 *
 * Vingt-trois boutons portaient ✏️ ou 🗑️ comme unique contenu. Deux défauts
 * s'y mêlaient, et ils n'ont pas la même portée :
 *
 * **Annonce parasite — partout.** Un émoji dans un bouton est lu par le
 * lecteur d'écran EN PLUS de l'`aria-label`. Le bouton s'appelait donc
 * « crayon Modifier ». Une icône SVG `aria-hidden` laisse l'étiquette seule
 * porter le nom, ce qui est aussi ce qui garde
 * `getByRole("button", { name })` fonctionnel dans les recettes.
 *
 * **Cible trop petite — au cas par cas.** Certains n'avaient AUCUN
 * remplissage : `<button class="text-primary-600">✏️</button>` donne une zone
 * de tap d'environ 20 px, contre 36 px de cible sur une ligne de tableau et
 * 44 px sur mobile. D'autres portaient `px-3 py-2` et allaient très bien de
 * ce côté. Ne pas confondre les deux : dire que les vingt-trois étaient trop
 * petits aurait été faux.
 *
 * ── Le cliquet ─────────────────────────────────────────────────────────
 *
 * MESURÉ le 2026-09-10 après conversion de trois composants, commentaires
 * dépouillés. Il ne peut que baisser.
 *
 * Le chiffre vient d'un comptage, pas d'un souvenir. Une heure plus tôt,
 * j'ai posé un cliquet à 4 sur une estimation quand la mesure en donnait 7,
 * et sa garde a refusé — à raison. Un cliquet non mesuré décide à l'avance de
 * ce qu'on va trouver.
 */

const RACINE = join(process.cwd(), "src");

/** Mesuré le 2026-09-10, après conversion de BuildingList, UnitList, UnitOwners. */
const EMOJIS_DACTION_AU_2026_09_10 = 17;

function sources(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      // Les tests citent l'émoji pour dire pourquoi ils l'interdisent.
      if (entree === "__tests__" || entree === "node_modules") continue;
      sources(chemin, sortie);
    } else if (/\.(svelte|astro)$/.test(entree)) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

function releve(): string[] {
  const trouves: string[] = [];
  for (const chemin of sources(RACINE)) {
    // Trois formes de commentaire. Le `(^|[^:])` épargne `https://`.
    const code = readFileSync(chemin, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/(^|[^:])\/\/[^\n]*/g, "$1");

    for (const m of code.matchAll(/✏️|🗑️/gu)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      trouves.push(
        `  ${chemin.replace(process.cwd() + "/", "")}:${ligne} — ${m[0]}`,
      );
    }
  }
  return trouves;
}

describe("les actions de ligne n'emploient pas d'émoji", () => {
  it("n'en ajoute pas", () => {
    const trouves = releve();
    expect(
      trouves.length,
      `${trouves.length} émojis d'action, contre ` +
        `${EMOJIS_DACTION_AU_2026_09_10} au 2026-09-10 :\n` +
        `${trouves.join("\n")}\n\n` +
        "Un émoji dans un bouton est annoncé EN PLUS de l'`aria-label` : le " +
        "bouton s'appelle « crayon Modifier ».\n\n" +
        'Employez `<BoutonAction nom="edit" ariaLabel={…} testId={…} />`. ' +
        "Il pose une cible de 36 px et une icône `aria-hidden`.",
    ).toBeLessThanOrEqual(EMOJIS_DACTION_AU_2026_09_10);
  });

  it("le remède existe, et exige un nom accessible", () => {
    // Vérification d'aveuglement, premier volet : si le composant
    // disparaissait, le cliquet resterait vert sans remède à proposer.
    const source = readFileSync(
      join(RACINE, "components/ui/BoutonAction.svelte"),
      "utf-8",
    );
    // `ariaLabel` sans `?` : un bouton sans texte visible qui n'aurait pas de
    // nom accessible serait muet pour un lecteur d'écran, et introuvable par
    // `getByRole("button", { name })`.
    expect(source).toMatch(/ariaLabel:\s*string;/);
    expect(source).toContain("h-9 w-9");
  });

  it("lit bien les fichiers, et n'est pas vert par vacuité", () => {
    const fichiers = sources(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);
    expect(fichiers.some((f) => f.endsWith(".svelte"))).toBe(true);
  });
});
