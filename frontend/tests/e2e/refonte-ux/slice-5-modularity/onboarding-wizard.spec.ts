/**
 * Story 5.7 — OnboardingWizard E2E (slice 5 / refonte-ux).
 *
 * Couverture 4 catégories (sur des `test` distincts) :
 *   @happy    superadmin parcourt les 5 étapes → ACP créée → modules
 *             activés → confirmation avec temps écoulé < 5 min.
 *   @edge     l'utilisateur saute la recommandation → seuls les modules par
 *             défaut (community + identity) sont activés.
 *   @security un rôle non-superadmin (syndic) qui tape directement l'URL
 *             `/admin/acps/onboarding` est redirigé avant le montage — la
 *             garde vit dans `roleGuards["/admin/*"]`
 *             (frontend/src/lib/guards.ts) + `RouteGuard.svelte`, pas dans le
 *             composant. Aucun `data-testid` de l'assistant ne doit
 *             apparaître.
 *   @negative l'assistant est interrompu après l'étape 1 (rechargement de
 *             page) → la reprise depuis IndexedDB ramène directement à
 *             l'étape où l'utilisateur s'était arrêté.
 *
 * Plus un test axe-core (@happy) sur les étapes 1 et 2 — DoD « accessible
 * clavier seul / axe-core VERT ».
 *
 * NOTE — endpoint `PUT /acps/{id}/modules/{module}/enable` pas encore
 * implémenté backend (Story 5.1, cf. en-tête de OnboardingWizard.svelte).
 * Comme pour `pwa-contractor.spec.ts` (Story 3.3) avec `/c/{token}/respond`,
 * on intercepte cet appel via `page.route` pour que le parcours puisse
 * aboutir en CI. Supprimer le mock quand 5.1 atterrit et garder le reste du
 * flux intact.
 */
import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { loginAsAdmin, loginAsSyndic } from "../../helpers/auth";

const WCAG_AA_TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"];

/**
 * Attend que le brouillon soit réellement écrit dans IndexedDB avant de
 * recharger la page.
 *
 * L'effet de persistance du composant est fire-and-forget (`void
 * saveDraft(...)`) : `toBeVisible()` sur l'étape 2 confirme que le DOM a
 * changé, pas que l'écriture asynchrone dans IndexedDB a abouti. Un
 * `page.reload()` immédiatement après risquerait de courir devant cette
 * écriture — flake par construction, pas par défaut du produit.
 */
async function waitForDraftPersisted(
  page: import("@playwright/test").Page,
  expectedStep: number,
) {
  await page.waitForFunction((step) => {
    return new Promise<boolean>((resolve) => {
      const req = indexedDB.open("koprogo-onboarding");
      req.onerror = () => resolve(false);
      req.onsuccess = () => {
        const db = req.result;
        if (!db.objectStoreNames.contains("wizard-drafts")) {
          resolve(false);
          return;
        }
        const tx = db.transaction(["wizard-drafts"], "readonly");
        const getReq = tx.objectStore("wizard-drafts").get("current-draft");
        getReq.onsuccess = () => resolve(getReq.result?.step === step);
        getReq.onerror = () => resolve(false);
      };
    });
  }, expectedStep);
}

async function mockModuleActivation(page: import("@playwright/test").Page) {
  await page.route(/\/acps\/[^/]+\/modules\/[^/]+\/enable$/, async (route) => {
    if (route.request().method() === "PUT") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ ok: true }),
      });
      return;
    }
    await route.fallback();
  });
}

async function fillProfileStep(
  page: import("@playwright/test").Page,
  opts: { name: string; unitsCount: string; sharedSpaces: boolean },
) {
  await page.getByTestId("onboarding-name-input").fill(opts.name);
  await page.getByTestId("onboarding-street-input").fill("Rue des Tilleuls 12");
  await page.getByTestId("onboarding-postal-code-input").fill("1000");
  await page.getByTestId("onboarding-city-input").fill("Bruxelles");
  await page.getByTestId("onboarding-units-count-input").fill(opts.unitsCount);
  if (opts.sharedSpaces) {
    await page.getByTestId("onboarding-shared-spaces-checkbox").check();
  }
}

