import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Comptable — Budgets, parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée un budget annuel de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-budget");
    await page.goto("/budgets", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    await page.getByTestId("create-budget-button").click();

    await page.getByTestId("budget-building-select").selectOption({ index: 1 });
    await page.getByTestId("budget-ordinary-amount").fill("50000");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/budgets") && r.request().method() === "POST",
      ),
      page.getByTestId("budget-submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
