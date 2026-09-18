/**
 * Playwright Global Setup — creates the TestWorld.
 *
 * Runs once before all tests. Seeds the database via API with:
 *   - 1 organization
 *   - 1 syndic user (with JWT token)
 *   - 1 owner user (with JWT token)
 *   - 1 building with 3 units
 *   - Owner assigned to unit 1A
 *   - 1 meeting (for governance tests)
 *
 * Like the BDD World pattern but for Playwright.
 */
import { request } from "@playwright/test";
import { ADMIN_PASSWORD } from "./helpers/identifiants";
import { saveWorld, type TestWorld } from "./helpers/test-world";

import { API_BASE } from "./helpers/adresses";

export default async function globalSetup() {
  const ts = Date.now();
  const ctx = await request.newContext();

  // 1. Login as admin
  const adminResp = await ctx.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
  });
  const admin = await adminResp.json();
  const adminToken = admin.token;
  const adminHeaders = { Authorization: `Bearer ${adminToken}` };

  // 2. Create organization
  const orgResp = await ctx.post(`${API_BASE}/organizations`, {
    data: {
      name: `E2E World ${ts}`,
      slug: `e2e-world-${ts}`,
      contact_email: `e2e-${ts}@koprogo.test`,
      subscription_plan: "professional",
    },
    headers: adminHeaders,
  });
  const org = await orgResp.json();

  // 3. Create syndic user
  const syndicEmail = `syndic-${ts}@koprogo.test`;
  const syndicPassword = "test123456";
  const syndicRegResp = await ctx.post(`${API_BASE}/auth/register`, {
    data: {
      email: syndicEmail,
      password: syndicPassword,
      first_name: "Syndic",
      last_name: `World${ts}`,
      role: "syndic",
      organization_id: org.id,
    },
  });
  const syndicData = await syndicRegResp.json();

  // 4. Create owner user
  const ownerEmail = `owner-${ts}@koprogo.test`;
  const ownerPassword = "test123456";
  const ownerRegResp = await ctx.post(`${API_BASE}/auth/register`, {
    data: {
      email: ownerEmail,
      password: ownerPassword,
      first_name: "Owner",
      last_name: `World${ts}`,
      role: "owner",
      organization_id: org.id,
    },
  });
  const ownerData = await ownerRegResp.json();

  // 5. Create ACP first (post-#602 — buildings.acp_id is FK to acps.id).
  //    Inline (not using ensureAcp helper) because globalSetup runs without
  //    a Page context — only an APIRequestContext is available.
  const acpResp = await ctx.post(`${API_BASE}/acps`, {
    data: {
      organization_id: org.id,
      name: `E2E ACP ${ts}`,
      address_street: `${ts} Rue Test`,
      address_postal_code: "1000",
      address_city: "Brussels",
    },
    headers: adminHeaders,
  });
  if (!acpResp.ok()) {
    throw new Error(
      `globalSetup: POST /acps failed ${acpResp.status()} : ${await acpResp.text()}`,
    );
  }
  const acp = await acpResp.json();

  // 6. Create building
  const buildingResp = await ctx.post(`${API_BASE}/buildings`, {
    data: {
      name: `Résidence E2E ${ts}`,
      address: `${ts} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 3,
      construction_year: 2010,
      acp_id: acp.id,
    },
    headers: adminHeaders,
  });
  const building = await buildingResp.json();

  // 6. Trois lots dont les quotités somment EXACTEMENT à l'acte de base.
  //
  // ── Le monde de scénario n'a jamais été conforme ─────────────────────────
  //
  // Les trois lots portaient `quota: 333.33`. Somme : 999,99 sur une base de
  // 1000, soit un `quota_delta` de 0,01. Le serveur refusait donc en 422
  // `ACP_NOT_CONFORMANT` toute dépense, toute répartition, tout appel de
  // fonds et tout état daté fondés sur ce monde (ADR-0010).
  //
  // Ça n'a jamais rougi, pour deux raisons qui se sont additionnées :
  // ce fichier n'était exécuté par personne (`globalSetup` n'était pas
  // déclaré, #955), et ses créations d'entités avalent leurs échecs en
  // `console.warn`. Au premier lancement réel, le 2026-09-18, six entités
  // sur quatorze manquaient à l'appel.
  //
  // Le dernier lot absorbe le reste : c'est la seule façon d'atteindre la
  // somme exacte quand la base ne se divise pas rondement. 1000 / 3 ne
  // tombe pas juste, et un acte de base ne tolère pas l'à-peu-près.
  const BASE_TANTIEMES = 1000;
  const REPARTITION = [
    ["1A", 1],
    ["2A", 2],
    ["3A", 3],
  ] as const;
  const quotaCourant =
    Math.floor((BASE_TANTIEMES / REPARTITION.length) * 100) / 100;

  const units: Array<{ id: string; unitNumber: string; floor: number }> = [];
  for (const [index, [num, floor]] of REPARTITION.entries()) {
    const dernier = index === REPARTITION.length - 1;
    const quota = dernier
      ? Number(
          (BASE_TANTIEMES - quotaCourant * (REPARTITION.length - 1)).toFixed(2),
        )
      : quotaCourant;

    const unitResp = await ctx.post(`${API_BASE}/units`, {
      data: {
        building_id: building.id,
        unit_number: num,
        unit_type: "Apartment",
        floor,
        surface_area: 75.0,
        quota,
      },
      headers: adminHeaders,
    });
    if (!unitResp.ok()) {
      // Un lot qui manque rend le monde non conforme, et tout ce qui suit
      // échouera en 422 sans qu'on sache pourquoi. Celui-ci ne s'avale pas.
      throw new Error(
        `global-setup : lot ${num} (quota ${quota}) refusé en ` +
          `HTTP ${unitResp.status()} — ${(await unitResp.text()).slice(0, 200)}`,
      );
    }
    const unit = await unitResp.json();
    units.push({ id: unit.id, unitNumber: num, floor });
  }

  // 7. La fiche de copropriétaire, LIÉE au compte, puis rattachée au lot 1A.
  //
  // ── Sans `user_id`, trois modules communautaires ne se sèment pas ────────
  //
  // La fiche était créée sans lien vers le compte. Le serveur ne trouvait
  // donc aucune fiche derrière l'utilisateur, et refusait en 403
  // `owner_profile_required` la création d'un échange, d'une compétence et
  // d'un objet partagé :
  //
  //   « Cette action est réservée aux copropriétaires : elle engage une
  //     personne, pas la copropriété. »
  //
  // Le refus est JUSTE — c'est le produit qui a raison. C'était le monde de
  // scénario qui prétendait avoir un copropriétaire sans lui en donner
  // l'identité.
  const ownerUserId =
    ownerData.user?.id ?? ownerData.id ?? ownerData.user_id ?? null;
  if (!ownerUserId) {
    throw new Error(
      "global-setup : impossible de lire l'identifiant du compte " +
        "copropriétaire. Sans lui, sa fiche reste non liée et les modules " +
        "communautaires ne se sèment pas.",
    );
  }

  const ownerRecordResp = await ctx.post(`${API_BASE}/owners`, {
    data: {
      organization_id: org.id,
      first_name: "Owner",
      last_name: `World${ts}`,
      email: ownerEmail,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      user_id: ownerUserId,
    },
    headers: { Authorization: `Bearer ${syndicData.token}` },
  });
  const ownerRecord = await ownerRecordResp.json();

  // Assign owner to unit 1A
  await ctx.post(`${API_BASE}/units/${units[0].id}/owners`, {
    data: {
      owner_id: ownerRecord.id,
      ownership_percentage: 1.0,
      is_primary_contact: true,
    },
    headers: { Authorization: `Bearer ${syndicData.token}` },
  });

  // 8. Create a meeting (for governance tests)
  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 30);
  const meetingResp = await ctx.post(`${API_BASE}/meetings`, {
    data: {
      building_id: building.id,
      organization_id: org.id,
      title: `AG E2E ${ts}`,
      scheduled_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      location: "Salle de réunion E2E",
    },
    headers: { Authorization: `Bearer ${syndicData.token}` },
  });
  if (!meetingResp.ok()) {
    console.error(
      "Meeting creation failed:",
      meetingResp.status(),
      await meetingResp.text(),
    );
  }
  const meeting = await meetingResp.json();

  // 9. Create pre-seeded entities for detail page tests
  const syndicHeaders = { Authorization: `Bearer ${syndicData.token}` };

  async function tryCreate(name: string, promise: Promise<any>): Promise<any> {
    const resp = await promise;
    if (!resp.ok()) {
      console.warn(
        `⚠️  ${name} creation failed (${resp.status()}): ${await resp.text()}`,
      );
      return null;
    }
    return resp.json();
  }

  // Owner-authenticated headers for community features
  const ownerHeaders = { Authorization: `Bearer ${ownerData.token}` };

  const ticket = await tryCreate(
    "ticket",
    ctx.post(`${API_BASE}/tickets`, {
      data: {
        building_id: building.id,
        title: `Fuite robinet ${ts}`,
        description: "Fuite 3ème étage",
        priority: "Medium",
        category: "Plumbing",
      },
      headers: syndicHeaders,
    }),
  );

  const expense = await tryCreate(
    "expense",
    ctx.post(`${API_BASE}/expenses`, {
      data: {
        building_id: building.id,
        category: "Maintenance",
        description: `Réparation ascenseur ${ts}`,
        amount: 1500.0,
        expense_date: new Date().toISOString(),
      },
      headers: syndicHeaders,
    }),
  );

  const budget = await tryCreate(
    "budget",
    ctx.post(`${API_BASE}/budgets`, {
      data: {
        building_id: building.id,
        organization_id: org.id,
        fiscal_year: 2026,
        total_budget_amount: 75000.0,
        ordinary_budget: 50000.0,
        extraordinary_budget: 25000.0,
      },
      headers: syndicHeaders,
    }),
  );

  const notice = await tryCreate(
    "notice",
    ctx.post(`${API_BASE}/notices`, {
      data: {
        building_id: building.id,
        title: `Travaux parking ${ts}`,
        content: "Parking fermé pour travaux.",
        notice_type: "Announcement",
        category: "Maintenance",
      },
      headers: syndicHeaders,
    }),
  );

  // Exchange/Skill/SharedObject need an owner-linked user

  const exchange = await tryCreate(
    "exchange",
    ctx.post(`${API_BASE}/exchanges`, {
      data: {
        building_id: building.id,
        provider_id: ownerRecord.id,
        exchange_type: "Service",
        title: `Cours cuisine ${ts}`,
        description: "Cours cuisine belge",
        credits: 2,
      },
      headers: ownerHeaders,
    }),
  );

  const skill = await tryCreate(
    "skill",
    ctx.post(`${API_BASE}/skills`, {
      data: {
        building_id: building.id,
        skill_category: "Technology",
        skill_name: `Dépannage PC ${ts}`,
        expertise_level: "Advanced",
        description: "Aide informatique",
        is_available_for_help: true,
      },
      headers: ownerHeaders,
    }),
  );

  const sharedObject = await tryCreate(
    "sharedObject",
    ctx.post(`${API_BASE}/shared-objects`, {
      data: {
        building_id: building.id,
        object_category: "Tools",
        object_name: `Perceuse ${ts}`,
        description: "Perceuse avec coffret",
        condition: "Good",
        is_available: true,
      },
      headers: ownerHeaders,
    }),
  );

  const pollEndDate = new Date();
  pollEndDate.setDate(pollEndDate.getDate() + 14);
  const poll = await tryCreate(
    "poll",
    ctx.post(`${API_BASE}/polls`, {
      data: {
        building_id: building.id,
        poll_type: "yes_no",
        title: `Repeindre hall ? ${ts}`,
        description: "Consultation avant AG",
        ends_at: pollEndDate.toISOString(),
        is_anonymous: false,
        allow_multiple_votes: false,
        require_all_owners: false,
        options: [
          { option_text: "Oui", display_order: 1 },
          { option_text: "Non", display_order: 2 },
        ],
      },
      headers: syndicHeaders,
    }),
  );

  const quote = await tryCreate(
    "quote",
    ctx.post(`${API_BASE}/quotes`, {
      data: {
        building_id: building.id,
        contractor_id: "00000000-0000-0000-0000-000000000000",
        project_title: `Rénovation façade ${ts}`,
        project_description: "Ravalement façade",
        amount_excl_vat: 250.0,
        // `vat_rate DECIMAL(5, 4)` : la colonne stocke une FRACTION, pas un
        // pourcentage — 21 % s'écrit 0,2100, et le commentaire de la
        // migration `20251120150000_create_quotes.sql:25` le dit. Envoyer
        // 21.0 dépassait la capacité de la colonne, et le devis ne se semait
        // pas : « numeric field overflow », un message qui ne nomme ni le
        // champ ni la convention.
        vat_rate: 0.21,
        estimated_duration_days: 30,
        warranty_years: 10,
        validity_date: new Date(Date.now() + 30 * 86400000).toISOString(),
      },
      headers: syndicHeaders,
    }),
  );

  const workReport = await tryCreate(
    "workReport",
    ctx.post(`${API_BASE}/work-reports`, {
      data: {
        building_id: building.id,
        organization_id: org.id,
        work_type: "repair",
        title: `Réparation toiture ${ts}`,
        description: "Tuiles cassées",
        contractor_name: "Toitures Dupont SPRL",
        work_date: new Date().toISOString(),
        start_date: new Date().toISOString(),
        cost: 3500.0,
        warranty_type: "decennial",
        warranty_years: 10,
      },
      headers: syndicHeaders,
    }),
  );

  const nextInspDate = new Date();
  nextInspDate.setFullYear(nextInspDate.getFullYear() + 1);
  const inspection = await tryCreate(
    "inspection",
    ctx.post(`${API_BASE}/technical-inspections`, {
      data: {
        building_id: building.id,
        organization_id: org.id,
        title: `Contrôle ascenseur ${ts}`,
        inspection_type: "elevator",
        inspector_name: "Bureau Véritas",
        inspection_date: new Date().toISOString(),
        next_inspection_date: nextInspDate.toISOString(),
      },
      headers: syndicHeaders,
    }),
  );

  const convocDate = new Date();
  convocDate.setDate(convocDate.getDate() + 30);
  const convocation = await tryCreate(
    "convocation",
    ctx.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meeting.id,
        building_id: building.id,
        organization_id: org.id,
        meeting_type: "Ordinary",
        meeting_date: convocDate.toISOString(),
        language: "fr",
      },
      headers: syndicHeaders,
    }),
  );

  const etatDate = await tryCreate(
    "etatDate",
    ctx.post(`${API_BASE}/etats-dates`, {
      data: {
        unit_id: units[0].id,
        building_id: building.id,
        organization_id: org.id,
        language: "fr",
        reference_date: new Date().toISOString(),
        notary_name: "Me Dupont",
        notary_email: "notaire@test.be",
      },
      headers: syndicHeaders,
    }),
  );

  let paymentReminder = null;
  if (expense) {
    paymentReminder = await tryCreate(
      "paymentReminder",
      ctx.post(`${API_BASE}/payment-reminders`, {
        data: {
          expense_id: expense.id,
          owner_id: ownerRecord.id,
          organization_id: org.id,
          level: "FirstReminder",
          amount_due: 1500.0,
          amount_owed: 1500.0,
          due_date: new Date(Date.now() + 30 * 86400000).toISOString(),
          days_overdue: 15,
        },
        headers: syndicHeaders,
      }),
    );
  }

  // Save world
  const world: TestWorld = {
    adminToken,
    orgId: org.id,
    orgName: org.name,
    syndic: {
      token: syndicData.token,
      email: syndicEmail,
      password: syndicPassword,
      userId: syndicData.user?.id || syndicData.id,
      firstName: "Syndic",
      lastName: `World${ts}`,
    },
    owner: {
      token: ownerData.token,
      email: ownerEmail,
      password: ownerPassword,
      userId: ownerData.user?.id || ownerData.id,
      ownerId: ownerRecord.id,
      firstName: "Owner",
      lastName: `World${ts}`,
    },
    building: {
      id: building.id,
      name: building.name,
      address: building.address,
      city: "Brussels",
    },
    units,
    meeting: {
      id: meeting.id,
      title: meeting.title,
    },
    ticket: ticket ? { id: ticket.id, title: ticket.title } : undefined,
    expense: expense
      ? { id: expense.id, description: expense.description }
      : undefined,
    budget: budget
      ? { id: budget.id, fiscalYear: budget.fiscal_year }
      : undefined,
    notice: notice ? { id: notice.id, title: notice.title } : undefined,
    exchange: exchange ? { id: exchange.id, title: exchange.title } : undefined,
    skill: skill ? { id: skill.id, skillName: skill.skill_name } : undefined,
    sharedObject: sharedObject
      ? { id: sharedObject.id, objectName: sharedObject.object_name }
      : undefined,
    poll: poll ? { id: poll.id, title: poll.title } : undefined,
    quote: quote
      ? { id: quote.id, projectTitle: quote.project_title }
      : undefined,
    workReport: workReport
      ? { id: workReport.id, title: workReport.title }
      : undefined,
    inspection: inspection
      ? { id: inspection.id, inspectorName: inspection.inspector_name }
      : undefined,
    convocation: convocation ? { id: convocation.id } : undefined,
    etatDate: etatDate ? { id: etatDate.id } : undefined,
    paymentReminder: paymentReminder ? { id: paymentReminder.id } : undefined,
    booking: undefined, // TODO: add when resource booking API is ready
    createdAt: new Date().toISOString(),
  };

  saveWorld(world);

  // Log created entities
  const entities = [
    ticket && "ticket",
    expense && "expense",
    budget && "budget",
    notice && "notice",
    exchange && "exchange",
    skill && "skill",
    sharedObject && "sharedObject",
    poll && "poll",
    quote && "quote",
    workReport && "workReport",
    inspection && "inspection",
    convocation && "convocation",
    etatDate && "etatDate",
    paymentReminder && "reminder",
  ].filter(Boolean);
  console.log(
    `✅ TestWorld created: org=${org.id}, building=${building.id}, ${units.length} units, meeting=${meeting.id}, +${entities.length} entities (${entities.join(", ")})`,
  );

  await ctx.dispose();
}
