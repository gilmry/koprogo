import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : la dette d'ancrage ne grossit pas.
 *
 * ── Le constat ─────────────────────────────────────────────────────────────
 *
 * Au 2026-09-06, sur les éléments interactifs du code de production —
 * `<button>`, `<a href>`, `<input>`, `<select>`, `<textarea>`, `<form>` :
 *
 *     avec data-testid : 340
 *     sans             : 821
 *     couverture       : 29 %
 *
 * Le contrat des 882 identifiants existants est figé par
 * `garde-data-testid.test.ts`. Mais figer ne sert à rien là où il n'y a rien :
 * **821 éléments interactifs n'ont aucun ancrage**, dont 335 boutons.
 *
 * ── Pourquoi cela devient urgent ───────────────────────────────────────────
 *
 * La refonte UX (#797 à #802) va déplacer la barre latérale, les quatre
 * tableaux de bord, le motif de liste et le périmètre applicatif. Un élément
 * sans ancrage aujourd'hui sera **irrepérable demain** : on ne pourra ni
 * vérifier qu'il a survécu, ni écrire le test qui manquait.
 *
 * Et le motif dominant des défauts de ce produit, confirmé par cinq recettes
 * navigateur, est la **capacité écrite, testée et inatteignable**. Sept cents
 * boutons sans ancrage, c'est sept cents endroits où ce motif peut se loger
 * sans qu'aucun test ne puisse le voir.
 *
 * ── Pourquoi un cliquet et non une correction en bloc ──────────────────────
 *
 * Poser 821 identifiants d'un coup serait invérifiable : personne ne relit 821
 * diffs de gabarit, et un identifiant mal nommé vaut moins que pas
 * d'identifiant — il fait croire à une couverture. La dette se résorbe au fil
 * des lots, chacun ancrant ce qu'il touche et **faisant baisser ce nombre dans
 * le même commit**.
 *
 * C'est la règle déjà retenue pour `garde_ecriture` et `garde_champs_ignores`
 * côté backend, et celle que la revue de design pose elle-même.
 *
 * **Baisser DETTE_AU_2026_09_06 fait partie de la correction.** Suivi en #803.
 */

/**
 * Éléments interactifs sans `data-testid`. **Ne doit que BAISSER.**
 *
 * 821 au relevé initial du 2026-09-06 ; 806 après l'ancrage de la page
 * d'accueil publique, faite dans le même commit que ce cliquet — parce qu'un
 * cliquet posé sans une première baisse n'est qu'une constatation.
 *
 * **778** au 2026-09-07, après l'ancrage des deux écrans des personas que
 * six recettes n'ont jamais éprouvés : la page à lien magique du prestataire
 * (#815) et le tableau de bord du copropriétaire (#807).
 *
 * Le choix n'est pas arbitraire. Ancrer 806 éléments d'un coup serait
 * invérifiable — personne ne relit 806 diffs de gabarit — et un identifiant
 * mal nommé vaut moins que pas d'identifiant, puisqu'il fait croire à une
 * couverture. On ancre donc les écrans dont on a besoin, quand on en a
 * besoin, et ce sont ceux des rôles qu'on n'a jamais pu observer qui en ont
 * le plus besoin.
 *
 * Baisser ce nombre fait partie de chaque lot de la refonte : celui qui touche
 * un écran l'ancre.
 */
/// Éléments interactifs sans ancrage. **Ne doit que BAISSER.**
///
/// 672 au 2026-09-06, **343 au 2026-09-08**. Les trois cent douze posés le sont
/// sur les écrans que #803 nomme en priorité, et selon la convention relevée
/// le même jour : `<domaine>-<objet>-<rôle>`, en casse kebab.
///
/// **Le cycle de vie d'une AG est intégralement ancré** : assemblées,
/// résolutions, votes, convocations et destinataires ne comptent plus un seul
/// élément interactif nu. C'est le premier des quatre parcours de recette de
/// #803 à être complet, et c'est celui de #780 — les trois verrous qu'une
/// recette navigateur doit pouvoir franchir.
///
/// **La saisie comptable l'est aussi**, depuis le 2026-09-08 : écritures,
/// dépenses, appels de fonds, relances, factures, budgets, paiements et
/// rapports financiers ne comptent plus un seul élément interactif nu. C'est
/// un parcours bien plus large que celui de l'AG — cent un éléments contre
/// quinze — et le second des quatre de #803 à être complet.
///
/// **La création d'immeuble et de lot l'est également** : immeubles, lots,
/// copropriétaires, liens lot↔propriétaire, ACP et contributions ne comptent
/// plus un seul élément interactif nu.
///
/// **Les modules communautaires le sont aussi** — SEL, annonces, compétences,
/// partage, réservations, gamification.
///
/// ── Les quatre parcours de recette de #803 sont donc complets ────────────
///
/// L'issue demandait de « prioriser les parcours de recette : création
/// d'immeuble et de lot, cycle de vie d'une AG, saisie comptable, modules
/// communautaires. Ce sont ceux que Cowork parcourt, et où l'absence
/// d'ancrage lui a déjà coûté. »
///
/// La dette est à zéro. Chaque élément interactif du produit porte une
/// ancre : boutons, champs, listes déroulantes, formulaires et liens, sur
/// l'application comme sur les pages publiques. 755 au premier relevé.
///
/// Le cliquet devient donc une interdiction. Il reste écrit avec
/// `toBeLessThanOrEqual` et non `toBe` : la forme du cliquet vaut par
/// elle-même, et un décompte qui baisserait pour une bonne raison — un écran
/// retiré — ne doit pas faire échouer la garde.
///
/// Trois exclusions, chacune avec sa raison écrite plus bas :
///
///   - les liens `mailto:` et `tel:`, qu'aucune recette ne pilote ;
///   - les primitives qui répandent `{...restProps}`, ancrables par leur
///     appelant, et tenues de le rester par le test de passe-plat ;
///   - les éléments cités en commentaire, qui ne sont pas du code.
///
/// Aucune ancre n'a été inventée au hasard : chacune nomme l'écran, l'objet
/// et le rôle, ou dérive de l'`id` que l'élément portait déjà. Un identifiant
/// mal nommé vaut moins que pas d'identifiant, puisqu'il fait croire à une
/// couverture.
const DETTE_AU_2026_09_06 = 0;

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".astro"]);

