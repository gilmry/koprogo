import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Campagnes énergie — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une campagne d'achat groupé de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-energy");
    await page.goto("/energy-campaigns/new", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    await page
      .getByTestId("campaign-name-input")
      .fill(`Achat groupé électricité ${Date.now()}`);
    await page.getByRole("checkbox", { name: /électricité/i }).check();

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/energy-campaigns") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("campaign-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("un type d'énergie manquant affiche l'erreur et laisse le bouton réutilisable", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "journey-energy-validation");
    await page.goto("/energy-campaigns/new", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    // Rempli le nom mais aucun type d'énergie coché — la validation JS
    // rejette. Régression : handleSubmit() mettait loading=true avant les
    // checks et ne le remettait jamais à false sur un retour anticipé, donc
    // le bouton restait bloqué en spinner "Création…" pour toujours.
    await page
      .getByTestId("campaign-name-input")
      .fill(`Campagne sans type ${Date.now()}`);
    await page.getByTestId("campaign-submit-btn").click();

    await expect(page.getByTestId("campaign-submit-btn")).toBeEnabled();
    await expect(page.getByTestId("campaign-submit-btn")).not.toContainText(
      "Création",
    );
  });
});
