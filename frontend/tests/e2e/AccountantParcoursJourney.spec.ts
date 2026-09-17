import { test, expect, type Page } from "@playwright/test";
import { randomUUID } from "node:crypto";
import { uiLoginWithRetry, adminLogin, ensureAcp } from "./helpers/auth";
import { seedBuildingWithUnitsViaPage } from "./helpers/building";
import { failOnPageErrors } from "./helpers/pageErrors";
import { confirmerSiDemande, amorce } from "./helpers/amorcage";
import { API_BASE } from "./helpers/adresses";

/**
 * Parcours filmé du rôle comptable (#808), un test par étape numérotée de
 * `docs/personas/accountant.md`. Chaque test se connecte pour de vrai (pas
 * d'injection localStorage — cf. la justification dans
 * `AccountantJournalEntriesJourney.spec.ts`) et produit sa propre vidéo
 * (`playwright.config.ts`, `video: { mode: "on" }`).
 *
 * Données uniquement fictives (@security, #808) : emails et immeubles
 * générés avec un timestamp, jamais de copropriété réelle.
 */

const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

interface ComptableContext {
  email: string;
  token: string;
  orgId: string;
  acpId: string;
  adminToken: string;
}

/**
 * Comptable connecté par un VRAI login UI — la preuve de session est le
 * cookie HttpOnly posé par `/auth/register`, pas un cache localStorage
 * (`auth.ts:182`). `token` (retourné par `/auth/register`) sert aux appels
 * d'amorçage API qui précèdent certaines étapes ; l'action documentée,
 * elle, passe toujours par l'écran.
 */
async function loginAsAccountant(
  page: Page,
  prefix: string,
): Promise<ComptableContext> {
  const timestamp = Date.now();
  const email = `${prefix}-${timestamp}@example.com`;

  const adminToken = await adminLogin(page);
  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: `${prefix} Org ${timestamp}`,
      slug: `${prefix}-${timestamp}`,
      contact_email: email,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await amorce(orgResp, `org ${prefix}`);

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Comptable",
      last_name: `Recette${timestamp}`,
      role: "accountant",
      organization_id: org.id,
    },
  });
  const userData = await amorce(regResp, `register ${prefix}`);

  await uiLoginWithRetry(page, email, TEST_PASSWORD, /\/accountant/);
  await page.goto("/accountant", { waitUntil: "networkidle" });

  const acpId = await ensureAcp(page, org.id, adminToken, prefix);

  return {
    email,
    token: userData.token,
    orgId: org.id,
    acpId,
    adminToken,
  };
}

/**
 * Immeuble CONFORME : Σ(quotités des lots) == total_tantiemes déclaré.
 * Nécessaire pour les étapes verrouillées par la conformité (3, 7, 8, 9).
 */
async function immeubleConforme(
  page: Page,
  ctx: ComptableContext,
): Promise<string> {
  const { buildingId } = await seedBuildingWithUnitsViaPage(
    page,
    ctx.adminToken,
    ctx.orgId,
    4,
    1000,
  );
  return buildingId;
}

/**
 * Immeuble NON CONFORME : acte de base à 1000 millièmes, AUCUN lot créé —
 * Σ quotités = 0 ≠ 1000. C'est le cas caractérisé par #783 (avant, on ne
 * pouvait pas provoquer volontairement un immeuble incohérent en recette).
 */
