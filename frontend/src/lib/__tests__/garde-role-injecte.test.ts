import { describe, expect, it } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

/**
 * On n'établit pas un rôle en écrivant dans `localStorage`.
 *
 * TROIS recettes injectaient un utilisateur dans `koprogo_user` pour « se
 * connecter » en comptable ou en copropriétaire — les deux parcours
 * comptables et le test `@security` du sélecteur d'immeubles.
 *
 * (J'avais d'abord annoncé neuf. C'était un artefact de `grep` : le motif
 * attrapait toute mention de la clé, or six autres fichiers ne font que
 * `removeItem` entre deux connexions, ce qui est légitime. Quinzième mesure
 * exagérée de cette session, et à peu près toutes étaient les miennes.)
 *
 * Ces injections ne pouvaient rien établir, et c'est une bonne nouvelle : `stores/auth.ts:182` dit que
 * cette clé est « un cache d'affichage NON sensible, jamais une preuve
 * d'authentification », et qu'`init()` fait un silent-refresh via le cookie
 * HttpOnly pour confirmer la session.
 *
 * Le cache injecté est donc écrasé par le VRAI utilisateur — celui du cookie,
 * c'est-à-dire l'admin qui a servi à créer les données. **On ne peut pas
 * changer de rôle en modifiant `localStorage`, et c'est exactement ce qu'on
 * veut d'un produit.**
 *
 * Ce que ça produisait, mesuré sur le run 34347631686 :
 *
 *   - `AccountantJournalEntriesJourney` attendait `#description` alors que
 *     l'instantané de page montre « Bienvenue, Admin » et le tableau de bord
 *     administrateur. Ni comptable, ni sur la page des écritures ;
 *   - `building-selector.spec.ts` portait une assertion `@security` — « un
 *     copropriétaire ne voit pas le sélecteur » — qui n'a jamais éprouvé ce
 *     qu'elle annonce, puisqu'aucun copropriétaire n'était connecté.
 *
 * Une recette qui croit avoir un rôle qu'elle n'a pas ne teste pas ce qu'elle
 * dit. C'est plus grave qu'un test rouge : c'est un test qui ment sur son
 * sujet.
 *
 * `removeItem("koprogo_user")` reste permis et n'est pas compté : vider le
 * cache entre deux connexions est légitime. Seule l'ÉCRITURE est interdite.
 *
 * Le remède est en place et éprouvé : `uiLoginWithRetry(page, email, mot de
 * passe, /\/role/)`. Ces recettes créent déjà l'utilisateur avec son mot de
 * passe ; il suffit de s'en servir.
 */

const RACINE = join(process.cwd(), "tests/e2e");

/**
 * Zéro : c'est une interdiction, pas une dette. Les trois seules injections
 * ont été remplacées par une connexion réelle le 2026-09-09.
 */
const DETTE_AU_2026_09_09 = 0;

const INJECTION = /localStorage\.setItem\(\s*["']koprogo_user["']/g;

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

function injections(): string[] {
  const trouvees: string[] = [];
  for (const fichier of recettes(RACINE)) {
    const code = readFileSync(fichier, "utf-8")
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\/\/[^\n]*/g, "");
    for (const m of code.matchAll(INJECTION)) {
      const ligne = code.slice(0, m.index).split("\n").length;
      trouvees.push(`  ${relative(process.cwd(), fichier)}:${ligne}`);
    }
  }
  return trouvees;
}

describe("les recettes n'injectent pas leur rôle dans localStorage", () => {
  it("n'ajoute pas d'injection de session", () => {
    const trouvees = injections();
    expect(
      trouvees.length,
      `${trouvees.length} injections de \`koprogo_user\`, contre ` +
        `${DETTE_AU_2026_09_09} au 2026-09-09.\n\n` +
        `Cette clé est un cache d'affichage : le silent-refresh la remplace ` +
        `par l'utilisateur du cookie. La recette croit alors avoir un rôle ` +
        `qu'elle n'a pas, et ne teste plus ce qu'elle annonce.\n\n` +
        `Utilisez \`uiLoginWithRetry(page, email, motDePasse, /\\/role/)\`.\n\n` +
        trouvees.join("\n"),
    ).toBeLessThanOrEqual(DETTE_AU_2026_09_09);
  });

  it("lit encore les recettes", () => {
    // Contrôle d'aveuglement. Il ne porte PAS sur le nombre d'infractions :
    // une garde qui exige des violations punit sa propre réussite.
    const fichiers = recettes(RACINE);
    expect(fichiers.length).toBeGreaterThan(50);
    const connexions = fichiers
      .map((f) => readFileSync(f, "utf-8"))
      .join("\n")
      .match(/login|Login/g);
    expect(
      connexions?.length ?? 0,
      "plus aucune connexion dans les recettes : le répertoire a bougé.",
    ).toBeGreaterThan(50);
  });
});
