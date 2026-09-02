import { test, expect } from "@playwright/test";
import {
  loginAsAccountantEmetteur,
  loginAsSyndicWithBuilding,
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
    // `loginAsSyndicWithUnit` et non `loginAsSyndicWithOwner` : le second passe
    // par la configuration par défaut (12 lots attendus), et l'appel de fonds
    // est refusé sur un immeuble en dérive par rapport à son acte de base
    // (Art. 3.85 CC, gate `validate-before-compute`). Le premier produit un
    // immeuble conforme : 1 lot, 1000 millièmes.
    const ctx = await loginAsSyndicWithUnit(page, "fin-f2");
    const entetes = { Authorization: `Bearer ${ctx.adminToken}` };
    const unitId = ctx.unitId;

    // 1. Un copropriétaire à rattacher au lot.
    const prop = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: ctx.orgId,
        first_name: "Jean",
        last_name: `Peeters${Date.now()}`,
        email: `f2-${Date.now()}@example.com`,
        address: "1 Rue Test",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(prop.status(), await prop.text()).toBe(201);
    const ownerId = (await prop.json()).id;

    // 2. Le rattachement — par `unit_owners`, PAS par `units.owner_id`.
    const detention = await page.request.post(
      `${API_BASE}/units/${unitId}/owners`,
      {
        data: {
          owner_id: ownerId,
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
    // Corps `{}` obligatoire : le handler prend un `web::Json<...>`, même si
    // la structure est vide. Sans lui, la requête est rejetée en 400
    // « Content type error » — ce que le nouveau gestionnaire d'erreurs JSON
    // dit désormais explicitement, au lieu d'un corps `text/plain`.
    const envoi = await page.request.post(
      `${API_BASE}/call-for-funds/${appelId}/send`,
      { data: {}, headers: { Authorization: `Bearer ${ctx.token}` } },
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
      `${API_BASE}/owner-contributions?owner_id=${ownerId}`,
      { headers: { Authorization: `Bearer ${ctx.token}` } },
    );
    expect(quotes.status()).toBe(200);
    const liste = await quotes.json();
    expect(liste.length).toBeGreaterThan(0);
    expect(Number(liste[0].amount)).toBe(10000);
    expect(liste[0].payment_status).toBe("pending");
  });

  // ───────────────────────────────────────────────────────────────────────
  // A1/A2 — répartition selon les tantièmes, sur un immeuble à PLUSIEURS lots
  // ───────────────────────────────────────────────────────────────────────

  /// Le test précédent (F2) porte sur un immeuble à UN lot détenu à 100 % —
  /// le seul cas où le défaut est invisible, puisque 100 % du lot vaut aussi
  /// 100 % de l'immeuble. Celui-ci utilise quatre lots aux tantièmes
  /// distincts, ce qui rend la clé de répartition observable.
  ///
  /// Mesuré en production le 2026-09-02, sur un immeuble CONFORME
  /// (4 lots / 4, 1000 millièmes / 1000) :
  ///
  ///   - `POST /call-for-funds/{id}/send` de 10 000 € générait QUATRE
  ///     quotes-parts de 10 000 €, soit **40 000 € appelés**, chacune
  ///     étiquetée « Quote-part: 100 % » — et répondait 200 ;
  ///   - `POST /invoices/{id}/calculate-distribution` refusait toute
  ///     répartition avec « Total quota percentage exceeds 100%
  ///     (got: 400.00000) » — 400 % = 4 lots × 100 %.
  ///
  /// Cause commune : `find_active_by_building` renvoyait le pourcentage de
  /// détention BRUT (1.0 par propriétaire unique), utilisé tel quel comme
  /// quote-part du montant total. La formule légale (Art. 3.84)
  /// `(quota / total_tantiemes) × ownership_percentage` existait dans
  /// `ChargeDistribution::resolve_owner_quota`, testée, et n'avait aucun
  /// appelant en production.
  test("A1/A2 — charges et appels de fonds suivent les tantièmes sur 4 lots", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "regr-repartition", {
      totalUnits: 4,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const admin = { Authorization: `Bearer ${ctx.adminToken}` };
    const syndic = { Authorization: `Bearer ${ctx.token}` };

    const TANTIEMES = [200, 200, 300, 300];
    for (let i = 0; i < TANTIEMES.length; i++) {
      const lot = await page.request.post(`${API_BASE}/units`, {
        data: {
          acp_id: ctx.acpId,
          building_id: ctx.buildingId,
          unit_number: `T${i + 1}`,
          floor: i,
          surface_area: 50 + i * 10,
          unit_type: "Apartment",
          quota: TANTIEMES[i],
        },
        headers: admin,
      });
      expect(lot.status(), `lot ${i + 1}: ${await lot.text()}`).toBe(201);
      const unitId = (await lot.json()).id;

      const prop = await page.request.post(`${API_BASE}/owners`, {
        data: {
          organization_id: ctx.orgId,
          first_name: `Copro${i + 1}`,
          last_name: `Repartition${Date.now()}`,
          email: `repartition-${i}-${Date.now()}@example.com`,
          address: `${i + 1} Rue Test`,
          city: "Bruxelles",
          postal_code: "1000",
          country: "Belgium",
        },
        headers: syndic,
      });
      expect(prop.status(), await prop.text()).toBe(201);
      const ownerId = (await prop.json()).id;

      const det = await page.request.post(`${API_BASE}/units/${unitId}/owners`, {
        data: {
          owner_id: ownerId,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: admin,
      });
      expect(det.status(), `détention ${i + 1}: ${await det.text()}`).toBe(201);
    }

    // Prérequis : sans conformité, la répartition est refusée en amont
    // (Art. 3.85 CC) et le test ne prouverait rien.
    const bat = await page.request.get(
      `${API_BASE}/buildings/${ctx.buildingId}`,
      { headers: syndic },
    );
    const b = await bat.json();
    expect(b.is_conformant, `immeuble non conforme: ${JSON.stringify(b)}`).toBe(
      true,
    );

    // ── Répartition d'une charge de 2 420 € TTC ─────────────────────────
    const dep = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Ascenseur ${Date.now()}`,
        amount: 2420.0,
        amount_excl_vat: 2000.0,
        vat_rate: 21.0,
        expense_date: new Date().toISOString(),
        account_code: "611002",
        supplier: "Kone SA",
      },
      headers: syndic,
    });
    expect(dep.status(), await dep.text()).toBe(201);
    const depenseId = (await dep.json()).id;

    await page.request.put(`${API_BASE}/invoices/${depenseId}/submit`, {
      data: { submitted_by_user_id: ctx.userId },
      headers: syndic,
    });
    await page.request.put(`${API_BASE}/invoices/${depenseId}/approve`, {
      data: { approved_by_user_id: ctx.userId },
      headers: syndic,
    });

    const rep = await page.request.post(
      `${API_BASE}/invoices/${depenseId}/calculate-distribution`,
      { data: {}, headers: syndic },
    );
    expect(rep.status(), `répartition: ${await rep.text()}`).toBeLessThan(300);

    const brut = await rep.json();
    const lignes = Array.isArray(brut) ? brut : (brut.distributions ?? []);
    expect(lignes.length, "une ligne par lot").toBe(4);

    // 2420 × 200/1000 = 484,00   |   2420 × 300/1000 = 726,00
    const dus = lignes
      .map((l: any) => Number(l.amount_due))
      .sort((a: number, b: number) => a - b);
    expect(dus).toEqual([484, 484, 726, 726]);

    const sommeRepartie = dus.reduce((a: number, b: number) => a + b, 0);
    expect(
      sommeRepartie,
      "la somme répartie doit égaler la charge, ni plus ni moins",
    ).toBeCloseTo(2420, 2);

    // ── Appel de fonds de 10 000 € ──────────────────────────────────────
    const maintenant = new Date();
    const appel = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        building_id: ctx.buildingId,
        title: `Charges Q3 ${Date.now()}`,
        description: "Non-régression répartition",
        total_amount: 10000.0,
        contribution_type: "regular",
        call_date: maintenant.toISOString(),
        due_date: new Date(maintenant.getTime() + 30 * 864e5).toISOString(),
      },
      headers: syndic,
    });
    expect(appel.status(), await appel.text()).toBe(201);
    const appelId = (await appel.json()).id;

    const envoi = await page.request.post(
      `${API_BASE}/call-for-funds/${appelId}/send`,
      { data: {}, headers: syndic },
    );
    expect(envoi.status(), `envoi: ${await envoi.text()}`).toBe(200);
    expect((await envoi.json()).contributions_generated).toBe(4);

    const quotes = await page.request.get(`${API_BASE}/owner-contributions`, {
      headers: syndic,
    });
    const liste = await quotes.json();
    const montants = liste
      .map((q: any) => Number(q.amount))
      .sort((a: number, b: number) => a - b);
    expect(montants).toEqual([2000, 2000, 3000, 3000]);

    const sommeAppelee = montants.reduce((a: number, b: number) => a + b, 0);
    expect(
      sommeAppelee,
      "avant correction, un appel de 10 000 € en appelait 40 000",
    ).toBe(10000);
  });

  /// A3 — un lot en INDIVISION ne doit pas bloquer la répartition.
  ///
  /// `charge_distributions` portait `UNIQUE (expense_id, unit_id)`. Un lot
  /// détenu par deux copropriétaires — un couple, une succession, un
  /// démembrement — produit deux lignes pour ce lot, que la contrainte
  /// refusait. L'insertion étant groupée, l'échec ne portait pas sur la seule
  /// ligne fautive : AUCUNE répartition n'était enregistrée pour l'immeuble.
  ///
  /// Mesuré en production le 2026-09-02 :
  ///   400 — duplicate key value violates unique constraint "unique_expense_unit"
  ///
  /// L'indivision est le cas ORDINAIRE en copropriété belge, pas un cas limite.
  /// Le domaine la gérait déjà (`resolve_owner_quota` multiplie par
  /// `ownership_percentage`, testé sur « 50/50 ») ; c'est le schéma qui
  /// l'interdisait.
  test("A3 — indivision 50/50 : la répartition passe et les montants tiennent", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "regr-indivis", {
      totalUnits: 2,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const admin = { Authorization: `Bearer ${ctx.adminToken}` };
    const syndic = { Authorization: `Bearer ${ctx.token}` };

    const creerProprio = async (nom: string) => {
      const r = await page.request.post(`${API_BASE}/owners`, {
        data: {
          organization_id: ctx.orgId,
          first_name: nom,
          last_name: `Indivis${Date.now()}`,
          email: `${nom.toLowerCase()}-${Date.now()}@example.com`,
          address: "1 Rue Test",
          city: "Bruxelles",
          postal_code: "1000",
          country: "Belgium",
        },
        headers: syndic,
      });
      expect(r.status(), await r.text()).toBe(201);
      return (await r.json()).id;
    };

    const creerLot = async (numero: string, quota: number) => {
      const r = await page.request.post(`${API_BASE}/units`, {
        data: {
          acp_id: ctx.acpId,
          building_id: ctx.buildingId,
          unit_number: numero,
          floor: 0,
          surface_area: 70,
          unit_type: "Apartment",
          quota,
        },
        headers: admin,
      });
      expect(r.status(), await r.text()).toBe(201);
      return (await r.json()).id;
    };

    // Lot A : 400 millièmes, en indivision 50/50 (un couple).
    // Lot B : 600 millièmes, propriétaire unique.
    const lotA = await creerLot("IA", 400);
    const lotB = await creerLot("IB", 600);
    const epoux1 = await creerProprio("Jean");
    const epouse = await creerProprio("Anne");
    const seul = await creerProprio("Philippe");

    for (const [lot, owner, pct, principal] of [
      [lotA, epoux1, 0.5, true],
      [lotA, epouse, 0.5, false],
      [lotB, seul, 1.0, true],
    ] as const) {
      const d = await page.request.post(`${API_BASE}/units/${lot}/owners`, {
        data: {
          owner_id: owner,
          ownership_percentage: pct,
          is_primary_contact: principal,
        },
        headers: admin,
      });
      expect(d.status(), `détention ${pct}: ${await d.text()}`).toBe(201);
    }

    const dep = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Indivision ${Date.now()}`,
        amount: 1000.0,
        expense_date: new Date().toISOString(),
        account_code: "611002",
      },
      headers: syndic,
    });
    expect(dep.status(), await dep.text()).toBe(201);
    const depenseId = (await dep.json()).id;

    await page.request.put(`${API_BASE}/invoices/${depenseId}/submit`, {
      data: { submitted_by_user_id: ctx.userId },
      headers: syndic,
    });
    await page.request.put(`${API_BASE}/invoices/${depenseId}/approve`, {
      data: { approved_by_user_id: ctx.userId },
      headers: syndic,
    });

    const rep = await page.request.post(
      `${API_BASE}/invoices/${depenseId}/calculate-distribution`,
      { data: {}, headers: syndic },
    );
    expect(rep.status(), `répartition: ${await rep.text()}`).toBeLessThan(300);

    const brut = await rep.json();
    const lignes = Array.isArray(brut) ? brut : (brut.distributions ?? []);
    expect(lignes.length, "trois lignes : deux indivisaires + un seul").toBe(3);

    // 400‰ × 50 % = 200 € chacun ; 600‰ × 100 % = 600 €.
    const dus = lignes
      .map((l: any) => Number(l.amount_due))
      .sort((a: number, b: number) => a - b);
    expect(dus).toEqual([200, 200, 600]);
    expect(
      dus.reduce((a: number, b: number) => a + b, 0),
      "la somme répartie doit égaler la charge",
    ).toBeCloseTo(1000, 2);
  });

  /// A5 — l'écart budgétaire ne doit ni planter, ni contredire le grand livre.
  ///
  /// Deux défauts sur la même requête, mesurés en production le 2026-09-02.
  ///
  /// 1. `SELECT category` lisait une colonne de type énuméré PostgreSQL
  ///    (`expense_category`) en `String`, via `row.get()` — qui PANIQUE sur
  ///    une erreur de décodage au lieu de la remonter. Le worker actix
  ///    mourait et l'appelant recevait un 502.
  ///
  ///    Le défaut restait invisible tant qu'aucune dépense ne correspondait au
  ///    filtre : sans ligne, la boucle ne s'exécute pas. Il suffisait d'une
  ///    seule dépense pour faire tomber l'écran de suivi budgétaire.
  ///
  /// 2. Le filtre portait sur `payment_status = 'paid'` — le décaissé — alors
  ///    que le grand livre comptabilise la charge à l'APPROBATION. Les deux
  ///    rapports se contredisaient sur le même engagement : grand livre
  ///    2 420 € de charges, suivi budgétaire 0 consommé.
  test("A5 — écart budgétaire : pas de 502, et accord avec le grand livre", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "regr-budget", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const admin = { Authorization: `Bearer ${ctx.adminToken}` };
    const syndic = { Authorization: `Bearer ${ctx.token}` };
    const ts = Date.now();

    const lot = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: ctx.acpId,
        building_id: ctx.buildingId,
        unit_number: "BG1",
        floor: 0,
        surface_area: 80,
        unit_type: "Apartment",
        quota: 1000,
      },
      headers: admin,
    });
    expect(lot.status(), await lot.text()).toBe(201);

    const annee = new Date().getFullYear();
    const bud = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: ctx.buildingId,
        fiscal_year: annee,
        ordinary_budget: 40000.0,
        extraordinary_budget: 8000.0,
      },
      headers: syndic,
    });
    expect(bud.status(), await bud.text()).toBe(201);
    const budget = await bud.json();
    expect(Number(budget.total_budget)).toBe(48000);
    // 48 000 / 12 mois.
    expect(Number(budget.monthly_provision_amount)).toBe(4000);

    // Une charge approuvée de 2 420 €.
    const dep = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: ctx.buildingId,
        category: "Maintenance",
        description: `Budget ${ts}`,
        amount: 2420.0,
        expense_date: new Date().toISOString(),
        account_code: "611002",
      },
      headers: syndic,
    });
    expect(dep.status(), await dep.text()).toBe(201);
    const depenseId = (await dep.json()).id;

    await page.request.put(`${API_BASE}/invoices/${depenseId}/submit`, {
      data: { submitted_by_user_id: ctx.userId },
      headers: syndic,
    });
    await page.request.put(`${API_BASE}/invoices/${depenseId}/approve`, {
      data: { approved_by_user_id: ctx.userId },
      headers: syndic,
    });

    // Le point qui produisait un 502 : une ligne existe désormais.
    const ecart = await page.request.get(
      `${API_BASE}/budgets/${budget.id}/variance`,
      { headers: syndic },
    );
    expect(
      ecart.status(),
      `l'écart ne doit pas faire tomber le worker: ${await ecart.text()}`,
    ).toBe(200);

    const v = await ecart.json();
    expect(
      Number(v.actual_total),
      "une facture approuvée consomme le budget, qu'elle soit payée ou non",
    ).toBe(2420);
    expect(Number(v.variance_total)).toBe(48000 - 2420);

    // Et le grand livre doit dire la même chose.
    const reg = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: `budget-regr-${ts}@example.com`,
        password: process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456",
        first_name: "Marie",
        last_name: `Compta${ts}`,
        role: "accountant",
        organization_id: ctx.orgId,
      },
    });
    expect(reg.status(), await reg.text()).toBe(201);
    const compta = { Authorization: `Bearer ${(await reg.json()).token}` };

    const debut = new Date(annee, 0, 1).toISOString();
    const fin = new Date(annee, 11, 31).toISOString();
    const cr = await page.request.get(
      `${API_BASE}/reports/income-statement?period_start=${debut}&period_end=${fin}`,
      { headers: compta },
    );
    expect(cr.status()).toBe(200);
    const r = await cr.json();
    expect(
      Number(r.total_expenses),
      "suivi budgétaire et grand livre doivent s'accorder sur le même engagement",
    ).toBe(Number(v.actual_total));
  });

  // ───────────────────────────────────────────────────────────────────────
  // A6 — garde-fous du cycle de vie d'une facture
  // ───────────────────────────────────────────────────────────────────────

  /// Audité contre la production le 2026-09-02 : les huit transitions
  /// interdites sont TOUTES refusées, avec un message juste. Rien à corriger.
  ///
  /// Elles sont verrouillées ici parce qu'elles gardent des mouvements
  /// d'argent : approuver un brouillon, payer sans approbation ou modifier une
  /// facture approuvée sont des contournements du contrôle interne, pas des
  /// détails d'ergonomie. Un audit qui constate « c'est bon » sans laisser de
  /// preuve durable ne protège rien.
  test("A6 — les transitions interdites du cycle de facture sont refusées", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "regr-wf", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const admin = { Authorization: `Bearer ${ctx.adminToken}` };
    const syndic = { Authorization: `Bearer ${ctx.token}` };

    const lot = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: ctx.acpId,
        building_id: ctx.buildingId,
        unit_number: "WF1",
        floor: 0,
        surface_area: 80,
        unit_type: "Apartment",
        quota: 1000,
      },
      headers: admin,
    });
    expect(lot.status(), await lot.text()).toBe(201);

    const creerFacture = async (categorie = "Maintenance") => {
      const r = await page.request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: ctx.buildingId,
          category: categorie,
          description: `WF ${Date.now()}-${Math.random()}`,
          amount: 1000.0,
          expense_date: new Date().toISOString(),
          account_code: "611002",
        },
        headers: syndic,
      });
      expect(r.status(), await r.text()).toBe(201);
      return (await r.json()).id;
    };
    const soumettre = (id: string) =>
      page.request.put(`${API_BASE}/invoices/${id}/submit`, {
        data: { submitted_by_user_id: ctx.userId },
        headers: syndic,
      });
    const approuver = (id: string) =>
      page.request.put(`${API_BASE}/invoices/${id}/approve`, {
        data: { approved_by_user_id: ctx.userId },
        headers: syndic,
      });

    // 1. Approuver un brouillon sans l'avoir soumis.
    const brouillon = await creerFacture();
    const r1 = await approuver(brouillon);
    expect(r1.status()).toBe(400);
    expect(await r1.text()).toContain("must be submitted first");

    // 2. Payer une facture non approuvée — le contournement le plus coûteux.
    const r2 = await page.request.put(
      `${API_BASE}/expenses/${brouillon}/mark-paid`,
      { data: {}, headers: syndic },
    );
    expect(r2.status()).toBe(400);
    expect(await r2.text()).toContain("must be approved first");

    // 3. Le cycle nominal, puis une seconde approbation.
    const facture = await creerFacture();
    expect((await soumettre(facture)).status()).toBe(200);
    expect((await approuver(facture)).status()).toBe(200);
    const r3 = await approuver(facture);
    expect(r3.status()).toBe(400);
    expect(await r3.text()).toContain("already approved");

    // 4. Modifier une facture approuvée.
    const r4 = await page.request.put(`${API_BASE}/invoices/${facture}`, {
      data: { description: "Modifiée après approbation" },
      headers: syndic,
    });
    expect(r4.status()).toBe(400);
    expect(await r4.text()).toContain("cannot be modified");

    // 5. Rejet sans motif : un refus doit être motivé.
    const rejetee = await creerFacture();
    await soumettre(rejetee);
    const r5 = await page.request.put(`${API_BASE}/invoices/${rejetee}/reject`, {
      data: { rejected_by_user_id: ctx.userId, rejection_reason: "   " },
      headers: syndic,
    });
    expect(r5.status()).toBe(400);
    expect(await r5.text()).toContain("reason cannot be empty");

    // 6. Rejet motivé, puis approbation d'une facture rejetée.
    const r6 = await page.request.put(`${API_BASE}/invoices/${rejetee}/reject`, {
      data: {
        rejected_by_user_id: ctx.userId,
        rejection_reason: "Montant erroné",
      },
      headers: syndic,
    });
    expect(r6.status()).toBe(200);
    const r7 = await approuver(rejetee);
    expect(r7.status()).toBe(400);
    expect(await r7.text()).toContain("resubmit first");

    // 7. Mais la re-soumission après rejet reste possible.
    expect((await soumettre(rejetee)).status()).toBe(200);

    // 8. Une dépense « Travaux » sans rapport de prestataire validé
    //    (chaîne de bon de commande, issue #309).
    const travaux = await creerFacture("Works");
    await soumettre(travaux);
    const r8 = await approuver(travaux);
    expect(r8.status()).toBe(400);
    expect(await r8.text()).toContain("contractor report");
  });

  // ───────────────────────────────────────────────────────────────────────
  // A7 — cloisonnement entre cabinets syndics, côté ÉCRITURE
  // ───────────────────────────────────────────────────────────────────────

  /// Une ACP est une entité à part entière : sa comptabilité, ses lots et ses
  /// mutations lui appartiennent. Un syndic n'en est que le mandataire, et ce
  /// mandat change au gré des assemblées générales.
  ///
  /// Les LECTURES étaient correctement cloisonnées (403 dans toutes les
  /// directions, aucune fuite dans les listes — Hotfix #603). Les ÉCRITURES ne
  /// l'étaient pas : 29 routes acceptent un `building_id` dans leur corps sans
  /// vérifier qu'il relève d'une ACP gérée par l'appelant.
  ///
  /// Mesuré le 2026-09-02 entre deux cabinets indépendants. Le cabinet B a pu,
  /// sur l'immeuble du cabinet A :
  ///
  ///   POST /expenses         → 201, dépense de 50 000 € VISIBLE dans la
  ///                            liste des charges de l'immeuble de A ;
  ///   POST /call-for-funds   → 201, puis `send` → 200, générant une
  ///                            quote-part de 25 000 € réclamée à une
  ///                            copropriétaire de A.
  ///
  /// La garde existante protégeait le mauvais champ : écraser
  /// `organization_id` avec celui du JWT empêche d'ESTAMPILLER un
  /// enregistrement au nom d'autrui, pas de le RATTACHER au patrimoine
  /// d'autrui.
  test("A7 — un cabinet ne peut pas écrire dans le patrimoine d'un autre", async ({
    page,
  }) => {
    const ts = Date.now();

    const a = await loginAsSyndicWithBuilding(page, "iso-a", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const adminA = { Authorization: `Bearer ${a.adminToken}` };
    const synA = { Authorization: `Bearer ${a.token}` };

    const lot = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: a.acpId,
        building_id: a.buildingId,
        unit_number: "ISO1",
        floor: 0,
        surface_area: 80,
        unit_type: "Apartment",
        quota: 1000,
      },
      headers: adminA,
    });
    expect(lot.status(), await lot.text()).toBe(201);
    const unitA = (await lot.json()).id;

    const prop = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: a.orgId,
        first_name: "Alice",
        last_name: `Iso${ts}`,
        email: `iso-${ts}@example.com`,
        address: "1 Rue A",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: synA,
    });
    const ownerA = (await prop.json()).id;
    await page.request.post(`${API_BASE}/units/${unitA}/owners`, {
      data: {
        owner_id: ownerA,
        ownership_percentage: 1.0,
        is_primary_contact: true,
      },
      headers: adminA,
    });

    // Cabinet B, totalement indépendant.
    const b = await loginAsSyndicWithBuilding(page, "iso-b", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const synB = { Authorization: `Bearer ${b.token}` };
    expect(b.orgId).not.toBe(a.orgId);

    // ── Écritures sur le patrimoine de A, depuis B ────────────────────
    const intrusionDepense = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: a.buildingId,
        category: "Maintenance",
        description: `INTRUSION ${ts}`,
        amount: 50000.0,
        expense_date: new Date().toISOString(),
        account_code: "611002",
      },
      headers: synB,
    });
    expect(
      intrusionDepense.status(),
      "un cabinet tiers ne doit pas pouvoir imputer une charge",
    ).toBe(403);

    const intrusionAppel = await page.request.post(`${API_BASE}/call-for-funds`, {
      data: {
        building_id: a.buildingId,
        title: `INTRUSION ${ts}`,
        description: "x",
        total_amount: 25000.0,
        contribution_type: "regular",
        call_date: new Date().toISOString(),
        due_date: new Date(Date.now() + 30 * 864e5).toISOString(),
      },
      headers: synB,
    });
    expect(
      intrusionAppel.status(),
      "un cabinet tiers ne doit pas pouvoir appeler des fonds",
    ).toBe(403);

    const intrusionBudget = await page.request.post(`${API_BASE}/budgets`, {
      data: {
        building_id: a.buildingId,
        fiscal_year: new Date().getFullYear(),
        ordinary_budget: 1.0,
        extraordinary_budget: 0.0,
      },
      headers: synB,
    });
    expect(intrusionBudget.status(), "ni voter un budget").toBe(403);

    const intrusionEtat = await page.request.post(`${API_BASE}/etats-dates`, {
      data: {
        building_id: a.buildingId,
        unit_id: unitA,
        reference_date: new Date().toISOString(),
        language: "fr",
        notary_name: "Intrus",
        notary_email: `intrus-${ts}@example.com`,
      },
      headers: synB,
    });
    expect(intrusionEtat.status(), "ni délivrer un état daté").toBe(403);

    // ── Le patrimoine de A doit être intact ───────────────────────────
    const charges = await page.request.get(
      `${API_BASE}/buildings/${a.buildingId}/expenses`,
      { headers: synA },
    );
    expect(await charges.text()).not.toContain(`INTRUSION ${ts}`);

    const quotes = await page.request.get(
      `${API_BASE}/owner-contributions?owner_id=${ownerA}`,
      { headers: synA },
    );
    const liste = await quotes.json();
    expect(
      Array.isArray(liste) ? liste.length : 0,
      "aucune quote-part ne doit avoir été réclamée à la copropriétaire de A",
    ).toBe(0);

    // ── Et les lectures restent cloisonnées ───────────────────────────
    for (const url of [
      `${API_BASE}/buildings/${a.buildingId}`,
      `${API_BASE}/units/${unitA}`,
      `${API_BASE}/owners/${ownerA}`,
      `${API_BASE}/buildings/${a.buildingId}/expenses`,
    ]) {
      const r = await page.request.get(url, { headers: synB });
      expect(r.status(), `lecture cloisonnée: ${url}`).toBe(403);
    }
  });

  // ───────────────────────────────────────────────────────────────────────
  // A8 — passation d'ACP : l'entrant hérite, le sortant perd l'accès
  // ───────────────────────────────────────────────────────────────────────

  /// Une ACP est une entité juridique : sa comptabilité lui appartient et la
  /// suit quand l'assemblée générale change de syndic.
  ///
  /// Mesuré le 2026-09-02, AVANT correction, sur une passation réelle :
  ///
  ///                    charges  écritures  budgets  quotes-parts  proprios
  ///   avant, sortant       1        1         1          1           1
  ///   après, ENTRANT       1        0         0          0           0
  ///   après, SORTANT     403        1         1          1           1
  ///
  /// Deux défauts symétriques. L'entrant héritait d'une copropriété amputée —
  /// alors que l'Art. 3.94 § 1er lui impose de transmettre à un notaire les
  /// décomptes des deux derniers exercices. Et le sortant, mandat révoqué,
  /// gardait accès aux noms, adresses et ARRIÉRÉS des copropriétaires : un
  /// traitement de données personnelles sans base légale.
  ///
  /// Cause unique : les lectures filtraient sur `organization_id`, l'estampille
  /// du cabinet ayant SAISI l'enregistrement. Le mandat change, l'estampille
  /// non. La portée se lit désormais « cette charge relève-t-elle d'une ACP
  /// dont ce cabinet a la gestion ? ».
  ///
  /// Ce test ne couvre encore que les CHARGES : c'est la première entité
  /// migrée. Budgets, écritures et copropriétaires suivront le même patron.
  test("A8 — une ACP qui change de syndic emporte sa comptabilité", async ({
    page,
  }) => {
    const ts = Date.now();

    const sortant = await loginAsSyndicWithBuilding(page, "passa-s", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const adminH = { Authorization: `Bearer ${sortant.adminToken}` };
    const synS = { Authorization: `Bearer ${sortant.token}` };

    const lot = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: sortant.acpId,
        building_id: sortant.buildingId,
        unit_number: "PA1",
        floor: 0,
        surface_area: 80,
        unit_type: "Apartment",
        quota: 1000,
      },
      headers: adminH,
    });
    expect(lot.status(), await lot.text()).toBe(201);

    const marqueur = `Historique ${ts}`;
    const dep = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: sortant.buildingId,
        category: "Maintenance",
        description: marqueur,
        amount: 2420.0,
        expense_date: new Date().toISOString(),
        account_code: "611002",
      },
      headers: synS,
    });
    expect(dep.status(), await dep.text()).toBe(201);
    // La charge porte l'ACP à laquelle elle appartient, pas seulement le
    // cabinet qui l'a saisie.
    expect((await dep.json()).acp_id).toBe(sortant.acpId);

    const chargesVisibles = async (entetes: any) => {
      const r = await page.request.get(
        `${API_BASE}/expenses?page=1&per_page=200`,
        { headers: entetes },
      );
      if (!r.ok()) return { http: r.status(), trouve: false };
      return { http: 200, trouve: (await r.text()).includes(marqueur) };
    };

    expect(
      (await chargesVisibles(synS)).trouve,
      "prérequis : le syndic en fonction voit la charge",
    ).toBe(true);

    // ── L'assemblée générale nomme un nouveau syndic ──────────────────
    const entrant = await loginAsSyndicWithBuilding(page, "passa-e", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    const synE = { Authorization: `Bearer ${entrant.token}` };

    const acp = await (
      await page.request.get(`${API_BASE}/acps/${sortant.acpId}`, {
        headers: adminH,
      })
    ).json();

    const passation = await page.request.put(
      `${API_BASE}/acps/${sortant.acpId}`,
      {
        data: {
          organization_id: entrant.orgId,
          name: acp.name,
          address_street: acp.address_street,
          address_postal_code: acp.address_postal_code,
          address_city: acp.address_city,
          address_country: acp.address_country ?? "Belgium",
          total_tantiemes: acp.total_tantiemes,
          legal_status: acp.legal_status,
        },
        headers: adminH,
      },
    );
    expect(passation.status(), await passation.text()).toBe(200);

    // ── Les deux moitiés du problème ──────────────────────────────────
    const chezEntrant = await chargesVisibles(synE);
    expect(
      chezEntrant.trouve,
      "le syndic entrant hérite de la comptabilité de l'ACP qu'il reprend",
    ).toBe(true);

    const chezSortant = await chargesVisibles(synS);
    expect(
      chezSortant.trouve,
      "le syndic sortant n'a plus de base légale pour consulter cette ACP",
    ).toBe(false);
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
