/**
 * Régression : boucle de redirection infinie sur /login en prod (koprogo.com).
 *
 * nginx (try_files $uri $uri/ /index.html) redirige /login -> /login/ (slash
 * final, comportement standard pour un répertoire statique). isPublicRoute
 * ne matchait que la forme exacte "/login" (sans slash), donc RouteGuard
 * traitait /login/ comme une route protégée et redirigeait vers /login,
 * que nginx renvoyait de nouveau vers /login/ - boucle infinie constatée
 * en session (Playwright, 100+ navigations en 6s, jamais stabilisé).
 */
import { describe, it, expect } from "vitest";
import { isPublicRoute } from "./guards";

describe("isPublicRoute", () => {
  it("reconnaît les routes publiques avec ou sans slash final", () => {
    expect(isPublicRoute("/login")).toBe(true);
    expect(isPublicRoute("/login/")).toBe(true);
    expect(isPublicRoute("/register")).toBe(true);
    expect(isPublicRoute("/register/")).toBe(true);
    expect(isPublicRoute("/mentions-legales")).toBe(true);
    expect(isPublicRoute("/mentions-legales/")).toBe(true);
  });

  it("garde la racine intacte", () => {
    expect(isPublicRoute("/")).toBe(true);
  });

  it("reconnaît /blog et ses sous-routes", () => {
    expect(isPublicRoute("/blog")).toBe(true);
    expect(isPublicRoute("/blog/mon-article")).toBe(true);
  });

  it("rejette les routes protégées, slash final ou non", () => {
    expect(isPublicRoute("/admin")).toBe(false);
    expect(isPublicRoute("/admin/")).toBe(false);
    expect(isPublicRoute("/syndic")).toBe(false);
  });
});
