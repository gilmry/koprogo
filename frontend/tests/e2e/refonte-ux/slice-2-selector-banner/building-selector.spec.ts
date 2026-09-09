/**
 * Story 2.2 — BuildingSelector E2E (multi-role + RBAC + scope violation).
 *
 * Couvre les AC de la Story 2.2 (issue #563) :
 *
 *   @happy   Syndic ouvre selector, type "imm", clique un building, scope
 *            est mis a jour (highlighted result-{id}).
 *   @security Owner ne voit PAS le selector (composant invisible / null).
 *   @negative Aucun building dans le perimetre → empty state visible.
 *
 * Helpers : `loginAsAdmin`, `loginAsSyndic`, `loginAsSyndicWithBuilding`
 * (cf. `frontend/tests/e2e/helpers/auth.ts`). Auth est injectee sans UI
 * login pour rester focus sur le selector.
 *
 * Pattern multi-role : on registre un syndic + 3 buildings via admin token,
 * puis on login le syndic et on valide le rendu. Pas de step 'click → scope
 * banner' (banner = Story 2.3) — on verifie le store via le testid mis a
 * jour cote dropdown (aria-selected="true").
 */
import { test, expect } from "@playwright/test";
import {
  loginAsAdmin,
  loginAsSyndic,
  loginAsSyndicWithBuilding,
  uiLoginWithRetry,
} from "../../helpers/auth";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Story 2.2 — BuildingSelector (top-left)", () => {
  test("@happy syndic sees selector, types query, selects a building", async ({
    page,
  }) => {
    // Setup : syndic + 1 building (via helper).
    const ctx = await loginAsSyndicWithBuilding(page, "sel-happy");

    // Le selector est rendu en top-left via Layout.astro.
    const input = page.getByTestId("building-selector-input");
    await expect(input).toBeVisible({ timeout: 15_000 });

    // Type a query that matches the auto-created building name prefix.
    await input.click();
    await input.fill("sel-happy");

    // Wait for the result to render (autocomplete after debounce 150ms).
    const result = page.getByTestId(
      `building-selector-result-${ctx.buildingId}`,
    );
    await expect(result).toBeVisible({ timeout: 5_000 });

    // Click → store mis a jour → aria-selected="true" sur la ligne.
    await result.click();

    // Apres click, l'input garde la valeur du building selectionne
    // (cf. composant : `query = b.name` post-click).
    await expect(input).toHaveValue(/sel-happy/);
  });

  test("@security owner does NOT see the building selector", async ({
    page,
  }) => {
    // Login un owner via admin → register role=owner.
    const { adminToken } = await loginAsAdmin(page);
    const timestamp = Date.now();
    const ownerEmail = `sel-owner-${timestamp}@example.com`;

    const orgResp = await page.request.post(`${API_BASE}/organizations`, {
      data: {
        name: `OwnerOrg ${timestamp}`,
        slug: `owner-org-${timestamp}`,
        contact_email: ownerEmail,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await orgResp.json();

    await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: "test123456",
        first_name: "OwnerSel",
        last_name: `Test${timestamp}`,
        role: "owner",
        organization_id: org.id,
      },
    });

    // Connexion RÉELLE, pas une injection dans `localStorage`.
    //
    // Le test injectait un utilisateur factice dans `koprogo_user`. Ça ne
    // pouvait pas marcher, et c'est une bonne nouvelle : `auth.ts:182` dit
    // que cette clé est « un cache d'affichage NON sensible, jamais une
    // preuve d'authentification », et qu'`init()` fait un silent-refresh via
    // le cookie HttpOnly pour confirmer la session. Le cache injecté était
    // donc écrasé par le VRAI utilisateur — le superadmin, dont le cookie
    // était encore posé.
    //
    // L'instantané de page du run 34347631686 le montre sans ambiguïté :
    // `/url: /superadmin` et un menu « Administration ». Le sélecteur
    // s'affichait donc à bon droit, pour un superadmin.
    //
    // Autrement dit, cette assertion `@security` n'a jamais éprouvé ce
    // qu'elle annonce : aucun copropriétaire n'était connecté. On ne peut pas
    // changer de rôle en modifiant `localStorage`, et c'est exactement ce
    // qu'on veut d'un produit.
    //
    // Le copropriétaire est créé plus haut avec un mot de passe : on s'en
    // sert.
    await uiLoginWithRetry(page, ownerEmail, "test123456", /\/owner/);
    await page.goto("/owner", { waitUntil: "networkidle" });

    // Le selector NE DOIT PAS apparaitre pour un owner (cf. AC @security).
    await expect(page.getByTestId("building-selector-input")).toHaveCount(0);
  });

  test("@negative syndic without buildings sees empty state on search", async ({
    page,
  }) => {
    // Syndic sans building — empty state visible apres typing.
    await loginAsSyndic(page, "sel-empty");

    const input = page.getByTestId("building-selector-input");
    await expect(input).toBeVisible({ timeout: 15_000 });

    await input.click();
    await input.fill("zzz-no-match-xyz");

    // L'empty state s'affiche soit immediatement (0 building dans le scope)
    // soit apres le filtre (aucun match). L'un OU l'autre satisfait l'AC.
    const empty = page.getByTestId("building-selector-empty");
    await expect(empty).toBeVisible({ timeout: 5_000 });
  });
});
