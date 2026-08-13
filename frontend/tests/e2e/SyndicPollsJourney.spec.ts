import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding, ensureAcp } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Sondages — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée un sondage Oui/Non de bout en bout", async ({ page }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "journey-poll");

    // Un sondage exige total_eligible_voters > 0, calculé côté backend à
    // partir des unit_owners actifs de l'immeuble (poll_use_cases.rs) — un
    // immeuble sans aucun propriétaire lié à un lot ne peut jamais avoir de
    // sondage créé (règle métier légitime, pas un bug produit). Un vrai
    // immeuble a toujours au moins un propriétaire.
    const acpId = await ensureAcp(
      page,
      ctx.orgId,
      ctx.adminToken,
      "journey-poll",
    );
    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: ctx.orgId,
        first_name: "Voter",
        last_name: `Test${Date.now()}`,
        email: `voter-${Date.now()}@test.com`,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    const owner = await ownerResp.json();
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: ctx.buildingId,
        unit_number: `POLL-${Date.now()}`,
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
          owner_id: owner.id,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${ctx.token}` },
      },
    );
    expect(linkResp.status()).toBe(201);

    await page.goto("/polls/new", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    await page
      .getByTestId("create-poll-question-input")
      .fill(`Faut-il repeindre le hall ? ${Date.now()}`);

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/polls") && r.request().method() === "POST",
      ),
      page.getByTestId("create-poll-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