/**
 * Balises ouvrantes d'éléments avec lesquels un utilisateur interagit.
 *
 * On ne peut PAS s'arrêter au premier `>` : une fonction fléchée en attribut
 * en contient un.
 *
 *     <button onclick={() => close()} data-testid="x">
 *                             ↑ ici
 *
 * Le motif `[^>]*?` tronquait la balise à cette flèche, et tout `data-testid`
 * placé APRÈS devenait invisible. **Quatre-vingt-quatre éléments ancrés
 * étaient ainsi comptés comme non ancrés** — 755 annoncés pour 671 réels.
 *
 * `balisesInteractives` suit donc les accolades et ne s'arrête qu'au `>` de
 * fermeture réel.
 *
 * Un détecteur qui accuse à tort use la même chose qu'un détecteur aveugle :
 * la confiance qu'on lui accorde. Ici, il poussait à ancrer une seconde fois
 * un élément déjà ancré — j'ai créé un attribut en double avant de le voir.
 */
function* balisesInteractives(
  texte: string,
): Generator<{ nom: string; balise: string; index: number }> {
  const debut = /<(button|input|select|textarea|form|a)(?=[\s>])/g;
  let m: RegExpExecArray | null;
  while ((m = debut.exec(texte)) !== null) {
    let i = debut.lastIndex;
    let profondeur = 0;
    while (i < texte.length) {
      const c = texte[i];
      if (c === "{") profondeur += 1;
      else if (c === "}") profondeur -= 1;
      else if (c === ">" && profondeur === 0) break;
      i += 1;
    }
    yield {
      nom: m[1],
      balise: texte.slice(m.index, i + 1),
      index: m.index,
    };
  }
}

