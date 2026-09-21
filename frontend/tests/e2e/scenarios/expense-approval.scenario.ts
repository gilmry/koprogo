/**
 * SCENARIO: Workflow d'approbation d'une facture (SINGLE ROLE - syndic)
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet du syndic Francois :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la page Workflow Factures via le menu lateral
 *   3. Visualisation d'une facture en statut Draft
 *   4. Soumission pour approbation (Draft -> PendingApproval)
 *   5. Approbation de la facture (PendingApproval -> Approved)
 *   6. Verification du statut final Approved
 *
 * Duree video attendue : ~45-60 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import { ADMIN_PASSWORD } from "../helpers/identifiants";
import {
  amorce,
  confirmerSiDemande,
  aucuneErreurAffichee,
} from "../helpers/amorcage";
import { nameContains } from "../helpers/name-match";
import {
  humanLogin,
  humanClick,
  humanClickLocator,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

import { API_BASE } from "../helpers/adresses";

test.describe("Scenario: Workflow d'approbation d'une facture", () => {
  test.setTimeout(120_000);

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login");
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

    // 3. Create a Draft expense for the scenario via Francois
    const syndicResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "francois@syndic-leroy.be", password: "francois123" },
    });
    const syndic = await amorce(syndicResp, "POST /auth/login");
    const syndicHeaders = { Authorization: `Bearer ${syndic.token}` };

    // Get buildings to find Residence du Parc
    const buildingsResp = await request.get(`${API_BASE}/buildings`, {
      headers: syndicHeaders,
    });
    const buildings = await amorce(buildingsResp, "GET /buildings");
    // `GET /buildings` rend un `PageResponse` — `{ data, pagination }`,
    // jamais un tableau nu. `Array.isArray` valait donc TOUJOURS faux,
    // `building` TOUJOURS null, et le `if (building)` ci-dessous sautait
    // l'amorçage entier sans rien dire. Le scénario échouait 150 lignes
    // plus loin sur une liste vide, en accusant l'affichage.
    const listeImmeubles = Array.isArray(buildings)
      ? buildings
      : (buildings?.data ?? []);
    const building = listeImmeubles.find(
      (b: any) => b.name && nameContains(b.name, "Résidence du Parc"),
    );

    if (!building) {
      throw new Error(
        `Amorçage : immeuble introuvable parmi ${listeImmeubles.length} ` +
          `renvoyé(s) par GET /buildings : ` +
          `${listeImmeubles.map((b: any) => b.name).join(", ") || "(aucun)"}. ` +
          `Sans lui, aucune donnée n'est créée et le scénario échouera ` +
          `plus loin sur une liste vide.`,
      );
    }

    {
      const reponseAmorce1 = await request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: building.id,
          category: "Maintenance",
          description: "Reparation toiture - infiltrations eau",
          amount: 1250.0,
          expense_date: new Date().toISOString(),
        },
        headers: syndicHeaders,
      });
      await amorce(reponseAmorce1, "POST /expenses");
    }
  });

  // PAS de suppression du monde de scénario.
  //
  // Il est PARTAGÉ : huit fichiers le sèment, dix emploient ses comptes, et
  // ce même bloc de teardown était copié dans QUATORZE d'entre eux. Chacun
  // détruisait donc la précondition des autres.
  //
  // Le semer coûte 44 s, le supprimer une seconde. Pendant une campagne, la
  // spec suivante devait le reconstruire contre un plafond de requête de
  // 10 s : elle échouait, et ses tests en série étaient sautés. C'est ce qui
  // rendait `AccessibiliteEcransAuthentifies` inexécutable, alors que son
  // propre `beforeAll` est tolérant et dit même que le monde « peut déjà
  // exister, semé par un autre fichier de la même campagne ».
  //
  // Le monde est un scénario FIXE (« Résidence du Parc Royal ») : le laisser
  // en place n'accumule rien. Le nettoyage, si on en veut un, appartient à un
  // `globalTeardown` — c'est-à-dire à quelqu'un qui possède le fixture.
  // Cf. #942 et #876.

  test("@happy Francois soumet et approuve une facture via l'interface", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "francois@syndic-leroy.be", "francois123");
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers le Workflow Factures via le menu
    // ============================================================
    await humanClick(page, "nav-link-invoice-workflow");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Trouver la facture Draft dans la liste
    // ============================================================
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const invoiceCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(invoiceCard).toBeVisible({ timeout: 15000 });

    await invoiceCard.scrollIntoViewIfNeeded();
    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Soumettre pour approbation (Draft -> PendingApproval)
    // ============================================================
    const submitButton = invoiceCard.getByTestId("submit-approval-button");
    await expect(submitButton).toBeVisible({ timeout: 5000 });

    page.on("dialog", (dialog) => dialog.accept());

    await submitButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await submitButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    // La soumission demande confirmation, et le scenario ne confirmait pas.
    //
    // Capture d'ecran du run du 2026-09-08 : « Êtes-vous sûr de vouloir
    // soumettre cette facture pour approbation ? », Annuler / Confirmer, et le
    // scenario qui attend derriere `approve-button` un bouton qu'il ne verra
    // jamais.
    //
    // #844 a remplace soixante `confirm()` natifs par de vraies modales. Un
    // navigateur pilote SUPPRIME les dialogues natifs : le geste passait donc
    // tout seul avant la conversion. L'etape d'approbation, plus bas, gere
    // bien sa modale — celle de la soumission avait ete oubliee.
    await confirmerSiDemande(page);
    await aucuneErreurAffichee(
      page,
      "soumission de la facture pour approbation",
    );

    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    const updatedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(updatedCard).toBeVisible({ timeout: 15000 });

    const approveButton = updatedCard.getByTestId("approve-button");
    await expect(approveButton).toBeVisible({ timeout: 10000 });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Approuver la facture (PendingApproval -> Approved)
    // ============================================================
    await approveButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await approveButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    const modal = page.locator(".modal").first();
    await expect(modal).toBeVisible({ timeout: 5000 });
    await page.waitForTimeout(PACE.BETWEEN_STEPS);

    await expect(modal.locator("text=Reparation toiture")).toBeVisible();

    const confirmApproveButton = modal.locator("button.btn-success").last();
    await confirmApproveButton.scrollIntoViewIfNeeded();
    await page.waitForTimeout(PACE.BEFORE_CLICK);
    await confirmApproveButton.click();
    await page.waitForTimeout(PACE.AFTER_CLICK);

    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // ============================================================
    // ETAPE 6 : Verifier que la facture est Approved
    // ============================================================
    const approvedCard = page
      .getByTestId("invoice-card")
      .filter({ hasText: "Reparation toiture" })
      .first();
    await expect(approvedCard).toBeVisible({ timeout: 15000 });

    const markPaidButton = approvedCard.getByTestId("mark-paid-button");
    await expect(markPaidButton).toBeVisible({ timeout: 10000 });

    await expect(
      approvedCard.getByTestId("submit-approval-button"),
    ).not.toBeVisible();
    await expect(approvedCard.getByTestId("approve-button")).not.toBeVisible();

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
