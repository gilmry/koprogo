import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Les en-têtes de sécurité doivent survivre à chaque bloc de nginx.
 *
 * `add_header` ne s'hérite pas comme on le croit. La documentation le dit en
 * une phrase facile à manquer : les directives sont héritées du niveau
 * précédent **si et seulement si** aucun `add_header` n'est déclaré au niveau
 * courant. Un seul suffit donc à tout effacer.
 *
 * C'est ce qui se passait. Le bloc des ressources statiques déclarait
 * `Cache-Control`, et perdait par là les trois en-têtes de sécurité du bloc
 * parent. Mesuré en production le 2026-09-08 :
 *
 *     GET /                       nosniff, DENY, 1; mode=block
 *     GET /_astro/....js          cache-control seul
 *
 * La ressource qui perdait `nosniff` était le fichier JavaScript — celle que
 * cet en-tête existe précisément pour protéger contre le reniflage de type.
 * Vérifié après correction, conteneur nginx monté sur un fichier réel :
 * `GET /app.js` rend 200 avec les cinq en-têtes.
 *
 * Le défaut est invisible à la lecture : les en-têtes sont bien écrits, à un
 * niveau qui ne s'applique pas. Seule une garde le voit.
 */

const NGINX = join(process.cwd(), "nginx.conf");
const SNIPPET = "include /etc/nginx/snippets/securite.conf;";

/**
 * Découpe grossièrement en blocs `{ ... }` de premier niveau sous `server`.
 * On ne cherche pas à analyser nginx, seulement à repérer les blocs qui
 * déclarent un `add_header` — ce sont eux qui coupent l'héritage.
 */
function blocsAvecAddHeader(source: string): string[] {
  const sansCommentaires = source.replace(/^\s*#.*$/gm, "");
  const blocs: string[] = [];
  const debut = /location\s+[^{]*\{/g;
  let m: RegExpExecArray | null;
  while ((m = debut.exec(sansCommentaires)) !== null) {
    let i = debut.lastIndex;
    let profondeur = 1;
    while (i < sansCommentaires.length && profondeur > 0) {
      if (sansCommentaires[i] === "{") profondeur += 1;
      else if (sansCommentaires[i] === "}") profondeur -= 1;
      i += 1;
    }
    const corps = sansCommentaires.slice(m.index, i);
    if (/add_header/.test(corps)) blocs.push(corps);
  }
  return blocs;
}

describe("les en-têtes de sécurité survivent aux blocs nginx", () => {
  it("réinclut les en-têtes dans chaque bloc qui déclare un add_header", () => {
    const source = readFileSync(NGINX, "utf-8");
    const fautifs = blocsAvecAddHeader(source)
      .filter((b) => !b.includes(SNIPPET))
      .map((b) => b.split("\n")[0].trim());
    expect(
      fautifs.join("\n"),
      "Ces blocs déclarent un `add_header` sans réinclure les en-têtes de " +
        "sécurité. nginx REMPLACE la liste héritée : tout ce que le bloc " +
        "parent posait disparaît pour les URL que ces blocs servent.\n\n" +
        `Ajoutez-y \`${SNIPPET}\`.`,
    ).toBe("");
  });

  it("le bloc parent pose bien les en-têtes", () => {
    // Contrôle d'aveuglement : si l'inclusion disparaissait du niveau
    // `server`, le test ci-dessus passerait sans rien garder, puisqu'il ne
    // regarde que les blocs `location`.
    const source = readFileSync(NGINX, "utf-8");
    const avantPremierLocation = source.slice(0, source.indexOf("location"));
    expect(
      avantPremierLocation.includes(SNIPPET),
      "le niveau `server` n'inclut plus les en-têtes de sécurité : les blocs " +
        "`location` peuvent bien les réinclure, il ne reste rien à hériter " +
        "pour tout le reste du site.",
    ).toBe(true);
  });

  it("le fichier inclus porte encore les en-têtes attendus", () => {
    // Second contrôle d'aveuglement : les inclusions peuvent être en place et
    // le fichier inclus vidé.
    const snippet = readFileSync(
      join(process.cwd(), "nginx-securite.conf"),
      "utf-8",
    );
    for (const entete of [
      "X-Content-Type-Options",
      "X-Frame-Options",
      "Referrer-Policy",
      "Permissions-Policy",
      "Content-Security-Policy-Report-Only",
    ]) {
      expect(snippet, `${entete} a disparu du fichier inclus`).toContain(
        entete,
      );
    }
  });
});

/**
 * La CSP doit autoriser ce que la page charge elle-même.
 *
 * `Layout.astro` charge le script Plausible depuis plausible.io, qui envoie
 * ensuite ses événements à `https://plausible.io/api/event`. La CSP n'en
 * savait rien : en Report-Only, le navigateur le signale à chaque page ; le
 * jour où l'on retire `-Report-Only`, l'analytics s'éteint en silence.
 */
describe("la CSP couvre les origines externes de la mise en page", () => {
  const snippet = readFileSync(
    join(process.cwd(), "nginx-securite.conf"),
    "utf-8",
  );
  const layout = readFileSync(
    join(process.cwd(), "src/layouts/Layout.astro"),
    "utf-8",
  );

  function directive(nom: string): string[] {
    const csp = /Content-Security-Policy-Report-Only\s+"([^"]+)"/.exec(snippet);
    expect(csp, "CSP introuvable dans le fichier inclus").not.toBeNull();
    const valeur = csp![1]
      .split(";")
      .map((d) => d.trim())
      .find((d) => d.startsWith(`${nom} `));
    return valeur ? valeur.split(/\s+/).slice(1) : [];
  }

  const originesScripts = [
    ...layout.matchAll(/<script[^>]*\ssrc="(https?:\/\/[^"]+)"/g),
  ].map((m) => new URL(m[1]).origin);

  it("trouve au moins un script externe (contrôle d'aveuglement)", () => {
    expect(originesScripts.length).toBeGreaterThan(0);
  });

  it("autorise dans script-src chaque script externe de la mise en page", () => {
    const manquantes = originesScripts.filter(
      (o) => !directive("script-src").includes(o),
    );
    expect(manquantes, "origines de script absentes de script-src").toEqual([]);
  });

  it("autorise dans connect-src le point de collecte Plausible", () => {
    expect(directive("connect-src")).toContain("https://plausible.io");
  });
});
