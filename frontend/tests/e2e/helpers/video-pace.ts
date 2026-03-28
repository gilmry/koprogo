/**
 * Video Pace Helpers - "Documentation Vivante"
 *
 * These helpers add human-readable pauses between Playwright actions
 * so that recorded videos can serve as living documentation (YouTube, demos).
 *
 * Usage: import { human, PACE } from "./helpers/video-pace";
 *
 * Each function waits before AND after the action so the viewer can:
 * 1. See what's about to happen (cursor moves to element)
 * 2. See the action happen (typing, click)
 * 3. See the result (page update, spinner, navigation)
 */
import type { Page, Locator } from "@playwright/test";

// ---------------------------------------------------------------------------
// Timing constants (milliseconds) - tune for comfortable YouTube viewing
// ---------------------------------------------------------------------------
export const PACE = {
  /** Pause before typing into a field */
  BEFORE_TYPE: 600,
  /** Pause after typing (let viewer read what was typed) */
  AFTER_TYPE: 400,
  /** Pause before clicking a button/link */
  BEFORE_CLICK: 500,
  /** Pause after clicking (see the result) */
  AFTER_CLICK: 1000,
  /** Pause after a page navigation completes */
  AFTER_NAVIGATION: 1500,
  /** Polling interval to check if spinner disappeared */
  SPINNER_POLL: 300,
  /** Pause after spinner disappears (let viewer see loaded state) */
  AFTER_SPINNER: 1000,
  /** Pause between major logical steps (e.g., "login done, now create ticket") */
  BETWEEN_STEPS: 2500,
  /** Pause at the very end of a scenario (let viewer see final state) */
  FINAL_PAUSE: 3000,
  /** Pause before selecting a dropdown option */
  BEFORE_SELECT: 400,
  /** Pause after selecting a dropdown option */
  AFTER_SELECT: 600,
};

// ---------------------------------------------------------------------------
// Core helpers
// ---------------------------------------------------------------------------

/**
 * Fill a form field at human pace.
 * Scrolls into view, highlights briefly, types, then pauses.
 */
export async function humanFill(
  page: Page,
  testId: string,
  text: string,
): Promise<void> {
  const locator = page.getByTestId(testId);
  await locator.scrollIntoViewIfNeeded();
  await page.waitForTimeout(PACE.BEFORE_TYPE);
  await locator.clear();
  await locator.fill(text);
  await page.waitForTimeout(PACE.AFTER_TYPE);
}

/**
 * Click an element identified by data-testid at human pace.
 */
export async function humanClick(page: Page, testId: string): Promise<void> {
  const locator = page.getByTestId(testId);
  await locator.scrollIntoViewIfNeeded();
  await page.waitForTimeout(PACE.BEFORE_CLICK);
  await locator.click();
  await page.waitForTimeout(PACE.AFTER_CLICK);
}

/**
 * Click a locator directly (for non-testid selectors) at human pace.
 */
export async function humanClickLocator(
  page: Page,
  locator: Locator,
): Promise<void> {
  await locator.scrollIntoViewIfNeeded();
  await page.waitForTimeout(PACE.BEFORE_CLICK);
  await locator.click();
  await page.waitForTimeout(PACE.AFTER_CLICK);
}

/**
 * Select an option in a <select> element at human pace.
 */
export async function humanSelect(
  page: Page,
  testId: string,
  value: string,
): Promise<void> {
  const locator = page.getByTestId(testId);
  await locator.scrollIntoViewIfNeeded();
  await page.waitForTimeout(PACE.BEFORE_SELECT);
  await locator.selectOption(value);
  await page.waitForTimeout(PACE.AFTER_SELECT);
}

/**
 * Navigate to a URL and wait for the page to settle visually.
 */
export async function humanGoto(page: Page, url: string): Promise<void> {
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await page.waitForTimeout(PACE.AFTER_NAVIGATION);
}

/**
 * Wait for a loading spinner to appear then disappear.
 * Handles cases where the spinner is too fast to catch.
 *
 * Looks for common spinner patterns:
 * - data-testid="loading-spinner"
 * - .spinner / .loading CSS classes
 * - [aria-busy="true"] attribute
 */
export async function waitForSpinner(
  page: Page,
  options?: { timeout?: number },
): Promise<void> {
  const timeout = options?.timeout ?? 15000;
  const spinnerSelector = [
    '[data-testid="loading-spinner"]',
    '[data-testid="loading"]',
    ".spinner",
    ".loading",
    '[aria-busy="true"]',
  ].join(", ");

  const spinner = page.locator(spinnerSelector).first();

  try {
    // Wait briefly for spinner to appear
    await spinner.waitFor({ state: "visible", timeout: 2000 });
    // Then wait for it to disappear (content loaded)
    await spinner.waitFor({ state: "hidden", timeout });
    await page.waitForTimeout(PACE.AFTER_SPINNER);
  } catch {
    // Spinner was too fast or didn't appear — that's fine
    await page.waitForTimeout(PACE.AFTER_SPINNER);
  }
}

/**
 * Pause between major logical steps.
 * Use this to give the viewer time to understand what just happened.
 */
export async function stepPause(page: Page): Promise<void> {
  await page.waitForTimeout(PACE.BETWEEN_STEPS);
}

/**
 * Final pause at the end of a scenario.
 * Lets the viewer see the final state before the video ends.
 */
export async function finalPause(page: Page): Promise<void> {
  await page.waitForTimeout(PACE.FINAL_PAUSE);
}

/**
 * Override the frontend API URL for in-container Playwright runs.
 *
 * When Chromium runs inside a Docker container, `localhost` doesn't reach
 * Traefik/backend. This injects `window.__ENV__` before any page JS loads
 * so that API calls go to the correct Docker service hostname.
 *
 * Call this ONCE per test before any page.goto().
 */
export async function setupContainerApiUrl(page: Page): Promise<void> {
  const apiBase = process.env.PLAYWRIGHT_API_BASE;
  if (!apiBase) return; // not in container mode

  await page.addInitScript((url) => {
    (window as any).__ENV__ = { API_URL: url };
  }, apiBase);
}

/**
 * Login via the UI at human pace (not injected via localStorage).
 * This is the "documentation vivante" version — the viewer sees every step.
 */
export async function humanLogin(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  await setupContainerApiUrl(page);
  await humanGoto(page, "/login");
  await humanFill(page, "login-email", email);
  await humanFill(page, "login-password", password);
  await humanClick(page, "login-submit");
  await waitForSpinner(page);
  await page.waitForURL(/\/(syndic|owner|accountant|admin)/, {
    timeout: 15000,
  });
  await page.waitForTimeout(PACE.AFTER_NAVIGATION);
}
