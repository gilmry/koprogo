import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Une recette ne doit jamais chercher un élément par sa FORMULATION.
 *
 * Le guide de style le dit déjà (§ 3, « Pourquoi ancrer plutôt que nommer ») :
 * une assertion peut exiger une ancre, une valeur interpolée ou une structure,
 * jamais une formulation. Rien ne l'imposait ; cette garde l'impose.
 *
 * Ce que coûte l'infraction, mesuré et non supposé :
 *
 *   - `playwright.config.ts` déclare quatre projets actifs, tous épinglés à
 *     `locale: "fr-BE"`. La ligne 172 porte l'aveu : « Force French locale so
 *     nav testids match hardcoded expectations ». Quelqu'un a buté sur le
 *     problème et l'a contourné en figeant la langue.
 *   - KoproGo est livré en quatre langues (fr, nl, de, en : `src/lib/i18n.ts`).
 *     Aucun projet Playwright n'en exerce trois. Elles n'ont, de bout en bout,
 *     jamais été ouvertes.
 *   - On ne peut pas les ouvrir sans retirer d'abord ces formulations : un
 *     projet `nl-BE` ferait échouer chaque spec qui cherche « Approuver ».
 *
 * **Ce cliquet a dû être élargi deux fois, et chaque fois parce qu'un échec
 * réel a montré ce qu'il ne voyait pas.** Écrit en regardant `getByText`,
 * `getByRole` et `text=`, il ignorait `filter({ hasText })` puis les
 * expressions régulières. Le chiffre monte donc en corrigeant la garde, pas
 * en dégradant le dépôt : 79 relevés, 78 après deux corrections de
 * sélecteurs, 106 une fois les expressions régulières comptées, sous LEURS DEUX formes — la
 * propriété `hasText: /.../` et l'appel `getByText(/.../)`, que mon premier
 * motif manquait.
 *
 * C'est le reproche que cette journée a fait à six gardes existantes —
 * écrites en regardant le défaut trouvé, pas la classe de défauts — et il
 * valait pour celle-ci.
 *
 * 79 au premier relevé, 78 après avoir étendu la garde à
 * `filter({ hasText: ... })` — qui en ajoutait un — et corrigé les deux
 * sélecteurs par libellé de `SyndicDocumentsJourney.spec.ts`, dont les ancres
 * existaient depuis l'ancrage du même jour.
 *
 * Sur les 79 relevés d'origine, 72 visent un mot traduit — c'est la part qui tient les
 * trois langues fermées. Les 7 autres visent un emoji (`text=🔴`, `text=💰`) :
 * ceux-là ne parient pas sur la langue, ils parient sur la présentation, et
 * la revue de design a justement relevé que cinq emojis de navigation servent
 * chacun deux entrées de menu. Ils sont comptés, mais pour l'autre raison.
 *
 * Le cliquet borne donc une dette qui n'est pas un détail de style : c'est ce
 * qui tient trois langues hors de portée de la recette.
 */

const RACINE_E2E = "tests/e2e";

/** Le décompte relevé le 2026-09-08. Il ne doit que baisser. */
// 106 → 121 : la garde ne lisait que `.spec.ts` et manquait les douze
// `.scenario.ts`. Quatrième fois que ce cliquet monte en se corrigeant.
const DETTE_AU_2026_09_08 = 118;

/** Le nombre de specs ce jour-là : on ne solde pas la dette en les supprimant. */
const SPECS_AU_2026_09_08 = 100;

function specs(racine: string): string[] {
  const trouvees: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouvees.push(...specs(chemin));
    // `.scenario.ts` AUTANT que `.spec.ts`.
    //
    // Le filtre ne retenait que `.spec.ts`, et les douze fichiers du projet
    // `scenarios` s'appellent `.scenario.ts` : la garde ne les lisait pas.
    // Elle a donc laissé passer `filter({ hasText: /vote/i }).last()`, qui a
    // fait cliquer « Voir les votes » au lieu de « Soumettre le vote », et
    // `text=2026` qui résolvait deux éléments dont le copyright du pied de
    // page.
    //
    // Ce sont précisément les recettes filmées comme documentation vivante :
    // les moins surveillées étaient les plus visibles.
    else if (entree.endsWith(".spec.ts") || entree.endsWith(".scenario.ts"))
      trouvees.push(chemin);
  }
  return trouvees;
}