test.describe("Story 5.7 — OnboardingWizard (slice 5)", () => {
  // `public/service-worker.js` intercepte tout `/api/*` et refait le fetch
  // lui-même (networkFirstStrategy) : sans ce blocage, `page.route` sur
  // `/acps/.../modules/.../enable` n'aurait aucun effet, silencieusement
  // (cf. garde-interception-reseau.test.ts).
  test.use({ serviceWorkers: "block" });

  test("@happy superadmin completes the 5 steps → ACP created → modules activated → confirmation under 5 min", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await mockModuleActivation(page);

    const enableRequests: string[] = [];
    page.on("request", (req) => {
      if (
        req.method() === "PUT" &&
        /\/modules\/[^/]+\/enable$/.test(req.url())
      ) {
        enableRequests.push(req.url());
      }
    });

    await page.goto("/admin/acps/onboarding", { waitUntil: "networkidle" });

    await expect(page.getByTestId("onboarding-step-1")).toBeVisible();

    await fillProfileStep(page, {
      name: `ACP E2E ${Date.now()}`,
      unitsCount: "3",
      sharedSpaces: true,
    });
    await page.getByTestId("onboarding-next").click();

    await expect(page.getByTestId("onboarding-step-2")).toBeVisible({
      timeout: 15_000,
    });
    // "community" recommandé (espaces communs) + "identity" (socle).
    await expect(
      page.getByTestId("onboarding-module-toggle-community"),
    ).toBeChecked();
    await expect(
      page.getByTestId("onboarding-module-toggle-identity"),
    ).toBeChecked();

    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-step-3")).toBeVisible();

    await page.getByTestId("onboarding-activate-submit").click();
    await expect(page.getByTestId("onboarding-step-4")).toBeVisible({
      timeout: 10_000,
    });
    expect(enableRequests.some((u) => u.includes("/modules/community/"))).toBe(
      true,
    );

    await page.getByTestId("onboarding-demo-next").click();
    await expect(page.getByTestId("onboarding-step-5")).toBeVisible();
    await expect(page.getByTestId("onboarding-kpi-status")).toContainText(
      /Objectif atteint/,
    );

    await page.getByTestId("onboarding-finish-submit").click();
  });

  test("@edge skipping the recommendation activates only the default modules (community + identity)", async ({
    page,
  }) => {
    await loginAsAdmin(page);
    await mockModuleActivation(page);

    const enableRequests: string[] = [];
    page.on("request", (req) => {
      if (
        req.method() === "PUT" &&
        /\/modules\/[^/]+\/enable$/.test(req.url())
      ) {
        enableRequests.push(req.url());
      }
    });

    await page.goto("/admin/acps/onboarding", { waitUntil: "networkidle" });
    await expect(page.getByTestId("onboarding-step-1")).toBeVisible();

    // Grande ACP (12 lots, sans espaces communs) : un moteur de recommandation
    // suggérerait accounting + governance. On saute quand même l'étape.
    await fillProfileStep(page, {
      name: `ACP E2E Skip ${Date.now()}`,
      unitsCount: "12",
      sharedSpaces: false,
    });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-step-2")).toBeVisible({
      timeout: 15_000,
    });

    await page.getByTestId("onboarding-skip-recommendation").click();
    await expect(page.getByTestId("onboarding-step-3")).toBeVisible();

    await page.getByTestId("onboarding-activate-submit").click();
    await expect(page.getByTestId("onboarding-step-4")).toBeVisible({
      timeout: 10_000,
    });

    // Seul "community" est appelé (identity toujours actif) — jamais
    // accounting ni governance, que la recommandation aurait pourtant
    // suggérés pour 12 lots.
    expect(enableRequests).toHaveLength(1);
    expect(enableRequests[0]).toContain("/modules/community/");
  });

  test("@security a syndic cannot reach the wizard by typing the admin URL directly", async ({
    page,
  }) => {
    await loginAsSyndic(page);

    await page.goto("/admin/acps/onboarding", { waitUntil: "networkidle" });

    // Assertion POSITIVE d'abord : `getDefaultRedirect("syndic")` (guards.ts)
    // renvoie `/syndic`. Sans elle, les deux assertions suivantes passeraient
    // aussi si l'île n'avait simplement pas fini de s'hydrater — un faux vert
    // qui ne prouverait aucune redirection.
    await expect(page).toHaveURL(/\/syndic/, { timeout: 10_000 });
    await expect(page).not.toHaveURL(/\/admin\/acps\/onboarding/);
    await expect(page.getByTestId("onboarding-step-1")).toHaveCount(0);
    await expect(page.getByTestId("onboarding-wizard-root")).toHaveCount(0);
  });

  test("@happy steps 1 and 2 have no WCAG 2.1 AA violations (axe-core)", async ({
    page,
  }) => {
    await loginAsAdmin(page);

    await page.goto("/admin/acps/onboarding", { waitUntil: "networkidle" });
    await expect(page.getByTestId("onboarding-step-1")).toBeVisible();

    const step1Results = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();
    expect(step1Results.violations).toEqual([]);

    await fillProfileStep(page, {
      name: `ACP E2E A11y ${Date.now()}`,
      unitsCount: "3",
      sharedSpaces: true,
    });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-step-2")).toBeVisible({
      timeout: 15_000,
    });

    // L'étape 2 porte les cases à cocher désactivées (identity) et les
    // badges "recommandé" — le gabarit le plus à risque du wizard.
    const step2Results = await new AxeBuilder({ page })
      .withTags(WCAG_AA_TAGS)
      .analyze();
    expect(step2Results.violations).toEqual([]);
  });

  test("@negative wizard interrupted after step 1 (page reload) resumes from IndexedDB", async ({
    page,
  }) => {
    await loginAsAdmin(page);

    await page.goto("/admin/acps/onboarding", { waitUntil: "networkidle" });
    await expect(page.getByTestId("onboarding-step-1")).toBeVisible();

    await fillProfileStep(page, {
      name: `ACP E2E Reprise ${Date.now()}`,
      unitsCount: "3",
      sharedSpaces: false,
    });
    await page.getByTestId("onboarding-next").click();
    await expect(page.getByTestId("onboarding-step-2")).toBeVisible({
      timeout: 15_000,
    });
    await waitForDraftPersisted(page, 2);

    // Interruption : rechargement brutal, comme un onglet fermé puis rouvert.
    await page.reload({ waitUntil: "networkidle" });

    await expect(page.getByTestId("onboarding-resumed-banner")).toBeVisible({
      timeout: 10_000,
    });
    await expect(page.getByTestId("onboarding-step-2")).toBeVisible();
  });
});
