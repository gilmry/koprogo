import { test, expect } from "@playwright/test";
import { loginAsSyndicWithUnit } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Unit Owners - Multi-Owner Support", () => {
  test("should display units page", async ({ page }) => {
    await loginAsSyndicWithUnit(page, "unitowner");
    await page.goto("/units");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='units-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create an owner and assign to unit", async ({ page }) => {
    const { token, unitId, orgId } = await loginAsSyndicWithUnit(
      page,
      "unitowner",
    );
    const timestamp = Date.now();

    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Marie",
        last_name: `Copropriétaire${timestamp}`,
        email: `owner-unit-${timestamp}@test.com`,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const owner = await ownerResp.json();

    const assignResp = await page.request.post(
      `${API_BASE}/units/${unitId}/owners`,
      {
        data: {
          owner_id: owner.id,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect([200, 201].includes(assignResp.status())).toBeTruthy();
  });

  test("should list owners for a unit", async ({ page }) => {
    const { token, unitId } = await loginAsSyndicWithUnit(page, "unitowner");

    const listResp = await page.request.get(
      `${API_BASE}/units/${unitId}/owners`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const owners = await listResp.json();
    expect(Array.isArray(owners)).toBeTruthy();
  });

  test("should get total ownership percentage for unit", async ({ page }) => {
    const { token, unitId } = await loginAsSyndicWithUnit(page, "unitowner");

    const pctResp = await page.request.get(
      `${API_BASE}/units/${unitId}/owners/total-percentage`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(pctResp.ok()).toBeTruthy();
  });
});