function fichiersDeGabarit(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeGabarit(chemin));
    } else if (
      EXTENSIONS.has(extname(entree)) &&
      !entree.includes(".test.") &&
      !HORS_PRODUIT.has(entree)
    ) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/**
 * Retire les commentaires avant de chercher des balises.
 *
 * ── Pourquoi ────────────────────────────────────────────────────────────────
 *
 * Sans cela le détecteur compte les balises CITÉES dans un commentaire.
 * `JournalEntryList.svelte` en portait une :
 *
 * ```ts
 * // L'API attend du RFC3339, pas la date nue du champ `<input type=date>`.
 * ```
 *
 * Ce commentaire était relevé comme un `<input>` sans ancrage, et il aurait
 * fallu l'ancrer pour faire baisser le cliquet — c'est-à-dire ancrer une
 * phrase.
 *
 * C'est le même défaut que la garde d'identité du backend, qui comptait un
 * commentaire mentionnant `AuthenticatedUser` comme une preuve d'identité.
 * Deux gardes, deux jours, la même cause : **elles lisaient ce que le code
 * DIT au lieu de ce qu'il FAIT.**
 *
 * Les lignes sont conservées — on ne remplace que le contenu — pour que les
 * numéros de ligne du relevé restent justes.
 */
function sansCommentaires(source: string): string {
  return source
    .replace(/<!--[\s\S]*?-->/g, (bloc) => bloc.replace(/[^\n]/g, " "))
    .replace(/\/\*[\s\S]*?\*\//g, (bloc) => bloc.replace(/[^\n]/g, " "))
    .split("\n")
    .map((ligne) => (ligne.trim().startsWith("//") ? "" : ligne))
    .join("\n");
}

/**
 * Les fichiers qui vivent dans `src/` sans être du produit.
 *
 * ── Pourquoi une liste NOMMÉE et non un motif ────────────────────────────
 *
 * `BuildingListExample.svelte` porte quatorze éléments interactifs sans
 * ancrage, et son en-tête dit ce qu'il est :
 *
 * ```
 * Example: Translated Building List Component
 * This component demonstrates: …
 * ```
 *
 * **Il n'est monté nulle part.** Aucun `.astro`, aucun `.svelte`, aucun `.ts`
 * ne l'importe. Ses quatorze éléments ne seront jamais à l'écran, et les
 * ancrer reviendrait à ancrer de la documentation pour faire baisser un
 * chiffre.
 *
 * Une liste nommée plutôt qu'un motif `*Example*` : un motif se remplirait
 * tout seul, et il suffirait de renommer un composant pour le sortir de la
 * mesure. Chaque entrée est un engagement à vérifier — ce fichier n'est pas
 * du produit, et on peut le contrôler.
 *
 * C'est le même choix que la liste `PUBLIQUES` de `garde_identite_absente`,
 * et pour la même raison : une exemption sans justification se remplit
 * d'elle-même.
 */
const HORS_PRODUIT = new Set(["BuildingListExample.svelte"]);

function recenser(): { ancres: number; sansAncre: string[] } {
  let ancres = 0;
  const sansAncre: string[] = [];

  for (const chemin of fichiersDeGabarit(RACINE)) {
    const texte = sansCommentaires(readFileSync(chemin, "utf8"));
    for (const { nom, balise, index } of balisesInteractives(texte)) {
      // Une ancre sans `href` est décorative, pas un point d'interaction.
      if (nom === "a" && !balise.includes("href")) continue;
      // `mailto:` et `tel:` sortent du produit : cliquer y ouvre un client de
      // messagerie ou un téléphone, qu'aucune recette ne pilote. Les ancrer
      // reviendrait à promettre un test qu'on ne peut pas écrire.
      //
      // Onze `mailto:` et un `tel:`, presque tous dans les pages juridiques et
      // les fiches de contact du syndic. Le lien reste utile à l'utilisateur ;
      // il n'est simplement pas un point d'interaction mesurable.
      if (nom === "a" && /href=[{"'`]*(mailto|tel):/.test(balise)) continue;
      // Une primitive qui répand `{...restProps}` est ancrable par son
      // appelant : `<Button data-testid="x">` arrive jusqu'au `<button>`.
      // Lui poser une ancre fixe donnerait le même identifiant à chaque
      // bouton de l'application, ce qui est pire que pas d'ancre du tout —
      // `getByTestId` deviendrait ambigu partout à la fois.
      //
      // L'exclusion a une contrepartie, dans le test qui suit : ces cinq
      // primitives doivent CONTINUER de transmettre. Sans lui, il suffirait
      // de retirer le `{...restProps}` pour sortir du décompte en cassant
      // l'ancrage de tous les appelants.
      if (/\{\.\.\.[a-zA-Z_$][\w$]*\}/.test(balise)) continue;
      if (balise.includes("data-testid")) {
        ancres += 1;
      } else {
        const ligne = texte.slice(0, index).split("\n").length;
        sansAncre.push(
          `${chemin.replace(process.cwd() + "/", "")}:${ligne} <${nom}>`,
        );
      }
    }
  }
  return { ancres, sansAncre };
}

describe("la dette d'ancrage ne grossit pas (#803)", () => {
  it("ne laisse pas augmenter le nombre d'éléments interactifs sans data-testid", () => {
    const { ancres, sansAncre } = recenser();

    expect(
      sansAncre.length,
      `La dette d'ancrage a GROSSI : ${sansAncre.length} éléments interactifs ` +
        `sans \`data-testid\`, contre ${DETTE_AU_2026_09_06} au 2026-09-06 ` +
        `(couverture actuelle : ${Math.round((ancres * 100) / (ancres + sansAncre.length))} %).\n\n` +
        `Un élément sans ancrage ne peut pas être vérifié après la refonte, et ` +
        `c'est là que se logent les capacités écrites mais inatteignables — le ` +
        `motif dominant des défauts trouvés en cinq recettes.\n\n` +
        `Convention : \`<domaine>-<objet>-<action>\`, par exemple ` +
        `\`building-create-submit\`.\n\n` +
        `Derniers relevés :\n` +
        sansAncre.slice(-15).join("\n"),
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_06);
  });

  /// Sans ce contrôle, une refonte du répertoire rendrait le cliquet
  /// silencieusement vert en ne trouvant plus rien à compter.
  it("trouve encore des éléments interactifs à recenser", () => {
    const { ancres, sansAncre } = recenser();
    expect(
      ancres + sansAncre.length,
      "le recensement ne reconnaît plus les éléments interactifs",
    ).toBeGreaterThan(500);
    expect(
      ancres,
      "plus aucun élément ancré : le motif a changé",
    ).toBeGreaterThan(200);
  });
});

/**
 * Les primitives exclues du décompte ci-dessus le sont parce qu'elles
 * transmettent `{...restProps}` : leur appelant peut y passer une ancre. Ce
 * test est la contrepartie de l'exclusion. Sans lui, retirer le
 * `{...restProps}` ferait sortir la primitive du décompte tout en cassant
 * l'ancrage de tous ses appelants d'un coup.
 */
const PRIMITIVES_A_PASSE_PLAT = [
  "src/components/ui/AccessibleButton.svelte",
  "src/components/ui/Button.svelte",
  "src/components/ui/FormInput.svelte",
  "src/components/ui/FormSelect.svelte",
  "src/components/ui/FormTextarea.svelte",
];

describe("les primitives restent ancrables par leur appelant (#803)", () => {
  it("transmet encore les attributs jusqu'à l'élément interactif", () => {
    const fautives: string[] = [];
    for (const chemin of PRIMITIVES_A_PASSE_PLAT) {
      const source = readFileSync(join(process.cwd(), chemin), "utf-8");
      const transmet = [...balisesInteractives(source)].some((b) =>
        /\{\.\.\.[a-zA-Z_$][\w$]*\}/.test(b.balise),
      );
      if (!transmet) fautives.push(chemin);
    }
    expect(
      fautives.join("\n"),
      "Ces primitives ne répandent plus leurs attributs sur leur élément " +
        "interactif. Elles sortent du décompte d'ancrage PARCE QU'ELLES le " +
        'font : sans cela, `<Button data-testid="x">` n\'atteint plus le ' +
        "`<button>`, et tous leurs appelants perdent leur ancre en silence.",
    ).toBe("");
  });
});
