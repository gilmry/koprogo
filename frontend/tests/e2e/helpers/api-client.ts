/**
 * Typed API client for Playwright E2E tests.
 *
 * Wraps Playwright's page.request with type-safe methods derived from
 * the OpenAPI spec. TypeScript validates field names, required fields,
 * and enum values at compile time — no more silent 400 errors from
 * wrong field names like "question" instead of "title".
 *
 * Usage:
 *   import { createApiClient } from "./helpers/api-client";
 *   const api = createApiClient(page, token);
 *   const poll = await api.post("/polls", { building_id: "...", ... });
 */
import type { Page } from "@playwright/test";
import type { paths, components } from "../../../src/types/api.d.ts";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

export type Schemas = components["schemas"];
export type { paths, components };

// Extract the JSON request body type for a given path + method
type RequestBody<P extends keyof paths, M extends string> =
  paths[P] extends Record<M, infer Op>
    ? Op extends {
        requestBody?: { content: { "application/json": infer Body } };
      }
      ? Body
      : never
    : never;

// Extract path params type
type PathParams<P extends keyof paths, M extends string> =
  paths[P] extends Record<M, infer Op>
    ? Op extends { parameters: { path: infer Params } }
      ? Params
      : Record<string, never>
    : Record<string, never>;

/**
 * Type-safe API client backed by Playwright's request context.
 * Uses the same network stack as the browser (same cookies, same proxy).
 */
export function createApiClient(page: Page, token?: string) {
  const authHeaders: Record<string, string> = token
    ? { Authorization: `Bearer ${token}` }
    : {};

  function buildUrl(path: string, params?: Record<string, string>): string {
    let url = `${API_BASE}${path}`;
    if (params) {
      for (const [key, value] of Object.entries(params)) {
        url = url.replace(`{${key}}`, value);
      }
    }
    return url;
  }

  return {
    /** Type-safe POST — body validated against OpenAPI schema */
    async post<P extends keyof paths>(
      path: P,
      body: RequestBody<P, "post">,
      pathParams?: PathParams<P, "post">,
    ) {
      const resp = await page.request.post(
        buildUrl(path as string, pathParams as Record<string, string>),
        {
          data: body as unknown as Record<string, unknown>,
          headers: { ...authHeaders },
        },
      );
      return {
        response: resp,
        status: resp.status(),
        ok: resp.ok(),
        data: resp.ok() ? await resp.json() : null,
      };
    },

    /** Type-safe GET */
    async get<P extends keyof paths>(
      path: P,
      pathParams?: PathParams<P, "get">,
    ) {
      const resp = await page.request.get(
        buildUrl(path as string, pathParams as Record<string, string>),
        { headers: { ...authHeaders } },
      );
      return {
        response: resp,
        status: resp.status(),
        ok: resp.ok(),
        data: resp.ok() ? await resp.json() : null,
      };
    },

    /** Type-safe PUT */
    async put<P extends keyof paths>(
      path: P,
      body?: RequestBody<P, "put">,
      pathParams?: PathParams<P, "put">,
    ) {
      const resp = await page.request.put(
        buildUrl(path as string, pathParams as Record<string, string>),
        {
          data: (body ?? {}) as unknown as Record<string, unknown>,
          headers: { ...authHeaders },
        },
      );
      return {
        response: resp,
        status: resp.status(),
        ok: resp.ok(),
        data: resp.ok() ? await resp.json() : null,
      };
    },

    /** Type-safe DELETE */
    async del<P extends keyof paths>(
      path: P,
      pathParams?: PathParams<P, "delete">,
    ) {
      const resp = await page.request.delete(
        buildUrl(path as string, pathParams as Record<string, string>),
        { headers: { ...authHeaders } },
      );
      return {
        response: resp,
        status: resp.status(),
        ok: resp.ok(),
        data: resp.ok() ? await resp.json() : null,
      };
    },
  };
}
