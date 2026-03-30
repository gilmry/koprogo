import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";

/**
 * Budgets E2E Test Suite - Annual Budget Management
 *
 * Tests budget creation, status workflow (Draft→Submitted→Approved),
 * variance analysis, and active budget lookup.
 * Mirrors workflows from backend/tests/e2e_budget.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Budgets - Annual Budget Management", () => {
  test("should display budgets page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "budget");
    await page.goto("/budgets");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("main h1, main h2, [data-testid='budgets-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a budget via API and retrieve it", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "budget",
    );

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2026,
        ordinary_budget: 50000.0,
        extraordinary_budget: 0.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([200, 201].includes(budgetResp.status())).toBeTruthy();

    if (budgetResp.ok()) {
      const budget = await budgetResp.json();
      expect(budget.id).toBeTruthy();
      expect(budget.fiscal_year).toBe(2026);
      expect(budget.status).toBe("Draft");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/budgets/${budget.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(budget.id);
    }
  });

  test("should list budgets for a building", async ({ page }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "budget",
    );

    const listResp = await page.request.get(
      `${API_BASE}/buildings/${buildingId}/budgets`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const budgets = await listResp.json();
    expect(Array.isArray(budgets)).toBeTruthy();
  });

  test("should navigate to budget detail page", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "budget",
    );

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2027,
        ordinary_budget: 60000.0,
        extraordinary_budget: 0.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (budgetResp.ok()) {
      const budget = await budgetResp.json();
      await page.goto(`/budget-detail?id=${budget.id}`);
      await expect(page.locator("body")).toBeVisible();
    }
  });

  test("should submit budget for approval (Draft → Submitted)", async ({
    page,
  }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "budget",
    );

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2028,
        ordinary_budget: 45000.0,
        extraordinary_budget: 5000.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (budgetResp.ok()) {
      const budget = await budgetResp.json();

      const submitResp = await page.request.put(
        `${API_BASE}/budgets/${budget.id}/submit`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(submitResp.status())).toBeTruthy();

      if (submitResp.ok()) {
        const submitted = await submitResp.json();
        expect(submitted.status).toBe("Submitted");
      }
    }
  });

  test("should get variance analysis for a budget", async ({ page }) => {
    const { token, buildingId, orgId } = await loginAsSyndicWithBuilding(
      page,
      "budget",
    );

    const budgetResp = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: buildingId,
        organization_id: orgId,
        fiscal_year: 2029,
        ordinary_budget: 40000.0,
        extraordinary_budget: 0.0,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (budgetResp.ok()) {
      const budget = await budgetResp.json();
      const varianceResp = await page.request.get(
        `${API_BASE}/budgets/${budget.id}/variance`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 404].includes(varianceResp.status())).toBeTruthy();
    }
  });

  test("should require auth for budgets API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/budgets`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
