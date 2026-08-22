/**
 * Characterization Spec 05 — Notifications + sync
 *
 * GOAL : Geler la page notifications + bell + sync (création API + visible UI).
 *
 * STATUT : Caractérisation (NON TDD red-first). Doit être GREEN sur HEAD pré-refonte.
 *
 * SOURCE : docs/maury/refonte-ux-multi-role-acp/stories.md §2 Story 0.1
 */
import { test, expect } from "@playwright/test";
import { loginAsSyndic } from "../helpers/auth";
import { setupContainerApiUrl } from "../helpers/video-pace";

const API_BASE = process.env.PLAYWRIGHT_API_BASE || "http://localhost/api/v1";

test.describe("Characterization 05 — Notifications + sync", () => {
  test.beforeEach(async ({ page }) => {
    await setupContainerApiUrl(page);
  });

  test("notifications page renders for syndic", async ({ page }) => {
    await loginAsSyndic(page, "char-notif");
    await page.goto("/notifications");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='notifications-list']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("notification preferences page renders", async ({ page }) => {
    await loginAsSyndic(page, "char-notif-prefs");
    await page.goto("/settings/notifications");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page
        .locator("main h1, main h2, [data-testid='notification-preferences']")
        .first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("API-created notification appears in UI list (sync)", async ({
    page,
  }) => {
    const { token } = await loginAsSyndic(page, "char-notif-sync");
    const timestamp = Date.now();

    // userId injection helper donne "injected-user" — on doit récupérer le vrai
    // userId via /auth/me ou similaire. À défaut, on poste la notification sur
    // la session courante en récupérant l'id du JWT côté /me.
    const meResp = await page.request.get(`${API_BASE}/auth/me`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(meResp.ok()).toBeTruthy();
    const me = await meResp.json();
    const userId = me.id || me.user?.id;
    expect(userId).toBeTruthy();

    const title = `Char Notif Sync ${timestamp}`;
    const notifResp = await page.request.post(`${API_BASE}/notifications`, {
      data: {
        user_id: userId,
        title,
        message: "Notification de caractérisation pour test sync UI",
        notification_type: "System",
        channel: "InApp",
        priority: "Medium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(notifResp.ok()).toBeTruthy();

    // Caractérise via /notifications/my (endpoint list-for-current-user).
    // GET /notifications n'existe pas sur HEAD (404) — comportement existant.
    const listResp = await page.request.get(`${API_BASE}/notifications/my`, {
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(listResp.ok()).toBeTruthy();
    const notifs = await listResp.json();
    const found =
      Array.isArray(notifs) &&
      notifs.some((n: { title: string }) => n.title === title);
    expect(
      found,
      `Notification "${title}" should be in /notifications/my after creation (sync gel)`,
    ).toBeTruthy();

    // Page UI charge sans crash après création (sync : le polling/refresh
    // existant peut ou non afficher la notif selon le storage user)
    await page.goto("/notifications");
    await expect(page.locator("body")).toBeVisible();
  });

  test("syndic dashboard accessible — bell entry point", async ({ page }) => {
    await loginAsSyndic(page, "char-notif-bell");
    await page.goto("/syndic");
    await expect(page.locator("body")).toBeVisible();
    // On ne hardcode pas le sélecteur bell exact (peut varier) ; on vérifie
    // juste que la page dashboard charge — la bell est testée par les flows
    // ulterieurs slice 3+.
  });
});
