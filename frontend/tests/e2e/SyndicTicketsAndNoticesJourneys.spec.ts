import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

test.describe("Syndic — parcours de création remplis jusqu'au bout (tickets, notices)", () => {
  test("tickets: crée un ticket de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-ticket");
    await page.goto("/tickets", { waitUntil: "networkidle" });
    // La sélection du building se fait automatiquement (un seul building) via
    // BuildingSelector, mais le warning ne se cache qu'après le callback
    // onSelect — laisser le temps au montage Svelte de résoudre.
    await page.waitForTimeout(500);
    await page.locator("#create-ticket-btn").click();
    await page.getByTestId("ticket-title-input").fill("Fuite robinet cuisine");
    await page
      .getByTestId("ticket-description-input")
      .fill("Fuite constatée sous l'évier, à réparer rapidement.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/tickets") && r.request().method() === "POST",
      ),
      page.getByTestId("ticket-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("notices: crée une annonce de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-notice");
    await page.goto("/notices", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);
    await page.locator("#create-notice-btn").click();
    await page.getByTestId("notice-title-input").fill("Travaux dans le hall");
    await page
      .getByTestId("notice-content-input")
      .fill("Des travaux de peinture auront lieu la semaine prochaine.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/notices") && r.request().method() === "POST",
      ),
      page.getByTestId("notice-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
