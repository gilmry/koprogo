import { test, expect } from "@playwright/test";
import {
  loginAsAccountantEmetteur,
  loginAsSyndicWithBuilding,
  loginAsSyndicWithOwner,
  loginAsSyndicWithExpense,
  loginAsSyndicWithUnit,
} from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

/**
 * Non-régression du rapport « workflows financiers » du 2026-09-01.
 *
 * ── Ce que ce fichier couvre, et pourquoi il est bâti ainsi ───────────────
 *
 * Sur les 21 constats du rapport, une part notable ne se reproduisait pas.
 * Trois causes distinctes, qu'il vaut la peine de nommer parce qu'elles
 * dictent la forme des tests ci-dessous :
 *
 *   1. F5 « boutons inopérants sur toutes les pages financières » : les
 *      boutons de /payment-reminders et /invoice-workflow sont gardés par un
 *      `confirm()`. Une automatisation de navigateur REJETTE les dialogues par
 *      défaut — le clic aboutissait, la confirmation était refusée, et il ne
 *      se passait rien. D'où `page.on("dialog", d => d.accept())` ici : sans
 *      lui, ces tests reproduiraient le faux positif du rapport.
 *
 *   2. F1/F2 « impossible de lier propriétaires aux lots » : le testeur
 *      passait `owner_id` à `PUT /units`. Ce champ est DÉPRÉCIÉ depuis la
 *      migration `20250127000000_refactor_owners_multitenancy` ; la relation
 *      vit dans `unit_owners`. L'API répondait 200 en jetant le champ, ce qui
 *      rendait l'erreur indétectable. Elle répond désormais 400.
 *
 *   3. F3 « rapports comptables en échec » : testé avec un compte sans
 *      organisation, qui reçoit un 401 sur toute route scopée.
 *
 * Les tests vérifient donc AUSSI que ces comportements-là sont les bons, pas
 * seulement que les vrais défauts sont corrigés. Un rapport qui se trompe sur
 * un point mérite une preuve durable, sans quoi le même constat reviendra au
 * prochain audit.
 */

