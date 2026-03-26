/**
 * SCENARIO: Tableau d'affichage communautaire
 *
 * Documentation Vivante — vidéo exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Annonces via le menu latéral
 *   3. Sélection d'un immeuble
 *   4. Consultation de la liste des annonces existantes
 *   5. Clic sur une annonce pour voir le détail
 *   6. Retour à la liste et création d'une nouvelle annonce
 *
 * Durée vidéo attendue : ~50-70 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  humanClick,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scénario: Tableau d'affichage communautaire", () => {
  test.setTimeout(120_000);

  // ----- Données de test (créées via API, invisibles en vidéo) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let buildingName: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-notice-${ts}@koprogo.test`;
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
        name: `Notice Org ${ts}`,
        slug: `notice-${ts}`,
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
        first_name: "Claire",
        last_name: "Janssens",
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
    buildingName = `Résidence des Érables ${ts}`;
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "8 Avenue des Arts",
        city: "Liège",
        postal_code: "4000",
        country: "Belgium",
        total_units: 16,
        construction_year: 2005,
        organization_id: org.id,
      },
      headers: adminHeaders,
    });
    const building = await buildingResp.json();

    // 6. Create a few notices via API
    const notice1Resp = await request.post(`${API_BASE}/notices`, {
      data: {
        building_id: building.id,
        notice_type: "Announcement",
        category: "General",
        title: "Nettoyage des communs ce samedi",
        content:
          "Un nettoyage complet des parties communes est prévu ce samedi de 9h à 12h. " +
          "Merci de ne pas laisser d'objets dans les couloirs.",
      },
      headers: syndicHeaders,
    });
    const notice1 = await notice1Resp.json();
    // Publish notice so it appears in the default "Published" filter
    await request.post(`${API_BASE}/notices/${notice1.id}/publish`, {
      headers: syndicHeaders,
    });

    const notice2Resp = await request.post(`${API_BASE}/notices`, {
      data: {
        building_id: building.id,
        notice_type: "Event",
        category: "Social",
        title: "Barbecue de quartier - Fête des voisins",
        content:
          "À l'occasion de la fête des voisins, un barbecue est organisé dans le " +
          "jardin commun. Chacun apporte un plat à partager. Ambiance conviviale garantie !",
        event_date: new Date(
          Date.now() + 14 * 24 * 60 * 60 * 1000,
        ).toISOString(),
        event_location: "Jardin commun, rez-de-chaussée",
        contact_info: "Claire Janssens - syndic@residence-erables.be",
      },
      headers: syndicHeaders,
    });
    const notice2 = await notice2Resp.json();
    await request.post(`${API_BASE}/notices/${notice2.id}/publish`, {
      headers: syndicHeaders,
    });

    const notice3Resp = await request.post(`${API_BASE}/notices`, {
      data: {
        building_id: building.id,
        notice_type: "LostAndFound",
        category: "General",
        title: "Trouvé : clé avec porte-clé bleu",
        content:
          "Une clé avec un porte-clé bleu a été trouvée dans le hall d'entrée ce matin. " +
          "Contactez la loge du concierge pour la récupérer.",
        contact_info: "Loge du concierge, RDC",
      },
      headers: syndicHeaders,
    });
    const notice3 = await notice3Resp.json();
    await request.post(`${API_BASE}/notices/${notice3.id}/publish`, {
      headers: syndicHeaders,
    });
  });

  test("Un syndic consulte les annonces et en crée une nouvelle", async ({
    page,
  }) => {
    // ============================================================
    // ÉTAPE 1 : Connexion (visible dans la vidéo)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 2 : Navigation vers les Annonces via le menu latéral
    // ============================================================
    await humanClick(page, "nav-link-annonces");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page Annonces est chargée
    await expect(page.locator("main h1").first()).toContainText("Annonces");
    await stepPause(page);

    // ============================================================
    // ÉTAPE 3 : Sélectionner l'immeuble (ou attendre auto-sélection)
    // ============================================================
    await waitForSpinner(page);

    // Wait for building selection to be ready (either selector or auto-selected single building)
    const buildingReady = page.locator('[data-testid="building-selector"], [data-testid="building-selected"]').first();
    await expect(buildingReady).toBeVisible({ timeout: 15000 });

    const buildingSelect = page.getByTestId("building-selector");
    if (await buildingSelect.isVisible({ timeout: 2000 }).catch(() => false)) {
      await buildingSelect.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Résidence des Érables")) {
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
    // ÉTAPE 4 : Vérifier que la liste des annonces s'affiche
    // ============================================================
    await expect(page.getByTestId("notice-list")).toBeVisible({
      timeout: 15000,
    });

    // Vérifier qu'au moins une annonce apparaît
    await expect(
      page.getByTestId("notice-list-row").first(),
    ).toBeVisible({ timeout: 15000 });

    // Vérifier que les annonces créées en beforeAll sont visibles
    await expect(
      page.locator("text=Nettoyage des communs"),
    ).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ÉTAPE 5 : Cliquer sur une annonce pour voir le détail
    // ============================================================
    const noticeRow = page
      .getByTestId("notice-list-row")
      .filter({ hasText: "Barbecue de quartier" })
      .first();
    await humanClickLocator(page, noticeRow.locator("a"));
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que la page de détail est chargée
    await expect(page.getByTestId("notice-detail")).toBeVisible({
      timeout: 15000,
    });

    // Vérifier le titre de l'annonce
    await expect(page.locator("h1")).toContainText("Barbecue de quartier");
    await stepPause(page);

    // ============================================================
    // ÉTAPE 6 : Retour à la liste des annonces
    // ============================================================
    await humanClick(page, "nav-link-annonces");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Re-sélectionner l'immeuble après navigation (ou attendre auto-sélection)
    await waitForSpinner(page);
    const buildingReady2 = page.locator('[data-testid="building-selector"], [data-testid="building-selected"]').first();
    await expect(buildingReady2).toBeVisible({ timeout: 15000 });

    const buildingSelect2 = page.getByTestId("building-selector");
    if (await buildingSelect2.isVisible({ timeout: 2000 }).catch(() => false)) {
      await buildingSelect2.scrollIntoViewIfNeeded();
      await page.waitForTimeout(PACE.BEFORE_SELECT);
      const options = await buildingSelect2.locator("option").all();
      for (const option of options) {
        const text = await option.textContent();
        if (text && text.includes("Résidence des Érables")) {
          const value = await option.getAttribute("value");
          if (value) {
            await buildingSelect2.selectOption(value);
            break;
          }
        }
      }
      await page.waitForTimeout(PACE.AFTER_SELECT);
    }
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // ÉTAPE 7 : Créer une nouvelle annonce
    // ============================================================
    await humanClick(page, "notices-create-btn");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Vérifier que le modal de création est ouvert
    await expect(page.getByTestId("notice-create-modal")).toBeVisible({
      timeout: 10000,
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Remplir le formulaire
    await humanFill(
      page,
      "notice-title-input",
      "Travaux ascenseur - Interruption prévue",
    );

    await humanFill(
      page,
      "notice-content-input",
      "L'ascenseur sera hors service du lundi au mercredi de la semaine " +
        "prochaine pour une maintenance préventive obligatoire. " +
        "Nous vous prions d'utiliser les escaliers durant cette période. " +
        "Merci de votre compréhension.",
    );

    await stepPause(page);

    // ============================================================
    // ÉTAPE 8 : Soumettre l'annonce
    // ============================================================
    await humanClick(page, "notice-submit-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // La page recharge après création réussie
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la vidéo montre le résultat
    // ============================================================
    await finalPause(page);
  });
});