/**
 * Les commentaires ne sont pas du code. Deux gardes de ce dépôt ont déjà
 * compté des exemples cités en commentaire comme des infractions réelles.
 */
function sansCommentaires(source: string): string {
  return source.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/[^\n]*/g, "");
}

/**
 * Un littéral de chaîne figé — guillemets simples, doubles, ou gabarit SANS
 * interpolation. Un gabarit qui contient `${` porte une valeur, pas une
 * formulation : le guide de style l'autorise explicitement.
 */
const LITTERAL_FIGE = String.raw`(?:"[^"\n]*"|'[^'\n]*'|\`[^\`$\n]*\`)`;

const MOTIFS: { nom: string; motif: RegExp }[] = [
  {
    nom: "getByText / getByLabel / getByPlaceholder",
    motif: new RegExp(
      String.raw`getBy(?:Text|Label|Placeholder)\(\s*${LITTERAL_FIGE}`,
      "g",
    ),
  },
  {
    nom: "getByRole(..., { name: ... })",
    // `name:` doit être borné à l'intérieur d'un `getByRole(` : le mot est
    // aussi une clé de données de test (`{ name: "Immeuble test" }`), et le
    // capturer partout gonflait la dette de 65 à 144.
    motif: new RegExp(
      String.raw`getByRole\([^)]*?name:\s*${LITTERAL_FIGE}`,
      "g",
    ),
  },
  {
    nom: "sélecteur text= ou :has-text(...)",
    motif: new RegExp(
      // `text=` s'arrête au premier `$` : `text=${user.email}` porte une
      // valeur, pas une formulation, et le guide de style l'autorise.
      String.raw`(?:text=[^"'\`$\n]+|:has-text\(\s*${LITTERAL_FIGE})`,
      "g",
    ),
  },
];

/**
 * `filter({ hasText: "..." })` était l'angle mort de cette garde.
 *
 * C'est le même pari sur la langue que `getByText`, et il mordait : le
 * scénario `poll-vote` cherchait un `button` contenant « Oui ». Douze usages
 * existaient sans qu'aucun ne soit compté.
 *
 * Mais dix d'entre eux sont LÉGITIMES : ils visent une donnée que le test
 * vient lui-même de créer — « Reparation toiture », « Barbecue de quartier »,
 * « repeindre le hall ». Le guide de style autorise expressément une valeur ;
 * ce qu'il interdit est une formulation du produit.
 *
 * Le discriminant est mécanique : une valeur créée par le test apparaît
 * AILLEURS dans le même fichier — dans le `data:` du POST qui l'a créée. Un
 * libellé du produit n'apparaît qu'ici. Sur les douze, un seul est dans ce
 * cas : « Nouveau document ».
 */
