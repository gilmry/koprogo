import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

test.describe("Sessions AG vidéo — parcours de création rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("crée une session vidéo puis la démarre, depuis la fiche réunion", async ({
    page,
  }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(
      page,
      "journey-agsession",
    );
    await page.goto(`/meeting-detail?id=${meetingId}`, {
      waitUntil: "networkidle",
    });

    await page.getByTestId("ag-session-create-btn").click();
    await page
      .getByTestId("ag-session-video-url-input")
      .fill("https://meet.jit.si/journey-test-room");

    const scheduledStart = new Date(Date.now() + 24 * 60 * 60 * 1000)
      .toISOString()
      .slice(0, 16);
    await page
      .getByTestId("ag-session-scheduled-start-input")
      .fill(scheduledStart);

    const [createResp] = await Promise.all([
      page.waitForResponse(
        (r) =>
          r.url().includes("/ag-session") && r.request().method() === "POST",
      ),
      page.getByTestId("ag-session-submit-btn").click(),
    ]);
    expect(createResp.status()).toBe(201);

    const startBtn = page.getByTestId("ag-session-start-btn");
    await expect(startBtn).toBeVisible();

    const [startResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/start") && r.request().method() === "PUT",
      ),
      startBtn.click(),
    ]);
    expect(startResp.status()).toBe(200);

    await expect(page.getByTestId("ag-session-end-btn")).toBeVisible();
  });
});
