import { test, expect } from "@playwright/test";
import { confirmerSiDemande } from "./helpers/amorcage";
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

    // Par les ancres, pas par les libellés.
    //
    // Ces deux sélecteurs pariaient sur la langue : « Téléverser un
    // document » et « Nouveau document » ne trouvent rien dès que l'écran
    // rend en néerlandais. Ils ne tenaient que parce que les quatre projets
    // Playwright sont épinglés à `fr-BE`, ce que la configuration admet
    // elle-même ligne 172.
    //
    // Les deux ancres existent depuis l'ancrage du 2026-09-08.
    await page.getByTestId("documents-upload-button").click();

    const modal = page.getByTestId("document-upload-form");
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

    // Par les ancres, posées le 2026-09-08, plutôt que par les libellés :
    // « Télécharger » et « Supprimer » ne trouvent rien dès que l'écran rend
    // en néerlandais.
    const [downloadResp] = await Promise.all([
      page.waitForResponse((r) => r.url().includes("/download")),
      row.getByTestId("documents-download-button").click(),
    ]);
    expect(downloadResp.status()).toBe(200);

    // Plus de dialogue natif depuis #844 : la suppression demande une modale.
    // Le gestionnaire installé ici ne se déclenchait plus.
    // Attente armée AVANT le clic, confirmation entre les deux : la
    // suppression ouvre une modale depuis #844, et le `Promise.all` ne
    // laissait aucune place à ce geste.
    const attenteSuppression = page.waitForResponse(
      (r) =>
        r.url().includes("/documents/") && r.request().method() === "DELETE",
    );
    await row.getByTestId("documents-delete-button").click();
    await confirmerSiDemande(page);
    const deleteResp = await attenteSuppression;
    expect(deleteResp.status()).toBe(204);

    await expect(page.locator("tr", { hasText: title })).toHaveCount(0);
  });
});
