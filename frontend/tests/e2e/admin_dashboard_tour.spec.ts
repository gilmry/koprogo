import { test, expect } from "@playwright/test";

test("Admin Dashboard Tour - Idempotent", async ({ page }) => {
  // Generate unique data for this test run
  const timestamp = Date.now();
  const buildingName = `Test Building ${timestamp}`;
  const userEmail = `testuser${timestamp}@example.com`;
  const firstName = `First${timestamp}`;
  const lastName = `Last${timestamp}`;

  // Login
  await page.goto("http://localhost/");
  await page.getByRole("link", { name: "Se connecter" }).click();
  await page.getByRole("textbox", { name: "Email" }).click();
  await page.getByRole("textbox", { name: "Email" }).fill("admin@koprogo.com");
  await page.getByRole("textbox", { name: "Email" }).press("Tab");
  await page.getByRole("textbox", { name: "Mot de passe" }).fill("admin123");
  await expect(
    page.getByRole("button", { name: "Se connecter" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Se connecter" }).click();
  await expect(page.getByText("Dashboard SuperAdmin - Vue d'")).toBeVisible();

  // Navigate to Buildings
  await page.getByRole("link", { name: "🏢 Buildings" }).click();
  await expect(page.getByText("Gérer les immeubles de votre")).toBeVisible();

  // Create new building with unique name
  await page.getByRole("button", { name: "➕ Nouvel immeuble" }).click();

  // Select first organization option dynamically
  const orgSelect = page.getByLabel("Organisation *");
  const firstOrgValue = await orgSelect.locator("option").nth(1).getAttribute("value"); // nth(1) to skip placeholder
  if (firstOrgValue) {
    await orgSelect.selectOption(firstOrgValue);
  }
  await page.getByRole("textbox", { name: "Nom de l'immeuble *" }).click();
  await page
    .getByRole("textbox", { name: "Nom de l'immeuble *" })
    .fill(buildingName);
  await page.getByRole("textbox", { name: "Adresse *" }).click();
  await page
    .getByRole("textbox", { name: "Adresse *" })
    .fill("123 Test Street");
  await page.getByRole("textbox", { name: "Code postal *" }).click();
  await page.getByRole("textbox", { name: "Code postal *" }).fill("1000");
  await page.getByRole("textbox", { name: "Code postal *" }).press("Tab");
  await page.getByRole("textbox", { name: "Ville *" }).fill("Brussels");
  await page.getByRole("textbox", { name: "Ville *" }).press("Tab");
  await page.getByRole("spinbutton", { name: "Nombre de lots *" }).click();
  await page.getByRole("spinbutton", { name: "Nombre de lots *" }).fill("20");
  await page.getByRole("spinbutton", { name: "Année de construction" }).click();
  await page
    .getByRole("spinbutton", { name: "Année de construction" })
    .fill("2000");
  await expect(
    page.getByRole("button", { name: "Créer l'immeuble" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Créer l'immeuble" }).click();

  // Verify building was created
  await expect(page.getByRole("heading", { name: buildingName })).toBeVisible();

  // View building details
  await page.getByRole("link", { name: "Détails →" }).first().click();
  await expect(page.getByText("123 Test Street")).toBeVisible();

  // Navigate to Organizations
  await page.getByRole("link", { name: "🏛️ Organisations" }).click();

  // Expand/collapse organizations
  await page.getByRole("button", { name: "▶️" }).first().click();
  await page.getByRole("button", { name: "▶️" }).nth(1).click();
  await page.getByRole("button", { name: "▶️" }).first().click();
  await page.getByRole("button", { name: "▶️" }).click();

  // Navigate to Users
  await page.getByRole("link", { name: "👥 Utilisateurs" }).click();

  // Create new user with unique email
  await page.getByRole("button", { name: "➕ Nouvel utilisateur" }).click();
  await page.getByRole("textbox", { name: "Email *" }).click();
  await page.getByRole("textbox", { name: "Email *" }).fill(userEmail);
  await page.getByRole("textbox", { name: "Prénom *" }).click();
  await page.getByRole("textbox", { name: "Prénom *" }).fill(firstName);
  await page.getByRole("textbox", { name: "Nom *", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Nom *", exact: true })
    .fill(lastName);
  await page
    .getByRole("textbox", { name: "Mot de passe *", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Mot de passe *", exact: true })
    .fill("Test123456!");
  await page
    .getByRole("textbox", { name: "Confirmation du mot de passe *" })
    .click();
  await page
    .getByRole("textbox", { name: "Confirmation du mot de passe *" })
    .fill("Test123456!");
  await expect(page.getByRole("button", { name: "Créer" })).toBeVisible();
  await page.getByRole("button", { name: "Créer" }).click();

  // Select first organization option dynamically for user role
  const roleOrgSelect = page.getByLabel("", { exact: true });
  const firstRoleOrgValue = await roleOrgSelect
    .locator("option")
    .nth(1)
    .getAttribute("value"); // nth(1) to skip placeholder
  if (firstRoleOrgValue) {
    await roleOrgSelect.selectOption(firstRoleOrgValue);
  }
  await page.getByRole("button", { name: "Créer" }).click();

  // Verify user was created
  await expect(page.getByText(`${firstName} ${lastName}`)).toBeVisible();

  // Logout
  await page.getByRole("button", { name: "SA Super Admin" }).click();
  await page.getByRole("button", { name: "🚪 Logout" }).click();
});
