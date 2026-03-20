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
import { saveWorld, type TestWorld } from "./helpers/test-world";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

export default async function globalSetup() {
  const ts = Date.now();
  const ctx = await request.newContext();

  // 1. Login as admin
  const adminResp = await ctx.post(`${API_BASE}/auth/login`, {
    data: { email: "admin@koprogo.com", password: "admin123" },
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

  // 5. Create building
  const buildingResp = await ctx.post(`${API_BASE}/buildings`, {
    data: {
      name: `Résidence E2E ${ts}`,
      address: `${ts} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 3,
      construction_year: 2010,
      organization_id: org.id,
    },
    headers: adminHeaders,
  });
  const building = await buildingResp.json();

  // 6. Create 3 units
  const units: Array<{ id: string; unitNumber: string; floor: number }> = [];
  for (const [num, floor] of [
    ["1A", 1],
    ["2A", 2],
    ["3A", 3],
  ] as const) {
    const unitResp = await ctx.post(`${API_BASE}/units`, {
      data: {
        organization_id: org.id,
        building_id: building.id,
        unit_number: num,
        unit_type: "Apartment",
        floor,
        surface_area: 75.0,
        quota: 333.33,
      },
      headers: adminHeaders,
    });
    const unit = await unitResp.json();
    units.push({ id: unit.id, unitNumber: num, floor });
  }

  // 7. Create an owner record and assign to unit 1A
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
      headers: syndicHeaders,
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
      headers: syndicHeaders,
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
      headers: syndicHeaders,
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
        vat_rate: 21.0,
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
