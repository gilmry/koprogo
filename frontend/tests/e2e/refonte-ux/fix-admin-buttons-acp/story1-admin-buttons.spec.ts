/**
 * Story 1 (#697) — docs/maury/fix-admin-buttons-acp/stories.md
 *
 * `<Button on:click={...}>` (syntaxe Svelte 4) sur `components/ui/Button.svelte`
 * (Svelte 5 runes-mode) ne déclenche jamais l'action : `on:click` est une
 * directive de compilation, jamais reçue par `...restProps`. RED avant fix :
 * ces tests échouent tant que les 13 occurrences ne sont pas passées en
 * `onclick={...}`.
 */
import { test, expect, type Page } from "@playwright/test";
import { loginAsAdmin, loginAsSyndicWithExpense } from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

/**
 * `mark-paid` exige `approval_status: Approved` côté backend
 * (`check_can_emit_expenses` + `expense.rs::approve` — invoice must be
 * approved first). Une dépense fraîchement créée est `Draft`. Fait
 * avancer Draft → PendingApproval → Approved via les endpoints
 * `/invoices/{id}/submit` puis `/invoices/{id}/approve` (syndic autorisé
 * pour les deux) avant de tester le bouton "Marquer payé".
 */
async function approveExpense(
  page: Page,
  token: string,
  expenseId: string,
  approvedByUserId: string,
): Promise<void> {
  await page.request.put(`${API_BASE}/invoices/${expenseId}/submit`, {
    data: {},
    headers: { Authorization: `Bearer ${token}` },
  });
  await page.request.put(`${API_BASE}/invoices/${expenseId}/approve`, {
    data: { approved_by_user_id: approvedByUserId },
    headers: { Authorization: `Bearer ${token}` },
  });
}

test.describe("Story 1 (#697) — boutons admin morts (Svelte 5)", () => {
  test('@happy clic "Nouvelle organisation" ouvre la modale', async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.goto("/admin/organizations");
    const createBtn = page.getByTestId("create-organization-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await createBtn.click();

    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test('@happy clic "Nouvel immeuble" ouvre la modale', async ({ page }) => {
    await loginAsAdmin(page);
    await page.goto("/buildings");
    const createBtn = page.getByTestId("create-building-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await createBtn.click();

    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test('@happy clic "Nouvel utilisateur" ouvre la modale', async ({ page }) => {
    await loginAsAdmin(page);
    await page.goto("/admin/users");
    const createBtn = page.getByTestId("create-user-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await expect(page.getByRole("dialog")).toHaveCount(0);

    await createBtn.click();

    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test('@edge double-clic rapide sur "Nouvelle organisation" — une seule modale', async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.goto("/admin/organizations");

    const btn = page.getByTestId("create-organization-button");
    await expect(btn).toBeVisible({ timeout: 15_000 });
    await btn.click();
    await btn.click({ force: true }).catch(() => {
      /* la modale peut déjà intercepter le 2e clic — attendu */
    });

    await expect(page.getByRole("dialog")).toHaveCount(1);
  });

  test('@negative bouton "retour" (branche erreur ExpenseDetail) ramène en arrière', async ({
    page,
  }) => {
    // Navigue d'abord vers une page connue (pour que "retour" ait une cible),
    // puis vers un id de dépense inexistant pour déclencher la branche
    // `error` (Button ligne 206).
    await loginAsAdmin(page);
    await page.goto("/expenses");
    await expect(page.locator("body")).toBeVisible({ timeout: 15_000 });
    await page.goto("/expense-detail?id=00000000-0000-0000-0000-000000000000");
    await expect(page.getByTestId("back-button")).toBeVisible({
      timeout: 15_000,
    });

    await page.getByTestId("back-button").click();

    await expect(page).toHaveURL(/\/expenses/);
  });

  test("@happy panneau dépense — marquer payé change le statut", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithExpense(page, "btnfix1");
    await approveExpense(page, ctx.token, ctx.expenseId, ctx.userId);
    await page.goto(`/expense-detail?id=${ctx.expenseId}`);
    await expect(page.getByTestId("status-badge")).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByTestId("status-badge")).toContainText(
      /pending|attente/i,
    );

    await page.getByTestId("mark-paid-button").click();

    await expect(page.getByTestId("status-badge")).toContainText(/paid|payé/i);
  });

  test("@happy panneau dépense — marquer en retard puis annuler", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithExpense(page, "btnfix2");
    await page.goto(`/expense-detail?id=${ctx.expenseId}`);
    await expect(page.getByTestId("status-badge")).toBeVisible({
      timeout: 15_000,
    });

    await page.getByTestId("mark-overdue-button").click();
    await expect(page.getByTestId("status-badge")).toContainText(
      /overdue|retard/i,
    );

    page.once("dialog", (d) => d.accept());
    await page.getByTestId("cancel-button").click();

    await expect(page.getByTestId("status-badge")).toContainText(
      /cancelled|annul/i,
    );
  });

  test("@happy panneau dépense — payé puis dé-payer puis réactiver", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithExpense(page, "btnfix3");
    await approveExpense(page, ctx.token, ctx.expenseId, ctx.userId);
    await page.goto(`/expense-detail?id=${ctx.expenseId}`);
    await expect(page.getByTestId("status-badge")).toBeVisible({
      timeout: 15_000,
    });

    await page.getByTestId("mark-paid-button").click();
    await expect(page.getByTestId("status-badge")).toContainText(/paid|payé/i);

    page.once("dialog", (d) => d.accept());
    await page.getByTestId("unpay-button").click();
    await expect(page.getByTestId("status-badge")).toContainText(
      /pending|attente/i,
    );

    // Repasse par "annuler" pour pouvoir tester "réactiver".
    page.once("dialog", (d) => d.accept());
    await page.getByTestId("cancel-button").click();
    await expect(page.getByTestId("status-badge")).toContainText(
      /cancelled|annul/i,
    );

    await page.getByTestId("reactivate-button").click();
    await expect(page.getByTestId("status-badge")).toContainText(
      /pending|attente/i,
    );
  });

  test("@security le fix de câblage n'ouvre aucune nouvelle route ni contournement d'autorisation", async ({
    page,
  }) => {
    // Un syndic (non-superadmin) reste bloqué par le backend même une fois
    // le bouton "vivant" — le clic ne fait qu'appeler un endpoint déjà gaté.
    const ctx = await loginAsSyndicWithExpense(page, "btnfix4");
    const resp = await page.request.get(
      `${process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1"}/organizations`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(resp.status()).toBe(403);
  });
});
