import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Une interception réseau se déclare par une EXPRESSION RÉGULIÈRE, pas par un
 * motif en chaîne.
 *
 * Playwright résout un motif sans schéma contre `baseURL`. Dans ce dépôt,
 * `baseURL` vaut la racine du FRONT (`http://localhost:3000` en CI) et l'API
 * vit sur un autre port (`8080`). Un `page.route("**\/api/v1/acps", …)` ne
 * correspond donc **jamais** : l'interception n'a aucun effet, silencieusement.
 *
 * Ce que ça produisait, prouvé par la trace du run 34377060293 :
 *
 *   - `story2-acp-organization` simulait `GET /acps` avec `[]` pour éprouver
 *     l'état vide « Aucune ACP disponible ». La réponse réelle faisait
 *     **56 706 octets**. Le test exerçait la vraie liste, qui n'est pas vide,
 *     et échouait en accusant l'affichage. **Il n'a jamais testé ce qu'il
 *     annonce.**
 *   - `forceLargePageSize` réécrivait `per_page=10000` pour contourner la
 *     pagination. Il n'a jamais rien réécrit non plus.
 *
 * Une interception qui ne s'applique pas est pire qu'une absente : elle donne
 * au lecteur la certitude que le cas limite est couvert.
 *
 * Une expression régulière est comparée à l'URL complète, sans résolution
 * relative. Les autres recettes du dépôt le font déjà — `page.route(/\/c\?t=/)`,
 * `page.route(/\/organizations/)`. Seul ce fichier employait des chaînes.
 *
 * Le cliquet est à zéro : c'est une interdiction.
 */

const RACINE = join(process.cwd(), "tests/e2e");

const ROUTE_EN_CHAINE = /page\.route\(\s*["'`]/g;

function recettes(racine: string): string[] {
  const trouvees: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) trouvees.push(...recettes(chemin));
    else if (entree.endsWith(".spec.ts") || entree.endsWith(".scenario.ts"))
      trouvees.push(chemin);
  }
  return trouvees;
}

function interceptionsFragiles(): string[] {
  const trouvees: string[] = [];
  for (const fichier of recettes(RACINE)) {
    const code = readFileSync(fichier, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    for (const m of code.matchAll(ROUTE_EN_CHAINE)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      trouvees.push(`  ${relative(process.cwd(), fichier)}:${ligne}`);
    }
  }
  return trouvees;
}

/**
 * Une interception d'API exige de bloquer le service worker.
 *
 * `public/service-worker.js:118` intercepte TOUT ce qui commence par `/api/`
 * et refait le `fetch` lui-même (`networkFirstStrategy`). La requête part donc
 * du service worker, pas de la page, et `page.route` ne la voit jamais.
 *
 * L'interception est alors sans effet, **sans que rien ne le signale**. Deux
 * recettes en dépendaient : l'une échouait en accusant l'affichage, l'autre
 * PASSAIT en croyant éprouver une branche qu'elle n'atteignait pas.
 *
 * Le remède est `test.use({ serviceWorkers: "block" })` au niveau du bloc.
 * `pwa-contractor.spec.ts` est la seule exception légitime : elle éprouve
 * précisément la PWA, et intercepte une URL de PAGE (`/c/…`), pas d'API.
 */
function interceptionsApiSansBlocage(): string[] {
  const fautives: string[] = [];
  for (const fichier of recettes(RACINE)) {
    const code = readFileSync(fichier, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    const intercepteUneApi =
      /page\.route\([^)]*(?:api|organizations|buildings|acps|tickets|invoices|expenses)/.test(
        code,
      );
    if (!intercepteUneApi) continue;
    if (/serviceWorkers:\s*["']block["']/.test(code)) continue;
    fautives.push(`  ${relative(process.cwd(), fichier)}`);
  }
  return fautives;
}

describe("les interceptions réseau visent l'URL complète", () => {
  it("bloque le service worker quand elle intercepte une API", () => {
    expect(
      interceptionsApiSansBlocage().join("\n"),
      "Ces recettes interceptent une route d'API sans bloquer le service " +
        "worker. `service-worker.js` refait le `fetch` lui-même pour tout " +
        "`/api/`, si bien que `page.route` ne voit jamais la requête : " +
        "l'interception est sans effet, et le test croit éprouver un cas " +
        "qu'il n'atteint pas.\n\n" +
        'Ajoutez `test.use({ serviceWorkers: "block" })` au bloc.',
    ).toBe("");
  });

  it("n'emploie aucun motif en chaîne", () => {
    expect(
      interceptionsFragiles().join("\n"),
      "Ces interceptions emploient un motif en chaîne, que Playwright résout " +
        "contre `baseURL`. L'API n'étant pas sur la même origine que le " +
        "front, elles ne correspondent jamais et n'ont aucun effet — sans " +
        "que rien ne le signale.\n\n" +
        "Employez une expression régulière : `page.route(/\\/api\\/v1\\/x$/, …)`.",
    ).toBe("");
  });

  it("lit encore les recettes", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite. On vérifie
    // que le détecteur voit encore des interceptions — sous leur bonne forme.
    const fichiers = recettes(RACINE);
    expect(fichiers.length).toBeGreaterThan(50);
    const routes = fichiers
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/page\.route\(/g);
    expect(
      routes?.length ?? 0,
      "plus aucune interception réseau dans les recettes : le motif a changé.",
    ).toBeGreaterThan(0);
  });
});
