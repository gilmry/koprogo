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
      },
      testIgnore: /scenarios\//,
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
      use: {
        ...devices["Desktop Chrome"],
        viewport: { width: 1280, height: 720 },
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
