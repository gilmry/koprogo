import { describe, expect, it } from "vitest";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * Tout lien interne doit mener quelque part.
 *
 * L'hébergement sert une page de repli en **200** pour n'importe quelle URL.
 * Un lien mort ne rend donc pas 404 : il rend l'accueil, avec un code de
 * succès. Rien ne le signale — ni à l'utilisateur, ni à la supervision.
 * Vérifié en production le 2026-09-08 :
 *
 *     /gdpr/export                              200, 24193 octets
 *     /gdpr/cette-page-nexiste-absolument-pas   200, 24193 octets
 *
 * Deux URL au même octet près, dont une que je venais d'inventer.
 *
 * Ce que cela cachait :
 *
 *   - `privacy-policy.astro` nommait cinq URL pour exercer les droits RGPD
 *     (accès, rectification, effacement, limitation, opposition au marketing).
 *     Aucune n'existait : c'étaient les chemins d'API que `GdprDataPanel`
 *     appelle, recopiés dans la politique comme s'ils étaient des pages. Un
 *     copropriétaire exerçant son droit d'accès atterrissait sur la page
 *     d'accueil commerciale. Les cinq droits existent, sur `/settings/gdpr`.
 *   - La tuile « Factures » du tableau de bord comptable pointait `/invoices`
 *     quand la page s'appelle `/invoice-workflow`. La recette
 *     `AccountantInvoiceWorkflowJourney` ouvre l'URL directement et n'a donc
 *     jamais cliqué la tuile.
 *   - `Layout.astro` déclarait `/favicon.svg`, absent de `public/`. Seul lien
 *     du lot à rendre un vrai 404.
 *
 * Le cliquet est à zéro : c'est une interdiction, pas une dette. Un lien mort
 * qui rend 200 ne se découvre jamais à l'usage.
 */

const RACINE = join(process.cwd(), "src");
const PAGES = join(RACINE, "pages");
const PUBLIC = join(process.cwd(), "public");

function fichiers(racine: string, garder: (f: string) => boolean): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory())
      trouves.push(...fichiers(chemin, garder));
    else if (garder(entree)) trouves.push(chemin);
  }
  return trouves;
}

/** Les routes qu'Astro sait servir, déduites de l'arborescence des pages. */
function routes(): { exactes: Set<string>; dynamiques: RegExp[] } {
  const exactes = new Set<string>();
  const dynamiques: RegExp[] = [];
  for (const f of fichiers(PAGES, (e) => e.endsWith(".astro"))) {
    let r = relative(PAGES, f).replace(/\.astro$/, "");
    r = r === "index" ? "/" : "/" + r.replace(/\/index$/, "");
    if (r.includes("[")) {
      dynamiques.push(
        new RegExp(
          "^" +
            r
              .replace(/\[\.\.\.[^\]]+\]/g, ".+")
              .replace(/\[[^\]]+\]/g, "[^/]+") +
            "$",
        ),
      );
    } else {
      exactes.add(r);
    }
  }
  return { exactes, dynamiques };
}

function liensMorts(): { lien: string; ou: string[] }[] {
  const { exactes, dynamiques } = routes();
  const par: Map<string, Set<string>> = new Map();
  const sources = fichiers(
    RACINE,
    (e) => e.endsWith(".svelte") || e.endsWith(".astro"),
  );
  for (const f of sources) {
    // Les commentaires ne sont pas du code : deux gardes de ce dépôt ont déjà
    // compté des exemples cités en commentaire comme des infractions réelles.
    const code = readFileSync(f, "utf-8")
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\*[\s\S]*?\*\//g, "");
    // Seuls les `href` littéraux : un `href={...}` porte une expression que
    // l'analyse statique ne peut pas résoudre sans se mentir.
    for (const m of code.matchAll(/href="(\/[^"{}\s#?]*)"/g)) {
      const lien = m[1].replace(/\/+$/, "") || "/";
      par.set(
        lien,
        (par.get(lien) ?? new Set()).add(relative(process.cwd(), f)),
      );
    }
  }
  const morts: { lien: string; ou: string[] }[] = [];
  for (const [lien, ou] of par) {
    if (exactes.has(lien)) continue;
    if (dynamiques.some((d) => d.test(lien))) continue;
    // Un lien peut viser un fichier servi tel quel depuis `public/`.
    if (existsSync(join(PUBLIC, lien))) continue;
    morts.push({ lien, ou: [...ou].sort() });
  }
  // Le manifeste PWA déclare des chemins d'icônes que ni Svelte ni Astro ne
  // portent : sans cette lecture, la garde ne les verrait pas. Les trois
  // icônes qu'il nommait étaient absentes, toutes trois en 404 vérifié.
  // Chrome n'émet `beforeinstallprompt` que si le manifeste offre une icône
  // récupérable ; sans elle, la bannière d'installation de
  // `PWAInstallPrompt.svelte` ne peut pas apparaître, et le bouton
  // « Installer » est inatteignable.
  const manifeste = join(PUBLIC, "manifest.webmanifest");
  if (existsSync(manifeste)) {
    const m = JSON.parse(readFileSync(manifeste, "utf-8"));
    for (const icone of m.icons ?? []) {
      if (!existsSync(join(PUBLIC, icone.src))) {
        morts.push({ lien: icone.src, ou: ["public/manifest.webmanifest"] });
      }
    }
  }
  return morts.sort((a, b) => a.lien.localeCompare(b.lien));
}

describe("les liens internes mènent quelque part (#803)", () => {
  it("ne pointe vers aucune page ni fichier inexistant", () => {
    const morts = liensMorts();
    expect(
      morts
        .map((m) => `  ${m.lien}\n      ← ${m.ou.join("\n      ← ")}`)
        .join("\n"),
      "Des liens internes ne mènent nulle part. L'hébergement rend la page " +
        "d'accueil en 200 pour ces URL : le défaut ne se voit ni à l'usage ni " +
        "en supervision.\n\nVisez une page de `src/pages` ou un fichier de " +
        "`public/`.",
    ).toBe("");
  });

  it("lit encore les sources et y trouve des liens résolus", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite. Il vérifie
    // que le détecteur trouve encore des pages ET des liens qui pointent
    // dessus — ce qui subsiste quand la dette est à zéro.
    const { exactes } = routes();
    expect(exactes.size, "plus aucune page dans src/pages").toBeGreaterThan(50);
    const liens = fichiers(
      RACINE,
      (e) => e.endsWith(".svelte") || e.endsWith(".astro"),
    )
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/href="\/[^"{}\s#?]*"/g);
    expect(
      liens?.length ?? 0,
      "plus aucun lien interne littéral : le motif a changé, ou le " +
        "répertoire a bougé. Vérifiez avant de vous réjouir.",
    ).toBeGreaterThan(50);
    const manifeste = join(PUBLIC, "manifest.webmanifest");
    expect(
      existsSync(manifeste) &&
        (JSON.parse(readFileSync(manifeste, "utf-8")).icons ?? []).length,
      "le manifeste PWA ne déclare plus d'icône : la garde ne vérifierait " +
        "plus rien de ce côté.",
    ).toBeGreaterThan(0);
  });
});
