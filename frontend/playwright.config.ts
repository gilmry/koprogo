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
  retries: process.env.CI ? 2 : 0,

  /* Opt out of parallel tests on CI. */
  workers: process.env.CI ? 1 : undefined,

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
      // Stabilisation Documentation Vivante e2e"). Les specs restent dans le
      // repo et peuvent être lancées en local pour debug.
      testIgnore: [
        /scenarios\//,
        /smoke\//,
        /characterization\//,
        /refonte-ux\/phase-b-fe\//,
      ],
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
