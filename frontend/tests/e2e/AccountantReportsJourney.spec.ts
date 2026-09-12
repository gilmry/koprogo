import { test, expect, type Page } from "@playwright/test";
import { uiLoginWithRetry } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";
import { adminLogin, ensureAcp } from "./helpers/auth";

import { API_BASE } from "./helpers/adresses";
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

  // Connexion RÉELLE, pas une injection dans `localStorage`.
  //
  // L'injection ci-dessous ne pouvait rien établir : `auth.ts:182` dit que
  // `koprogo_user` est « un cache d'affichage NON sensible, jamais une preuve
  // d'authentification », et qu'`init()` fait un silent-refresh via le cookie
  // HttpOnly pour confirmer la session. Le cache injecté était donc écrasé
  // par le VRAI utilisateur — l'admin, dont le cookie était encore posé.
  //
  // L'instantané de page du run 34347631686 le montre : « Admin System »,
  // « Bienvenue, Admin », tableau de bord administrateur. Le test n'était ni
  // comptable ni sur la page des écritures ; il attendait `#description` sur
  // un écran qui ne le porte pas.
  //
  // Le comptable est créé plus haut avec un mot de passe : on s'en sert. On
  // ne peut pas changer de rôle en modifiant `localStorage`, et c'est
  // exactement ce qu'on veut d'un produit.
  await uiLoginWithRetry(page, email, TEST_PASSWORD, /\/accountant/);
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
