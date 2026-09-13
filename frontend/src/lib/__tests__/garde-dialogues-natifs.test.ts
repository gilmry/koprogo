import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";

/**
 * Cliquet : les dialogues natifs du navigateur ne se multiplient pas.
 *
 * ── Le constat ────────────────────────────────────────────────────────────
 *
 * `confirm()`, `prompt()` et `alert()` sont des dialogues du NAVIGATEUR, pas
 * de la page. Trois conséquences, et aucune n'est cosmétique.
 *
 * **Un navigateur piloté les supprime.** Playwright les rejette par défaut,
 * et les outils de recette automatisée aussi. L'action garde alors
 * exactement la forme d'une panne : aucun dialogue, aucune requête, aucun
 * message. C'est le diagnostic qui a coûté le plus cher en recette — le
 * bouton « Reporter » d'une assemblée a été déclaré mort deux recettes
 * durant (#780, RN-9) alors que sa source était correcte.
 *
 * **Ils ne se traduisent pas.** Les boutons « OK » et « Annuler » viennent de
 * la locale du navigateur, pas de celle du produit. Un écran par ailleurs
 * traduit en quatre langues pose une question en français et propose des
 * réponses en anglais.
 *
 * **Ils ne sont pas accessibles.** Pas de piège de focus contrôlable, pas de
 * `role`, pas d'annonce maîtrisée. `AccessibleModal.svelte` existe dans le
 * dépôt et gère déjà tout cela.
 *
 * ── Ce que ce cliquet garde ───────────────────────────────────────────────
 *
 * Il n'en interdit pas l'usage : soixante appels existent dans vingt-huit
 * fichiers, et les remplacer tous demande d'écrire soixante modales. Il
 * empêche le soixante-et-unième.
 *
 * `MeetingDetail.svelte` montre la sortie : ses trois dialogues natifs ont
 * été remplacés par des modales du dépôt, et il n'en porte plus aucun.
 *
 * Suivi en #844.
 */

const RACINE = join(process.cwd(), "src");

/**
 * Appels aux dialogues natifs. **Zéro, et cela ne remonte pas.**
 *
 * La dette était de soixante le matin du 2026-09-08, répartie sur vingt-huit
 * fichiers. Elle est épuisée le soir même.
 */
const DETTE_AU_2026_09_08 = 0;

const APPEL_NATIF = /(?<![.\w$])(?:window\.)?(?:confirm|prompt|alert)\s*\(/g;

function fichiersDeGabarit(dossier: string): string[] {
  const sortie: string[] = [];
  for (const entree of readdirSync(dossier)) {
    const chemin = join(dossier, entree);
    if (statSync(chemin).isDirectory()) {
      sortie.push(...fichiersDeGabarit(chemin));
    } else if (chemin.endsWith(".svelte") || chemin.endsWith(".astro")) {
      sortie.push(chemin);
    }
  }
  return sortie;
}

/**
 * Retire les commentaires avant de chercher.
 *
 * Sans cela, la garde compterait les commentaires qui EXPLIQUENT le défaut —
 * dont ceux de `MeetingDetail.svelte`, le seul fichier qui s'en est
 * débarrassé. Une garde qui punit la documentation de sa propre règle
 * décourage de l'écrire.
 */
function sansCommentaires(source: string): string {
  return source
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .split("\n")
    .filter((l) => !l.trim().startsWith("//"))
    .join("\n");
}

function appelsNatifs(): string[] {
  const trouves: string[] = [];
  for (const chemin of fichiersDeGabarit(RACINE)) {
    const source = sansCommentaires(readFileSync(chemin, "utf8"));
    const n = source.match(APPEL_NATIF)?.length ?? 0;
    if (n > 0) {
      trouves.push(`${n} × ${chemin.slice(RACINE.length + 1)}`);
    }
  }
  return trouves.sort();
}

function total(): number {
  return appelsNatifs().reduce((s, l) => s + Number(l.split(" ")[0]), 0);
}

describe("les dialogues natifs ne se multiplient pas (#844)", () => {
  it("n'ajoute pas de confirm(), prompt() ou alert() natif", () => {
    const n = total();
    expect(
      n,
      `${n} appels à un dialogue natif du navigateur, contre ` +
        `${DETTE_AU_2026_09_08} au 2026-09-08.\n\n` +
        `Un navigateur piloté les supprime : l'action prend alors la forme ` +
        `exacte d'une panne, sans dialogue, sans requête et sans message. ` +
        `Ils ne se traduisent pas, et ils ne sont pas accessibles.\n\n` +
        `Employez \`AccessibleModal.svelte\`, qui gère déjà le piège de ` +
        `focus. \`MeetingDetail.svelte\` montre la sortie.\n\n` +
        appelsNatifs().join("\n"),
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_08);
  });

  /**
   * Le contrôle d'aveuglement, refait pour un cliquet à ZÉRO.
   *
   * Il exigeait « au moins un appel natif », ce qui prouvait que le détecteur
   * lisait encore les gabarits. À zéro, cette formulation punirait le succès —
   * l'erreur que `garde_classement_par_souschaines` a commise le même jour, et
   * qui a fait échouer sa propre correction.
   *
   * Il vérifie donc que les fichiers sont LUS, par leur nombre, sans rien
   * exiger du compte de violations.
   */
  it("lit encore les gabarits du dépôt", () => {
    expect(
      fichiersDeGabarit(RACINE).length,
      "moins de trois cents gabarits lus : le détecteur ne parcourt plus " +
        "`src/`, et le cliquet serait vert faute de matière. Vérifiez avant " +
        "de vous réjouir.",
    ).toBeGreaterThan(300);
  });

  /**
   * Les fichiers qui ont fait le chemin ne doivent pas le refaire à l'envers.
   *
   * `MeetingDetail.svelte` est celui dont les trois dialogues ont été
   * remplacés après que le bouton « Reporter » a été déclaré mort deux
   * recettes durant. `ConvocationDetailView.svelte` est l'écran du deuxième
   * verrou de #780, et il en portait cinq — le plus chargé du dépôt.
   *
   * Un cliquet global se satisferait de n'importe quels autres fichiers :
   * ceux-ci sont nommés.
   */
  it.each([
    "components/MeetingDetail.svelte",
    "components/convocations/ConvocationDetailView.svelte",
    "components/etats-dates/EtatDateDetail.svelte",
    "components/ExpenseDetail.svelte",
    "components/budgets/BudgetDetail.svelte",
    "components/CallForFundsList.svelte",
    "components/PaymentReminderDetail.svelte",
    "components/polls/PollDetail.svelte",
    "components/local-exchanges/ExchangeDetail.svelte",
    "components/convocations/ConvocationPanel.svelte",
    "components/quotes/QuoteDetail.svelte",
    "components/gamification/AdminChallengeList.svelte",
  ])("garde %s exempt de dialogue natif", (relatif) => {
    const source = sansCommentaires(
      readFileSync(join(RACINE, relatif), "utf8"),
    );
    expect(
      source.match(APPEL_NATIF)?.length ?? 0,
      `un dialogue natif est revenu dans ${relatif}. Les deux premiers sont ` +
        `les écrans des deux verrous de #780 ; le troisième est l'état daté, ` +
        `que le notaire demande à la vente d'un lot sous quinze jours ` +
        `ouvrables (Art. 3.94). Les deux derniers gardent des actes ` +
        `destructeurs sur des montants notifiés aux copropriétaires. Aucun ne ` +
        `peut se permettre d'être intestable par un navigateur piloté.`,
    ).toBe(0);
  });
});
