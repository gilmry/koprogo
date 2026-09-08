import { test, expect, type Page } from "@playwright/test";
import { failOnPageErrors } from "./helpers/pageErrors";
import { adminLogin, ensureAcp } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// /reports est gated ACCOUNTANT-only (guards.ts, même schéma que
// /journal-entries) — un syndic y est silencieusement redirigé.
async function loginAsAccountant(page: Page, prefix: string) {
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
  const org = await orgResp.json();

  await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Accountant",
      last_name: `Test${timestamp}`,
      role: "accountant",
      organization_id: org.id,
    },
  });

  await page.addInitScript(
    (value) => {
      try {
        localStorage.setItem("koprogo_user", value);
      } catch {
        /* ignore */
      }
    },
    JSON.stringify({
      id: "injected-user",
      email,
      first_name: "Accountant",
      last_name: `Test${timestamp}`,
      role: "accountant",
      roles: [
        {
          id: "injected-role-1",
          role: "accountant",
          organization_id: null,
          is_primary: true,
        },
      ],
      active_role: {
        id: "injected-role-1",
        role: "accountant",
        organization_id: null,
        is_primary: true,
      },
    }),
  );
  await page.goto("/accountant", { waitUntil: "networkidle" });

  // Un immeuble, et son identifiant rendu à l'appelant.
  //
  // Les écrans comptables lisent le périmètre d'immeuble, et le frontend est
  // une application Astro MULTI-PAGE : le `$state` de module du store repart à
  // zéro à chaque navigation. Naviguer directement vers `/journal-entries`
  // donnait donc toujours un périmètre nul, et l'écran affichait à juste titre
  // « sélectionnez un immeuble » — le formulaire n'existait pas.
  //
  // Ce n'était pas seulement un défaut de fixture : un utilisateur qui ouvre un
  // lien faisait exactement la même chose. Le store réhydrate désormais depuis
  // `?buildingId=`, en faisant VALIDER l'immeuble par le serveur (#841).
  const acpId = await ensureAcp(page, org.id, adminToken, prefix);
  const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
    data: {
      name: `${prefix} Building ${timestamp}`,
      address: `${timestamp} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 4,
      total_tantiemes: 1000,
      construction_year: 2010,
      acp_id: acpId,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await buildingResp.json();
  return {
    buildingId: building.id as string,
    orgId: org.id as string,
    adminToken,
  };
}

test.describe("Comptable — Rapports PCMN, parcours rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("génère le bilan comptable de bout en bout", async ({ page }) => {
    const { buildingId } = await loginAsAccountant(page, "journey-reports");
    await page.goto(`/reports?buildingId=${buildingId}`, {
      waitUntil: "networkidle",
    });

    const [resp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/reports/balance-sheet") &&
          r.request().method() === "GET",
      ),
      // Ancré, et non formulé. Cette assertion cherchait le bouton par son
      // libellé FRANÇAIS, sur un composant traduit en quatre langues : dès
      // que la langue résolue n'est pas le français, le bouton est
      // introuvable, le clic ne part jamais, et c'est `waitForResponse` qui
      // expire — un symptôme qui ne dit rien de la cause. Cf. #832.
      page.getByTestId("financial-reports-generate").click(),
    ]);
    expect(resp.status()).toBe(200);

    await expect(page.getByText("Bilan").first()).toBeVisible();
  });
});
