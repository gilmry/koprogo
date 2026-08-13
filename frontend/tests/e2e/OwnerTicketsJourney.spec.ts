import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Copropriétaire — Tickets de maintenance, parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée un ticket de maintenance de bout en bout", async ({ page }) => {
    const ctx = await loginAsSyndicWithLinkedOwner(
      page,
      "journey-owner-ticket",
    );

    const unitsResp = await page.request.get(
      `${API_BASE}/buildings/${ctx.buildingId}/units`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    const unit = (await unitsResp.json())[0];
    const linkResp = await page.request.post(
      `${API_BASE}/units/${unit.id}/owners`,
      {
        data: {
          owner_id: ctx.ownerId,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${ctx.token}` },
      },
    );
    expect(linkResp.status()).toBe(201);

    await page.goto("/owner/tickets", { waitUntil: "networkidle" });

    // Un seul bâtiment lié : BuildingSelector auto-sélectionne, ce qui
    // active le bouton "Créer un ticket" (disabled tant qu'aucun bâtiment
    // n'est sélectionné, cf. owner/tickets.astro:mountList).
    const createBtn = page.locator("#create-ticket-btn");
    await expect(createBtn).toBeEnabled();
    await createBtn.click();

    const form = page.getByTestId("ticket-create-form");
    await expect(form).toBeVisible();

    const title = `Fuite robinet ${Date.now()}`;
    await page.getByTestId("ticket-title-input").fill(title);
    await page
      .getByTestId("ticket-description-input")
      .fill(
        "Le robinet de la cuisine fuit depuis hier soir, intervention nécessaire.",
      );

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().endsWith("/tickets") && r.request().method() === "POST",
      ),
      page.getByTestId("ticket-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);

    await expect(form).toBeHidden();
    await expect(page.locator("body")).toContainText(title);
  });
});