function hasTextDeLibelle(code: string): RegExpMatchArray[] {
  const trouves: RegExpMatchArray[] = [];
  for (const m of code.matchAll(/hasText:\s*(["'`])([^"'`\n$]+)\1/g)) {
    const valeur = m[2];
    const echappee = valeur.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const total = (code.match(new RegExp(echappee, "g")) ?? []).length;
    const dansHasText = (
      code.match(new RegExp(`hasText:\\s*["'\`]${echappee}`, "g")) ?? []
    ).length;
    if (total - dansHasText === 0) trouves.push(m);
  }
  return trouves;
}

function infractions(): { fichier: string; motif: string; extrait: string }[] {
  const trouvees: { fichier: string; motif: string; extrait: string }[] = [];
  for (const fichier of specs(RACINE_E2E)) {
    const code = sansCommentaires(readFileSync(fichier, "utf-8"));
    for (const { nom, motif } of MOTIFS) {
      for (const m of code.matchAll(motif)) {
        trouvees.push({ fichier, motif: nom, extrait: m[0].slice(0, 60) });
      }
    }
    // Les EXPRESSIONS RÉGULIÈRES sur du texte visible étaient exclues, au
    // motif qu'elles ne sont pas une formulation figée. C'est faux :
    // `/démarrer|start/i` est un pari sur la langue, simplement étalé sur
    // deux d'entre elles. Plusieurs de ces motifs sont d'ailleurs des
    // contournements bilingues explicites — quelqu'un avait vu le problème et
    // l'a évité au lieu d'ancrer. Aucun ne survit au néerlandais.
    //
    // Le cas qui a forcé cette extension : `meeting-vote` prenait
    // `filter({ hasText: /vote/i }).last()` pour soumettre un vote, et
    // attrapait « Voir les votes (0) » — ambigu jusque DANS la langue choisie.
    //
    // Un motif sans aucune lettre (`/^\d+$/`) est structurel, pas une
    // formulation : il ne compte pas.
    // Deux formes : la propriété (`hasText: /.../`, `name: /.../`) et l'appel
    // (`getByText(/.../)`, `filter({ hasText: /.../ })`). Mon premier motif
    // n'acceptait que la propriété, à cause du `:` — c'est un témoin qui l'a
    // montré, en ajoutant un `getByText(/inexistant/i)` que la garde n'a pas
    // vu.
    for (const m of code.matchAll(
      /(?:(?:hasText|name):\s*|getBy(?:Text|Label|Placeholder)\(\s*)(\/[^/\n]+\/[gimsuy]*)/g,
    )) {
      if (!/[a-zA-ZÀ-ÿ]/.test(m[1])) continue;
      trouvees.push({
        fichier,
        motif: "expression régulière sur du texte visible",
        extrait: m[0].slice(0, 60),
      });
    }
    for (const m of hasTextDeLibelle(code)) {
      trouvees.push({
        fichier,
        motif: "filter({ hasText: ... })",
        extrait: m[0].slice(0, 60),
      });
    }
  }
  return trouvees;
}

describe("les recettes ne cherchent pas par formulation (#803)", () => {
  it("ne crée pas de nouveau sélecteur fondé sur un libellé figé", () => {
    const trouvees = infractions();
    const parFichier = new Map<string, number>();
    for (const i of trouvees)
      parFichier.set(i.fichier, (parFichier.get(i.fichier) ?? 0) + 1);
    const pires = [...parFichier.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, 8)
      .map(([f, n]) => `  ${n.toString().padStart(3)}  ${f}`)
      .join("\n");

    expect(
      trouvees.length,
      `${trouvees.length} sélecteurs cherchent un élément par sa formulation, ` +
        `contre ${DETTE_AU_2026_09_08} au 2026-09-08.\n\n` +
        `Chercher « Approuver » est un pari sur la langue résolue. Il ne tient ` +
        `aujourd'hui que parce que les quatre projets Playwright sont épinglés ` +
        `à fr-BE, et c'est ce qui empêche d'en ouvrir un en nl, de ou en.\n\n` +
        `Si le sélecteur vise un emoji, l'objection est autre : cinq emojis de ` +
        `navigation désignent déjà deux entrées de menu chacun.\n\n` +
        `Visez une ancre \`data-testid\`, une valeur interpolée, ou une ` +
        `structure.\n\nLes plus chargés :\n${pires}`,
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_08);
  });

  it("ne solde pas la dette en supprimant des recettes", () => {
    expect(
      specs(RACINE_E2E).length,
      "le nombre de specs a baissé : la dette peut avoir été « résolue » en " +
        "retirant des recettes plutôt qu'en corrigeant leurs sélecteurs.",
    ).toBeGreaterThanOrEqual(SPECS_AU_2026_09_08);
  });

  it("lit encore les specs et sait y voir des sélecteurs", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite. Il vérifie
    // que le détecteur ouvre bien des fichiers et qu'il y trouve des
    // sélecteurs — la forme recommandée, celle qui subsistera quand la dette
    // sera à zéro.
    const fichiers = specs(RACINE_E2E);
    expect(fichiers.length).toBeGreaterThan(0);
    const ancres = fichiers
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/getByTestId\(|\[data-testid/g);
    expect(
      ancres?.length ?? 0,
      "plus aucune requête par ancre dans les specs : le répertoire a bougé, " +
        "ou l'API Playwright a changé. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(100);
  });
});
