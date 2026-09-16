/**
 * SCENARIO: Le cycle de vie d'une assemblee generale (MULTI-ROLE)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Compagnon du document docs/workflows/assemblee-generale.md (#810) : ce
 * test rejoue au navigateur les douze etapes du fil syndic -> coproprietaire
 * -> syndic, en alternant CINQ sessions distinctes (jamais un seul login
 * pour tout — regle 9 de .claude/rules/CRITICAL.md) :
 *
 *   1. Francois (syndic)  : cree l'AG, l'ordre du jour + resolution (API,
 *      aucune UI n'existe pour cette etape — voir le document workflow),
 *      convoque.
 *   2. Alice (coproprietaire) : recoit et consulte l'AG.
 *   3. Francois (syndic)  : constate le quorum a l'ouverture.
 *   4. Alice (coproprietaire) : vote, avec procuration pour un second
 *      coproprietaire absent (Bob).
 *   5. Francois (syndic)  : cloture le vote, verifie le resultat proclame,
 *      puis tente de cloturer l'assemblee.
 *
 * La derniere etape N'EST PAS censee reussir : le workflow documente un
 * verrou circulaire (le PV ne peut etre attache qu'apres cloture, et la
 * cloture exige qu'un PV existe deja — voir docs/workflows/assemblee-generale.md
 * §"verrou circulaire"). Le test l'affirme comme ROUGE plutot que de la
 * contourner pour que la video soit jolie (@negative).
 *
 * Duree video attendue : ~3-4 minutes (rythme humain, cinq sessions).
 */
import { test, expect } from "@playwright/test";
import { ADMIN_PASSWORD } from "../helpers/identifiants";
import {
  amorce,
  aucuneErreurAffichee,
  confirmerSiDemande,
} from "../helpers/amorcage";
import { nameContains } from "../helpers/name-match";
import {
  humanLogin,
  humanClick,
  humanClickLocator,
  humanFill,
  humanSelect,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

import { API_BASE } from "../helpers/adresses";

const SYNDIC_EMAIL = "francois@syndic-leroy.be";
const SYNDIC_PASSWORD = "francois123";
const OWNER_EMAIL = "alice@residence-parc.be";
const OWNER_PASSWORD = "alice123";

/** Format attendu par <input type="datetime-local"> : "YYYY-MM-DDTHH:mm". */
function commeDatetimeLocal(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, "0");
  return (
    `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}` +
    `T${pad(date.getHours())}:${pad(date.getMinutes())}`
  );
}

