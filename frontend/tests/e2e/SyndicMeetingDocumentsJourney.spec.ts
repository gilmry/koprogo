import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

// Régression : MeetingDocuments.svelte téléchargeait via fetch() manuel +
// localStorage.getItem('token'), un token qui n'existe jamais (WP-FE1,
// access token en mémoire) — le téléchargement échouait silencieusement.
// Fixé en passant par api.download().
test.describe("Syndic — documents liés à une AG, upload puis téléchargement de bout en bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("upload un document lié à l'AG puis le télécharge avec succès", async ({
    page,
  }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(
      page,
      "journey-meeting-doc",
    );
    await page.goto(`/meeting-detail?id=${meetingId}`, {
      waitUntil: "networkidle",
    });

    const title = `PV réunion ${Date.now()}`;

    await page.getByRole("button", { name: "+ Ajouter un document" }).click();

    await page.getByTestId("upload-title").fill(title);
    await page.getByTestId("upload-file").setInputFiles({
      name: "pv.pdf",
      mimeType: "application/pdf",
      buffer: Buffer.from("%PDF-1.4 test meeting document"),
    });

    const [uploadResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/documents") && r.request().method() === "POST",
      ),
      page
        .getByRole("button", { name: "Ajouter un document", exact: true })
        .click(),
    ]);
    expect(uploadResp.status()).toBe(201);

    const row = page
      .getByTestId("document-list")
      .locator("div", {
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
      page.getByTestId("document-download-btn").click(),
    ]);
    expect(await downloadReq.headerValue("authorization")).toMatch(
      /^Bearer ey/,
    );
  });
});
