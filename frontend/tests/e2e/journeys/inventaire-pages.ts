/**
 * L'inventaire des écrans, recalculé depuis le disque.
 *
 * ── Pourquoi il ne se recopie pas ─────────────────────────────────────────
 *
 * Une liste d'écrans tenue à la main dérive. Elle dérive silencieusement :
 * la page ajoutée hier n'y est pas, et le balayage annonce « tout couvert »
 * sur un produit qu'il ne connaît plus entièrement. C'est le défaut que
 * `garde-projets-playwright` a déjà eu à corriger ailleurs — une liste tenue
 * de mémoire contre une liste reconstruite.
 *
 * Mesuré le 2026-09-19 : 104 fichiers sous `src/pages`, dont 23 chemins
 * seulement traversés par les sept parcours existants. L'écart n'était
 * visible qu'en recalculant.
 *
 * ── Ce que l'inventaire écarte, et pourquoi ───────────────────────────────
 *
 * Les routes PUBLIQUES ne sont pas balayées par un rôle connecté : les
 * ouvrir depuis une session active ne montre pas le produit, cela montre une
 * redirection. Elles méritent leur propre parcours, pas une ligne dans
 * celui-ci.
 *
 * Les routes DYNAMIQUES (`[...slug]`, `[id]`) sont écartées parce qu'un
 * balayage ne sait pas quel identifiant leur donner. Les ouvrir sans
 * paramètre filmerait un 404 et le ferait passer pour un défaut d'écran.
 * Leurs versions « detail » sont visitées par les parcours métier, qui eux
 * savent de quoi ils parlent.
 */
import { readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";

const ICI = dirname(new URL(import.meta.url).pathname);
const DOSSIER_PAGES = join(ICI, "..", "..", "..", "src", "pages");

/**
 * Ce qu'un utilisateur connecté n'a aucune raison d'ouvrir.
 *
 * `/` est la page commerciale : elle apparaît déjà, et c'est précisément
 * l'observation utile, quand un rôle y ATTERRIT faute d'écran à lui (#961).
 * L'y envoyer exprès ne prouverait rien.
 */
const PUBLIQUES = new Set([
  "/",
  "/login",
  "/register",
  "/forgot-password",
  "/reset-password",
  "/mentions-legales",
  "/privacy-policy",
  "/blog",
  // La page d'atterrissage des liens magiques : elle n'a de sens qu'avec un
  // jeton, et le parcours prestataire la couvre déjà avec un vrai.
  "/c",
]);

function fichiersAstro(racine: string, prefixe = ""): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(racine)) {
    const chemin = join(racine, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersAstro(chemin, `${prefixe}/${entree}`));
    } else if (entree.endsWith(".astro")) {
      trouves.push(`${prefixe}/${entree.replace(/\.astro$/, "")}`);
    }
  }
  return trouves;
}

/**
 * Toutes les routes balayables, triées.
 *
 * `index` se replie sur son parent : `src/pages/owner/index.astro` est la
 * route `/owner`, pas `/owner/index`.
 */
export function inventaireDesEcrans(): string[] {
  const routes = new Set<string>();
  for (const brut of fichiersAstro(DOSSIER_PAGES)) {
    // Un segment dynamique ne se balaie pas : on ne sait pas quoi y mettre.
    if (brut.includes("[")) continue;
    const route = brut.replace(/\/index$/, "") || "/";
    if (PUBLIQUES.has(route)) continue;
    routes.add(route);
  }
  return [...routes].sort();
}
