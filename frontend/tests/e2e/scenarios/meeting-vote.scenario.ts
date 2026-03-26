/**
 * SCENARIO: Vote sur une resolution en assemblee generale
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un syndic :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Assemblees via le menu lateral
 *   3. Clic sur une assemblee pour afficher le detail
 *   4. Visualisation de la section Resolutions
 *   5. Creation d'une resolution via le formulaire UI
 *   6. Vote "Pour" avec pouvoir de vote (tantiemes)
 *   7. Verification que le vote est enregistre
 *   8. Cloture du scrutin
 *   9. Verification du statut final (Adoptee/Rejetee)
 *
 * Duree video attendue : ~60-90 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import {
  humanLogin,
  humanFill,
  humanClick,
  humanSelect,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Scenario: Vote sur une resolution en assemblee generale", () => {
  test.setTimeout(120_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let meetingTitle: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-vote-${ts}@koprogo.test`;
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
        name: `Scenario Vote Org ${ts}`,
        slug: `scenario-vote-${ts}`,
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
        first_name: "Jean",
        last_name: "Syndic",
        role: "syndic",
        organization_id: org.id,
      },
    });

    // 4. Login as syndic to get syndic token
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Create building
    const buildingResp = await request.post(`${API_BASE}/buildings`, {
      data: {
        name: `Residence du Parc ${ts}`,
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

    // 6. Create a meeting (Ordinary, scheduled 30 days from now)
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    meetingTitle = `AG Ordinaire - Budget ${ts}`;

    await request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: building.id,
        organization_id: org.id,
        meeting_type: "Ordinary",
        title: meetingTitle,
        scheduled_date: meetingDate.toISOString(),
        location: "Salle communale, 15 Rue de la Loi, 1000 Bruxelles",
      },
      headers: syndicHeaders,
    });
  });

  test("Un syndic cree une resolution, vote et cloture le scrutin", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Assemblees via le menu lateral
    // ============================================================
    await humanClick(page, "nav-link-assemblées");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Assemblees est chargee
    await expect(page.locator("main h1").first()).toContainText("Assemblées");
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Cliquer sur l'assemblee pour ouvrir le detail
    // ============================================================
    // Attendre que la liste des reunions charge
    await expect(
      page.getByTestId("meeting-list-container"),
    ).toBeVisible({ timeout: 15000 });
    await waitForSpinner(page);

    // Trouver la carte de la reunion et cliquer sur "Details"
    const meetingCard = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG Ordinaire" })
      .first();
    await expect(meetingCard).toBeVisible({ timeout: 15000 });

    const detailsLink = meetingCard.locator("a").filter({ hasText: /[Dd]étails|Details/ }).first();
    await humanClickLocator(page, detailsLink);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le detail de la reunion est affiche
    await expect(page.locator("h1").first()).toContainText("AG Ordinaire", {
      timeout: 15000,
    });
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Verifier la section Resolutions
    // ============================================================
    // Faire defiler jusqu'a la section Resolutions
    const resolutionSection = page.getByTestId("resolution-list");
    await resolutionSection.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // La section devrait etre visible (meme sans resolutions)
    await expect(resolutionSection).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Creer une resolution via le formulaire UI
    // ============================================================
    // Cliquer sur le bouton "+ Ajouter"
    await humanClick(page, "resolution-create-btn");
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Remplir le formulaire de creation
    await humanFill(
      page,
      "resolution-title-input",
      "Approbation des travaux de facade",
    );

    await humanFill(
      page,
      "resolution-description-textarea",
      "Resolution pour approuver les travaux de ravalement de facade " +
        "du batiment, incluant la reparation des fissures et " +
        "la peinture exterieure. Devis retenu: 45.000 EUR HTVA.",
    );

    await humanSelect(page, "resolution-type-select", "works");
    await humanSelect(page, "resolution-majority-select", "Simple");

    await stepPause(page);

    // Soumettre la resolution
    await humanClick(page, "resolution-submit-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la resolution apparait dans la liste
    const resolutionItem = page
      .getByTestId("resolution-item")
      .filter({ hasText: "Approbation des travaux de facade" })
      .first();
    await expect(resolutionItem).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 6 : Voter "Pour" avec pouvoir de vote (tantiemes)
    // ============================================================
    // Trouver le panneau de vote dans la resolution
    const voteSection = resolutionItem.locator(
      '[data-testid="vote-btn-pour"]',
    );
    await voteSection.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);

    // Selectionner "Pour"
    await humanClickLocator(page, voteSection);
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Saisir le pouvoir de vote (tantiemes/milliemes)
    const votingPowerInput = resolutionItem.locator(
      '[data-testid="vote-voting-power"]',
    );
    await votingPowerInput.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_TYPE);
    await votingPowerInput.clear();
    await votingPowerInput.fill("150");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    await stepPause(page);

    // Soumettre le vote (cliquer sur le bouton "Enregistrer le vote")
    const submitVoteBtn = resolutionItem.locator(
      "button:has-text('vote')",
    ).last();
    await humanClickLocator(page, submitVoteBtn);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 7 : Verifier que le vote est enregistre
    // ============================================================
    // Les barres de progression devraient montrer le vote
    const progressPour = resolutionItem.locator(
      '[data-testid="vote-progress-pour"]',
    );
    await progressPour.scrollIntoViewIfNeeded();
    await expect(progressPour).toBeVisible({ timeout: 10000 });

    // Verifier que le compteur de votes "Pour" a augmente (n'est plus "0")
    await expect(progressPour).not.toContainText("0 vote");

    await stepPause(page);

    // ============================================================
    // ETAPE 8 : Cloturer le scrutin
    // ============================================================
    const closeBtn = resolutionItem.locator(
      '[data-testid="vote-close-btn"]',
    );
    await closeBtn.scrollIntoViewIfNeeded();
    await expect(closeBtn).toBeVisible({ timeout: 10000 });

    // Intercepter le dialogue de confirmation
    page.on("dialog", (dialog) => dialog.accept());

    await humanClickLocator(page, closeBtn);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 9 : Verifier le statut final de la resolution
    // ============================================================
    // Apres cloture, le statut devrait etre "Adoptee" (vote Pour majoritaire)
    // Le badge de statut est dans le ResolutionStatusBadge
    const statusBadge = resolutionItem.locator("span").filter({
      hasText: /Adoptée|Rejetée/,
    });
    await expect(statusBadge).toBeVisible({ timeout: 15000 });

    // Le formulaire de vote ne devrait plus etre visible
    await expect(
      resolutionItem.locator('[data-testid="vote-btn-pour"]'),
    ).not.toBeVisible();

    // Verifier le compteur dans l'en-tete (1 adoptee ou 1 rejetee)
    const adoptedOrRejected = page.locator(
      '[data-testid="resolution-adopted-count"], [data-testid="resolution-rejected-count"]',
    ).first();
    await expect(adoptedOrRejected).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
