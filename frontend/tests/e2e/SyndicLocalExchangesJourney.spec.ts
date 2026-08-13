import { test, expect } from "@playwright/test";
import { loginAsSyndicWithLinkedOwner, ensureAcp } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Échanges locaux (SEL) — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une offre d'échange de bout en bout, en tant que propriétaire lié", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithLinkedOwner(page, "journey-exch");

    // GET /buildings est scopé via unit_owners pour le rôle owner (Story
    // 1.3 / BUG-WF14-2) — un Owner sans lot lié ne voit aucun immeuble,
    // donc le BuildingSelector du formulaire reste vide et le submit ne
    // peut jamais aboutir. Un vrai propriétaire SEL a toujours un lot.
    const acpId = await ensureAcp(
      page,
      ctx.orgId,
      ctx.adminToken,
      "journey-exch",
    );
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: ctx.buildingId,
        unit_number: `SEL-${Date.now()}`,
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

    await page.goto("/exchanges/new", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    const title = `Aide jardinage ${Date.now()}`;
    await page.getByTestId("exchange-title-input").fill(title);
    await page
      .getByTestId("exchange-description-input")
      .fill("Tonte pelouse et taille de haie pour un voisin.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/exchanges") && r.request().method() === "POST",
      ),
      page.getByTestId("exchange-submit-btn").click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
