import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner, ensureAcp } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Compétences — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("propose une compétence de bout en bout, en tant que propriétaire lié", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithLinkedOwner(page, "journey-skill");

    // GET /buildings est scopé via unit_owners pour le rôle owner — même
    // piège de fixture que SEL/Sondages, cf. findings.md.
    const acpId = await ensureAcp(
      page,
      ctx.orgId,
      ctx.adminToken,
      "journey-skill",
    );
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: ctx.buildingId,
        unit_number: `SKILL-${Date.now()}`,
        floor: 1,
        surface_area: 85.0,
        unit_type: "Apartment",
        quota: 1000.0,
      },
      headers: { Authorization: `Bearer ${ctx.adminToken}` },
    });
    const unit = await unitResp.json();
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

    await page.goto("/skills", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    await page.locator("#create-offer-btn").click();
    const form = page.getByTestId("skill-offer-create-form");
    await expect(form).toBeVisible();

    const skillName = `Plomberie ${Date.now()}`;
    await form.locator("#skill_name").fill(skillName);
    await form
      .locator("#description")
      .fill("Je peux aider pour des petites réparations de plomberie.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().endsWith("/skills") && r.request().method() === "POST",
      ),
      page.getByTestId("submit-skill-offer-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
