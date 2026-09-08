import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname } from "node:path";

/**
 * Cliquet : la dette de libellés non traduits ne grossit pas.
 *
 * ── Le constat ─────────────────────────────────────────────────────────────
 *
 * #833 a traité les messages de toast : trente-six, deux portes, dette
 * éteinte. Mais un toast n'est pas le seul endroit où l'on parle à
 * l'utilisateur. **Le texte des gabarits est la porte principale**, et elle
 * n'avait jamais été mesurée.
 *
 * Au 2026-09-07, après la traduction de la navigation : **362 libellés écrits
 * en dur dans 52 fichiers**. Titres de sections, en-têtes de colonnes,
 * étiquettes de champs, phrases d'aide.
 *
 * Un produit qui vise quatre langues et sert une clientèle belge ne peut pas
 * afficher « Informations personnelles » à un néerlandophone. Ce n'est pas une
 * traduction approximative : c'est un écran qu'il ne peut pas lire.
 *
 * ── Pourquoi un cliquet et non un interdit ─────────────────────────────────
 *
 * Contrairement à `garde-messages-en-dur`, qui part de zéro parce que sa dette
 * est éteinte, celle-ci est vivante. Traduire 362 libellés d'un coup serait
 * invérifiable, et une traduction posée sans regarder l'écran vaut moins que
 * pas de traduction — elle fait croire que la langue est couverte.
 *
 * La dette se résorbe au fil des lots : celui qui touche un écran le traduit,
 * **et fait baisser ce nombre dans le même commit**. C'est la règle déjà
 * retenue pour `garde_ecriture`, `garde_champs_ignores` et #803.
 *
 * Le nombre part de 362 et non de 380 : la navigation a été traduite dans le
 * commit qui pose ce cliquet, parce qu'un cliquet posé sans une première
 * baisse n'est qu'une constatation. Et l'écran choisi n'est pas au hasard —
 * c'est celui que voient les rôles sans interface, dont le prestataire, un
 * tiers extérieur à la copropriété qui a toutes les chances de ne pas être
 * francophone.
 *
 * ── Ce que le détecteur ne compte pas ──────────────────────────────────────
 *
 * Le texte à l'intérieur de `<script>` et `<style>`, les expressions `{...}`
 * (donc tout ce qui passe déjà par `$_()`), les URL, et le nom du produit.
 * « KoproGo » n'est pas un libellé : le traduire serait une faute.
 *
 * Suivi en #834.
 */

/**
 * Libellés de gabarit non traduits. **Ne doit que BAISSER.**
 *
 * 380 au premier relevé ; 362 après la navigation ; 332 après le profil ;
 * 309 après le formulaire de ticket ; 281 après la liste des ACP ;
 * **258** après l'écran du prestataire et la création d'assemblée —
 * vingt-trois libellés dont quinze réécrivaient en dur `tickets.categories.*`
 * et `tickets.priorities.*`, que `TicketPriorityBadge` employait déjà.
 *
 * Le motif se répète : ces écrans ne manquaient pas de traductions, ils étaient
 * DÉBRANCHÉS de celles qui existaient. `ProfilePanel` dupliquait de même
 * `gdpr.article15.title`, `gdpr.myPersonalData` et `common.edit`. Traduire
 * revient donc surtout à raccorder — ce qui explique qu'on puisse descendre
 * vite au début, et pourquoi le reste sera plus lent.
 *
 * Un effet de bord notable : le formulaire affichait « Basse » là où la liste
 * affiche « Basse (7 jours) ». Le même ticket portait deux libellés selon
 * l'écran, et le délai que le produit s'engage à tenir n'apparaissait pas au
 * moment où l'utilisateur choisit sa priorité.
 *
 * Un `confirm()` natif y portait aussi son message en dur, hors de portée du
 * détecteur qui ne regarde que le gabarit.
 */
const DETTE_AU_2026_09_07 = 128;

