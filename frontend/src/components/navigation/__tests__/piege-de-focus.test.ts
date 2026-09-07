import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * Garde-fou : le tiroir mobile retient le focus, et son overlay est un vrai
 * bouton.
 *
 * ── Ce qui a été trouvé ────────────────────────────────────────────────────
 *
 * Revue de design du 2026-09-06, point 0.9. Deux défauts sur le même élément :
 *
 * **Le focus s'échappait.** Il était bien DÉPLACÉ à l'ouverture — sur le
 * bouton de fermeture — et rendu au hamburger à la fermeture. Mais rien ne le
 * RETENAIT : une tabulation de plus l'emmenait derrière l'overlay, dans une
 * page que l'utilisateur ne voit pas et dont il ne sort qu'en tabulant à
 * l'aveugle jusqu'au bout.
 *
 * Pour quelqu'un qui navigue au clavier ou au lecteur d'écran, c'est un
 * cul-de-sac : le contenu annoncé n'est pas celui qui est affiché.
 *
 * **L'overlay était un `<div role="button">`** — un bouton déguisé, que les
 * technologies d'assistance annoncent comme tel sans qu'il en soit un.
 *
 * ── Pourquoi cela compte plus qu'il n'y paraît ─────────────────────────────
 *
 * Le tiroir mobile est le seul chemin de navigation sur téléphone, et le
 * téléphone est là où ce produit vit vraiment : un syndic devant une cave
 * inondée, un copropriétaire qui vote depuis son canapé (#795, #825).
 *
 * `AccessibleModal.svelte` implémentait déjà ce piège. Il n'y avait rien à
 * inventer, seulement à réemployer — ce qui rend l'absence plus coûteuse à
 * expliquer qu'à corriger.
 *
 * Voir #794.
 */

const NAVIGATION = join(
  process.cwd(),
  "src/components/navigation/Navigation.svelte",
);

describe("le tiroir mobile retient le focus (#794)", () => {
  const source = readFileSync(NAVIGATION, "utf8");

  it("enferme la tabulation dans le tiroir ouvert", () => {
    expect(
      source,
      "Le piège de focus a disparu. Une tabulation emmène de nouveau " +
        "l'utilisateur derrière l'overlay, dans une page qu'il ne voit pas " +
        "(#794).",
    ).toContain("const piegerLeFocus");

    // Il ne suffit pas que la fonction EXISTE : il faut qu'elle soit
    // BRANCHÉE. Une première version de ce test se contentait de sa présence,
    // et passait encore après qu'on eut retiré son appel du gestionnaire de
    // touches — le piège était mort et le garde-fou restait vert.
    //
    // C'est le motif dominant de ce produit reproduit dans son propre
    // filet : une capacité écrite, testée, et qui ne s'exécute jamais.
    const fenetre = source.slice(source.indexOf("<svelte:window"));
    expect(
      fenetre,
      "`piegerLeFocus` n'est plus appelée par le gestionnaire de touches de " +
        "la fenêtre. La fonction existe et ne s'exécute jamais.",
    ).toContain("piegerLeFocus(e)");

    // Les deux sens comptent : Maj+Tab depuis le premier élément doit
    // ramener au dernier, sans quoi le piège fuit par le haut.
    expect(
      source,
      "Le piège ne traite plus Maj+Tab : le focus s'échappe par le haut du " +
        "tiroir, ce qui est aussi gênant que par le bas.",
    ).toMatch(/shiftKey/);

    expect(
      source,
      "Le tiroir n'est plus lié : le piège n'a plus de quoi énumérer ses " +
        "éléments focusables.",
    ).toContain("bind:this={drawerElement}");
  });

  it("n'emploie pas de bouton déguisé pour l'overlay", () => {
    // On retire les commentaires AVANT de chercher : celui qui explique le
    // défaut cite `role="button"`, et le compter reviendrait à signaler la
    // documentation du correctif comme si elle était le défaut.
    //
    // `garde-secrets-console.test.ts` a le même réglage, pour la même raison :
    // il retire les littéraux de chaîne avant de chercher un jeton journalisé.
    const sansCommentaires = source
      .replace(/<!--[\s\S]*?-->/g, "")
      .replace(/\/\/[^\n]*/g, "");

    const ancre = sansCommentaires.indexOf("Fermer le menu");
    const overlay = sansCommentaires.slice(ancre - 400, ancre + 100);

    expect(
      overlay,
      'L\'overlay est redevenu un `<div role="button">`. Les technologies ' +
        "d'assistance l'annoncent comme un bouton sans qu'il en soit un " +
        '(#794). Employez un `<button type="button">`.',
    ).not.toMatch(/role="button"/);

    expect(
      overlay,
      "L'overlay n'est plus un bouton du tout : son clic ne serait plus " +
        "annoncé, et fermer le menu deviendrait impossible autrement que par " +
        "Échap.",
    ).toMatch(/<button/);
  });

  /// Le focus doit aussi ALLER quelque part à l'ouverture, et REVENIR à la
  /// fermeture. Sans cela, le piège enferme un focus qui n'est pas entré.
  it("déplace le focus à l'ouverture et le rend à la fermeture", () => {
    expect(source).toContain("drawerCloseButton?.focus()");
    expect(source).toContain("hamburgerButton?.focus()");
  });
});
