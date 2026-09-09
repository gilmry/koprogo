import { test, expect, type Page } from "@playwright/test";
import { uiLoginWithRetry } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";
import { adminLogin, ensureAcp } from "./helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";
const TEST_PASSWORD = process.env.PLAYWRIGHT_TEST_PASSWORD || "test123456";

// /journal-entries est gated ACCOUNTANT-only (guards.ts) — un syndic est
// silencieusement redirigé vers /syndic sans jamais voir le formulaire.
// loginAsSyndicWithBuilding (utilisé partout ailleurs dans ce sweep) ne
// convient donc pas ici ; il faut un vrai compte accountant.
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

  const regResp = await page.request.post(`${API_BASE}/auth/register`, {
    data: {
      email,
      password: TEST_PASSWORD,
      first_name: "Accountant",
      last_name: `Test${timestamp}`,
      role: "accountant",
      organization_id: org.id,
    },
  });
  const userData = await regResp.json();

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

test.describe("Comptable — Écritures comptables, parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une écriture comptable équilibrée de bout en bout", async ({
    page,
  }) => {
    const { buildingId } = await loginAsAccountant(page, "journey-journal");
    await page.goto("/journal-entries", { waitUntil: "networkidle" });

    // L'immeuble est choisi dans la BARRE DE CONTEXTE, pas par l'URL.
    //
    // Le test naviguait vers `/journal-entries?buildingId=…`. Le paramètre
    // n'est pas lu : `JournalEntriesPanel` dérive son immeuble du périmètre
    // sélectionné (`buildingId ?? scope.selectedBuildingId`), et
    // `stores/scope.svelte.ts` annonce que « le rehydrate sur reload sera
    // porté par Story 2.5 (deep-links) ». C'est l'arbitrage #841, non tranché.
    //
    // L'instantané de page du run 34354176811 le montre : le comptable est
    // bien connecté, la page est bien celle des écritures, et elle affiche
    // « Sélectionnez un immeuble — choisissez-en un dans la barre de
    // contexte ». Le formulaire n'est donc pas monté, et `#description`
    // n'existe pas.
    //
    // On emprunte le chemin supporté aujourd'hui. À simplifier en un
    // deep-link le jour où #841 sera tranchée.
    const selecteur = page.getByTestId("building-selector-input");
    await expect(selecteur).toBeVisible({ timeout: 15_000 });
    await selecteur.click();
    await selecteur.fill("journey-journal");
    const resultat = page.getByTestId(`building-selector-result-${buildingId}`);
    await expect(resultat).toBeVisible({ timeout: 10_000 });
    await resultat.click();

    await expect(page.getByTestId("journal-entry-form")).toBeVisible({
      timeout: 15_000,
    });
    await page
      .getByTestId("journal-entry-description-input")
      .fill(`Facture eau ${Date.now()}`);

    // 604002 "Eau" / 440 "Fournisseurs" — comptes PCMN réels du seed belge
    // (get_belgian_pcmn_seed_data), contrairement au "6100" du placeholder
    // du formulaire qui n'existe dans aucun plan comptable seedé.
    await page.locator("#journal-line-0-account-code").fill("604002");
    await page.locator("#journal-line-0-debit").fill("100");
    await page.locator("#journal-line-1-account-code").fill("440");
    await page.locator("#journal-line-1-credit").fill("100");

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
});
