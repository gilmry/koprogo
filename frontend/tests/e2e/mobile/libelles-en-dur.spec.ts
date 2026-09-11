import { test, expect } from "@playwright/test";
import { ongletsPour } from "../../../src/lib/onglets-mobiles";
import { ouvreEnTantQue, ROLES, type Role } from "./socle";

/**
 * Les libellés qui ne passent pas par la traduction (#834).
 *
 * ── Comment on les trouve sans les lister ──────────────────────────────────
 *
 * Le même écran est chargé deux fois, en `fr` puis en `nl`, et les deux textes
 * rendus sont comparés. **Ce qui est identique dans les deux langues n'est pas
 * passé par le catalogue.**
 *
 * C'est ce qui rend #834 mesurable. « 362 libellés en dur » est un chiffre
 * qu'on ne peut ni vérifier ni faire baisser ; « tel écran affiche seize
 * textes identiques en français et en néerlandais » se corrige, et se
 * recompte.
 *
 * La mesure ne lit pas le code : peu importe qu'un libellé soit dans un
 * gabarit Astro, dans une chaîne Svelte ou construit à l'exécution. Elle lit
 * ce que l'utilisateur voit.
 *
 * ── Ce qu'elle a trouvé au premier passage ────────────────────────────────
 *
 * Six textes sur les VINGT destinations : le lien d'évitement — la première
 * chose qu'un utilisateur au clavier rencontre —, les quatre lignes du pied de
 * page, et le libellé du rôle. Un néerlandophone lisait « Passer au contenu
 * principal » et « Mentions légales » sur chaque écran du produit.
 *
 * Corrigé : `Layout.astro` est un gabarit Astro, son texte est figé à la
 * construction alors que la langue se choisit dans le navigateur. Le pied de
 * page et le lien d'évitement sont devenus des îles Svelte.
 *
 * ── Pourquoi un cliquet, et pas zéro ──────────────────────────────────────
 *
 * Parce que la dette existe : les titres et descriptions de page sont encore
 * écrits en dur, et les corriger tous est le travail de #834, pas de ce
 * commit. Le cliquet interdit qu'elle grossisse.
 *
 * Et parce que la mesure a un bruit irréductible : « Contact », « Niveau »,
 * « Formule » s'écrivent pareil en français et en néerlandais. Exiger zéro
 * obligerait à truquer la règle pour des mots qui sont corrects.
 */

/**
 * Ce qui est légitimement identique dans les deux langues.
 *
 * Court, et le restera : chaque ajout ici est une exception au contrôle, donc
 * une raison de moins de lui faire confiance.
 */
const LEGITIMES = new Set([
  "KoproGo", // La marque.
  "Banc Mobile", // Le nom de l'utilisateur du banc.
  "Banc", // Idem, quand le prénom est seul dans son nœud.
  "Mobile",
]);

/**
 * Mesuré le 2026-09-11, après correction de la coquille.
 *
 * Ce nombre a été RELEVÉ, jamais choisi. J'avais d'abord écrit 118 de tête ;
 * la mesure dit 97. Un cliquet posé de mémoire aurait laissé passer vingt et
 * un nouveaux libellés en dur sans rien dire — c'est précisément le genre de
 * vert qui ne garde rien.
 *
 * Il ne peut que descendre, et il descend : 97, puis **91**, puis **73**,
 * après la coquille, puis le rôle affiché sur l'écran « Plus », puis les
 * en-têtes de onze pages passés par `EnTeteDePage`.
 *
 * Un cliquet qu'on ne rabaisse pas après une correction cesse de garder le
 * terrain qu'on vient de gagner : il autorise en silence le retour de ce qu'on
 * vient de corriger.
 */
const CLIQUET = 73;

