/**
 * Magic-link helpers for Playwright E2E tests (Story Tx.2).
 *
 * Magic-link auth is used for one-shot, narrow-scope access (e.g. a
 * contractor opening a quote form, an owner approving a meeting agenda).
 * The full backend endpoint lands with **story 3.2**; in the meantime we
 * expose stable helper signatures so refonte-ux scenarios can be written
 * now and the implementation can be swapped in without touching the tests.
 */
import type { Page } from "@playwright/test";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

/**
 * Scope of a magic link — narrows the set of resources / actions the bearer
 * can touch. Mirrors the shape we expect in the `magic_links` table once
 * story 3.2 lands (`scope_type` + `scope_id` columns).
 */
export interface MagicLinkScope {
  /** Resource type the link is bound to (building, quote, meeting, ...). */
  resource: "building" | "quote" | "meeting" | "ticket";
  /** UUID of the resource. */
  resourceId: string;
  /** Optional explicit expiry override (ISO 8601). Defaults: backend policy. */
  expiresAt?: string;
}

export interface IssuedMagicLink {
  token: string;
  /** Frontend URL the recipient navigates to: `/c/<token>`. */
  url: string;
  /** Echo of the scope the backend bound the token to. */
  scope: MagicLinkScope;
}

/**
 * Ask the backend to issue a magic-link token for `subjectUserId` scoped to
 * `scope`. Requires an authenticated syndic (or higher) bearer in
 * `syndicToken`.
 *
 * TODO(story 3.2): implement against `POST /magic-links` once the endpoint
 * exists. Until then this function throws so callers know they're early.
 * The signature is the contract — do not change it.
 *
 * @param syndicToken    Bearer JWT of the syndic issuing the link
 * @param subjectUserId  UUID of the user the token authenticates as (can be
 *                       a placeholder until the contractor account exists)
 * @param scope          Resource scope the token is bound to
 */
export async function issueMagicLink(
  syndicToken: string,
  subjectUserId: string,
  scope: MagicLinkScope,
): Promise<IssuedMagicLink> {
  if (!syndicToken) {
    throw new Error("issueMagicLink: syndicToken required");
  }
  if (!subjectUserId) {
    throw new Error("issueMagicLink: subjectUserId required");
  }
  if (!scope?.resourceId) {
    throw new Error("issueMagicLink: scope.resourceId required");
  }

  // TODO(story 3.2): replace with real API call:
  //   const resp = await fetch(`${API_BASE}/magic-links`, {
  //     method: "POST",
  //     headers: {
  //       "Content-Type": "application/json",
  //       Authorization: `Bearer ${syndicToken}`,
  //     },
  //     body: JSON.stringify({
  //       subject_user_id: subjectUserId,
  //       scope_type: scope.resource,
  //       scope_id: scope.resourceId,
  //       expires_at: scope.expiresAt,
  //     }),
  //   });
  //   if (!resp.ok) throw new Error(`issueMagicLink: ${resp.status}`);
  //   return resp.json();

  // For now: explicit failure so green tests don't lull us into thinking
  // the endpoint exists. Do NOT silently fake a token — that would mask
  // the missing implementation in CI.
  throw new Error(
    "issueMagicLink: POST /magic-links not yet implemented (blocked by story 3.2)",
  );
}

/**
 * Navigate the Playwright `page` to the magic-link landing route
 * (`/c/<token>`). The frontend exchanges the token for a session cookie,
 * then redirects to the scoped dashboard (e.g. the contractor quote form).
 *
 * Safe to call from any test once a token exists. Never logs the token.
 *
 * @param page  Playwright Page
 * @param token Magic-link token (opaque to the test)
 */
export async function openMagicLinkPage(
  page: Page,
  token: string,
): Promise<void> {
  if (!token) {
    throw new Error("openMagicLinkPage: token required");
  }
  await page.goto(`/c/${encodeURIComponent(token)}`, {
    waitUntil: "networkidle",
  });
}

// Re-export the API base so dependent helpers can reuse it without leaking
// the env var name into every call site.
export { API_BASE };