async function immeubleNonConforme(
  page: Page,
  ctx: ComptableContext,
  prefix: string,
): Promise<string> {
  const timestamp = Date.now();
  const resp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `${prefix} Non Conforme ${timestamp}`,
      address: `${timestamp} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 4,
      total_tantiemes: 1000,
      construction_year: 2010,
      acp_id: ctx.acpId,
    },
    headers: { Authorization: `Bearer ${ctx.adminToken}` },
  });
  const building = await amorce(resp, `immeuble non conforme ${prefix}`);
  return building.id as string;
}

test.describe("Comptable — parcours documenté (docs/personas/accountant.md, #808)", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("1. @happy se connecter et arriver sur son périmètre comptable", async ({
    page,
  }) => {
    await loginAsAccountant(page, "parcours-1-login");

    await expect(page).toHaveURL(/\/accountant/);
    await expect(page.getByTestId("accountant-dashboard")).toBeVisible();
  });

  test("1. @edge un compte comptable tout neuf arrive sur un écran utilisable", async ({
    page,
  }) => {
    // Un comptable qui vient d'être créé n'a ni immeuble, ni dépense, ni
    // transaction. C'est l'étape que personne ne teste et que tout le monde
    // voit en premier (@edge, #808).
    await loginAsAccountant(page, "parcours-1-vide");

    await expect(page.getByTestId("accountant-dashboard")).toBeVisible();
    // Ni le spinner de chargement ni l'état d'erreur ne doivent persister :
    // `failOnPageErrors` (beforeEach) aurait déjà fait échouer le test sur
    // une exception JS non interceptée.
    await expect(page.getByTestId("accountant-retry-button")).toHaveCount(0);
    await expect(page.getByTestId("accountant-expenses-tile")).toBeVisible();
  });

  test("2. @happy consulter le plan comptable PCMN (au fil des écrans, pas de liste dédiée)", async ({
    page,
  }) => {
    await loginAsAccountant(page, "parcours-2-pcmn");

    // Aucun écran ne liste les 40+ comptes PCMN au 2026-09-16 (voir le
    // document persona, étape 2) : le plan se lit au fil des écrans. Celui
    // qui l'affiche effectivement est le rappel pédagogique de /reports.
    await page.goto("/reports", { waitUntil: "networkidle" });
    await expect(page.getByTestId("reports")).toBeVisible();
    await expect(page.getByTestId("reports-pcmn-mention")).toBeVisible();
  });

  test("3. @happy saisir une dépense sur un immeuble conforme", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-3-depense");
    await immeubleConforme(page, ctx);

    await page.goto("/expenses", { waitUntil: "networkidle" });
    await page.getByTestId("create-button").click();
    await page.getByTestId("building-select").selectOption({ index: 1 });
    const description = `Entretien chaufferie ${Date.now()}`;
    await page.getByTestId("description-input").fill(description);
    await page.getByTestId("amount-input").fill("640");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/expenses") && r.request().method() === "POST",
      ),
      page.getByTestId("submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);
    await expect(page.getByText(description)).toBeVisible();
  });

  test("3. @negative saisir une dépense sur un immeuble NON conforme est refusé (422)", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-3-degrade");
    await immeubleNonConforme(page, ctx, "parcours-3-degrade");

    await page.goto("/expenses", { waitUntil: "networkidle" });
    await page.getByTestId("create-button").click();
    await page.getByTestId("building-select").selectOption({ index: 1 });
    await page
      .getByTestId("description-input")
      .fill(`Dépense refusée ${Date.now()}`);
    await page.getByTestId("amount-input").fill("100");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/expenses") && r.request().method() === "POST",
      ),
      page.getByTestId("submit-button").click(),
    ]);
    // Cas dégradé documenté (docs/personas/accountant.md) : l'immeuble n'est
    // pas conforme à son acte de base (Σ quotités ≠ total_tantiemes), #770.
    expect(resp.status()).toBe(422);
    const body = await resp.json();

    // DEUX codes valent ici, et ce n'est pas un relâchement.
    //
    // Story H1 pose le constat au niveau IMMEUBLE, Story H5 au niveau
    // COPROPRIÉTÉ. Les deux sont légitimes et portent le même récit ; le
    // serveur choisit selon le niveau où la non-conformité se constate. Ce
    // test exigeait le code immeuble et échouait donc dès que le constat
    // remontait d'un cran — sans que rien ne soit cassé.
    //
    // Ce qui compte pour l'utilisateur n'est pas LEQUEL des deux revient,
    // c'est que le refus soit NARRATIF : `isConformityError` reconnaît
    // désormais les deux et déclenche le toast. Avant le 2026-09-17 il n'en
    // connaissait qu'un, et un refus au niveau ACP s'affichait en 422 nu
    // (#942). C'est ce contrat-là qu'on vérifie.
    const texte = JSON.stringify(body);
    expect(
      texte.includes("BUILDING_NOT_CONFORMANT") ||
        texte.includes("ACP_NOT_CONFORMANT"),
    ).toBe(true);
    // Le payload narratif doit porter de quoi écrire le message, quel que
    // soit le niveau : sans ces champs, le toast ne peut rien dire.
    expect(body.details).toMatchObject({
      units_delta: expect.any(Number),
      quota_basis: expect.any(Number),
    });
    expect(typeof body.details.quota_delta).toBe("string");
  });

  test("4. @happy suivre le workflow d'une facture, brouillon → soumission → approbation → paiement", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-4-facture");
    const buildingId = await immeubleConforme(page, ctx);
    const description = `Contrat entretien ascenseur ${Date.now()}`;

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description,
        amount: 950.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    await amorce(expenseResp, "seed expense pour workflow facture");

    await page.goto("/invoice-workflow", { waitUntil: "networkidle" });
    const card = page
      .getByTestId("invoice-card")
      .filter({ hasText: description });
    await expect(card).toBeVisible();

    const attenteSoumission = page.waitForResponse(
      (r) => r.url().includes("/submit") && r.request().method() === "PUT",
    );
    await card.getByTestId("submit-approval-button").click();
    await confirmerSiDemande(page);
    expect((await attenteSoumission).status()).toBe(200);
    await expect(card.getByTestId("approve-button")).toBeVisible();

    const approveModal = page.locator(".modal-footer").filter({
      has: page.getByTestId("invoice-approve-confirm-button"),
    });
    const [approveResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/approve") && r.request().method() === "PUT",
      ),
      (async () => {
        await card.getByTestId("approve-button").click();
        await approveModal
          .getByTestId("invoice-approve-confirm-button")
          .click();
      })(),
    ]);
    expect(approveResp.status()).toBe(200);
    await expect(card.getByTestId("mark-paid-button")).toBeVisible();

    const attentePaiement = page.waitForResponse(
      (r) => r.url().includes("/mark-paid") && r.request().method() === "PUT",
    );
    await card.getByTestId("mark-paid-button").click();
    await confirmerSiDemande(page);
    expect((await attentePaiement).status()).toBe(200);
  });

  test("5. @happy passer une écriture au journal, en partie double", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-5-journal");
    const buildingId = await immeubleConforme(page, ctx);

    await page.goto("/journal-entries", { waitUntil: "networkidle" });

    const selecteur = page.getByTestId("building-selector-input");
    await expect(selecteur).toBeVisible({ timeout: 15_000 });
    await selecteur.click();
    await selecteur.fill("parcours-5-journal");
    const resultat = page.getByTestId(`building-selector-result-${buildingId}`);
    await expect(resultat).toBeVisible({ timeout: 10_000 });
    await resultat.click();

    await expect(page.getByTestId("journal-entry-form")).toBeVisible({
      timeout: 15_000,
    });
    await page
      .getByTestId("journal-entry-description-input")
      .fill(`Facture entretien ${Date.now()}`);

    // 604002 "Eau" / 440 "Fournisseurs" — comptes PCMN réels du seed belge.
    await page.locator("#journal-line-0-account-code").fill("604002");
    await page.locator("#journal-line-0-debit").fill("120");
    await page.locator("#journal-line-1-account-code").fill("440");
    await page.locator("#journal-line-1-credit").fill("120");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/journal-entries") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("submit-journal-entry-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("6. @happy établir un budget, le soumettre à l'assemblée puis l'approuver", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-6-budget");
    await immeubleConforme(page, ctx);

    await page.goto("/budgets", { waitUntil: "networkidle" });
    await page.waitForTimeout(500);
    await page.getByTestId("create-budget-button").click();
    await page.getByTestId("budget-building-select").selectOption({ index: 1 });
    await page.getByTestId("budget-ordinary-amount").fill("42000");

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/budgets") && r.request().method() === "POST",
      ),
      page.getByTestId("budget-submit-button").click(),
    ]);
    expect(createResp.status()).toBe(201);
    const budget = await createResp.json();

    await page.goto(`/budget-detail?id=${budget.id}`, {
      waitUntil: "networkidle",
    });
    await expect(page.getByTestId("budget-detail")).toBeVisible();

    // « submit-budget-button » ouvre une modale de confirmation
    // (`actionEnAttente`, #844) : un clic qui s'arrête là paraît inerte.
    // C'est l'hypothèse documentée dans docs/personas/accountant.md pour le
    // signalement « R2-8 ».
    const attenteSoumission = page.waitForResponse(
      (r) => r.url().includes("/submit") && r.request().method() === "PUT",
    );
    await page.getByTestId("submit-budget-button").click();
    const confirme = await confirmerSiDemande(page);
    expect(
      confirme,
      "la modale de confirmation « soumettre » doit s'ouvrir",
    ).toBe(true);
    expect((await attenteSoumission).status()).toBe(200);

    await page.getByTestId("approve-budget-button").click();
    await page
      .getByTestId("budget-approve-meeting-id-input")
      .fill(randomUUID());
    const [approveResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/approve") && r.request().method() === "PUT",
      ),
      page.getByTestId("budget-approve-confirm-button").click(),
    ]);
    expect(approveResp.status()).toBe(200);
  });

  test("7. @happy répartir une dépense par tantièmes", async ({ page }) => {
    const ctx = await loginAsAccountant(page, "parcours-7-tantiemes");
    const buildingId = await immeubleConforme(page, ctx);

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        // « Common » n'est pas une variante d'ExpenseCategory — le serveur
        // rendait 400 « unknown variant ». Le libellé de la dépense dit
        // « Nettoyage parties communes » : la variante est `Cleaning`.
        category: "Cleaning",
        description: `Nettoyage parties communes ${Date.now()}`,
        amount: 300.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    const expense = await amorce(expenseResp, "seed expense pour répartition");

    await page.goto(`/expense-detail?id=${expense.id}`, {
      waitUntil: "networkidle",
    });
    await expect(page.getByTestId("distributions-section")).toBeVisible();
    await expect(page.getByTestId("no-distribution")).toBeVisible();

    const [resp] = await Promise.all([
      page.waitForResponse((r) => r.url().includes("/calculate-distribution")),
      page.getByTestId("calculate-distribution-button").click(),
    ]);
    expect(resp.ok()).toBe(true);
    await expect(page.getByTestId("distribution-row").first()).toBeVisible();
    await expect(page.getByTestId("distribution-total")).toBeVisible();
  });

  test("8. @happy émettre un appel de fonds collectif", async ({ page }) => {
    const ctx = await loginAsAccountant(page, "parcours-8-appel");
    await immeubleConforme(page, ctx);

    await page.goto("/call-for-funds", { waitUntil: "networkidle" });
    await page.getByTestId("call-for-funds-create-button").click();
    await expect(page.getByTestId("call-for-funds-form")).toBeVisible();

    await page
      .getByTestId("call-for-funds-building-select")
      .selectOption({ index: 1 });
    await page
      .getByTestId("call-for-funds-title-input")
      .fill(`Appel de fonds travaux ${Date.now()}`);
    await page.getByTestId("call-for-funds-amount-input").fill("8000");
    // La description est OBLIGATOIRE : `CallForFundsForm.svelte:62` refuse
    // la soumission si elle manque (`!title || !description`), affiche un
    // toast et ne POSTe jamais. Le test attendait donc une réponse qui ne
    // partait pas, et expirait au bout de 10 s sur `waitForResponse` — un
    // échec qui ressemble à un serveur muet alors que c'est le formulaire
    // qui refuse.
    await page
      .getByTestId("call-for-funds-description-textarea")
      .fill("Appel de fonds du parcours comptable");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/call-for-funds") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("call-for-funds-submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("8. @negative émettre un appel de fonds sur un immeuble NON conforme est refusé (422)", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-8-degrade");
    await immeubleNonConforme(page, ctx, "parcours-8-degrade");

    await page.goto("/call-for-funds", { waitUntil: "networkidle" });
    await page.getByTestId("call-for-funds-create-button").click();
    await page
      .getByTestId("call-for-funds-building-select")
      .selectOption({ index: 1 });
    await page
      .getByTestId("call-for-funds-title-input")
      .fill(`Appel refusé ${Date.now()}`);
    await page.getByTestId("call-for-funds-amount-input").fill("1000");
    // La description est OBLIGATOIRE : `CallForFundsForm.svelte:62` refuse
    // la soumission si elle manque (`!title || !description`), affiche un
    // toast et ne POSTe jamais. Le test attendait donc une réponse qui ne
    // partait pas, et expirait au bout de 10 s sur `waitForResponse` — un
    // échec qui ressemble à un serveur muet alors que c'est le formulaire
    // qui refuse.
    await page
      .getByTestId("call-for-funds-description-textarea")
      .fill("Appel de fonds du parcours comptable");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/call-for-funds") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("call-for-funds-submit-button").click(),
    ]);
    expect(resp.status()).toBe(422);
    // Le toast narratif documenté (docs/personas/accountant.md, § Cas
    // dégradé) : « Calcul bloqué — Immeuble non conforme ».
    //
    // L'ancre était `building-conformity-badge`, mais ce badge est un AUTRE
    // composant : il vit dans `BuildingDetail` et `ContextBanner`, pas sur
    // cet écran. Le test attendait donc un élément qui n'y est jamais, et
    // l'échec se lisait comme « pas de retour à l'utilisateur » alors que
    // c'était l'ancre qui visait à côté.
    //
    // `toast-error` est l'ancre du toast lui-même
    // (`ToastContainer.svelte:32`), indépendante de la langue comme le
    // voulait #803. Et le récit part maintenant pour de bon : avant le
    // 2026-09-17, `isConformityError` ne reconnaissait pas le constat au
    // niveau ACP et l'utilisateur recevait un 422 nu (#942).
    await expect(page.getByTestId("toast-error").first()).toBeVisible();
  });

  test("9. @happy produire un état daté", async ({ page }) => {
    const ctx = await loginAsAccountant(page, "parcours-9-etat-date");
    await immeubleConforme(page, ctx);

    await page.goto("/etats-dates", { waitUntil: "networkidle" });
    await page.getByTestId("etat-date-create-form").waitFor();

    await page.getByTestId("building").selectOption({ index: 1 });
    await page.getByTestId("unit").selectOption({ index: 1 });
    await page
      .getByTestId("reference-date")
      .fill(new Date().toISOString().slice(0, 10));
    await page.getByTestId("notary-name").fill("Étude Notariale Test");
    await page.getByTestId("notary-email").fill("notaire-test@example.com");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/etats-dates") && r.request().method() === "POST",
      ),
      page.getByTestId("etat-date-generate-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("10. @happy sortir le bilan et vérifier l'équilibre", async ({
    page,
  }) => {
    const ctx = await loginAsAccountant(page, "parcours-10-bilan");
    const buildingId = await immeubleConforme(page, ctx);

    await page.goto(`/reports?buildingId=${buildingId}`, {
      waitUntil: "networkidle",
    });

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/reports/balance-sheet") &&
          r.request().method() === "GET",
      ),
      page.getByTestId("financial-reports-generate").click(),
    ]);
    expect(resp.status()).toBe(200);
    const bilan = await resp.json();

    await expect(
      page.getByTestId("financial-reports-balance-sheet-title"),
    ).toBeVisible();
    // Équilibre comptable : actif == passif (à l'arrondi près, cf. Decimal
    // côté serveur — ADR-0007, jamais de f64 en comptabilité).
    if (
      typeof bilan.total_assets === "number" &&
      typeof bilan.total_liabilities === "number"
    ) {
      expect(
        Math.abs(bilan.total_assets - bilan.total_liabilities),
      ).toBeLessThan(0.01);
    }
  });
});
