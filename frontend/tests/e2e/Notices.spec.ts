import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function setupSyndicWithBuilding(page: import("@playwright/test").Page) {
  const ctx = await loginAsSyndicWithBuilding(page, "notice");
  return { token: ctx.token, buildingId: ctx.buildingId, orgId: ctx.orgId };
}

test.describe("Notices - Community Board", () => {
  test("should display notices page", async ({ page }) => {
    await setupSyndicWithBuilding(page);
    await page.goto("/notices");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("h1, h2, [data-testid='notices-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a notice via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const noticeResp = await page.request.post(`${API_BASE}/notices`, {
      data: {
        building_id: buildingId,
        title: `Travaux ascenseur ${timestamp}`,
        content: "L'ascenseur sera en maintenance le 20 mars de 8h à 12h.",
        notice_type: "Maintenance",
        priority: "Normal",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(noticeResp.status())).toBeTruthy();
    const notice = await noticeResp.json();
    expect(notice.building_id).toBe(buildingId);
  });

  test("should navigate to notice detail", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);
    const timestamp = Date.now();

    const noticeResp = await page.request.post(`${API_BASE}/notices`, {
      data: {
        building_id: buildingId,
        title: `Notice detail ${timestamp}`,
        content: "Contenu de l'annonce.",
        notice_type: "General",
        priority: "Low",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const notice = await noticeResp.json();

    await page.goto(`/notice-detail?id=${notice.id}`);
    await expect(page.locator("body")).toBeVisible();
  });

  test("should list building notices via API", async ({ page }) => {
    const { token, buildingId } = await setupSyndicWithBuilding(page);

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/notices`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });
});
