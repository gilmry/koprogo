import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

// /owner/profile redirige vers /profile (ProfilePanel.svelte), une page
// générique partagée par tous les rôles — jusqu'ici sans aucune couverture
// E2E dédiée (Gdpr.spec.ts couvre /settings/gdpr, une page différente).
test.describe("Copropriétaire — Mon profil, parcours de modification rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("modifie le prénom via le droit de rectification (Art. 16 RGPD)", async ({
    page,
  }) => {
    await loginAsSyndicWithLinkedOwner(page, "journey-owner-profile");

    await page.goto("/owner/profile", { waitUntil: "networkidle" });
    await expect(page).toHaveURL(/\/profile\/?$/);
    await expect(page.getByTestId("profile-panel")).toBeVisible();

    await page.getByRole("button", { name: "Modifier" }).click();

    const newFirstName = `Public${Date.now()}`;
    await page.locator("#firstName").fill(newFirstName);

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().endsWith("/gdpr/rectify") && r.request().method() === "PUT",
      ),
      page.getByRole("button", { name: "Enregistrer" }).click(),
    ]);
    expect(resp.status()).toBe(200);

    await expect(page.locator("body")).toContainText(newFirstName);
  });
});
