import { test, expect, type Page } from "@playwright/test";
import { failOnPageErrors } from "./helpers/pageErrors";
import { adminLogin } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// /reports est gated ACCOUNTANT-only (guards.ts, même schéma que
// /journal-entries) — un syndic y est silencieusement redirigé.
async function loginAsAccountant(page: Page, prefix: string) {
  const timestamp = Date.now();
  const email = `${prefix}-${timestamp}@example.com`;

  const adminToken = await adminLogin(page);
  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Accountant",
      last_name: `Test${timestamp}`,
      role: "accountant",
      organization_id: org.id,
    },
  });

  await page.addInitScript(
    (value) => {
      try {
        localStorage.setItem("koprogo_user", value);
      } catch {
        /* ignore */
      }
    },
    JSON.stringify({
      id: "injected-user",
      email,
      first_name: "Accountant",
      last_name: `Test${timestamp}`,
      role: "accountant",
      roles: [
        {
          id: "injected-role-1",
          role: "accountant",
          organization_id: null,
          is_primary: true,
        },
      ],
      active_role: {
        id: "injected-role-1",
        role: "accountant",
        organization_id: null,
        is_primary: true,
      },
    }),
  );
  await page.goto("/accountant", { waitUntil: "networkidle" });
}

test.describe("Comptable — Rapports PCMN, parcours rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("génère le bilan comptable de bout en bout", async ({ page }) => {
    await loginAsAccountant(page, "journey-reports");
    await page.goto("/reports", { waitUntil: "networkidle" });

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/reports/balance-sheet") &&
          r.request().method() === "GET",
      ),
      page.getByRole("button", { name: "Générer le rapport" }).click(),
    ]);
    expect(resp.status()).toBe(200);

    await expect(page.getByText("Bilan").first()).toBeVisible();
  });
});
