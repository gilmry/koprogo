import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Le voile de `RouteGuard` doit pouvoir se retirer, même quand rien ne
 * répond.
 *
 * ── Le dégât ──────────────────────────────────────────────────────────────
 *
 * `RouteGuard.svelte` couvre l'écran pendant qu'il vérifie l'accès, et il
 * s'accorde quinze secondes pour que `authStore.init()` aboutisse. Le
 * commentaire de ce plafond est explicite :
 *
 *   « L'alternative est un écran qui ne répond plus jamais, ce qui est
 *     strictement pire — et il n'a aucun moyen de le savoir. »
 *
 * C'était pourtant ce qui se passait. `init()` ne remet `isLoading` à faux
 * que s'il aboutit ; s'il pend — un rafraîchissement silencieux vers un
 * backend muet — `isLoading` reste vrai. La course rejetait bien au bout de
 * quinze secondes, mais `checkAccess()` retournait aussitôt sur
 * `if (isLoading)`, et la souscription ne le rappelait jamais puisque le
 * store n'émettait plus.
 *
 * **Le voile restait, indéfiniment, sur l'écran de connexion.**
 *
 * Constaté le 2026-09-18 sur le premier run de `vitrine.yml` : les cinq
 * parcours filmés bloqués, voile encore présent après 25 secondes. Rien ne
 * se voyait en local, où le serveur est chaud et l'init instantané.
 *
 * ── Ce que cette garde vérifie ────────────────────────────────────────────
 *
 * Elle lit le composant, pas son comportement — c'est une garde de
 * structure, comme les autres de ce dossier. Elle vérifie que le premier
 * contrôle après la course FORCE le passage, et que `isLoading` ne peut
 * plus le retenir seul.
 *
 * Elle ne remplace pas un test de rendu : elle empêche que le correctif
 * soit défait par inadvertance, ce qu'un refactor de ce fichier rendrait
 * facile — la ligne à supprimer n'a l'air de rien.
 */

const COMPOSANT = join(process.cwd(), "src", "components", "RouteGuard.svelte");

function source(): string {
  return readFileSync(COMPOSANT, "utf8");
}

describe("le voile de RouteGuard ne peut pas rester indéfiniment", () => {
  it("@security le premier contrôle après la course force le passage", () => {
    const t = source();
    expect(
      t,
      "`checkAccess(true)` est ce qui rend utile le plafond de quinze " +
        "secondes. Sans lui, un `init()` qui pend laisse `isLoading` à vrai " +
        "et le voile ne se retire JAMAIS.",
    ).toContain("checkAccess(true)");
  });

  it("@negative `isLoading` seul ne peut plus retenir le contrôle", () => {
    const t = source();
    expect(
      t,
      "la sortie anticipée doit pouvoir être forcée : `if (isLoading)` seul " +
        "était le blocage",
    ).toContain("if (isLoading && !forcer)");
    expect(
      /if \(isLoading\)\s*\{/.test(t),
      "il ne doit plus rester de sortie anticipée inconditionnelle sur " +
        "`isLoading`",
    ).toBe(false);
  });

  it("@edge le plafond de la course reste nommé et lisible", () => {
    const t = source();
    // Si ce plafond disparaît, le `checkAccess(true)` ci-dessus devient un
    // contrôle prématuré au lieu d'un filet. Les deux vont ensemble.
    expect(t).toContain("DELAI_INIT_MS");
  });

  it("@happy le voile reste conditionné à `isChecking`", () => {
    const t = source();
    // La garde ne doit pas pousser à supprimer le voile : il a une raison
    // d'être. Ce qu'on refuse, c'est qu'il devienne permanent.
    expect(t).toContain("{#if isChecking}");
    expect(t).toContain("isChecking = false");
  });
});
