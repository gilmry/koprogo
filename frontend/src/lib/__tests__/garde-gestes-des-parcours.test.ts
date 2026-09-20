/**
 * Un parcours de valeur AGIT. Il ne regarde pas.
 *
 * ── Le défaut que ce cliquet borne ────────────────────────────────────────
 *
 * Arbitrage du PO, le 2026-09-20, dans ses mots : « la vitrine se consacre
 * sur les écrans, parfois elle dit *sans cliquer* / *pour avoir testé
 * l'interface* ».
 *
 * Le reproche était mesurable, et la mesure était pire que l'impression. Sur
 * les sept parcours métier filmés à cette date, **43 étapes** produisaient :
 *
 *     7 clics · 3 saisies · 2 sélections · 25 navigations
 *
 * Autrement dit : douze gestes pour sept personas. Le motif dominant était
 * « aller à une URL, attendre, vérifier qu'un bouton s'affiche ». Un bouton
 * visible n'est pas un bouton qui marche — et c'est précisément ce que
 * `incident.journey.ts` a démontré en devenant le premier parcours à cliquer
 * le cycle entier : trois transitions sur cinq étaient MORTES depuis
 * toujours, et aucun écran ne le disait (#977).
 *
 * ── Ce que ce cliquet garantit, et ce qu'il ne garantit pas ───────────────
 *
 * Il compte des appels dans du texte. Il ne juge pas si le geste est
 * pertinent, ni si l'assertion qui suit prouve quoi que ce soit — un test ne
 * rend pas ce verdict-là.
 *
 * Ce qu'il empêche est plus étroit et plus utile : que la vitrine RETOMBE
 * vers la contemplation. Le total des gestes ne peut que monter, et tout
 * parcours métier neuf doit agir au lieu de constater.
 *
 * ── Pourquoi une mesure sur le texte, et pas à l'exécution ────────────────
 *
 * Compter à l'exécution demanderait de jouer les parcours, donc une pile
 * complète, donc deux minutes et demie par parcours. Ce cliquet tourne en
 * quelques millisecondes dans la suite unitaire, là où une dérive se voit
 * le jour où on l'écrit — pas trois semaines plus tard en CI.
 *
 * La contrepartie est assumée : un parcours qui appellerait `scene.cliquer`
 * dans une boucle serait compté une fois. C'est une borne, pas un recensement.
 */
import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

const JOURNEYS = resolve(__dirname, "../../..", "tests/e2e/journeys");

/**
 * Les gestes : ce qu'un utilisateur FAIT.
 *
 * `devenir()` n'en est pas un au sens de ce cliquet, bien qu'il clique
 * réellement (il remplit un formulaire de connexion). Le compter donnerait à
 * tout parcours au moins un point gratuit, et un parcours qui se connecte
 * puis regarde reste un parcours qui regarde.
 */
const GESTES = [
  "cliquer",
  "cliquerLePremier",
  "saisir",
  "choisir",
  "choisirQuiContient",
] as const;

/** Les regards : arriver quelque part, attendre, constater. */
const REGARDS = ["aller", "survoler"] as const;

/**
 * Compte les APPELS, pas les mentions.
 *
 * L'ancrage sur `.nom(` écarte deux faux positifs qui se seraient présentés
 * tout de suite : le nom cité dans la prose d'un en-tête, et la définition
 * de la méthode dans `scene.ts` — lequel n'est de toute façon pas un
 * `*.journey.ts`.
 *
 * `\b` avant le point évite qu'un `choisir` compte aussi pour
 * `choisirQuiContient` : la frontière est le point, et `(` ferme le nom.
 */
function compter(source: string, motifs: readonly string[]): number {
  return motifs.reduce((total, nom) => {
    const rx = new RegExp(`\\.${nom}\\s*\\(`, "g");
    return total + (source.match(rx)?.length ?? 0);
  }, 0);
}

interface Mesure {
  readonly fichier: string;
  readonly gestes: number;
  readonly regards: number;
}

/**
 * Les parcours MÉTIER. Les balayages en sont exclus, et c'est voulu.
 *
 * Un balayage ouvre 93 écrans pour dire lesquels se rendent : son travail EST
 * de regarder, et l'arbitrage du PO du 2026-09-19 le pose ainsi — « décrire,
 * ne rien casser ». Lui demander de cliquer le transformerait en autre chose.
 */
function parcoursMetier(): Mesure[] {
  return readdirSync(JOURNEYS)
    .filter((f) => f.endsWith(".journey.ts") && !f.startsWith("balayage-"))
    .sort()
    .map((fichier) => {
      const source = readFileSync(resolve(JOURNEYS, fichier), "utf8");
      return {
        fichier,
        gestes: compter(source, GESTES),
        regards: compter(source, REGARDS),
      };
    });
}

