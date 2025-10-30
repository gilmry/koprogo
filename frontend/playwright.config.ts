import { defineConfig, devices } from "@playwright/test";

/**
 * See https://playwright.dev/docs/test-configuration.
 */
const baseURL =
  process.env.PLAYWRIGHT_BASE_URL &&
  process.env.PLAYWRIGHT_BASE_URL.trim() !== ""
    ? process.env.PLAYWRIGHT_BASE_URL.trim()
    : "http://localhost";

const useTraefik =
  !baseURL.includes("://localhost:3000") && !baseURL.includes("127.0.0.1:3000");

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
  webServer: useTraefik
    ? undefined
    : [
        {
          command: "npm run dev",
          url: "http://localhost:3000",
          reuseExistingServer: !process.env.CI,
          timeout: 120 * 1000,
          env: process.env.CI
            ? {
                // En CI, le backend tourne sur localhost:8080 (pas de Traefik)
                PUBLIC_API_URL: "http://localhost:8080/api/v1",
              }
            : {
                // En local, le backend est accessible via Traefik sur port 80
                PUBLIC_API_URL: "http://localhost/api/v1",
              },
        },
      ],

  /* Output folders */
  outputDir: "test-results",
});
