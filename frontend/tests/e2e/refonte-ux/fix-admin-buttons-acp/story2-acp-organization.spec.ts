/**
 * Story 2 (#698) — docs/maury/fix-admin-buttons-acp/stories.md
 *
 * `BuildingForm.svelte` envoyait `organization_id` (champ qui n'existe plus
 * côté backend) au lieu de `acp_id` (requis) — création/édition d'immeuble
 * impossible. `AcpList.svelte` exigeait un UUID collé à la main pour lier
 * une ACP à son cabinet syndic. Bonus (même root cause, trouvé en
 * préparant cette story) : `UnitCreateModal.svelte` envoyait aussi
 * `organization_id` à `POST /units`.
 */
import { test, expect } from "@playwright/test";
import { loginAsAdmin } from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

/**
 * `GET /buildings` trie par défaut `created_at ASC` (le plus ancien en
 * premier) avec `per_page=20` — sur cet environnement de dev chargé
 * d'immeubles de test accumulés, un immeuble fraîchement créé atterrit sur
 * une page tardive, invisible dans la liste par défaut. Force `per_page`
 * large côté requête réseau pour rendre le test déterministe, sans changer
 * le code produit.
 */
/**
 * Force une page de liste assez grande pour contenir l'immeuble que le test
 * vient de creer.
 *
 * Necessaire parce que la recherche de `BuildingList.svelte` est CLIENTE :
 * `buildings.filter(...)` (l. 94) ne filtre que la page deja chargee, jamais
 * la base. Un immeuble hors de la page courante est donc introuvable par la
 * recherche — limitation produit reelle des qu'une ACP depasse une page.
 *
 * `per_page=500` ne suffisait plus : la base cible en comptait 609, le nouvel
 * immeuble tombait hors page et `toHaveCount(1)` voyait 0. Le test passait en
 * isolation et echouait en campagne, ce qui rendait le symptome trompeur.
 *
 * Un nombre fixe reste par nature fragile : il repousse le seuil, il ne le
 * supprime pas. Le seul correctif de fond est une recherche cote serveur.
 */
async function forceLargePageSize(
  page: import("@playwright/test").Page,
): Promise<void> {
  await page.route("**/buildings?*", (route) => {
    const url = new URL(route.request().url());
    url.searchParams.set("per_page", "10000");
    route.continue({ url: url.toString() });
  });
}