async function textesRendus(
  page: import("@playwright/test").Page,
  langue: string,
  role: Role,
  chemin: string,
): Promise<string[]> {
  await page.addInitScript(
    (l) => window.localStorage.setItem("preferred-language", l),
    langue,
  );
  await ouvreEnTantQue(page, role, chemin);

  // La page doit être POSÉE avant d'être lue.
  //
  // Sans cette attente, la mesure est instable : exécutée seule elle donnait
  // 91, exécutée en parallèle du reste du banc elle donnait 92. Sous charge,
  // un écran n'a pas fini de s'hydrater et montre encore un état de
  // chargement — identique dans les deux langues, donc compté comme un
  // libellé en dur.
  //
  // Un cliquet qui varie selon la charge de la machine ne garde rien : il
  // crie au loup, on finit par le relever « pour qu'il passe », et il cesse
  // d'être une mesure.
  await page.waitForLoadState("networkidle").catch(() => {});
  await page.waitForTimeout(400);

  return page.evaluate(() => {
    const vus: string[] = [];
    // `<script>` et `<style>` portent du texte qui n'est pas de l'interface :
    // sans cette exclusion, le module d'hydratation d'Astro — identique dans
    // toutes les langues, forcément — compte comme un libellé en dur.
    const ignores = new Set(["SCRIPT", "STYLE", "NOSCRIPT", "TEMPLATE"]);
    const marche = document.createTreeWalker(
      document.body,
      NodeFilter.SHOW_TEXT,
      {
        acceptNode: (n) =>
          n.parentElement && ignores.has(n.parentElement.tagName)
            ? NodeFilter.FILTER_REJECT
            : NodeFilter.FILTER_ACCEPT,
      },
    );
    for (let n = marche.nextNode(); n; n = marche.nextNode()) {
      const texte = (n.textContent ?? "").replace(/\s+/g, " ").trim();
      // Au moins trois lettres : écarte les nombres, les symboles et les
      // séparateurs, qui ne se traduisent pas.
      if (texte.length > 2 && /[a-zA-ZÀ-ÿ]{3}/.test(texte)) vus.push(texte);
    }
    return vus;
  });
}

const ECRANS = ROLES.flatMap((role) =>
  ongletsPour(role).map((onglet) => ({
    role: role as Role,
    chemin: `${onglet.href.replace(/\/$/, "")}/`,
  })),
);

test("@edge le nombre de libellés non traduits ne remonte pas", async ({
  browser,
}) => {
  test.setTimeout(180_000);

  const parEcran: string[] = [];
  let total = 0;

  // DEUX contextes pour quarante visites, un par langue.
  //
  // La première version en ouvrait un par écran et par langue — quarante en
  // tout — et une exécution sur deux dépassait son budget de temps. Un
  // contexte est cher, une page ne l'est pas : chaque visite prend une page
  // neuve dans un contexte déjà chaud, ce qui suffit à isoler le
  // `localStorage` accumulé par `addInitScript`.
  const contexteFr = await browser.newContext();
  const contexteNl = await browser.newContext();

  for (const { role, chemin } of ECRANS) {
    const fr = await textesRendus(
      await contexteFr.newPage(),
      "fr",
      role,
      chemin,
    );
    const nl = await textesRendus(
      await contexteNl.newPage(),
      "nl",
      role,
      chemin,
    );

    const enNeerlandais = new Set(nl);
    const communs = [
      ...new Set(fr.filter((t) => enNeerlandais.has(t) && !LEGITIMES.has(t))),
    ];

    total += communs.length;
    if (communs.length > 0) {
      parEcran.push(
        `  ${role} ${chemin} — ${communs.length} : ${communs
          .slice(0, 4)
          .map((t) => `« ${t.slice(0, 46)} »`)
          .join(", ")}`,
      );
    }
  }

  await contexteFr.close();
  await contexteNl.close();

  // Vérification d'aveuglement : zéro signifierait que la mesure n'a rien lu.
  // La dette existe, elle est même documentée par #834 ; un zéro serait la
  // preuve que le contrôle est cassé, pas que le produit est traduit.
  expect(
    total,
    "Aucun texte identique en français et en néerlandais sur vingt écrans. " +
      "Ce n'est pas crédible tant que #834 est ouverte : la mesure ne lit " +
      "probablement plus rien.",
  ).toBeGreaterThan(0);

  expect(
    total,
    `${total} textes s'affichent à l'identique en français et en ` +
      `néerlandais, contre ${CLIQUET} mesurés le 2026-09-11.\n\n` +
      `Ils ne passent pas par le catalogue : un néerlandophone les lit en ` +
      `français.\n\n${parEcran.join("\n")}`,
  ).toBeLessThanOrEqual(CLIQUET);
});
