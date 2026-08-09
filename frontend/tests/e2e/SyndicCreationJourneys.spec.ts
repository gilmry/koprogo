import { test, expect, type Page } from "@playwright/test";
import {
  loginAsSyndicWithBuilding,
  loginAsSyndicWithUnit,
} from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

async function seedOwner(
  page: Page,
  token: string,
  buildingId: string,
  prefix: string,
) {
  const ts = Date.now();
  const resp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      first_name: prefix,
      last_name: `Owner${ts}`,
      email: `${prefix}-${ts}@test.com`,
      address: "1 Rue Test",
      city: "Bruxelles",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  return resp.json();
}

test.describe("Syndic — parcours de création remplis jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("owner-contributions: crée une contribution de bout en bout", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "journey-contrib");
    await seedOwner(page, ctx.token, ctx.buildingId, "journey-contrib");

    await page.goto("/owner-contributions", { waitUntil: "networkidle" });
    await page
      .getByTestId("contribution-owner-select")
      .selectOption({ index: 1 });
    await page
      .getByTestId("contribution-description")
      .fill("Test contribution E2E");
    await page.getByTestId("contribution-amount").fill("150.50");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/owner-contributions") &&
          r.request().method() === "POST",
      ),
      page.getByTestId("contribution-submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("budgets: crée un budget de bout en bout", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "journey-budget");
    await page.goto("/budgets", { waitUntil: "networkidle" });
    await page.getByTestId("create-budget-button").click();
    await page.getByTestId("budget-building-select").selectOption({ index: 1 });
    await page.getByTestId("budget-fiscal-year").fill("2027");
    await page.getByTestId("budget-ordinary-amount").fill("12000");
    await page.getByTestId("budget-extraordinary-amount").fill("2000");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/budgets") && r.request().method() === "POST",
      ),
      page.getByTestId("budget-submit-button").click(),
    ]);
    expect(resp.status()).toBe(201);
  });

  test("etats-dates: génère un état daté de bout en bout", async ({ page }) => {
    // Un état daté porte sur un lot avec un propriétaire actif — il faut
    // lier explicitement le owner créé au unit via POST /units/{id}/owners
    // (sinon 400 "Unit has no active owners", règle métier légitime).
    const ctx = await loginAsSyndicWithUnit(page, "journey-etat");
    const owner = await seedOwner(
      page,
      ctx.token,
      ctx.buildingId,
      "journey-etat",
    );
    const linkResp = await page.request.post(
      `${API_BASE}/units/${ctx.unitId}/owners`,
      {
        data: {
          owner_id: owner.id,
          ownership_percentage: 1,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${ctx.token}` },
      },
    );
    expect(linkResp.status()).toBe(201);

    await page.goto("/etats-dates", { waitUntil: "networkidle" });
    await page.getByRole("button", { name: /nouvel état des dates/i }).click();
    await page.locator("#building").selectOption({ index: 1 });
    await page.waitForTimeout(500); // laisse le temps au select unit de se peupler
    await page.locator("#unit").selectOption({ index: 1 });
    await page.locator("#reference-date").fill("2027-01-15");
    await page.locator("#notary-name").fill("Maître Test");
    await page.locator("#notary-email").fill("notaire-test@example.com");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/etats-dates") && r.request().method() === "POST",
      ),
      page.getByTestId("etat-date-generate-button").click(),
    ]);
    expect([200, 201]).toContain(resp.status());
  });

  test("syndic/board-members: élit un membre de bout en bout", async ({
    page,
  }) => {
    // Conseil de copropriété requis uniquement > 20 lots (règle métier belge) —
    // le défaut du helper (12 lots) déclenche un 400 légitime, pas un bug.
    const ctx = await loginAsSyndicWithBuilding(page, "journey-board", {
      totalUnits: 24,
      totalTantiemes: 1000,
    });
    await seedOwner(page, ctx.token, ctx.buildingId, "journey-board");
    const meetingTs = Date.now();
    const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
      data: {
        building_id: ctx.buildingId,
        organization_id: ctx.orgId,
        title: `AG Journey ${meetingTs}`,
        scheduled_date: new Date(Date.now() + 30 * 86400000).toISOString(),
        meeting_type: "Ordinary",
        location: "Salle test",
        is_second_convocation: true,
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(meetingResp.status()).toBe(201);

    await page.goto("/syndic/board-members", { waitUntil: "networkidle" });
    await page.getByRole("button", { name: /élire un membre/i }).click();
    await page.locator("#board-elect-owner").selectOption({ index: 1 });
    await page.locator("#board-elect-meeting").selectOption({ index: 1 });
    // Mandat ~1 an (Art. 3.90 CC belge — durée exacte vérifiée côté backend).
    await page.locator("#board-elect-mandate-start").fill("2027-01-01");
    await page.locator("#board-elect-mandate-end").fill("2027-12-31");

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/board-members") && r.request().method() === "POST",
      ),
      page
        .getByRole("button", { name: /^élire$|^confirmer$|^valider$/i })
        .click(),
    ]);
    expect(resp.status()).toBe(201);
  });
});
