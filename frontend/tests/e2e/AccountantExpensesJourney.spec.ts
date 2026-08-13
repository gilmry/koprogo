import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Comptable — Dépenses, parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une dépense de bout en bout et la voit dans la liste", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "journey-expense");
    await page.goto("/expenses", { waitUntil: "networkidle" });

    await page.getByTestId("create-button").click();

    await page.getByTestId("building-select").selectOption({ index: 1 });
    const description = `Réparation ascenseur ${Date.now()}`;
    await page.getByTestId("description-input").fill(description);
    await page.getByTestId("amount-input").fill("1500");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/expenses") && r.request().method() === "POST",
      ),
      page.getByTestId("submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);

    await expect(page.getByText(description)).toBeVisible();
  });
});
