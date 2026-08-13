import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Comptable — Workflow factures, cycle de vie rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("soumet, approuve puis marque payée une dépense (draft → pending_approval → approved → paid)", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "journey-invoice-wf",
    );
    const description = `Entretien chaudière ${Date.now()}`;

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description,
        amount: 800.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResp.status()).toBe(201);

    await page.goto("/invoice-workflow", { waitUntil: "networkidle" });
    page.on("dialog", (dialog) => dialog.accept());

    const card = page
      .getByTestId("invoice-card")
      .filter({ hasText: description });
    await expect(card).toBeVisible();

    const [submitResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/submit") && r.request().method() === "PUT",
      ),
      card.getByTestId("submit-approval-button").click(),
    ]);
    expect(submitResp.status()).toBe(200);
    await expect(card.getByTestId("approve-button")).toBeVisible();

    await card.getByTestId("approve-button").click();
    const approveModal = page.locator(".modal-footer").filter({
      has: page.getByRole("button", { name: "Approuver" }),
    });
    const [approveResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/approve") && r.request().method() === "PUT",
      ),
      approveModal.getByRole("button", { name: "Approuver" }).click(),
    ]);
    expect(approveResp.status()).toBe(200);
    await expect(card.getByTestId("mark-paid-button")).toBeVisible();

    const [paidResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/mark-paid") && r.request().method() === "PUT",
      ),
      card.getByTestId("mark-paid-button").click(),
    ]);
    expect(paidResp.status()).toBe(200);
  });
});
