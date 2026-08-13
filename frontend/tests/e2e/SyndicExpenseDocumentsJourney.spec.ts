import { test, expect } from "@playwright/test";
import { loginAsSyndicWithExpense } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

// Régression : ExpenseDocuments.svelte téléchargeait via fetch() manuel +
// localStorage.getItem('token'), un token qui n'existe jamais (WP-FE1,
// access token en mémoire) — le téléchargement échouait silencieusement.
// Fixé en passant par api.download().
test.describe("Syndic — documents liés à une dépense, upload puis téléchargement de bout en bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("upload un document lié à la dépense puis le télécharge avec succès", async ({
    page,
  }) => {
    const { expenseId } = await loginAsSyndicWithExpense(
      page,
      "journey-expense-doc",
    );
    await page.goto(`/expense-detail?id=${expenseId}`, {
      waitUntil: "networkidle",
    });

    const title = `Facture ${Date.now()}`;

    await page
      .getByRole("button", { name: "Ajouter un document", exact: true })
      .click();

    await page.getByTestId("title-input").fill(title);
    await page.getByTestId("file-input").setInputFiles({
      name: "facture.pdf",
      mimeType: "application/pdf",
      buffer: Buffer.from("%PDF-1.4 test expense document"),
    });

    const [uploadResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/documents") && r.request().method() === "POST",
      ),
      page.getByTestId("upload-button").click(),
    ]);
    expect(uploadResp.status()).toBe(201);

    const row = page
      .getByTestId("documents-list")
      .locator('[data-testid="document-row"]', {
        hasText: title,
      })
      .first();
    await expect(row).toBeVisible();

    // Le backend n'exerce aucun contrôle d'auth sur /documents/{id}/download
    // (constat séparé, documenté dans findings.md) : un statut 200 ne prouve
    // donc rien ici. Ce qui distingue le code cassé (fetch + token mort en
    // localStorage) du code corrigé (api.download()), c'est l'en-tête
    // Authorization réellement envoyé.
    const [downloadReq] = await Promise.all([
      page.waitForRequest(
        (r) => r.url().includes("/download") && r.method() === "GET",
      ),
      row.getByTestId("download-button").click(),
    ]);
    expect(await downloadReq.headerValue("authorization")).toMatch(
      /^Bearer ey/,
    );
  });
});
