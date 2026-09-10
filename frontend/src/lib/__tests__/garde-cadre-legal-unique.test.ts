import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Un seul composant pour les encarts de cadre légal.
 *
 * ── Ce que ces encarts sont ─────────────────────────────────────────────
 *
 * Ils disent quelle règle de droit belge gouverne l'écran qu'on regarde. La
 * remise de design les appelle « the product's best differentiator » : aucun
 * concurrent ne dit sur quel fondement il agit.
 *
 * Ils étaient pourtant écrits à la main dans chaque module — jaunes ici, bleus
 * là, avec des structures différentes et un émoji ⚖️ en guise d'icône.
 *
 * ── Pourquoi l'émoji comptait, et pas seulement pour l'apparence ────────
 *
 * Un émoji ne peut pas être `aria-hidden` : il est **annoncé**. L'encart
 * s'ouvrait donc, pour un lecteur d'écran, par « balance Cadre légal ». Et il
 * se rend différemment selon le système, sans qu'on puisse l'accorder en
 * taille ni en graisse au reste.
 *
 * ── Ce que le composant unique apporte en plus ─────────────────────────
 *
 * Le lien « Voir la règle → » vers le registre. Un article cité sans moyen de
 * le lire demande de croire sur parole, ce qui est exactement le contraire de
 * ce que ces encarts promettent.
 *
 * ── Le cliquet ─────────────────────────────────────────────────────────
 *
 * Il ne peut que BAISSER. Quatre occurrences restent au 2026-09-10, dans des
 * composants où l'émoji sert d'autre chose qu'un encart — une méthode
 * d'envoi par huissier, une catégorie de compétence. Elles ne sont pas des
 * encarts de cadre légal et n'ont pas à devenir le composant ; elles sont
 * comptées pour que le chiffre ne remonte pas en silence.
 */

const RACINE = join(process.cwd(), "src");

/**
 * Le cliquet, MESURÉ le 2026-09-10 après unification de trois encarts.
 *
 * J'avais d'abord posé 4, de mémoire, sur un `grep` limité à
 * `src/components/**.svelte`. Il en restait sept : les pages `.astro` en
 * portaient trois de plus, dont deux vrais encarts, écrits en français dans
 * le source d'un produit servi en quatre langues.
 *
 * Poser un cliquet sans le mesurer, c'est décider à l'avance de ce qu'on va
 * trouver. La garde a refusé, et elle a eu raison.
 *
 * Les six restants ne sont PAS des encarts de cadre légal : une méthode
 * d'envoi par huissier (deux emplois), une catégorie de compétence, un
 * sous-titre d'aide, une question d'aide. Ils ont vocation à devenir des
 * icônes du jeu SVG, pas des `CadreLegal` — c'est un autre travail, et
 * le cliquet les tient d'ici là.
 */
const EMOJIS_BALANCE_AU_2026_09_10 = 6;

function sources(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      // Les tests CITENT l'émoji pour dire pourquoi ils l'interdisent.
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
    // Les commentaires expliquent souvent POURQUOI l'émoji est proscrit :
    // les compter reviendrait à punir l'explication. Trois formes, dont les
    // `//` — un cliquet de ce dépôt s'est déjà retourné contre sa propre note
    // faute de les dépouiller.
    const code = readFileSync(chemin, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/(^|[^:])\/\/[^\n]*/g, "$1");

    for (const m of code.matchAll(/⚖️/gu)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      trouves.push(`  ${chemin.replace(process.cwd() + "/", "")}:${ligne}`);
    }
  }
  return trouves;
}

describe("les encarts de cadre légal passent par un composant unique", () => {
  it("n'ajoute aucun encart écrit à la main", () => {
    const trouves = releve();
    expect(
      trouves.length,
      `${trouves.length} emplois de l'émoji ⚖️, contre ` +
        `${EMOJIS_BALANCE_AU_2026_09_10} au 2026-09-10 :\n${trouves.join("\n")}\n\n` +
        "Un émoji ne peut pas être `aria-hidden` : il est annoncé, et " +
        "l'encart s'ouvre par « balance Cadre légal » pour un lecteur " +
        "d'écran. Il se rend aussi différemment selon le système.\n\n" +
        "Employez `<CadreLegal titre corps ruleRef />`. Il apporte en plus " +
        "le lien « Voir la règle → » vers le registre : un article cité sans " +
        "moyen de le lire demande de croire sur parole.",
    ).toBeLessThanOrEqual(EMOJIS_BALANCE_AU_2026_09_10);
  });

  it("le composant unique existe, et offre le lien vers le registre", () => {
    // Vérification d'aveuglement, premier volet : si le composant disparaissait,
    // le cliquet ci-dessus resterait vert alors que le remède aurait disparu.
    const source = readFileSync(
      join(RACINE, "components/ui/CadreLegal.svelte"),
      "utf-8",
    );
    expect(source).toContain("cadre-legal-voir-la-regle");
    expect(source).toContain('nom="legalScale"');
  });

  it("lit bien les fichiers, et n'est pas vert par vacuité", () => {
    // Second volet : un répertoire renommé ou une extension oubliée rendrait
    // le compte nul, et ce zéro voudrait dire « je n'ai rien regardé ».
    const fichiers = sources(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);
    expect(fichiers.some((f) => f.endsWith(".svelte"))).toBe(true);
  });
});