test.describe("Scenario: Le cycle de vie d'une assemblee generale", () => {
  test.setTimeout(240_000);

  const timestamp = Date.now();
  const titreAg = `AG Cycle de vie ${timestamp}`;

  let buildingId: string;

  test.beforeAll(async ({ request }) => {
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login");
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    const seedResp = await request.post(`${API_BASE}/seed/scenario/world`, {
      headers: adminHeaders,
    });
    if (!seedResp.ok()) {
      console.log("Seed world already exists, continuing...");
    }

    // L'immeuble et l'organisation ne dependent pas de la reponse du seed
    // (qui peut etre vide si le monde existait deja) : on les relit toujours
    // via l'API, comme convocation-send.scenario.ts.
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: SYNDIC_EMAIL, password: SYNDIC_PASSWORD },
    });
    const syndic = await amorce(syndicResp, "POST /auth/login (syndic)");
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    const buildingsResp = await request.get(`${API_BASE}/buildings`, {
      headers: syndicHeaders,
    });
    const buildings = await amorce(buildingsResp, "GET /buildings");
    const listeImmeubles = Array.isArray(buildings)
      ? buildings
      : (buildings?.data ?? []);
    const building = listeImmeubles.find(
      (b: any) => b.name && nameContains(b.name, "Résidence du Parc"),
    );
    if (!building) {
      throw new Error(
        `Amorçage : immeuble introuvable parmi ${listeImmeubles.length} ` +
          `renvoyé(s) par GET /buildings.`,
      );
    }
    buildingId = building.id;
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

  test("le fil syndic -> coproprietaire -> syndic, jusqu'au verrou de cloture", async ({
    page,
    request,
  }) => {
    // ============================================================
    // SESSION 1 (SYNDIC) — Etapes 1-3 : creer l'AG, l'ordre du jour +
    // la resolution, convoquer.
    // ============================================================
    await humanLogin(page, SYNDIC_EMAIL, SYNDIC_PASSWORD);
    await stepPause(page);

    await humanClick(page, "nav-link-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // --- Etape 1 : creer l'AG (Art. 3.87 §2/§3), date a 30 jours ---------
    await humanClick(page, "btn-new-meeting");
    await page.waitForTimeout(PACE.AFTER_CLICK);
    await expect(page.getByTestId("meeting-create-form")).toBeVisible({
      timeout: 10000,
    });

    await humanFill(page, "input-meeting-title", titreAg);
    await humanSelect(page, "select-meeting-type", "Ordinary");

    const dateAg = new Date();
    dateAg.setDate(dateAg.getDate() + 30);
    await humanFill(page, "input-meeting-date", commeDatetimeLocal(dateAg));

    // Preuve filmee du verrou 1 de #780, leve le 2026-09-06 : la modale
    // avertit DES LA SAISIE que le delai de quinze jours est tenable,
    // plutot qu'au clic sur "creer une convocation" (docs/workflows/
    // assemblee-generale.md, ligne 1 du tableau).
    await expect(page.getByTestId("meeting-date-delai-tenable")).toBeVisible({
      timeout: 5000,
    });
    await stepPause(page);

    await humanFill(page, "input-meeting-location", "Salle communale");

    const [creationResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/meetings") && r.request().method() === "POST",
      ),
      humanClick(page, "btn-submit-meeting"),
    ]);
    const meeting = await creationResp.json();
    const meetingId: string = meeting.id;
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(
      page
        .getByTestId("meeting-card")
        .filter({ hasText: titreAg })
        .first(),
    ).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // --- Etape 2 : ordre du jour + resolution (Art. 3.87 §2) -------------
    //
    // Aucune UI n'existe pour ajouter un point d'ordre du jour :
    // MeetingCreateModal.svelte et MeetingDetail.svelte n'exposent que
    // l'affichage (`meeting-info-agenda`), jamais un formulaire de
    // creation. Documente comme constat dans docs/workflows/
    // assemblee-generale.md plutot que fabrique une UI qui n'existe pas.
    const syndicLoginResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: SYNDIC_EMAIL, password: SYNDIC_PASSWORD },
    });
    const syndicSession = await amorce(
      syndicLoginResp,
      "POST /auth/login (syndic, etape 2)",
    );
    const syndicHeaders = { Authorization: `Bearer ${syndicSession.token}` };

    const agendaResp = await request.post(
      `${API_BASE}/meetings/${meetingId}/agenda`,
      {
        data: { item: "Budget annuel" },
        headers: syndicHeaders,
      },
    );
    await amorce(agendaResp, "POST /meetings/{id}/agenda");

    const resolutionResp = await request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Résolution ${titreAg}`,
          description: "Adoption du budget annuel",
          resolution_type: "ordinary",
          majority_required: "absolute",
          agenda_item_index: 0,
        },
        headers: syndicHeaders,
      },
    );
    await amorce(resolutionResp, "POST /meetings/{id}/resolutions");

    // --- Etape 3 : convoquer, au moins quinze jours avant -----------------
    const convocationResp = await request.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meetingId,
        building_id: buildingId,
        meeting_type: "Ordinary",
        meeting_date: dateAg.toISOString(),
        language: "fr",
      },
      headers: syndicHeaders,
    });
    const convocation = await amorce(convocationResp, "POST /convocations");

    await page.goto(`/convocation-detail?id=${convocation.id}`);
    await waitForSpinner(page);
    await expect(page.getByTestId("convocation-detail")).toBeVisible({
      timeout: 15000,
    });
    await expect(
      page.getByTestId("convocation-detail-legal-deadline"),
    ).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    const sendBtn = page.getByTestId("convocation-detail-btn-send");
    if (await sendBtn.isVisible({ timeout: 5000 }).catch(() => false)) {
      await humanClickLocator(page, sendBtn);
      await confirmerSiDemande(page);
      await waitForSpinner(page);
      await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    }
    await stepPause(page);

    // ============================================================
    // SESSION 2 (COPROPRIETAIRE — Alice) — Etape 4 : recoit et
    // consulte l'AG.
    // ============================================================
    await humanLogin(page, OWNER_EMAIL, OWNER_PASSWORD);
    await stepPause(page);

    await humanClick(page, "owner-quick-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const carteAliceVoit = page
      .getByTestId("meeting-card")
      .filter({ hasText: titreAg })
      .first();
    await expect(carteAliceVoit).toBeVisible({ timeout: 15000 });
    await humanClickLocator(page, carteAliceVoit.locator("a").first());
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // Constat filme (pas un simple commentaire) : la regle de l'Art. 3.87
    // §3 al.3 — accord ecrit prealable pour un envoi par courriel — n'est
    // rattachee a aucun champ de l'API. #784, docs/workflows/
    // assemblee-generale.md.
    const recipientsResp = await request.get(
      `${API_BASE}/convocations/${convocation.id}/recipients`,
      { headers: syndicHeaders },
    );
    const recipients = await amorce(
      recipientsResp,
      "GET /convocations/{id}/recipients",
    );
    const listeDestinataires = Array.isArray(recipients)
      ? recipients
      : (recipients?.data ?? []);
    for (const destinataire of listeDestinataires) {
      expect(
        destinataire,
        "constat #810 : aucun champ d'accord écrit (Art. 3.87 §3 al.3) " +
          "n'existe sur un destinataire de convocation — la règle " +
          "envoi_convocation.rs n'est jamais appelée (#784)",
      ).not.toHaveProperty("email_consent_given_at");
    }

    // ============================================================
    // SESSION 3 (SYNDIC) — Etape 6 : constate le quorum a l'ouverture.
    // ============================================================
    await humanLogin(page, SYNDIC_EMAIL, SYNDIC_PASSWORD);
    await stepPause(page);

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await waitForSpinner(page);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const quorumPanel = page.getByTestId("quorum-panel");
    await quorumPanel.scrollIntoViewIfNeeded();
    await expect(quorumPanel).toBeVisible({ timeout: 15000 });

    await humanFill(page, "quorum-present-input", "600");
    await humanFill(page, "quorum-total-input", "1000");
    await humanClick(page, "quorum-validate-btn");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await expect(page.getByTestId("quorum-validated-badge")).toBeVisible({
      timeout: 10000,
    });
    await stepPause(page);

    // ============================================================
    // SESSION 4 (COPROPRIETAIRE — Alice) — Etapes 5+7 : vote, en
    // portant la procuration d'un second coproprietaire absent (Bob).
    //
    // Le produit ne separe pas "donner procuration" (etape 5) du vote
    // lui-meme (etape 7) : le mandataire saisit `vote-proxy-input` au
    // moment ou il vote (ResolutionVotePanel.svelte). Les deux etapes
    // du fil legal se jouent donc dans une seule action UI, documentee
    // ainsi dans docs/workflows/assemblee-generale.md.
    // ============================================================
    const ownersResp = await request.get(`${API_BASE}/owners`, {
      headers: syndicHeaders,
    });
    const owners = await amorce(ownersResp, "GET /owners");
    const listeOwners = Array.isArray(owners) ? owners : (owners?.data ?? []);
    const bob = listeOwners.find(
      (o: any) => o.email && o.email.includes("bob"),
    );

    await humanLogin(page, OWNER_EMAIL, OWNER_PASSWORD);
    await stepPause(page);

    await humanClick(page, "owner-quick-meetings");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const carteVote = page
      .getByTestId("meeting-card")
      .filter({ hasText: titreAg })
      .first();
    await expect(carteVote).toBeVisible({ timeout: 15000 });
    await humanClickLocator(page, carteVote.locator("a").first());
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const resolutionItem = page.getByTestId("resolution-item").first();
    await expect(resolutionItem).toBeVisible({ timeout: 15000 });
    await resolutionItem.scrollIntoViewIfNeeded();

    if (bob) {
      // Etape 5 (Art. 3.87 §7) : Alice porte la procuration de Bob, absent.
      const proxyInput = resolutionItem.locator(
        '[data-testid="vote-proxy-input"]',
      );
      await expect(proxyInput).toBeVisible({ timeout: 5000 });
      await proxyInput.fill(bob.id);
      await page.waitForTimeout(PACE.AFTER_TYPE);
    }

    const voteBtnPour = resolutionItem.locator('[data-testid="vote-btn-pour"]');
    await voteBtnPour.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await humanClickLocator(page, voteBtnPour);
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "vote d'Alice (avec procuration)");
    await page.waitForTimeout(PACE.AFTER_CLICK);

    const votingPowerInput = resolutionItem.locator(
      '[data-testid="vote-voting-power"]',
    );
    await votingPowerInput.scrollIntoViewIfNeeded();
    await votingPowerInput.clear();
    await votingPowerInput.fill("150");
    await page.waitForTimeout(PACE.AFTER_TYPE);

    const submitVoteBtn = resolutionItem.locator(
      '[data-testid="resolution-vote-submit-button"]',
    );
    await humanClickLocator(page, submitVoteBtn);
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "soumission du vote d'Alice");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await stepPause(page);

    // ============================================================
    // SESSION 5 (SYNDIC) — Etapes 8-9 : cloture le vote, verifie le
    // resultat ; puis etapes 10-12 : tente de cloturer l'assemblee,
    // et CONSTATE LE BLOCAGE (le PV ne peut jamais exister avant la
    // cloture — voir docs/workflows/assemblee-generale.md, "verrou
    // circulaire").
    // ============================================================
    await humanLogin(page, SYNDIC_EMAIL, SYNDIC_PASSWORD);
    await stepPause(page);

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await waitForSpinner(page);
    await page.waitForLoadState("domcontentloaded");
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const resolutionItem2 = page.getByTestId("resolution-item").first();
    await expect(resolutionItem2).toBeVisible({ timeout: 15000 });
    await resolutionItem2.scrollIntoViewIfNeeded();

    const closeBtn = resolutionItem2.locator('[data-testid="vote-close-btn"]');
    await closeBtn.scrollIntoViewIfNeeded();
    await expect(closeBtn).toBeVisible({ timeout: 10000 });

    page.on("dialog", (dialog) => dialog.accept());
    await humanClickLocator(page, closeBtn);
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(page, "clôture du vote");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Etape 9 : le systeme proclame le resultat (Art. 3.88 §1er).
    const statusBadge = resolutionItem2.locator("span").filter({
      hasText: /Adoptée|Rejetée|adoptée|rejetée/,
    });
    await expect(statusBadge).toBeVisible({ timeout: 15000 });
    await stepPause(page);

    // Etapes 10-12, RED — le bouton de cloture de l'assemblee reste
    // desactive : aucun PV n'a pu etre attache (et ne pourra jamais
    // l'etre avant cloture), donc `minutes_draft_exists` reste faux.
    // On affirme le blocage plutot que de le contourner (@negative).
    const completeBtn = page.getByTestId("meeting-complete-btn");
    await completeBtn.scrollIntoViewIfNeeded();
    await expect(completeBtn).toBeVisible({ timeout: 10000 });
    await expect(completeBtn).toBeDisabled();
    await expect(
      page.getByTestId("meeting-complete-button-disabled-reason"),
    ).toBeVisible({ timeout: 5000 });
    await stepPause(page);

    // ============================================================
    // FIN : pause finale pour que la video montre le blocage constate.
    // ============================================================
    await finalPause(page);
  });
});
