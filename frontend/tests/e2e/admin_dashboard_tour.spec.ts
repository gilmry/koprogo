import { test, expect } from "@playwright/test";

test("test", async ({ page }) => {
  await page.goto("http://localhost/");
  await page.getByRole("link", { name: "Se connecter" }).click();
  await expect(
    page.getByRole("button", { name: "Se connecter" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Email" }).click();
  await page.getByRole("textbox", { name: "Email" }).fill("admin@koprogo.com");
  await page.getByRole("textbox", { name: "Mot de passe" }).click();
  await page.getByRole("textbox", { name: "Mot de passe" }).fill("admin123");
  await page.getByRole("button", { name: "Se connecter" }).click();
  await page.getByRole("link", { name: "⚙️ Administration" }).click();
  await page.getByRole("link", { name: "🏢 Buildings" }).click();
  await expect(
    page.getByRole("heading", { name: "🏢 Gestion des Immeubles" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "✏️" }).first().click();
  await expect(
    page.getByRole("heading", { name: "Modifier l'Immeuble" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enregistrer les modifications" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Enregistrer les modifications" })
    .click();
  await page.getByRole("link", { name: "Détails →" }).first().click();
  await expect(
    page.getByRole("heading", { name: "Informations de l'immeuble" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "✏️ Modifier" })).toBeVisible();
  await page.getByRole("button", { name: "✏️ Modifier" }).click();
  await expect(
    page.getByRole("heading", { name: "Modifier l'Immeuble" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enregistrer les modifications" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Enregistrer les modifications" })
    .click();
  await page.getByRole("link", { name: "🏛️ Organisations" }).click();
  await expect(
    page.getByRole("link", { name: "🏛️ Organisations" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "➕ Nouvelle organisation" }),
  ).toBeVisible();
  await page.getByText("✏️ ⏸️ 🗑️").first().click();
  await page.getByRole("button", { name: "⏸️" }).first().click();
  await page.getByRole("button", { name: "✏️" }).first().click();
  await expect(
    page.getByRole("heading", { name: "Modifier l'Organisation" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Enregistrer les modifications" }),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Enregistrer les modifications" })
    .click();
  await page.getByRole("button", { name: "▶️" }).nth(1).click();
  await page.getByRole("button", { name: "▶️" }).click();
  await expect(
    page.getByRole("link", { name: "👥 Utilisateurs" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "👥 Utilisateurs" }).click();
  await expect(page.getByText("Gérer tous les utilisateurs")).toBeVisible();
  await page.getByRole("button", { name: "✏️" }).first().click();
  await expect(
    page.getByRole("heading", { name: "Modifier un utilisateur" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Mettre à jour" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Mettre à jour" }).click();
  await page.getByRole("button", { name: "Close" }).click();
});
