import { test, expect } from "@playwright/test";
import {
  loginAsSyndicWithLinkedOwner,
  ensureAcp,
  uiLoginWithRetry,
} from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";
import { attendCode } from "./helpers/reponse";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Échanges locaux (SEL) — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une offre d'échange de bout en bout, en tant que propriétaire lié", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithLinkedOwner(page, "journey-exch");

    // GET /buildings est scopé via unit_owners pour le rôle owner (Story
    // 1.3 / BUG-WF14-2) — un Owner sans lot lié ne voit aucun immeuble,
    // donc le BuildingSelector du formulaire reste vide et le submit ne
    // peut jamais aboutir. Un vrai propriétaire SEL a toujours un lot.
    const acpId = await ensureAcp(
      page,
      ctx.orgId,
      ctx.adminToken,
      "journey-exch",
    );
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: ctx.buildingId,
        unit_number: `SEL-${Date.now()}`,
        floor: 1,
        surface_area: 85.0,
        unit_type: "Apartment",
        quota: 1000.0,
      },
      headers: { Authorization: `Bearer ${ctx.adminToken}` },
    });
    const unit = await unitResp.json();
    const linkResp = await page.request.post(
      `${API_BASE}/units/${unit.id}/owners`,
      {
        data: {
          owner_id: ctx.ownerId,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${ctx.token}` },
      },
    );
    expect(linkResp.status()).toBe(201);

    // BASCULER vers le copropriétaire avant d'agir en son nom.
    //
    // `loginAsSyndicWithLinkedOwner` laisse volontairement la session du
    // SYNDIC en place : il crée le compte copropriétaire sans changer
    // d'identité. Sans cette bascule, le POST partait donc en tant que
    // syndic, et le serveur répondait 400 — à raison :
    //
    //   « Cette action est réservée aux copropriétaires : elle engage une
    //     personne, pas la copropriété. »
    //
    // Le test se disait « en tant que propriétaire lié » sans jamais le
    // devenir. Ce n'était pas le refus qui était faux, c'était l'acteur.
    // Cf. #832.
    await uiLoginWithRetry(page, ctx.ownerEmail, ctx.ownerPassword, /\/owner/);

    await page.goto("/exchanges/new", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);

    const title = `Aide jardinage ${Date.now()}`;
    await page.getByTestId("exchange-title-input").fill(title);
    await page
      .getByTestId("exchange-description-input")
      .fill("Tonte pelouse et taille de haie pour un voisin.");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/exchanges") && r.request().method() === "POST",
      ),
      page.getByTestId("exchange-submit-btn").click(),
    ]);
    await attendCode(resp, 201, "création d'une offre d'échange");
  });
});
