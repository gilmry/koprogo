import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding, loginAsAdmin } from "./helpers/auth";

/**
 * Non-régression des défauts relevés par l'audit du 2026-08-30.
 *
 * Contexte, qui explique la forme de ce fichier : sur les quatre bugs
 * « critiques » du rapport, trois ne se reproduisaient pas contre le code
 * courant. koprogo.com servait alors des images `:latest` construites depuis
 * une AUTRE branche que celle déployée — l'audit portait donc sur un build
 * sans rapport avec le dépôt. Le décalage a été corrigé le même jour.
 *
 * Ces tests couvrent quand même les parcours incriminés. Un rapport qui
 * signale un bouton mort mérite une preuve durable qu'il ne l'est pas, pas
 * une affirmation ponctuelle. Ils échoueront si le comportement se dégrade
 * réellement.
 */

test.describe("Audit 2026-08-30 — non-régression", () => {
  // B3 : le message existait, mais c'était la chaîne brute du backend.
  test("un identifiant erroné affiche un message dans la langue de l'interface", async ({
    page,
  }) => {
    await page.goto("/login");
    await page.fill('input[type="email"]', "inexistant@example.invalid");
    await page.fill('input[type="password"]', "mauvais-mot-de-passe");
    await page.click('button[type="submit"]');

    const error = page.getByTestId("login-error");
    await expect(error).toBeVisible({ timeout: 10000 });

    // Le cœur du correctif : plus jamais la chaîne technique du serveur.
    await expect(error).not.toHaveText(/Invalid credentials/i);
    await expect(error).not.toHaveText(/^\s*$/);
  });

  // B1 : signalé comme « bouton inopérant, pas de modal ».
  test("le bouton Nouvelle réunion ouvre le modal de création", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "audit-meeting");
    await page.goto("/meetings");

    const button = page.getByTestId("btn-new-meeting");
    await expect(button).toBeVisible({ timeout: 15000 });
    await button.click();

    await expect(
      page.locator('[role="dialog"][aria-label="Créer une assemblée"]'),
    ).toBeVisible({ timeout: 5000 });
  });

  // Défaut trouvé en reproduisant B1, absent du rapport : le modal ne se
  // fermait qu'à la souris, ce qui piège un utilisateur au clavier.
  test("le modal de création se ferme avec Échap", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "audit-escape");
    await page.goto("/meetings");

    await page.getByTestId("btn-new-meeting").click();
    const dialog = page.locator('[role="dialog"][aria-label="Créer une assemblée"]');
    await expect(dialog).toBeVisible({ timeout: 5000 });

    await page.keyboard.press("Escape");
    await expect(dialog).toHaveCount(0, { timeout: 5000 });
  });

  // B2 : signalé comme « dropdown ne s'ouvre pas ».
  test("la cloche de notifications ouvre son panneau", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "audit-bell");
    await page.goto("/syndic");

    // Deux cloches existent dans le DOM (entête large et menu mobile) ;
    // seule la visible est cliquable.
    const bell = page
      .locator('button[aria-label="Notifications"]:visible')
      .first();
    await expect(bell).toBeVisible({ timeout: 15000 });
    await expect(bell).toHaveAttribute("aria-expanded", "false");

    await bell.click();
    await expect(bell).toHaveAttribute("aria-expanded", "true", {
      timeout: 5000,
    });
  });

  // B4 : signalé comme « perte de session sur URL admin inconnue ».
  test("une URL admin inconnue ne détruit pas la session", async ({ page }) => {
    await loginAsAdmin(page);

    await page.goto("/admin/url-qui-nexiste-pas");
    await page.waitForLoadState("networkidle");

    // Le retour vers une page admin réelle doit rester possible sans
    // repasser par la connexion.
    await page.goto("/admin");
    await page.waitForLoadState("networkidle");
    expect(page.url()).not.toContain("/login");
  });
});
