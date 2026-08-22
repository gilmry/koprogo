/**
 * In-memory access token holder (WP-FE1 — JWT hors localStorage).
 *
 * L'access token (JWT court, 15 min) vit UNIQUEMENT en mémoire JS : jamais
 * `localStorage`/`sessionStorage`/cookie lisible. Un vol XSS ne peut donc
 * pas exfiltrer une session rejouable longue durée. Le refresh token, lui,
 * est dans un cookie `HttpOnly` posé par le backend (illisible par JS).
 *
 * Module dédié pour éviter tout cycle d'import entre `stores/auth.ts`
 * (écrit le token) et `lib/api.ts` (le lit pour le header Bearer).
 */

let accessToken: string | null = null;

export function getAccessToken(): string | null {
  return accessToken;
}

export function setAccessToken(token: string | null): void {
  accessToken = token;
}

export function clearAccessToken(): void {
  accessToken = null;
}
