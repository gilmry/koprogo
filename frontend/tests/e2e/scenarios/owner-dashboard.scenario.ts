/**
 * SCENARIO: Alice consulte son tableau de bord coproprietaire
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'Alice (coproprietaire, presidente CdC) :
 *   1. Connexion via le formulaire login
 *   2. Arrivee sur le tableau de bord proprietaire
 *   3. Consultation des widgets (stats, immeubles, lots)
 *   4. Navigation vers la section Paiements
 *   5. Retour au tableau de bord
 *
 * Duree video attendue : ~40-50 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanClick,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Alice consulte son tableau de bord", () => {
  test.setTimeout(120_000);

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Seed the world
    const seedResp = await request.post(`${API_BASE}/seed/scenario/world`, {
      headers: adminHeaders,
    });
    if (!seedResp.ok()) {
      console.log("Seed world already exists, continuing...");
    } else {
      seedData = await seedResp.json();
      seedData = seedData.data;
    }
  });

  test.afterAll(async ({ request }) => {
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: "admin123" },
    });
    const admin = await adminResp.json();
    await request.delete(`${API_BASE}/seed/scenario/world`, {
      headers: { Authorization: `Bearer ${admin.token}` },
    });
  });

  test("Alice se connecte et explore son tableau de bord", async ({ page }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Verifier l'arrivee sur le tableau de bord owner
    // ============================================================
    await expect(page.getByTestId("owner-dashboard")).toBeVisible({
      timeout: 15000,
    });

    // Attendre que le dashboard charge completement
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Le dashboard affiche le message de bienvenue
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Consulter les widgets du dashboard
    // ============================================================
    // Scroller doucement pour montrer les stats et les immeubles
    await page.evaluate(() => {
      window.scrollTo({ top: 300, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Scroller encore pour voir les lots et tickets
    await page.evaluate(() => {
      window.scrollTo({ top: 600, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Scroller pour voir les actions rapides
    await page.evaluate(() => {
      window.scrollTo({ top: 900, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    // Revenir en haut
    await page.evaluate(() => {
      window.scrollTo({ top: 0, behavior: "smooth" });
    });
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 4 : Navigation vers la section Paiements
    // ============================================================
    await humanClick(page, "nav-link-paiements");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Paiements est chargee
    await expect(page.locator("main").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Retour au tableau de bord
    // ============================================================
    await humanClick(page, "nav-link-tableau-de-bord");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("owner-dashboard")).toBeVisible({
      timeout: 15000,
    });

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
