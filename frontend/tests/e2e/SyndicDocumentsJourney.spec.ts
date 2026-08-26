import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Syndic — parcours de gestion documentaire rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("documents: upload, liste, téléchargement puis suppression de bout en bout", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "journey-doc");
    await page.goto("/documents", { waitUntil: "networkidle" });

    const title = `PV AGO ${Date.now()}`;

    await page.getByRole("button", { name: "Téléverser un document" }).click();

    const modal = page.locator("form").filter({ hasText: "Nouveau document" });
    await expect(modal).toBeVisible();

    await modal.locator("#doc-upload-title").fill(title);
    await modal.locator("input[type=file]").setInputFiles({
      name: "pv-ago.pdf",
      mimeType: "application/pdf",
      buffer: Buffer.from("%PDF-1.4 test document content"),
    });

    const [uploadResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/documents") && r.request().method() === "POST",
      ),
      modal.getByRole("button", { name: "Téléverser un document" }).click(),
    ]);
    // L'assertion porte le CORPS de la reponse : un « Expected 201, Received
    // 500 » nu ne dit rien de la cause, et c'est exactement ce qui a fait
    // perdre du temps sur ce defaut. Le handler renvoie `{"error": ...}`, on
    // le remonte donc dans le message d'echec.
    expect(
      uploadResp.status(),
      `upload document: ${await uploadResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(201);

    await expect(modal).toBeHidden();

    const row = page.locator("tr", { hasText: title });
    await expect(row).toBeVisible();

    const [downloadResp] = await Promise.all([
      page.waitForResponse((r) => r.url().includes("/download")),
      row.getByRole("button", { name: "Télécharger" }).click(),
    ]);
    expect(downloadResp.status()).toBe(200);

    page.once("dialog", (dialog) => dialog.accept());
    const [deleteResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/documents/") && r.request().method() === "DELETE",
      ),
      row.getByRole("button", { name: "Supprimer" }).click(),
    ]);
    expect(deleteResp.status()).toBe(204);

    await expect(page.locator("tr", { hasText: title })).toHaveCount(0);
  });
});
