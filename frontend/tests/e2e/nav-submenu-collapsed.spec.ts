import { test, expect } from "@playwright/test";
import { humanClick } from "./helpers/video-pace";

/**
 * Non-régression de la cause unique des 11 échecs `[scenarios]` en CI
 * (#696, WP-D1).
 *
 * `RoleSubmenu.svelte` range les liens de navigation dans un `<details>` dont
 * l'ouverture est dérivée : `isOpen = defaultOpen || containsActive`. Aucun
 * appelant ne passe `defaultOpen` — une section n'est donc ouverte QUE si elle
 * contient la page courante. Depuis le tableau de bord, « Budgets » est replié.
 *
 * Un `<details>` fermé laisse ses enfants dans le DOM mais les rend
 * INVISIBLES. Playwright résolvait donc le lien puis attendait sa visibilité
 * jusqu'au timeout de 30 s, sur les 11 scénarios, aux deux reprises, de façon
 * parfaitement déterministe :
 *
 *     locator resolved to <a href="/budgets" data-testid="nav-link-budgets">
 *     attempting click action
 *       - element is not visible          (x60, jusqu'au timeout)
 *
 * Ces tests n'ont besoin d'aucun serveur : ils reproduisent la structure DOM
 * en isolation via `page.setContent`, ce qui les rend rapides et insensibles
 * à l'état des seeds.
 */

// Structure minimale reproduisant RoleSubmenu.svelte.
const MENU = `<!doctype html><html><body>
  <details data-testid="navigation-menu-finance">
    <summary>Finance</summary>
    <ul role="list">
      <li>
        <a href="#budgets"
           data-testid="nav-link-budgets"
           onclick="document.title='NAVIGUE'">Budgets</a>
      </li>
    </ul>
  </details>
</body></html>`;

test.describe("Navigation — sections repliées (#696 / WP-D1)", () => {
  test("@negative un lien dans un <details> fermé est présent mais invisible", async ({
    page,
  }) => {
    await page.setContent(MENU);
    const link = page.getByTestId("nav-link-budgets");

    // Les deux assertions ensemble SONT la cause racine : Playwright trouve
    // l'élément (donc pas d'erreur de sélecteur), mais ne peut pas le cliquer.
    await expect(link).toHaveCount(1);
    await expect(link).not.toBeVisible();
  });

  test("@happy humanClick déplie la section puis clique le lien", async ({
    page,
  }) => {
    await page.setContent(MENU);
    await humanClick(page, "nav-link-budgets");
    await expect(page).toHaveTitle("NAVIGUE");
  });

  test("@edge humanClick reste correct sur une section déjà ouverte", async ({
    page,
  }) => {
    await page.setContent(MENU.replace("<details", "<details open"));
    await humanClick(page, "nav-link-budgets");
    await expect(page).toHaveTitle("NAVIGUE");
  });

  test("@security le dépliage ne masque pas un lien réellement absent", async ({
    page,
  }) => {
    // Garde-fou : le correctif ne doit pas transformer un sélecteur erroné en
    // succès silencieux. Un testid inexistant doit toujours échouer.
    await page.setContent(MENU);
    await expect(humanClick(page, "nav-link-inexistant")).rejects.toThrow();
  });
});
