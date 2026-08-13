import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Inspections techniques — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("planifie une inspection de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-inspection");
    await page.goto("/inspections", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    await page.getByTestId("create-inspection-button").click();
    await page
      .locator("#insp-new-title")
      .fill(`Contrôle ascenseur ${Date.now()}`);
    await page.locator("#insp-new-inspector").fill("Jean Dupont");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/technical-inspections") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("submit-inspection-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