test.describe("Workflows financiers 2026-09-01 — non-régression", () => {
  // ───────────────────────────────────────────────────────────────────────
  // F1 / F16 — perte silencieuse de champs inconnus
  // ───────────────────────────────────────────────────────────────────────

  test("F1 — PUT /units refuse `owner_id` au lieu de le jeter en silence", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithUnit(page, "fin-f1");

    const resp = await page.request.put(`${API_BASE}/units/${ctx.unitId}`, {
      data: {
        unit_number: "1A",
        floor: 1,
        surface_area: 85.0,
        quota: 1000.0,
        unit_type: "Apartment",
        // Le champ déprécié, cœur du constat F1.
        owner_id: "00000000-0000-0000-0000-000000000001",
      },
      headers: { Authorization: `Bearer ${ctx.adminToken}` },
    });

    // Le défaut d'origine : 200, et le champ perdu sans trace.
    expect(
      resp.status(),
      "un champ inconnu doit être refusé, pas ignoré",
    ).toBe(400);

    // Et le refus doit être du JSON exploitable, pas le `text/plain` d'Actix :
    // un appelant faisant `.json()` dessus recevait « Unexpected token 'J' ».
    const corps = await resp.json();
    expect(corps).toHaveProperty("error");
    expect(JSON.stringify(corps)).toContain("owner_id");
  });

  test("F1 — le même corps SANS `owner_id` passe toujours", async ({ page }) => {
    // Contre-épreuve indispensable : `deny_unknown_fields` ne doit pas avoir
    // cassé la modification de lot pour tout le monde.
    const ctx = await loginAsSyndicWithUnit(page, "fin-f1b");

    const resp = await page.request.put(`${API_BASE}/units/${ctx.unitId}`, {
      data: {
        unit_number: "1A-bis",
        floor: 2,
        surface_area: 90.0,
        quota: 1000.0,
        unit_type: "Apartment",
      },
      headers: { Authorization: `Bearer ${ctx.adminToken}` },
    });
    expect(resp.status(), await resp.text()).toBe(200);
    expect((await resp.json()).unit_number).toBe("1A-bis");
  });

  // ───────────────────────────────────────────────────────────────────────
  // F2 — le cycle complet d'appel de fonds
  // ───────────────────────────────────────────────────────────────────────

  /// Le rapport concluait : « L'ensemble de la chaîne appel de fonds →
  /// ventilation par tantièmes → contributions individuelles est non
  /// fonctionnel. »
  ///
  /// Elle l'est. Ce qui ne fonctionnait pas, c'est la façon d'y entrer : le
  /// testeur rattachait les propriétaires via `PUT /units { owner_id }`,
  /// champ déprécié que l'API acceptait puis jetait. La détention vit dans
  /// `unit_owners`, alimentée par `POST /units/{id}/owners`.
  ///
  /// Ce test parcourt la chaîne de bout en bout, par la bonne porte.
  test("F2 — un appel de fonds envoyé génère les quotes-parts par tantièmes", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithOwner(page, "fin-f2");
    const entetes = { Authorization: `Bearer ${ctx.adminToken}` };

    // 1. Un lot portant la totalité des tantièmes, pour que l'immeuble soit
    //    conforme (le `send` refuse un immeuble en dérive, Art. 3.85 CC).
    const lot = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: ctx.acpId,
        building_id: ctx.buildingId,
        unit_number: "F2-1A",
        floor: 1,
        surface_area: 90.0,
        unit_type: "Apartment",
        quota: 1000.0,
      },
      headers: entetes,
    });
    expect(lot.status(), await lot.text()).toBe(201);
    const unitId = (await lot.json()).id;

    // 2. Le rattachement — par `unit_owners`, PAS par `units.owner_id`.
    const detention = await page.request.post(
      `${API_BASE}/units/${unitId}/owners`,
      {
        data: {
          owner_id: ctx.ownerId,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: entetes,
      },
    );
    expect(
      detention.status(),
      `rattachement du propriétaire: ${await detention.text()}`,
    ).toBe(201);

    // 3. L'appel de fonds collectif.
    const maintenant = new Date();
    const echeance = new Date(maintenant.getTime() + 30 * 864e5);
    const appel = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        building_id: ctx.buildingId,
        title: `Charges Q3 ${Date.now()}`,
        description: "Non-régression F2",
        total_amount: 10000.0,
        contribution_type: "regular",
        call_date: maintenant.toISOString(),
        due_date: echeance.toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(appel.status(), await appel.text()).toBe(201);
    const appelId = (await appel.json()).id;

    // 4. L'envoi — c'est ici que le rapport butait sur « No active owners
    //    found for this building ».
    const envoi = await page.request.post(
      `${API_BASE}/call-for-funds/${appelId}/send`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(
      envoi.status(),
      `envoi de l'appel de fonds: ${await envoi.text()}`,
    ).toBe(200);

    const resultat = await envoi.json();
    expect(
      resultat.contributions_generated,
      "une quote-part par détention active",
    ).toBe(1);

    // 5. La quote-part existe, au bon montant : 100 % de 10 000 €.
    const quotes = await page.request.get(
      `${API_BASE}/owner-contributions?owner_id=${ctx.ownerId}`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(quotes.status()).toBe(200);
    const liste = await quotes.json();
    expect(liste.length).toBeGreaterThan(0);
    expect(Number(liste[0].amount)).toBe(10000);
    expect(liste[0].payment_status).toBe("pending");
  });

  // ───────────────────────────────────────────────────────────────────────
  // F14 — tantièmes : somme de `Decimal` sérialisés en chaîne
  // ───────────────────────────────────────────────────────────────────────

  test("F14 — le total des tantièmes s'affiche, et l'indicateur de conformité est fiable", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithUnit(page, "fin-f14");
    await page.goto(`/building-detail?id=${ctx.buildingId}`);

    // `data-testid` et non `text=/èmes/` : ce dernier attrapait le LIBELLÉ
    // « 📊 Total tantièmes: » avant la valeur.
    const total = page.getByTestId("quotas-total");
    await expect(total).toBeVisible({ timeout: 15000 });

    const texte = (await total.textContent()) ?? "";
    // Le symptôme exact du rapport.
    expect(texte, "le total ne doit jamais valoir NaN").not.toContain("NaN");
    // Un lot à 1000 sur un immeuble à 1000 : le total doit valoir 1000, pas 0
    // ni une concaténation.
    expect(texte).toMatch(/1000\s*\/\s*1000/);
  });

  // ───────────────────────────────────────────────────────────────────────
  // F5 — les boutons répondent (le rapport testait avec les dialogues rejetés)
  // ───────────────────────────────────────────────────────────────────────

  test("F5 — « Créer des relances automatiques » déclenche bien l'appel", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "fin-f5a");

    // SANS ceci, Playwright rejette le `confirm()` et rien ne part : c'est
    // très exactement le faux positif du rapport.
    let dialogueVu = false;
    page.on("dialog", async (d) => {
      dialogueVu = true;
      await d.accept();
    });

    const appel = page.waitForRequest(
      (r) =>
        r.url().includes("/payment-reminders/bulk-create") &&
        r.method() === "POST",
      { timeout: 15000 },
    );

    await page.goto("/payment-reminders");
    await page
      .getByRole("button", { name: /relances automatiques|automatic reminders/i })
      .first()
      .click();

    await appel;
    expect(dialogueVu, "le bouton est gardé par une confirmation").toBe(true);
  });

  test("F5 — « Nouveau budget » ouvre bien le formulaire", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "fin-f5b");
    await page.goto("/budgets");

    const bouton = page
      .getByRole("button", { name: /nouveau budget|new budget/i })
      .first();
    await expect(bouton).toBeVisible({ timeout: 15000 });
    await bouton.click();

    // Le formulaire n'a pas de conteneur porteur d'un `data-testid` : on vise
    // le premier champ qu'il rend, qui n'existe nulle part ailleurs.
    await expect(page.getByTestId("budget-building-select")).toBeVisible({
      timeout: 10000,
    });
    await expect(page.getByTestId("budget-fiscal-year")).toBeVisible();
  });

  // ───────────────────────────────────────────────────────────────────────
  // F6 — la page des écritures n'avait aucune vue liste
  // ───────────────────────────────────────────────────────────────────────

  test("F6 — /journal-entries affiche la liste des écritures, pas seulement le formulaire", async ({
    page,
  }) => {
    // Compte COMPTABLE et non admin : `superadmin` n'appartient à aucune
    // organisation et reçoit un 401 sur toute route scopée. C'est très
    // exactement la cause du constat F3, que le rapport attribuait à des noms
    // de paramètres erronés.
    await loginAsAccountantEmetteur(page, "fin-f6a");
    await page.goto("/journal-entries");

    const liste = page.getByTestId("journal-entry-list");
    await expect(liste, "la vue liste manquait entièrement").toBeVisible({
      timeout: 15000,
    });
    // Les filtres que l'API offrait déjà et que rien n'exposait.
    await expect(page.getByTestId("journal-type-filter")).toBeVisible();
    await expect(page.getByTestId("journal-start-filter")).toBeVisible();
  });

  test("F6 — une écriture créée est retrouvable dans la liste, avec ses lignes", async ({
    page,
  }) => {
    const ctx = await loginAsAccountantEmetteur(page, "fin-f6b");
    const reference = `NR-${Date.now()}`;

    const creation = await page.request.post(`${API_BASE}/journal-entries`, {
      data: {
        journal_type: "ODS",
        entry_date: new Date().toISOString(),
        description: `Non-regression F6 ${reference}`,
        document_ref: reference,
        lines: [
          { account_code: "611002", debit: 100.0, credit: 0.0, description: "Charge" },
          { account_code: "440", debit: 0.0, credit: 100.0, description: "Fournisseur" },
        ],
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(creation.status(), await creation.text()).toBe(201);

    await page.goto("/journal-entries");
    await expect(page.getByTestId("journal-entry-list")).toBeVisible({
      timeout: 15000,
    });

    const ligne = page
      .getByTestId("journal-entry-row")
      .filter({ hasText: reference });
    await expect(ligne, "l'écriture doit être retrouvable depuis l'UI").toBeVisible({
      timeout: 15000,
    });

    // Le détail : c'est ce que le rapport demandait explicitement.
    await ligne.getByTestId("toggle-lines-button").click();
    const lignes = page.getByTestId("journal-entry-lines");
    await expect(lignes).toBeVisible({ timeout: 10000 });
    await expect(lignes).toContainText("611002");
    await expect(lignes).toContainText("440");
  });

  test("F16 — POST /journal-entries refuse `operation_date` et `reference`", async ({
    page,
  }) => {
    const ctx = await loginAsAccountantEmetteur(page, "fin-f16");

    const resp = await page.request.post(`${API_BASE}/journal-entries`, {
      data: {
        journal_type: "ODS",
        // Les DEUX noms erronés cités par le rapport.
        operation_date: new Date().toISOString(),
        reference: "REF-KO",
        description: "Non-regression F16",
        lines: [
          { account_code: "611002", debit: 100.0, credit: 0.0, description: "x" },
          { account_code: "440", debit: 0.0, credit: 100.0, description: "y" },
        ],
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });

    expect(
      resp.status(),
      "des noms de champs erronés doivent produire un refus explicite",
    ).toBe(400);
  });

  // ───────────────────────────────────────────────────────────────────────
  // F3 — rapports comptables
  // ───────────────────────────────────────────────────────────────────────

  test("F3 — le bilan se génère sans erreur pour un compte scopé", async ({
    page,
  }) => {
    // Les rapports PCMN sont réservés aux comptables et superadmins (403
    // sinon). Le rapport du 2026-09-01 concluait « Erreur lors de la
    // génération » là où il s'agissait d'un refus d'autorisation.
    await loginAsAccountantEmetteur(page, "fin-f3");
    await page.goto("/reports");

    const requete = page.waitForResponse(
      (r) => r.url().includes("/reports/balance-sheet"),
      { timeout: 20000 },
    );
    await page
      .getByRole("button", { name: /générer|generate report/i })
      .first()
      .click();

    const reponse = await requete;
    // Le rapport annonçait « Erreur lors de la génération ». La cause réelle
    // était un 401 : le compte de test n'appartenait à aucune organisation.
    expect(reponse.status(), await reponse.text()).toBe(200);
  });

  // ───────────────────────────────────────────────────────────────────────
  // F8 / F11 / F15 — libellés
  // ───────────────────────────────────────────────────────────────────────

  test("F11 — le titre de la fiche budget porte son accent", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "fin-f11");
    await page.goto("/budget-detail?id=00000000-0000-0000-0000-000000000000");
    await expect(page).toHaveTitle(/Détail du Budget/);
  });

  test("F15 — /owner-contributions ne porte plus le titre de /call-for-funds", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "fin-f15");

    await page.goto("/call-for-funds");
    const titreAppels = await page.title();

    await page.goto("/owner-contributions");
    const titreContributions = await page.title();

    expect(
      titreContributions,
      "les deux pages portaient le même titre « Appels de fonds »",
    ).not.toBe(titreAppels);
    expect(titreContributions).toMatch(/Contributions/i);
  });

  // ───────────────────────────────────────────────────────────────────────
  // F19 / F20 — fiche dépense
  // ───────────────────────────────────────────────────────────────────────

  test("F19 — la ventilation par tantièmes est déclenchable depuis la fiche dépense", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithExpense(page, "fin-f19");
    await page.goto(`/expense-detail?id=${ctx.expenseId}`);

    const section = page.getByTestId("distributions-section");
    // La section n'était rendue QUE si des ventilations existaient déjà, et
    // rien dans l'interface ne permettait d'en créer : elle était donc
    // invisible en permanence.
    await expect(section, "la section doit exister même sans ventilation").toBeVisible({
      timeout: 15000,
    });
    await expect(page.getByTestId("calculate-distribution-button")).toBeVisible();
  });

  test("F20 — la fiche dépense montre la décomposition HT / TVA", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "fin-f20");

    // Une dépense AVEC détail TVA : 2000 HT + 21 % = 2420 TTC.
    const resp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Non-regression F20 ${Date.now()}`,
        amount: 2420.0,
        amount_excl_vat: 2000.0,
        vat_rate: 21.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(resp.status(), await resp.text()).toBe(201);
    const depense = await resp.json();

    await page.goto(`/expense-detail?id=${depense.id}`);

    const tva = page.getByTestId("vat-breakdown");
    await expect(tva, "seul le TTC était affiché").toBeVisible({ timeout: 15000 });
    await expect(page.getByTestId("vat-excl")).toContainText("2");
    await expect(page.getByTestId("vat-incl")).toContainText("2");
    // Le montant de TVA se déduit du HT et du TTC quand il n'est pas persisté.
    await expect(page.getByTestId("vat-amount")).toBeVisible();
  });
});
