import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Syndic — parcours de création remplis jusqu'au bout (work-reports)", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("work-reports: crée un rapport de travaux de bout en bout", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "journey-workreport");
    await page.goto("/work-reports", { waitUntil: "networkidle" });
    // BuildingSelector auto-sélectionne le seul immeuble du fixture, mais
    // le montage de WorkReportList dépend du callback onSelect asynchrone.
    await page.waitForTimeout(500);
    await page.getByRole("button", { name: /nouveau rapport/i }).click();
    await page.locator("#wr-new-title").fill("Remplacement chaudière");
    await page.locator("#wr-new-contractor").fill("Chauffage Dupont SPRL");
    await page
      .locator("#wr-new-desc")
      .fill("Remplacement complet de la chaudière collective.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/work-reports") && r.request().method() === "POST",
      ),
      page.getByRole("button", { name: "Créer" }).click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
