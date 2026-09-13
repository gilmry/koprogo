import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Les émojis ne sont pas un système d'icônes.
 *
 * ── Ce que la mesure donne, et pourquoi c'est un cliquet ────────────────
 *
 * **674 émojis répartis sur 118 fichiers** de gabarit au 2026-09-10. La garde
 * `garde-boutons-emoji` en tenait deux formes — ✏️ et 🗑️ — ramenées à zéro le
 * même jour. La classe réelle est cinquante fois plus grande.
 *
 * Un cliquet, donc, et pas une interdiction : convertir six cent soixante-
 * quatorze occurrences en une passe serait pire que de ne rien faire, parce
 * que la transformation N'EST PAS uniforme. L'expérience du matin en a
 * distingué trois :
 *
 * 1. **Bouton à icône seule** → composant `BoutonAction` : la cible passe de
 *    ~20 px à 36 px, et le nom accessible devient obligatoire.
 * 2. **Émoji à côté d'un libellé** → `Icone` en ligne, texte conservé. La
 *    cible et le nom étaient déjà bons ; seule l'annonce parasite gênait.
 *    Traiter ce cas comme le premier SUPPRIMERAIT des libellés lisibles.
 * 3. **Émoji dans une chaîne de gabarit** — `` `🗑️ ${…}` `` — le plus
 *    coriace : aucun `aria-hidden` ne s'applique à un caractère concaténé. Il
 *    faut sortir l'icône de la chaîne pour qu'elle redevienne un élément.
 *
 * ── Les trois reproches, et lesquels sont vérifiables ──────────────────
 *
 * **Ils sont annoncés par les lecteurs d'écran.** Un émoji ne peut pas porter
 * `aria-hidden` : il n'est pas un élément. Un lien vers les immeubles
 * s'appelait « bâtiment Immeubles ».
 *
 * **Ils se rendent différemment selon le système.** Un émoji est une police,
 * pas un dessin : le produit n'a pas la même allure sur macOS, Windows et
 * Android, et rien ne le contrôle.
 *
 * **Ils entrent en collision.** Mesuré sur la navigation avant conversion :
 * 42 entrées de menu pour 33 icônes distinctes, dont six collisions entre
 * destinations différentes. Deux entrées au même signe ne se distinguent plus
 * que par leur libellé.
 *
 * ── Ce que cette garde ne dit pas ──────────────────────────────────────
 *
 * Elle ne juge pas du CONTENU : un émoji dans un message de journal, dans une
 * chaîne de test ou dans un commentaire ne la concerne pas — les commentaires
 * sont dépouillés avant comptage. Elle porte sur ce que l'utilisateur voit.
 */

const RACINE = join(process.cwd(), "src");

/**
 * Mesuré le 2026-09-10, commentaires dépouillés.
 *
 * Le chiffre vient d'un comptage, jamais d'un souvenir. Un cliquet posé de
 * mémoire décide à l'avance de ce qu'on va trouver — c'est arrivé le même
 * jour, à 4 quand la mesure en donnait 7, et la garde a refusé.
 */
const EMOJIS_AU_2026_09_10 = 674;

/** Émojis pictographiques et symboles divers. */
const EMOJI = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}\u{2B00}-\u{2BFF}]/gu;

function gabarits(dossier: string, sortie: string[] = []): string[] {
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      // Les tests CITENT les émojis pour dire pourquoi ils les interdisent.
      if (entree === "__tests__" || entree === "node_modules") continue;
      gabarits(chemin, sortie);
    } else if (/\.(svelte|astro)$/.test(entree)) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

function compter(): { total: number; parFichier: [string, number][] } {
  const parFichier: [string, number][] = [];
  let total = 0;

  for (const chemin of gabarits(RACINE)) {
    // Trois formes de commentaire. Le `(^|[^:])` épargne `https://`.
    const code = readFileSync(chemin, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/(^|[^:])\/\/[^\n]*/g, "$1");

    const n = (code.match(EMOJI) ?? []).length;
    if (n > 0) {
      parFichier.push([chemin.replace(process.cwd() + "/", ""), n]);
      total += n;
    }
  }
  parFichier.sort((a, b) => b[1] - a[1]);
  return { total, parFichier };
}

describe("les émojis des gabarits ne se multiplient pas", () => {
  it("n'en ajoute pas", () => {
    const { total, parFichier } = compter();
    const pires = parFichier
      .slice(0, 8)
      .map(([f, n]) => `  ${String(n).padStart(3)}  ${f}`)
      .join("\n");

    expect(
      total,
      `${total} émojis dans les gabarits, contre ${EMOJIS_AU_2026_09_10} au ` +
        `2026-09-10. Les plus chargés :\n${pires}\n\n` +
        "Un émoji ne peut pas porter `aria-hidden` : il est ANNONCÉ par les " +
        "lecteurs d'écran. Il se rend aussi différemment selon le système, et " +
        "deux entrées au même signe cessent de se distinguer.\n\n" +
        'Employez `<Icone nom="…" />` pour un pictogramme décoratif, ou ' +
        "`<BoutonAction …/>` pour une action de ligne. Attention : la " +
        "transformation N'EST PAS uniforme — un émoji à côté d'un libellé " +
        "garde son texte, sinon vous supprimez un mot lisible pour corriger " +
        "un défaut d'accessibilité.",
    ).toBeLessThanOrEqual(EMOJIS_AU_2026_09_10);
  });

  it("lit bien les gabarits, et n'est pas vert par vacuité", () => {
    // Vérification d'aveuglement : un répertoire renommé ou une extension
    // oubliée rendrait le compte nul, et ce zéro voudrait dire « je n'ai rien
    // regardé » plutôt que « rien à signaler ».
    const fichiers = gabarits(RACINE);
    expect(fichiers.length).toBeGreaterThan(100);
    expect(fichiers.some((f) => f.endsWith(".svelte"))).toBe(true);
    expect(fichiers.some((f) => f.endsWith(".astro"))).toBe(true);

    // Et le motif doit reconnaître un émoji : sans cela le compte serait nul
    // quoi qu'il arrive.
    expect("🏢".match(EMOJI)?.length).toBe(1);
  });
});
