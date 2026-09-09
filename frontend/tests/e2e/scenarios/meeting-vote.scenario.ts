/**
 * SCENARIO: Vote sur une resolution en assemblee generale (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours multi-acteur d'une copropriete belge :
 *   1. Francois (syndic) se connecte et consulte l'AG avec sa resolution
 *   2. Alice (coproprietaire, CdC presidente) se connecte et vote "Pour"
 *   3. Francois revient, cloture le scrutin et verifie le resultat
 *
 * Le seed cree deja un meeting (2eme convocation) + resolution (Pending)
 * Duree video attendue : ~90-120 secondes (rythme humain, multi-role)
 */
import { test, expect } from "@playwright/test";
import { ADMIN_PASSWORD } from "../helpers/identifiants";
import {
  amorce,
  aucuneErreurAffichee,
  confirmerSiDemande,
} from "../helpers/amorcage";
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

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login");
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Seed the world (creates meeting + resolution)
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
      data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
    });
    const admin = await adminResp.json();
    await request.delete(`${API_BASE}/seed/scenario/world`, {
      headers: { Authorization: `Bearer ${admin.token}` },
    });
  });

  test("Francois prepare l'AG, Alice vote, Francois cloture", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Francois (syndic) se connecte et consulte l'AG
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // Navigation vers les Assemblees
    await humanClick(page, "nav-link-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("main h1").first()).toContainText("Assemblées");
    await stepPause(page);

    // Cliquer sur l'assemblee pour voir le detail
    await expect(page.getByTestId("meeting-list-container")).toBeVisible({
      timeout: 15000,
    });
    await waitForSpinner(page);

    const meetingCard = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG" })
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
    // ETAPE 2 : Alice (coproprietaire) se connecte et vote
    // ============================================================
    await humanLogin(page, "alice@residence-parc.be", "alice123");
    await stepPause(page);

    // Alice navigue vers les assemblees par SA TUILE, pas par la barre
    // laterale.
    //
    // `canSee` (permissions.ts) ne donne au coproprietaire que les menus
    // `communaute` et `mes-lots` : ni `/meetings` ni `/convocations` n'y
    // figurent. `nav-link-meetings` n'existe donc pas dans son DOM, et le
    // scenario attendait trente secondes un lien reserve au syndic.
    //
    // Le chemin prevu existe : `guards.ts:42` ouvre `/meetings` au role
    // OWNER, et `OwnerDashboard.svelte` porte la tuile
    // `owner-quick-meetings`. C'est par la qu'un coproprietaire rejoint son
    // assemblee.
    await humanClick(page, "owner-quick-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Trouver et ouvrir l'AG
    await expect(page.getByTestId("meeting-list-container")).toBeVisible({
      timeout: 15000,
    });
    await waitForSpinner(page);

    const meetingCard2 = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG" })
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
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "choix « Pour » d'Alice");
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

    // Soumettre le vote, par son ancre.
    //
    // Le scenario prenait `.locator("button").filter({ hasText: /vote/i })
    // .last()`. La carte de resolution contient DEUX boutons dont le texte
    // correspond : « Soumettre le vote » et « Voir les votes (0) ». Le second
    // vient apres dans le DOM (`ResolutionVotePanel.svelte:487` contre 471),
    // donc `.last()` choisissait le mauvais.
    //
    // Resultat : le scenario depliait la liste des votes au lieu de voter.
    // Aucune erreur, aucun toast, et « Total : 0 vote » sur la capture — puis
    // un echec cent lignes plus loin sur le bouton de cloture, absent parce
    // qu'aucun vote n'avait ete emis.
    //
    // Chercher un bouton par son texte est un pari sur la langue ; ici il
    // etait meme ambigu DANS la langue choisie.
    const submitVoteBtn = resolutionItem2.locator(
      '[data-testid="resolution-vote-submit-button"]',
    );
    await humanClickLocator(page, submitVoteBtn);
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "soumission du vote d'Alice");
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
    // ETAPE 3 : Francois revient, cloture le scrutin, verifie
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // Re-naviguer vers l'AG
    await humanClick(page, "nav-link-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.getByTestId("meeting-list-container")).toBeVisible({
      timeout: 15000,
    });
    await waitForSpinner(page);

    const meetingCard3 = page
      .getByTestId("meeting-card")
      .filter({ hasText: "AG" })
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

    const closeBtn = resolutionItem3.locator('[data-testid="vote-close-btn"]');
    await closeBtn.scrollIntoViewIfNeeded();
    await expect(closeBtn).toBeVisible({ timeout: 10000 });

    // Intercepter le dialogue de confirmation
    page.on("dialog", (dialog) => dialog.accept());

    await humanClickLocator(page, closeBtn);
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "closeBtn");
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
