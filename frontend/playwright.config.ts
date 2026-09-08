import { defineConfig, devices } from "@playwright/test";

/**
 * See https://playwright.dev/docs/test-configuration.
 * Local dev: Traefik on http://localhost (port 80)
 * CI: Astro dev server on http://localhost:3000 (PLAYWRIGHT_BASE_URL env var)
 */
const baseURL = process.env.PLAYWRIGHT_BASE_URL || "http://localhost"; // Traefik on port 80

const useTraefik = !process.env.PLAYWRIGHT_BASE_URL; // false in CI (no Traefik)

export default defineConfig({
  testDir: "./tests/e2e",

  /* Run tests in files in parallel */
  fullyParallel: false,

  /* Fail the build on CI if you accidentally left test.only in the source code. */
  forbidOnly: !!process.env.CI,

  /* Retry on CI only */
  /**
   * Une seule reprise en CI, plus deux.
   *
   * Une reprise sert à distinguer un aléa d'un défaut. Elle ne le fait que si
   * elle réussit parfois. Mesuré sur les runs du 2026-09-07 et du 2026-09-08 :
   * **40 reprises exécutées, zéro test « flaky »** — pas une seule n'a
   * transformé un échec en réussite. Les durées sont identiques à la seconde
   * près d'une tentative à l'autre (55,7 s / 56,8 s / 55,7 s), ce qui est la
   * signature d'un échec déterministe.
   *
   * Ce qu'elles coûtaient : le temps qui a manqué à `chromium` pour finir ses
   * 19 derniers tests avant son plafond. Une reprise qui n'apprend rien prend
   * la place d'un test qu'on n'a pas mesuré.
   *
   * On en garde UNE plutôt que zéro : l'absence de flake sur deux runs ne
   * prouve pas qu'il n'y en aura jamais, et un vrai aléa mérite encore d'être
   * distingué d'un défaut. Si un « flaky » réapparaît, c'est le signal qu'il
   * faut le corriger, pas remonter ce nombre.
   */
  retries: process.env.CI ? 1 : 0,

  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,

  /**
   * Plafond pour la suite ENTIÈRE, pas pour un test.
   *
   * Le 2026-09-07, ce job a tourné **2 h 43** sans rendre la main, contre
   * 36 minutes au run précédent, et rien ne l'a arrêté : `ci.yml` ne portait
   * aucun `timeout-minutes`, la limite GitHub par défaut étant de six heures.
   *
   * Le timeout par test (30 s par défaut) ne suffit pas à borner l'ensemble :
   * avec `retries: 2` et 319 tests, une dégradation multiplie les exécutions
   * sans qu'aucune ne dépasse individuellement sa limite. Un job qui traîne
   * devient alors indiscernable d'un job mort — et il occupe un runner
   * pendant ce temps.
   *
   * 70 minutes, soit un peu moins que le `timeout-minutes: 90` du job : la
   * suite doit rendre la main d'elle-même, avec son rapport, plutôt que
   * d'être fauchée par GitHub sans rien laisser à lire.
   *
   * ATTENTION — ce plafond s'applique à CHAQUE invocation de `playwright
   * test`, pas au job. Or `ci.yml` en lance trois : `--project=chromium`,
   * `--project=smoke`, `--project=scenarios`. Trois fois 70 minutes font 210
   * minutes possibles sous un plafond de 90, et le garde-fou écrit ici ne
   * peut structurellement pas jouer.
   *
   * C'est arrivé le 2026-09-08 : chromium 46 min, scenarios 40 min, GitHub a
   * fauché le job à 90 min. Aucun rapport, aucun détail d'erreur pour onze
   * scénarios en échec, aucune vidéo. La CI n'a rien pu dire.
   *
   * Chaque étape de `ci.yml` porte désormais son propre `--global-timeout`,
   * dont la somme tient sous le plafond du job. Cette valeur-ci reste comme
   * filet pour les exécutions locales et pour toute invocation qui n'en
   * passerait pas.
   */
  globalTimeout: process.env.CI ? 70 * 60 * 1000 : undefined,

  /* Reporter to use. See https://playwright.dev/docs/test-reporters */
  reporter: [
    ["html", { outputFolder: "playwright-report", open: "never" }],
    ["json", { outputFile: "test-results/results.json" }],
    ["list"],
  ],

  /* Shared settings for all the projects below. See https://playwright.dev/docs/api/class-testoptions. */
  use: {
    /* Base URL to use in actions like `await page.goto('/')`. */
    baseURL,

    /* Collect trace when retrying the failed test. See https://playwright.dev/docs/trace-viewer */
    trace: "on-first-retry",

    /* Screenshot on failure */
    screenshot: "only-on-failure",

    /* Video recording - DOCUMENTATION VIVANTE! */
    video: {
      mode: "on", // Enregistre toujours les vidéos
      size: { width: 1280, height: 720 },
    },

    /* Maximum time each action can take */
    actionTimeout: 10000,

    /* Maximum time for the entire test */
    navigationTimeout: 30000,
  },

  /* Configure projects for major browsers */
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
        locale: "fr-BE",
        trace: "on",
      },
      // Phase C ouverte : `refonte-ux/phase-b-fe/` exclu du gate CI le temps
      // de stabiliser seeds + multi-rôle login flow (issue GH "Phase C —
      // Stabilisation Documentation Vivante e2e"). Réactivation au fur et à
      // mesure par spec stabilisé — 2026-08-06 : C1 role-assignment.spec.ts
      // ✅ (root cause : `valid_until` jamais persisté en DB par
      // `user_role_repository_impl.rs`), C5 ticket-complaint.spec.ts ✅ (bug
      // de test), C6 syndic-response-sla.spec.ts ✅ (débloqué par le fix
      // casse email de C4), C7 technical-spec-flow.spec.ts ✅ (bug produit
      // réel : status snake_case du backend comparé en PascalCase côté FE)
      // et C8 contractor-eval.spec.ts ✅. 2026-08-07 (Story S2,
      // docs/maury/syndic-org-users-endpoint) : magic-link-issue.spec.ts et
      // mandate-issue.spec.ts ✅ — débloqués par l'endpoint org-scopé
      // `GET /organizations/{id}/users` (Story S1, #691), la branche de
      // création (précédemment vacuously skip faute de sélecteur peuplé)
      // s'exécute désormais réellement. 2026-08-08 (Story S3) :
      // role-delegation.spec.ts (C4) ✅ — même endpoint câblé sur
      // `RoleDelegationsPage.svelte`, dernière exclusion Phase C levée
      // (#617 clos, 8/8 sub-tasks).
      testIgnore: [/scenarios\//, /smoke\//, /characterization\//],
    },

    /**
     * Characterization suite (Story 0.1) — gel comportement HEAD pré-refonte.
     *
     * Ces specs DOIVENT rester VERTES sur toutes les slices ultérieures
     * de la refonte UX multi-rôle ACP. Tournent sur CHAQUE PR slices 1-5
     * et bloquent le merge si ROUGE (gate Tx.1).
     *
     * Run: npx playwright test --project=characterization
     * Source: docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1
     */
    {
      name: "characterization",
      testDir: "./tests/e2e/characterization",
      fullyParallel: false,
      // Single worker pour éviter les conflits (helpers réutilisent admin login,
      // state DB partagé). Suite caractérisation = ordre déterministe pour gel.
      workers: 1,
      // Retry sur HMR/dev server hiccups (ERR_ABORTED). Cible : zero-flake gate Tx.1.
      retries: 2,
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
        locale: "fr-BE",
        trace: "on",
        video: {
          mode: "on",
          size: { width: 1280, height: 720 },
        },
      },
    },

    /**
     * API smoke tests — no video, parallel workers, fast.
     * These test backend API contracts, not UI interactions.
     *
     * Run only smokes:  npx playwright test --project=smoke
     */
    {
      name: "smoke",
      testDir: "./tests/e2e/smoke",
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
        locale: "fr-BE",
        video: "off",
        screenshot: "off",
      },
    },

    /**
     * "Documentation Vivante" scenarios — human-paced UI tests
     * whose videos are meant to be uploaded to YouTube as living docs.
     *
     * Run only scenarios:  npx playwright test --project=scenarios
     * Run only smoke tests: npx playwright test --project=chromium
     */
    {
      name: "scenarios",
      testDir: "./tests/e2e/scenarios",
      testMatch: /\.scenario\.ts$/,
      timeout: 120_000, // Scenarios are human-paced, need more time
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
        // Force French locale so nav testids match hardcoded expectations
        locale: "fr-BE",
        // Seed endpoint can be slow on existing data
        actionTimeout: 30_000,
        // Slow down EVERY Playwright action by 50ms on top of explicit pauses
        launchOptions: { slowMo: 50 },
        video: {
          mode: "on",
          size: { width: 1280, height: 720 },
        },
      },
    },

    // Uncomment for cross-browser testing
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },

    /* Test against mobile viewports. */
    // {
    //   name: 'Mobile Chrome',
    //   use: { ...devices['Pixel 5'] },
    // },
    // {
    //   name: 'Mobile Safari',
    //   use: { ...devices['iPhone 12'] },
    // },
  ],

  /* Run your local dev server before starting the tests */
  // Traefik is already running via docker-compose, no need to start webServer
  webServer: undefined,

  /* Output folders */
  outputDir: "test-results",
});