const RACINE = join(process.cwd(), "src");

/** Ce qui n'est pas un libellé et ne doit pas être traduit. */
const NOMS_PROPRES = new Set(["KoproGo"]);

/** Du texte entre deux balises, sans expression Svelte à l'intérieur. */
const TEXTE_DE_GABARIT = />([^<>{}]+)</g;

/**
 * Au moins un mot capitalisé suivi de minuscules accentuées ou non.
 *
 * C'est ce qui distingue une phrase destinée à un humain d'un fragment
 * technique, d'un symbole ou d'un nombre. Le critère est volontairement
 * étroit : un cliquet qui crie à tort finit désactivé.
 */
const RESSEMBLE_A_UNE_PHRASE = /[A-ZÀÉÈÊÎÔÛÇ][a-zàéèêîôûçïü]{2,}/;

function fichiersDeGabarit(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeGabarit(chemin));
    } else if (extname(entree) === ".svelte" && !entree.includes(".test.")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

function recenser(): string[] {
  const fautes: string[] = [];
  for (const chemin of fichiersDeGabarit(RACINE)) {
    let texte = readFileSync(chemin, "utf8");
    texte = texte.replace(/<script[\s\S]*?<\/script>/g, "");
    texte = texte.replace(/<style[\s\S]*?<\/style>/g, "");
    for (const m of texte.matchAll(TEXTE_DE_GABARIT)) {
      const libelle = m[1].split(/\s+/).filter(Boolean).join(" ");
      if (libelle.length < 3) continue;
      if (!RESSEMBLE_A_UNE_PHRASE.test(libelle)) continue;
      if (libelle.startsWith("http") || libelle.startsWith("//")) continue;
      if (NOMS_PROPRES.has(libelle)) continue;
      fautes.push(
        `${chemin.replace(process.cwd() + "/", "")} — « ${libelle.slice(0, 55)} »`,
      );
    }
  }
  return fautes;
}

describe("la dette de libellés non traduits ne grossit pas (#834)", () => {
  it("ne laisse pas augmenter le nombre de libellés écrits en dur", () => {
    const fautes = recenser();

    expect(
      fautes.length,
      `La dette de traduction a GROSSI : ${fautes.length} libellés de gabarit ` +
        `écrits en dur, contre ${DETTE_AU_2026_09_07} au 2026-09-07.\n\n` +
        `Un libellé littéral s'affichera en français à un néerlandophone. ` +
        `Ajoutez la clé dans les QUATRE fichiers de \`src/locales/\` et ` +
        `appelez \`{$_("section.cle")}\`.\n\n` +
        `Celui qui touche un écran le traduit, et fait baisser ce nombre dans ` +
        `le même commit.\n\n` +
        `Derniers relevés :\n` +
        fautes.slice(-12).join("\n"),
      // `toBeLessThanOrEqual`, et non `toHaveLength`.
      //
      // La première version employait l'égalité stricte : le test échouait
      // aussi bien quand la dette montait que quand elle BAISSAIT. Découvert
      // en traduisant `ProfilePanel.svelte` — trente libellés de moins, et le
      // cliquet refusait le progrès qu'il était censé encourager.
      //
      // Un cliquet borne un maximum ; il ne fige pas une valeur.
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_07);
  });

  /// Sans ce contrôle, une refonte des gabarits rendrait le cliquet
  /// silencieusement vert en ne trouvant plus rien à compter.
  it("reconnaît encore un libellé et épargne ce qui est déjà traduit", () => {
    const attrape = (s: string) =>
      [...s.matchAll(TEXTE_DE_GABARIT)].filter((m) =>
        RESSEMBLE_A_UNE_PHRASE.test(m[1]),
      ).length;

    expect(attrape("<p>Informations personnelles</p>")).toBe(1);
    expect(attrape('<p>{$_("profile.title")}</p>')).toBe(0);
    expect(attrape("<span>42</span>")).toBe(0);
  });
});
