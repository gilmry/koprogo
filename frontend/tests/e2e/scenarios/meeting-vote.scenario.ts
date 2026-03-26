/**
 * SCENARIO: Vote sur une resolution en assemblee generale (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Le syndic se connecte et consulte l'AG avec sa resolution
 *   2. Un coproprietaire se connecte, trouve l'AG et vote "Pour"
 *   3. Le syndic revient, cloture le scrutin et verifie le resultat
 *
 * Duree video attendue : ~90-120 secondes (rythme humain, multi-role)
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

test.describe("Scenario: Vote multi-role sur une resolution en AG", () => {
  test.setTimeout(180_000);

  // ----- Donnees de test (creees via API, invisibles en video) -----
  let syndicEmail: string;
  let syndicPassword: string;
  let ownerEmail: string;
  let ownerPassword: string;
  let meetingTitle: string;
  let resolutionTitle: string;

  test.beforeAll(async ({ request }) => {
    const ts = Date.now();
    syndicEmail = `scenario-vote-syndic-${ts}@koprogo.test`;
    syndicPassword = "test123456";
    ownerEmail = `scenario-vote-owner-${ts}@koprogo.test`;
    ownerPassword = "test123456";
    resolutionTitle = `Approbation travaux facade ${ts}`;

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

    // 4. Login as syndic
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: syndicEmail, password: syndicPassword },
    });
    const syndic = await syndicResp.json();
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // 5. Register owner user account
    const ownerRegResp = await request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: ownerPassword,
        first_name: "Marie",
        last_name: "Dubois",
        role: "owner",
        organization_id: org.id,
      },
    });
    const ownerUser = await ownerRegResp.json();
    const ownerUserId =
      ownerUser.user?.id || ownerUser.id || ownerUser.user_id || "";

    // 6. Create building
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

    // 7. Create owner record linked to user
    await request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Marie",
        last_name: "Dubois",
        email: ownerEmail,
        address: "15 Rue de la Loi, Apt 3A",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        user_id: ownerUserId,
      },
      headers: syndicHeaders,
    });

    // 8. Create meeting (Ordinary, scheduled 30 days from now)
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    meetingTitle = `AG Ordinaire - Budget ${ts}`;

    const meetingResp = await request.post(`${API_BASE}/meetings`, {
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
    const meeting = await meetingResp.json();

    // 9. Create resolution via API
    await request.post(`${API_BASE}/meetings/${meeting.id}/resolutions`, {
      data: {
        meeting_id: meeting.id,
        title: resolutionTitle,
        description:
          "Resolution pour approuver les travaux de ravalement de facade " +
          "du batiment, incluant la reparation des fissures et " +
          "la peinture exterieure. Devis retenu: 45.000 EUR HTVA.",
        resolution_type: "works",
        majority_required: "Simple",
      },
      headers: syndicHeaders,
    });
  });

  test("Syndic prepare l'AG, coproprietaire vote, syndic cloture", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Le syndic se connecte et consulte l'AG
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // Navigation vers les Assemblees
    await humanClick(page, "nav-link-assemblées");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Assemblées");
    await stepPause(page);

    // Cliquer sur l'assemblee pour voir le detail
    await expect(
      page.getByTestId("meeting-list-container"),
    ).toBeVisible({ timeout: 15000 });
    await waitForSpinner(page);

    const meetingCard = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG Ordinaire" })
      .first();
    await expect(meetingCard).toBeVisible({ timeout: 15000 });

    const detailsLink = meetingCard.locator("a").first();
    await humanClickLocator(page, detailsLink);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier la section Resolutions
    const resolutionSection = page.getByTestId("resolution-list");
    await expect(resolutionSection).toBeVisible({ timeout: 15000 });
    await resolutionSection.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const resolutionItem = page.getByTestId("resolution-item").first();
    await expect(resolutionItem).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Le coproprietaire se connecte et vote
    // ============================================================
    await humanLogin(page, ownerEmail, ownerPassword);
    await stepPause(page);

    // L'owner navigue vers les assemblees (community section)
    await humanClick(page, "nav-link-assemblées");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Trouver et ouvrir l'AG
    await expect(
      page.getByTestId("meeting-list-container"),
    ).toBeVisible({ timeout: 15000 });
    await waitForSpinner(page);

    const meetingCard2 = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG Ordinaire" })
      .first();
    await expect(meetingCard2).toBeVisible({ timeout: 15000 });

    const detailsLink2 = meetingCard2.locator("a").first();
    await humanClickLocator(page, detailsLink2);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Trouver la resolution et voter "Pour"
    const resolutionSection2 = page.getByTestId("resolution-list");
    await expect(resolutionSection2).toBeVisible({ timeout: 15000 });
    await resolutionSection2.scrollIntoViewIfNeeded();

    const resolutionItem2 = page.getByTestId("resolution-item").first();
    await expect(resolutionItem2).toBeVisible({ timeout: 15000 });

    const voteBtnPour = resolutionItem2.locator(
      '[data-testid="vote-btn-pour"]',
    );
    await voteBtnPour.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await humanClickLocator(page, voteBtnPour);
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // Saisir le pouvoir de vote (tantiemes/milliemes)
    const votingPowerInput = resolutionItem2.locator(
      '[data-testid="vote-voting-power"]',
    );
    await votingPowerInput.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_TYPE);
    await votingPowerInput.clear();
    await votingPowerInput.fill("150");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    // Soumettre le vote
    const submitVoteBtn = resolutionItem2
      .locator("button")
      .filter({ hasText: /vote/i })
      .last();
    await humanClickLocator(page, submitVoteBtn);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que le vote est enregistre
    const progressPour = resolutionItem2.locator(
      '[data-testid="vote-progress-pour"]',
    );
    await progressPour.scrollIntoViewIfNeeded();
    await expect(progressPour).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Le syndic revient, cloture le scrutin, verifie
    // ============================================================
    await humanLogin(page, syndicEmail, syndicPassword);
    await stepPause(page);

    // Re-naviguer vers l'AG
    await humanClick(page, "nav-link-assemblées");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(
      page.getByTestId("meeting-list-container"),
    ).toBeVisible({ timeout: 15000 });
    await waitForSpinner(page);

    const meetingCard3 = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG Ordinaire" })
      .first();
    await expect(meetingCard3).toBeVisible({ timeout: 15000 });

    const detailsLink3 = meetingCard3.locator("a").first();
    await humanClickLocator(page, detailsLink3);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Trouver la resolution et cloturer le scrutin
    const resolutionSection3 = page.getByTestId("resolution-list");
    await expect(resolutionSection3).toBeVisible({ timeout: 15000 });
    await resolutionSection3.scrollIntoViewIfNeeded();

    const resolutionItem3 = page.getByTestId("resolution-item").first();
    await expect(resolutionItem3).toBeVisible({ timeout: 15000 });

    const closeBtn = resolutionItem3.locator(
      '[data-testid="vote-close-btn"]',
    );
    await closeBtn.scrollIntoViewIfNeeded();
    await expect(closeBtn).toBeVisible({ timeout: 10000 });

    // Intercepter le dialogue de confirmation
    page.on("dialog", (dialog) => dialog.accept());

    await humanClickLocator(page, closeBtn);
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier le statut final (Adoptee car vote Pour majoritaire)
    const statusBadge = resolutionItem3.locator("span").filter({
      hasText: /Adoptée|Rejetée|adoptée|rejetée/,
    });
    await expect(statusBadge).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
