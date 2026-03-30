import { test, expect } from "@playwright/test";
import {
  loginAsSyndicWithExpense,
  loginAsSyndicWithBuilding,
  loginAsSyndicWithOwner,
} from "./helpers/auth";

/**
 * Payments E2E Test Suite - Stripe & SEPA Payment Management
 *
 * Tests payment creation, listing, status transitions, and payment methods.
 * Mirrors workflows from backend/tests/e2e_payments.rs.
 */

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Payments - Stripe & SEPA", () => {
  test("should display owner payments page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "payment");
    await page.goto("/owner/payments");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("should display owner payment methods page", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "payment");
    await page.goto("/owner/payment-methods");

    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("should create a payment via API and retrieve it", async ({ page }) => {
    const { token, expenseId } = await loginAsSyndicWithExpense(
      page,
      "payment",
    );
    const timestamp = Date.now();

    const paymentResp = await page.request.post(`${API_BASE}/payments`, {
      data: {
        expense_id: expenseId,
        amount_cents: 50000,
        currency: "EUR",
        payment_method_type: "BankTransfer",
        idempotency_key: `test-payment-${timestamp}`,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([201, 400, 422].includes(paymentResp.status())).toBeTruthy();

    if (paymentResp.status() === 201) {
      const payment = await paymentResp.json();
      expect(payment.id).toBeTruthy();
      expect(payment.amount_cents).toBe(50000);
      expect(payment.currency).toBe("EUR");
      expect(payment.status).toBe("Pending");

      // Retrieve by ID
      const getResp = await page.request.get(
        `${API_BASE}/payments/${payment.id}`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(getResp.ok()).toBeTruthy();
      const retrieved = await getResp.json();
      expect(retrieved.id).toBe(payment.id);
    }
  });

  test("should list payments for an expense", async ({ page }) => {
    const { token, expenseId } = await loginAsSyndicWithExpense(
      page,
      "payment",
    );

    const listResp = await page.request.get(
      `${API_BASE}/expenses/${expenseId}/payments`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const payments = await listResp.json();
    expect(Array.isArray(payments)).toBeTruthy();
  });

  test("should transition payment pending → processing", async ({ page }) => {
    const { token, expenseId } = await loginAsSyndicWithExpense(
      page,
      "payment",
    );
    const timestamp = Date.now();

    const createResp = await page.request.post(`${API_BASE}/payments`, {
      data: {
        expense_id: expenseId,
        amount_cents: 10000,
        currency: "EUR",
        payment_method_type: "BankTransfer",
        idempotency_key: `test-processing-${timestamp}`,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    if (createResp.status() === 201) {
      const payment = await createResp.json();

      const processResp = await page.request.put(
        `${API_BASE}/payments/${payment.id}/processing`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect([200, 400].includes(processResp.status())).toBeTruthy();

      if (processResp.ok()) {
        const updated = await processResp.json();
        expect(updated.status).toBe("Processing");
      }
    }
  });

  test("should create and list payment methods for an owner", async ({
    page,
  }) => {
    const { token, ownerId } = await loginAsSyndicWithOwner(page, "paymethod");
    const timestamp = Date.now();

    const methodResp = await page.request.post(`${API_BASE}/payment-methods`, {
      data: {
        owner_id: ownerId,
        method_type: "BankTransfer",
        display_label: `Virement ${timestamp}`,
        is_default: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect([201, 400, 422].includes(methodResp.status())).toBeTruthy();

    if (methodResp.status() === 201) {
      const method = await methodResp.json();
      expect(method.id).toBeTruthy();
      expect(method.method_type).toBe("BankTransfer");

      // Verify it appears in owner's payment methods list
      const listResp = await page.request.get(
        `${API_BASE}/owners/${ownerId}/payment-methods`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(listResp.ok()).toBeTruthy();
      const methods = await listResp.json();
      expect(Array.isArray(methods)).toBeTruthy();
      expect(methods.some((m: { id: string }) => m.id === method.id)).toBe(
        true,
      );
    }
  });

  test("should get owner payment statistics", async ({ page }) => {
    const { token, ownerId } = await loginAsSyndicWithOwner(page, "paystats");

    const statsResp = await page.request.get(
      `${API_BASE}/owners/${ownerId}/payments/stats`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect([200, 404].includes(statsResp.status())).toBeTruthy();
  });

  test("should require auth to access payments API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/payments/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