/**
 * Mesuré le 2026-09-20, APRÈS l'arrivée de `incident.journey.ts`.
 *
 *     avant #974 :  12 gestes / 7 parcours
 *     après        :  voir le plancher ci-dessous
 *
 * Ce nombre ne doit que MONTER. Il valait 12 avant `incident.journey.ts`
 * et l'enrichissement du parcours comptable. Il est écrit en dur, et pas calculé : un
 * cliquet qui recalcule sa référence à chaque exécution vaut toujours sa
 * valeur courante et ne peut jamais mordre.
 */
const GESTES_AU_2026_09_20 = 89;

/**
 * Le plancher par parcours. Trois gestes, c'est le minimum en dessous duquel
 * un parcours ne démontre plus qu'il a ouvert un écran.
 *
 * Les parcours qui n'y sont pas encore sont nommés ici avec ce qui leur
 * manque. Une exception NOMMÉE se solde ; une exception implicite s'oublie.
 */
const PLANCHER_PAR_PARCOURS = 3;

const EN_DETTE: Record<string, string> = {
  "conseil.journey.ts":
    "le conseil lit des documents et un ordre du jour ; ses gestes " +
    "d'approbation ne sont pas câblés (#805)",
  "moderation.journey.ts":
    "le modérateur est REFUSÉ par les sept routes communautaires : il n'y a " +
    "rien à cliquer tant que #962 n'est pas tranchée",
  "prestataire.journey.ts":
    "le prestataire n'a pas d'écran (`ROLES_SANS_INTERFACE`) ; son point " +
    "d'entrée est un lien magique qui n'existe pas (story 3.2)",
  "perimetre-multi-role.journey.ts":
    "démontre un cloisonnement, qui se constate plus qu'il ne se clique",
};

describe("les parcours filmés agissent (#974)", () => {
  it("le total des gestes ne redescend pas", () => {
    const mesures = parcoursMetier();
    const total = mesures.reduce((n, m) => n + m.gestes, 0);

    expect(
      total,
      `Les parcours filmés ne totalisent plus que ${total} gestes.\n\n` +
        mesures
          .map(
            (m) => `  ${m.fichier.padEnd(34)} ${m.gestes} gestes · ` +
              `${m.regards} regards`,
          )
          .join("\n") +
        `\n\nUne vitrine qui perd ses gestes redevient une galerie de ` +
        `captures. Le PO l'a dit une fois ; le cliquet est là pour qu'il ` +
        `n'ait pas à le redire.`,
    ).toBeGreaterThanOrEqual(GESTES_AU_2026_09_20);
  });

  it("@negative aucun parcours neuf ne se contente de regarder", () => {
    const paresseux = parcoursMetier()
      .filter((m) => m.gestes < PLANCHER_PAR_PARCOURS)
      .filter((m) => !(m.fichier in EN_DETTE));

    expect(
      paresseux.map((m) => `${m.fichier} (${m.gestes} gestes)`),
      `Un parcours métier neuf totalise moins de ${PLANCHER_PAR_PARCOURS} ` +
        `gestes.\n\n` +
        `Soit il agit, soit il rejoint \`EN_DETTE\` avec la raison écrite : ` +
        `ce qui l'empêche de cliquer, et l'issue qui le débloquera. Une ` +
        `dette nommée se solde ; une dette implicite s'oublie.`,
    ).toEqual([]);
  });

  it("@edge les parcours en dette existent encore, et leur raison aussi", () => {
    // Une exception qui survit à la disparition de son parcours est une
    // exception qui protège du vide. Elle finirait par couvrir un vrai
    // défaut le jour où un fichier reprend ce nom.
    const presents = new Set(parcoursMetier().map((m) => m.fichier));
    const fantomes = Object.keys(EN_DETTE).filter((f) => !presents.has(f));

    expect(
      fantomes,
      "Ces parcours n'existent plus mais gardent leur exception : la retirer " +
        "de `EN_DETTE` fait partie de leur suppression.",
    ).toEqual([]);
  });

  it("@security au moins un parcours mène une écriture de bout en bout", () => {
    // La distinction qui compte. Un parcours peut cliquer beaucoup sans rien
    // créer — ouvrir des modales, plier des menus, filtrer des listes. Ce
    // qui prouve la valeur est qu'une donnée SOIT née d'un geste et qu'on la
    // retrouve ensuite.
    //
    // `incident` le fait : la copropriétaire remplit le formulaire, envoie,
    // et le syndic RETROUVE le signalement dans sa liste à l'étape suivante.
    // C'est ce chaînage qui a débusqué #977.
    const sources = parcoursMetier().map((m) =>
      readFileSync(resolve(JOURNEYS, m.fichier), "utf8"),
    );
    const avecEcriture = sources.filter(
      (s) => /-submit["'`]/.test(s) || /submit-button["'`]/.test(s),
    );

    expect(
      avecEcriture.length,
      "Aucun parcours filmé n'envoie de formulaire. La vitrine montre alors " +
        "un produit qu'on peut parcourir mais pas utiliser — et les défauts " +
        "d'écriture, qui sont les plus coûteux, restent invisibles.",
    ).toBeGreaterThanOrEqual(1);
  });
});
