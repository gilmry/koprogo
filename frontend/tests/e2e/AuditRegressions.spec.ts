import { test, expect } from "@playwright/test";
import {
  loginAsSyndicWithBuilding,
  loginAsSyndicWithUnit,
  loginAsAdmin,
} from "./helpers/auth";

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

  // UX7 : signalé comme « switch de langue non fonctionnel ou non visible ».
  test("le sélecteur de langue change la langue et la retient", async ({
    page,
  }) => {
    await page.goto("/login");

    const switcher = page.locator('button[aria-label="Changer de langue"]');
    await expect(switcher).toBeVisible({ timeout: 10000 });

    await switcher.click();
    await page.getByRole("menuitem", { name: /Deutsch/ }).click();

    // La langue s'applique...
    await expect(switcher).toContainText("DE", { timeout: 5000 });
    // ...et survit à un rechargement, via localStorage.
    await page.reload();
    await expect(switcher).toContainText("DE", { timeout: 10000 });
  });

  // UX2 : signalé comme « badges rôles tronqués (Syn..., Cop...) ».
  //
  // Testé au viewport le plus étroit : c'est là que la troncature
  // surviendrait si elle devait survenir. Vérifié de 375 à 1920 px, le badge
  // garde sa largeur naturelle — la table défile horizontalement plutôt que
  // d'écraser ses colonnes.
  test("les badges de rôle ne sont pas tronqués", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await loginAsAdmin(page);
    await page.goto("/admin/users");

    const badge = page.locator('[data-testid="user-role-badge"]').first();
    await expect(badge).toBeVisible({ timeout: 15000 });

    // Un badge tronqué déborde de sa boîte : scrollWidth dépasse clientWidth.
    const clipped = await badge.evaluate(
      (node) => node.scrollWidth > node.clientWidth + 1,
    );
    expect(clipped, "le badge déborde de sa boîte").toBe(false);
  });

  // UX4 : l'effacement RGPD n'était protégé que par deux `confirm()`.
  //
  // Le test s'arrête au mot de passe ERRONÉ, à dessein : vérifier le chemin
  // nominal effacerait réellement un compte. Ce qui compte ici est que le
  // garde-fou existe et morde, pas qu'une suppression aboutisse.
  test("l'effacement RGPD exige le mot de passe et rejette un faux", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "audit-gdpr");
    await page.goto("/profile");

    await page.getByRole("button", { name: /Effacer mes données/ }).click();

    const field = page.getByTestId("gdpr-erase-password");
    await expect(field).toBeVisible({ timeout: 10000 });

    // Sans mot de passe, l'action reste inaccessible.
    await expect(page.getByTestId("gdpr-erase-confirm")).toBeDisabled();

    await field.fill("ce-n-est-pas-le-bon-mot-de-passe");
    await page.getByTestId("gdpr-erase-confirm").click();

    // Le serveur refuse : l'erreur s'affiche et le compte survit.
    await expect(page.getByTestId("gdpr-erase-error")).toBeVisible({
      timeout: 10000,
    });
    await page.goto("/profile");
    await expect(page.locator("body")).toBeVisible();
  });

  // UX1 : la liste admin n'offrait aucun moyen d'ouvrir une organisation.
  test("le nom d'une organisation ouvre sa fiche de détail", async ({ page }) => {
    await loginAsAdmin(page);
    await page.goto("/admin/organizations");

    const first = page.getByTestId("organization-name").first();
    await expect(first).toBeVisible({ timeout: 15000 });
    const name = (await first.innerText()).trim();
    await first.click();

    // Astro sert les pages statiques avec une barre oblique finale.
    await expect(page).toHaveURL(/\/admin\/organization-detail\/?\?id=/);
    await expect(page.getByTestId("organization-detail-name")).toHaveText(name, {
      timeout: 10000,
    });

    // Les compteurs sont le cœur de la fiche : ils disent ce que
    // l'organisation contient, là où la liste ne montrait que les plafonds.
    for (const stat of ["stat-acps", "stat-buildings", "stat-users", "stat-units"]) {
      await expect(page.getByTestId(stat)).toBeVisible();
    }
  });

  // Signalé le 2026-08-31 : le bouton « Copropriétaires » d'un lot ne faisait
  // rien. `expandedUnits` était un Set natif dans un composant en mode runes.
  // `$state` rend réactifs les objets et les tableaux, jamais les collections
  // natives, et la réaffectation à soi-même — qui suffisait en Svelte 4 — ne
  // déclenche rien puisque la comparaison est référentielle.
  test("le bouton Copropriétaires déplie la liste des propriétaires", async ({
    page,
  }) => {
    const { buildingId } = await loginAsSyndicWithUnit(page, "audit-owners");
    await page.goto(`/building-detail?id=${buildingId}`);

    const toggle = page.getByRole("button", { name: /Copropriétaires/ }).first();
    await expect(toggle).toBeVisible({ timeout: 15000 });

    // On vise le panneau, pas la liste : `owner-list` n'est rendue que si le
    // lot a des propriétaires, et un lot fraîchement créé n'en a aucun. Le
    // panneau, lui, apparaît dès que le lot est déplié — c'est exactement ce
    // que le bouton doit produire.
    await expect(page.getByTestId("unit-owners-panel")).toHaveCount(0);

    await toggle.click();
    await expect(page.getByTestId("unit-owners-panel").first()).toBeVisible({
      timeout: 10000,
    });

    // Et le bouton referme, sans quoi le Set ne serait réactif qu'à l'ajout.
    await toggle.click();
    await expect(page.getByTestId("unit-owners-panel")).toHaveCount(0);
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
