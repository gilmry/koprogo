import { test, expect } from "@playwright/test";
import { loginAsSyndicWithOwner } from "../helpers/auth";

/**
 * Un lot de l'immeuble, réutilisé plutôt que créé.
 *
 * `loginAsSyndicWithOwner` sème douze lots conformes ; en ajouter un
 * romprait la somme des quotités et rendrait l'immeuble non conforme à son
 * acte de base — le produit refuserait alors, à juste titre.
 */
async function premierLot(
  page: import("@playwright/test").Page,
  token: string,
  buildingId: string,
): Promise<string> {
  const resp = await page.request.get(
    `${API_BASE}/buildings/${buildingId}/units`,
    { headers: { Authorization: `Bearer ${token}` } },
  );
  const corps = await resp.json();
  const lots = Array.isArray(corps) ? corps : (corps?.data ?? []);
  if (lots.length === 0) {
    throw new Error(
      `Aucun lot dans l'immeuble ${buildingId} : la quote-part ne peut pas ` +
        `désigner son ACP créancière.`,
    );
  }
  return lots[0].id;
}

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Owner Contributions - Payment Tracking", () => {
  test("should display owner contributions page", async ({ page }) => {
    await loginAsSyndicWithOwner(page, "contrib");
    await page.goto("/owner-contributions");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='contributions-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("should create a contribution via API", async ({ page }) => {
    const { token, ownerId, buildingId } = await loginAsSyndicWithOwner(
      page,
      "contrib",
    );
    const timestamp = Date.now();
    const unitId = await premierLot(page, token, buildingId);

    const contribResp = await page.request.post(
      `${API_BASE}/owner-contributions`,
      {
        data: {
          owner_id: ownerId,
          // `unit_id` est OBLIGATOIRE en pratique, malgré son `Option<Uuid>`
          // dans le DTO : `resoudre_lacp_creanciere` refuse `None` avec
          // « Impossible de déterminer l'ACP créancière : la quote-part doit
          // porter un lot ». Le lot porte son ACP depuis l'acte de base
          // (Story H15, ADR-0045), et une quote-part due à personne n'est pas
          // une quote-part.
          unit_id: unitId,
          description: `Provision T2 2026 ${timestamp}`,
          amount: 800.0,
          contribution_type: "regular",
          contribution_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(
      contribResp.status(),
      `contribResp : ${await contribResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(201);
  });

  test("should list contributions for owner", async ({ page }) => {
    const { token, ownerId } = await loginAsSyndicWithOwner(page, "contrib");

    const listResp = await page.request.get(
      `${API_BASE}/owner-contributions?owner_id=${ownerId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
  });

  test("should get outstanding contributions for owner", async ({ page }) => {
    const { token, ownerId } = await loginAsSyndicWithOwner(page, "contrib");

    const outstandingResp = await page.request.get(
      `${API_BASE}/owner-contributions/outstanding?owner_id=${ownerId}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(
      outstandingResp.status(),
      `outstandingResp : ${await outstandingResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(200);
  });

  test("should mark a contribution as paid", async ({ page }) => {
    const { token, ownerId, buildingId } = await loginAsSyndicWithOwner(
      page,
      "contrib",
    );
    const timestamp = Date.now();
    const unitId = await premierLot(page, token, buildingId);

    const contribResp = await page.request.post(
      `${API_BASE}/owner-contributions`,
      {
        data: {
          owner_id: ownerId,
          // `unit_id` est OBLIGATOIRE en pratique, malgré son `Option<Uuid>`
          // dans le DTO : `resoudre_lacp_creanciere` refuse `None` avec
          // « Impossible de déterminer l'ACP créancière : la quote-part doit
          // porter un lot ». Le lot porte son ACP depuis l'acte de base
          // (Story H15, ADR-0045), et une quote-part due à personne n'est pas
          // une quote-part.
          unit_id: unitId,
          description: `Provision T3 2026 ${timestamp}`,
          amount: 600.0,
          contribution_type: "regular",
          contribution_date: new Date().toISOString(),
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(
      contribResp.status(),
      `contribResp : ${await contribResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(201);
    const contrib = await contribResp.json();
    const paidResp = await page.request.put(
      `${API_BASE}/owner-contributions/${contrib.id}/mark-paid`,
      {
        data: {
          payment_date: new Date().toISOString(),
          payment_method: "bank_transfer",
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(
      paidResp.status(),
      `paidResp : ${await paidResp.text().catch(() => "<corps illisible>")}`,
    ).toBe(200);
  });

  test("should require auth for owner contributions API", async ({ page }) => {
    const resp = await page.request.get(`${API_BASE}/owner-contributions`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
