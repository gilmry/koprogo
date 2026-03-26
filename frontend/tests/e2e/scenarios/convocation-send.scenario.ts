/**
 * SCENARIO: Envoi d'une convocation d'assemblée générale
 *
 * Documentation Vivante — vidéo exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Convocations via le menu latéral
 *   3. Sélection d'un immeuble
 *   4. Consultation de la liste des convocations
 *   5. Clic sur une convocation pour voir le détail
 *   6. Vérification des informations de délai légal belge
 *
 * Durée vidéo attendue : ~45-60 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanClick,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scénario: Envoi d'une convocation d'AG", () => {
  test.setTimeout(120_000);

  // ----- Données de test (créées via API, invisibles en vidéo) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-convoc-${ts}@koprogo.test`;
    syndicPassword = "test123456";

    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Create org
    const orgResp = await request.post(`${API_BASE}/organizations`, {
      data: {
        name: `Convoc Org ${ts}`,
        slug: `convoc-${ts}`,
        contact_email: syndicEmail,
        subscription_plan: "professional",
      },
      headers: adminHeaders,
    });
    const org = await orgResp.json();

    // 3. Register syndic
    await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: syndicEmail,
        password: syndicPassword,
        first_name: "Sophie",
        last_name: "Lambert",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Login syndic for authed requests
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Create building
    buildingName = `Résidence du Parc ${ts}`;
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "15 Rue de la Loi",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 20,
        construction_year: 1998,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 6. Create a meeting 30 days out
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    const meetingResp = await request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: building.id,
        date: meetingDate.toISOString(),
        agenda: "Assemblée Générale Ordinaire annuelle",
        meeting_type: "Ordinary",
        organization_id: org.id,
      },
      headers: syndicHeaders,
    });
    const meeting = await meetingResp.json();

    // 7. Create a convocation for that meeting
    await request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meeting.id,
        building_id: building.id,
        meeting_type: "Ordinary",
        meeting_date: meetingDate.toISOString(),
        language: "fr",
      },
      headers: syndicHeaders,
    });
  });

  test("Un syndic consulte une convocation et vérifie le délai légal", async ({
    page,
  }) => {
    // ============================================================
    // ÉTAPE 1 : Connexion (visible dans la vidéo)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 2 : Navigation vers les Convocations via le menu latéral
    // ============================================================
    await humanClick(page, "nav-link-convocations");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page Convocations est chargée
    await expect(page.locator("main h1").first()).toContainText(
      "Convocations",
    );
    await stepPause(page);

    // ============================================================
    // ÉTAPE 3 : Sélectionner l'immeuble
    // ============================================================
    await waitForSpinner(page);

    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 5000 })) {
      await buildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Résidence du Parc")) {
          const value = await option.getAttribute("value");
          if (value) {
            await buildingSelect.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await waitForSpinner(page);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 4 : Vérifier que la liste de convocations est visible
    // ============================================================
    await expect(page.getByTestId("convocation-list")).toBeVisible({
      timeout: 15000,
    });

    // Vérifier qu'au moins une convocation apparaît
    await expect(
      page.getByTestId("convocation-rows").locator("li").first(),
    ).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ÉTAPE 5 : Cliquer sur la convocation pour voir le détail
    // ============================================================
    const convocationRow = page
      .getByTestId("convocation-rows")
      .locator("li")
      .first();
    await humanClickLocator(page, convocationRow.locator("a"));
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page de détail est chargée
    await expect(page.getByTestId("convocation-detail")).toBeVisible({
      timeout: 15000,
    });
    await stepPause(page);

    // ============================================================
    // ÉTAPE 6 : Vérifier les informations de délai légal
    // ============================================================
    const legalDeadline = page.getByTestId(
      "convocation-detail-legal-deadline",
    );
    await legalDeadline.scrollIntoViewIfNeeded();
    await expect(legalDeadline).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    // Vérifier que le statut s'affiche
    await expect(page.getByTestId("convocation-detail-status")).toBeVisible({
      timeout: 5000,
    });

    // Vérifier le résumé des destinataires
    await expect(
      page.getByTestId("convocation-detail-recipients-summary"),
    ).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la vidéo montre le résultat
    // ============================================================
    await finalPause(page);
  });
});