async function createOrgAndAcp(
  page: import("@playwright/test").Page,
  adminToken: string,
  prefix: string,
): Promise<{ orgId: string; acpId: string; orgName: string }> {
  const ts = Date.now();
  const orgName = `${prefix} Org ${ts}`;
  const orgResp = await page.request.post(`${API_BASE}/organizations`, {
    data: {
      name: orgName,
      slug: `${prefix}-${ts}`,
      contact_email: `${prefix}-${ts}@test.com`,
      subscription_plan: "professional",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const org = await orgResp.json();

  const acpResp = await page.request.post(`${API_BASE}/acps`, {
    data: {
      organization_id: org.id,
      name: `${prefix} ACP ${ts}`,
      address_street: "1 Rue Test",
      address_postal_code: "1000",
      address_city: "Bruxelles",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const acp = await acpResp.json();

  return { orgId: org.id, acpId: acp.id, orgName };
}

test.describe("Story 2 (#698) — ACP au lieu d'Organisation", () => {
  /**
   * Le point d'entrée, pas le formulaire.
   *
   * Tous les autres tests de ce fichier atteignent la page par
   * `page.goto("/admin/acps")`. Ils prouvent donc que la création FONCTIONNE,
   * et masquent en même temps qu'elle était INATTEIGNABLE : la page, livrée
   * par 7d9aab08 (« ACPs invisibles »), n'était liée depuis nulle part — ni
   * dans `getAdminItems()`, ni depuis le tableau de bord admin. Un admin ne
   * pouvait y arriver qu'en tapant l'URL à la main, et `noAcpAvailable` le
   * renvoyait vers un « Administration > ACP » qui n'existait pas.
   *
   * Ce test navigue donc par le MENU. C'est le seul de ce fichier capable
   * d'attraper la disparition du lien ; par construction, les autres ne le
   * peuvent pas.
   */
  test("@happy la gestion des ACP est atteignable depuis le menu admin", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.goto("/dashboard");

    // Déplier d'abord la section « Administration ».
    //
    // `RoleSubmenu.svelte` range ses liens dans un `<details>` dont l'ouverture
    // est dérivée de `defaultOpen || containsActive` — et aucun appelant ne
    // passe `defaultOpen`. Depuis /dashboard, la section admin ne contient pas
    // la page courante : elle est donc REPLIÉE, et ses liens sont présents dans
    // le DOM mais INVISIBLES. C'est exactement la cause des 11 échecs
    // `[scenarios]` corrigée en #730 ; le geste utilisateur est de cliquer le
    // `<summary>` avant le lien.
    const adminMenu = page.getByTestId("navigation-menu-admin");
    await expect(adminMenu).toBeAttached({ timeout: 15_000 });
    await adminMenu.locator("summary").first().click();

    // Ciblage par `href` et non par `data-testid` : `RoleSubmenu.svelte`
    // génère `nav-link-{slugify(item.label)}` à partir du libellé TRADUIT, donc
    // `nav-link-acp` en fr, `nav-link-acps` en en, `nav-link-vme-s` en nl. Son
    // propre en-tête documente pourtant un `stableSlug`, et la config
    // Playwright force `fr-BE` en admettant « so nav testids match hardcoded
    // expectations ». Le `href`, lui, ne dépend d'aucune locale.
    const navLink = page.locator('nav a[href="/admin/acps"]').first();
    await expect(navLink).toBeVisible({ timeout: 15_000 });
    await navLink.click();

    await expect(page).toHaveURL(/\/admin\/acps/);
    await expect(page.getByTestId("admin-acps-page")).toBeVisible({
      timeout: 15_000,
    });
    // Le geste de création doit être présent en arrivant par ce chemin.
    await expect(page.getByTestId("acp-create-toggle")).toBeVisible();
  });

  /**
   * Second point d'entrée : la carte du tableau de bord admin.
   */
  test("@happy la carte du tableau de bord admin mène aux ACP", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.goto("/admin");

    const card = page.getByTestId("admin-quick-action-acps");
    await expect(card).toBeVisible({ timeout: 15_000 });
    await card.click();

    await expect(page.getByTestId("admin-acps-page")).toBeVisible({
      timeout: 15_000,
    });
  });

  test("@happy création immeuble avec ACP sélectionnée — payload correct", async ({
    page,
  }) => {
    const { token } = await loginAsAdmin(page);
    const { acpId } = await createOrgAndAcp(page, token, "s2happy1");

    await forceLargePageSize(page);
    await page.goto("/buildings");
    const createBtn = page.getByTestId("create-building-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await createBtn.click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    const ts = Date.now();
    await dialog.getByTestId("building-acp-select").selectOption(acpId);
    await dialog.getByTestId("building-name-input").fill(`S2 Building ${ts}`);
    await dialog.getByTestId("building-address-input").fill("42 Rue Test");
    await dialog.getByTestId("building-postalcode-input").fill("1000");
    await dialog.getByTestId("building-city-input").fill("Bruxelles");
    await dialog.getByTestId("building-totalunits-input").fill("5");
    await dialog.getByTestId("building-totaltantiemes-input").fill("1000");

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/buildings") && r.request().method() === "POST",
      ),
      dialog.getByTestId("building-submit-button").click(),
    ]);
    expect(createResp.status()).toBe(201);
    const created = await createResp.json();
    expect(created.acp_id).toBe(acpId);
    expect(created).not.toHaveProperty("organization_id");

    await expect(dialog).not.toBeVisible();
    await expect(page.getByText(`S2 Building ${ts}`)).toBeVisible({
      timeout: 15_000,
    });
  });

  test("@happy (bis) création ACP avec organisation sélectionnée — payload correct", async ({
    page,
  }) => {
    const { token } = await loginAsAdmin(page);
    const ts = Date.now();
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `s2happy2 Org ${ts}`,
        slug: `s2happy2-${ts}`,
        contact_email: `s2happy2-${ts}@test.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const org = await orgResp.json();

    await page.goto("/admin/acps");
    const toggle = page.getByTestId("acp-create-toggle");
    await expect(toggle).toBeVisible({ timeout: 15_000 });
    await toggle.click();

    const form = page.getByTestId("acp-create-form");
    await expect(form).toBeVisible();
    await form.getByTestId("acp-form-name").fill(`S2 ACP ${ts}`);
    await form.getByTestId("acp-form-org-id").selectOption(org.id);
    await form.getByTestId("acp-form-street").fill("1 Rue Test");
    await form.getByTestId("acp-form-postal").fill("1000");
    await form.getByTestId("acp-form-city").fill("Bruxelles");

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/acps") && r.request().method() === "POST",
      ),
      form.getByTestId("acp-form-submit").click(),
    ]);
    expect(createResp.status()).toBe(201);
    const created = await createResp.json();
    expect(created.organization_id).toBe(org.id);

    await expect(page.getByText(`S2 ACP ${ts}`)).toBeVisible({
      timeout: 15_000,
    });
    await expect(page.getByText(`s2happy2 Org ${ts}`)).toBeVisible();
  });

  test("@negative submit immeuble sans ACP sélectionnée — erreur cliente, pas de requête", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.goto("/buildings");
    const createBtn = page.getByTestId("create-building-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await createBtn.click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    await dialog
      .getByTestId("building-name-input")
      .fill(`S2 Negative ${Date.now()}`);
    await dialog.getByTestId("building-address-input").fill("1 Rue Test");
    await dialog.getByTestId("building-postalcode-input").fill("1000");
    await dialog.getByTestId("building-city-input").fill("Bruxelles");
    await dialog.getByTestId("building-totalunits-input").fill("5");
    await dialog.getByTestId("building-totaltantiemes-input").fill("1000");

    let postFired = false;
    page.on("request", (r) => {
      if (r.url().includes("/buildings") && r.method() === "POST")
        postFired = true;
    });

    await dialog.getByTestId("building-submit-button").click();
    await page.waitForTimeout(500);

    expect(postFired).toBe(false);
    await expect(dialog.getByText(/acp est requise/i)).toBeVisible();
  });

  test("@edge édition d'un immeuble existant — dropdown ACP pré-rempli", async ({
    page,
  }) => {
    const { token } = await loginAsAdmin(page);
    const { acpId } = await createOrgAndAcp(page, token, "s2edge");
    const ts = Date.now();
    const buildingName = `S2 Edit Building ${ts}`;
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: buildingName,
        address: "1 Rue Test",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 3,
        total_tantiemes: 1000,
        construction_year: 2020,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(buildingResp.status()).toBe(201);

    await forceLargePageSize(page);
    await page.goto("/buildings");
    await expect(page.getByTestId("building-search-input")).toBeVisible({
      timeout: 15_000,
    });
    await page.getByTestId("building-search-input").fill(buildingName);
    await expect(page.getByTestId("building-card")).toHaveCount(1);

    await page.getByTestId("edit-building-button").click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    await expect(dialog.getByTestId("building-acp-select")).toHaveValue(acpId);
  });

  test("@edge (bis) aucune ACP disponible — état vide explicite, submit bloqué", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await page.route("**/api/v1/acps", (route) => {
      if (route.request().method() === "GET") {
        route.fulfill({
          status: 200,
          contentType: "application/json",
          body: "[]",
        });
      } else {
        route.continue();
      }
    });

    await page.goto("/buildings");
    const createBtn = page.getByTestId("create-building-button");
    await expect(createBtn).toBeVisible({ timeout: 15_000 });
    await createBtn.click();

    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText(/aucune acp disponible/i)).toBeVisible({
      timeout: 15_000,
    });

    await dialog
      .getByTestId("building-name-input")
      .fill(`S2 Empty ${Date.now()}`);
    await dialog.getByTestId("building-address-input").fill("1 Rue Test");
    await dialog.getByTestId("building-postalcode-input").fill("1000");
    await dialog.getByTestId("building-city-input").fill("Bruxelles");
    await dialog.getByTestId("building-totalunits-input").fill("5");
    await dialog.getByTestId("building-totaltantiemes-input").fill("1000");
    await dialog.getByTestId("building-submit-button").click();

    await expect(dialog.getByText(/acp est requise/i)).toBeVisible();
  });

  test("@security création immeuble reste superadmin-only (aucune nouvelle exposition)", async ({
    page,
  }) => {
    const { token: adminToken } = await loginAsAdmin(page);
    const ts = Date.now();
    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `s2sec Org ${ts}`,
        slug: `s2sec-${ts}`,
        contact_email: `s2sec-${ts}@test.com`,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();
    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: `s2sec-syndic-${ts}@test.com`,
        password: "test123456",
        first_name: "S2Sec",
        last_name: "Syndic",
        role: "syndic",
        organization_id: org.id,
      },
    });
    const syndic = await regResp.json();

    // acp_id doit être présent (même une valeur bidon) pour que le JSON se
    // désérialise et que le handler atteigne réellement le check de rôle —
    // sinon Actix rejette en 400 (champ requis manquant) avant toute logique
    // métier, ce qui ne teste pas l'autorisation.
    const resp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        acp_id: "00000000-0000-0000-0000-000000000000",
        name: "Should not be created",
        address: "1 Rue Test",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 1,
        total_tantiemes: 1000,
        construction_year: 2020,
      },
      headers: { Authorization: `Bearer ${syndic.token}` },
    });
    expect(resp.status()).toBe(403);
  });

  test("@happy (bonus) UnitCreateModal envoie acp_id — création de lot fonctionnelle", async ({
    page,
  }) => {
    const { token } = await loginAsAdmin(page);
    const { acpId } = await createOrgAndAcp(page, token, "s2unit");
    const buildingResp = await page.request.post(`${API_BASE}/buildings`, {
      data: {
        name: `S2 Unit Building ${Date.now()}`,
        address: "1 Rue Test",
        city: "Bruxelles",
        postal_code: "1000",
        country: "Belgium",
        total_units: 5,
        total_tantiemes: 1000,
        construction_year: 2020,
        acp_id: acpId,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const building = await buildingResp.json();

    await page.goto(`/building-detail?id=${building.id}`);
    const addUnitBtn = page.getByRole("button", { name: /ajouter un lot/i });
    await expect(addUnitBtn).toBeVisible({ timeout: 15_000 });
    await addUnitBtn.click();

    await page.getByLabel(/numéro de lot|unit number/i).fill("1A");
    await page.getByLabel(/type de lot|unit type/i).selectOption("Apartment");
    await page.getByLabel(/étage|floor/i).fill("1");
    await page.getByLabel(/surface/i).fill("50");
    await page
      .getByLabel(/tantièmes|quota/i)
      .first()
      .fill("200");

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/units") && r.request().method() === "POST",
      ),
      page.getByRole("button", { name: /créer le lot|create unit/i }).click(),
    ]);
    expect(createResp.status()).toBe(201);
    // UnitResponseDto n'expose pas acp_id (stocké mais non renvoyé) — la
    // preuve du fix est dans le payload envoyé, pas dans la réponse.
    const sentPayload = createResp.request().postDataJSON();
    expect(sentPayload.acp_id).toBe(acpId);
    expect(sentPayload).not.toHaveProperty("organization_id");
  });
});
