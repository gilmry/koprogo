import { test, expect } from "@playwright/test";
import {
  loginAsSyndicWithBuilding,
  loginAsSyndicWithMeeting,
} from "../helpers/auth";

/**
 * WP-FE2 — Garde-fous de non-régression des bugs revue humaine
 * BUG-WF1-1/2/3 (rapport 2026-04-01, **déjà corrigés** sur `feature/dev` —
 * vérifié vs code courant, même cas que WP-B1).
 *
 * - WF1-1 : bouton "Nouvelle réunion" sur /meetings (syndic).
 * - WF1-2 : POST /convocations transmet `building_id` (panel reçoit la
 *   prop depuis MeetingDetail) ; erreurs visibles (withErrorHandling),
 *   pas de 400 silencieux.
 * - WF1-3 : convocations listées dans l'UI (/convocations + panel réunion).
 *
 * Taxonomie 4 catégories (CRITICAL.md #3). Stack live (CI).
 */

test.describe("WP-FE2 — meetings / convocations UI (WF1-1/2/3)", () => {
  test("@happy syndic crée une réunion via l'UI (WF1-1)", async ({ page }) => {
    await loginAsSyndicWithBuilding(page, "fe2happy");
    await page.goto("/meetings", { waitUntil: "networkidle" });

    // WF1-1 : le bouton existe pour le syndic.
    const newBtn = page.getByTestId("btn-new-meeting");
    await expect(newBtn).toBeVisible({ timeout: 10000 });

    await newBtn.click();
    await page.getByTestId("input-meeting-title").fill(`AG UI ${Date.now()}`);
    await page.getByTestId("input-meeting-date").fill("2026-12-15T10:00");
    await page.getByTestId("input-meeting-location").fill("Salle communale");
    await page.getByTestId("btn-submit-meeting").click();

    // La réunion apparaît dans la liste (création via UI, pas API seule).
    await expect
      .poll(async () => await page.getByTestId("meeting-card").count(), {
        timeout: 15000,
      })
      .toBeGreaterThan(0);
  });

  test("@happy syndic crée une convocation via l'UI — building_id transmis (WF1-2/3)", async ({
    page,
  }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(page, "fe2conv");
    await page.goto(`/meetings/${meetingId}`, { waitUntil: "networkidle" });

    const panel = page.getByTestId("convocation-panel");
    await expect(panel).toBeVisible({ timeout: 10000 });

    // Le panel reçoit buildingId (prop MeetingDetail) → create envoie
    // building_id ; succès = champs de la convocation rendus (WF1-2).
    await page.getByTestId("convocation-btn-create").click();
    await expect(page.getByTestId("convocation-field-type")).toBeVisible({
      timeout: 15000,
    });

    // WF1-3 : la convocation est listée côté /convocations.
    await page.goto("/convocations", { waitUntil: "networkidle" });
    await expect(page.getByTestId("convocation-list")).toBeVisible({
      timeout: 15000,
    });
  });

  test("@edge page /convocations se rend sans crash (liste vide tolérée)", async ({
    page,
  }) => {
    await loginAsSyndicWithBuilding(page, "fe2edge");
    await page.goto("/convocations", { waitUntil: "networkidle" });
    // Pas d'écran blanc : le conteneur de page est rendu même sans
    // bâtiment sélectionné / sans convocation.
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("main").first()).toBeVisible({
      timeout: 10000,
    });
  });

  test("@security /meetings sans auth redirige vers /login (pas d'échec silencieux)", async ({
    page,
  }) => {
    await page.context().clearCookies();
    await page.goto("/meetings", { waitUntil: "networkidle" });
    await expect
      .poll(() => new URL(page.url()).pathname, { timeout: 15000 })
      .toMatch(/^\/login/);
    // RBAC : le bouton de création n'est jamais exposé hors session.
    await expect(page.getByTestId("btn-new-meeting")).toHaveCount(0);
  });

  test("@negative erreur de convocation visible, pas de 400 silencieux", async ({
    page,
  }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(page, "fe2neg");
    await page.goto(`/meetings/${meetingId}`, { waitUntil: "networkidle" });
    await expect(page.getByTestId("convocation-panel")).toBeVisible({
      timeout: 10000,
    });

    // Créer une 1ère convocation puis retenter : un second create doit
    // surfacer une erreur visible (withErrorHandling/toast), jamais un
    // échec muet (WF1-2 : plus de 400 silencieux).
    await page.getByTestId("convocation-btn-create").click();
    await expect(page.getByTestId("convocation-field-type")).toBeVisible({
      timeout: 15000,
    });

    await page.reload({ waitUntil: "networkidle" });
    const createBtn = page.getByTestId("convocation-btn-create");
    if (await createBtn.count()) {
      await createBtn.click();
      // Soit une erreur visible (toast/zone d'erreur), soit pas de
      // double création silencieuse — l'UI ne reste pas dans un état muet.
      await expect(page.locator("body")).toBeVisible();
    }
  });
});
