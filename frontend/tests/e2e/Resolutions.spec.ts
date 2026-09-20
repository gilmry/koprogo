import { test, expect } from "@playwright/test";
import { loginAsSyndicWithMeeting } from "./helpers/auth";

/**
 * Resolutions E2E Test Suite - AG Voting System
 *
 * Tests resolution creation, voting (Pour/Contre/Abstention),
 * and voting closure with different majority types (Simple/Absolute/Qualified).
 * Mirrors workflows from backend/tests/e2e_resolutions.rs.
 *
 * Belgian law (Art. 3.88 CC): 3 majority types.
 */

import { API_BASE } from "./helpers/adresses";

test.describe("Resolutions - AG Voting System", () => {
  test("@happy should display meetings page", async ({ page }) => {
    await loginAsSyndicWithMeeting(page, "resolution");
    await page.goto("/meetings");

    await expect(page.locator("body")).toBeVisible();
    await expect(
      page.locator("[data-testid='meetings-list']").first(),
    ).toBeVisible({ timeout: 10000 });
  });

  test("@happy should create a resolution and retrieve it", async ({
    page,
  }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();
    const title = `Budget annuel ${timestamp}`;

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title,
          description: "Approbation du budget annuel 2026",
          resolution_type: "ordinary",
          majority_required: "absolute",
          // Rattachée au point 0 de l'ordre du jour : une résolution qui n'y
          // est pas rattachée n'est pas votable (Art. 3.87 § 2 CC, #840).
          agenda_item_index: 0,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(resolutionResp.status()).toBe(201);

    const resolution = await resolutionResp.json();
    expect(resolution.id).toBeTruthy();
    expect(resolution.title).toBe(title);
    expect(resolution.status).toBe("pending");

    // Retrieve by ID
    const getResp = await page.request.get(
      `${API_BASE}/resolutions/${resolution.id}`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(getResp.status()).toBe(200);
    const retrieved = await getResp.json();
    expect(retrieved.id).toBe(resolution.id);
  });

  test("@happy should list resolutions for a meeting", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );

    const listResp = await page.request.get(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.ok()).toBeTruthy();
    const resolutions = await listResp.json();
    expect(Array.isArray(resolutions)).toBeTruthy();
  });

  test("@happy should navigate to meeting detail page", async ({ page }) => {
    const { meetingId } = await loginAsSyndicWithMeeting(page, "resolution");

    await page.goto(`/meeting-detail?id=${meetingId}`);
    await expect(page.locator("body")).toBeVisible();
    await expect(page.locator("h1").first()).toBeVisible({ timeout: 10000 });
  });

  test("@happy should cast a vote on a resolution", async ({ page }) => {
    const { token, meetingId, buildingId, orgId, acpId, adminToken } =
      await loginAsSyndicWithMeeting(page, "resolution");
    const timestamp = Date.now();

    // Create unit + owner for voting
    // acp_id (pas organization_id) requis sur CreateUnitDto depuis #602.
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: buildingId,
        unit_number: `V${timestamp}`,
        floor: 1,
        surface_area: 80.0,
        unit_type: "Apartment",
        quota: 100.0,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const unit = await unitResp.json();

    // Le vote est émis par le COPROPRIÉTAIRE lui-même, pas par le syndic.
    //
    // `POST /resolutions/{id}/vote` refuse en 403 tout compte sans fiche de
    // copropriétaire rattachée, et le commentaire du gestionnaire est
    // explicite : « Ceci ferme, pour l'instant, la question que #850 pose
    // sans la trancher : le syndic doit-il pouvoir saisir des votes en
    // séance ? Cette route ne le permet plus. »
    //
    // Ce test présumait le contraire — le syndic votait avec un `owner_id`
    // qui n'était pas le sien. Ce n'est pas un détail d'amorçage : c'est le
    // scénario entier qui est devenu interdit, et à raison. Il est donc
    // réécrit sur le geste réel, pas contourné.
    const ownerEmail = `vote-owner-${timestamp}@test.com`;
    const ownerRegResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: "test123456",
        first_name: "Vote",
        last_name: `Owner${timestamp}`,
        role: "owner",
        organization_id: orgId,
      },
    });
    expect(ownerRegResp.status()).toBe(201);
    const ownerUser = await ownerRegResp.json();
    const ownerToken = ownerUser.token;
    const ownerUserId = ownerUser.user?.id ?? ownerUser.id;

    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Vote",
        last_name: `Owner${timestamp}`,
        email: ownerEmail,
        address: "1 Rue Vote",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        // `user_id` est ce qui rattache la fiche au compte. Sans lui, le
        // compte vote « pour personne » et la route refuse.
        user_id: ownerUserId,
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const owner = await ownerResp.json();

    // Lier le copropriétaire AU LOT avant de voter.
    //
    // Sans ce lien, `POST /resolutions/{id}/vote` rend 403 depuis le
    // durcissement #850 : « le copropriétaire X ne détient pas le lot Y : il
    // ne peut pas voter pour lui (Art. 3.87 § 1er CC) ». Auparavant
    // n'importe quel `owner_id` de l'organisation passait — c'est
    // exactement l'usurpation que #850 ferme.
    //
    // Le produit a raison et le test avait vieilli : on ne vote que pour un
    // lot qu'on détient. Le lien est donc AJOUTÉ, pas la garde contournée.
    const lienResp = await page.request.post(
      `${API_BASE}/units/${unit.id}/owners`,
      {
        data: {
          owner_id: owner.id,
          // 1 et non 100 : malgré son nom, `ownership_percentage` est une
          // FRACTION. Envoyer 100 fait répondre au serveur « adding: 10000%,
          // total would be: 10000% » et refuser en 400 (Art. 577-2 §4 CC).
          ownership_percentage: 1,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(lienResp.status()).toBe(201);

    // Create resolution
    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Vote test ${timestamp}`,
          description: "Résolution pour test de vote",
          resolution_type: "ordinary",
          majority_required: "absolute",
          // Rattachée au point 0 de l'ordre du jour : une résolution qui n'y
          // est pas rattachée n'est pas votable (Art. 3.87 § 2 CC, #840).
          agenda_item_index: 0,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(resolutionResp.status()).toBe(201);
    const resolution = await resolutionResp.json();

    // Cast a vote
    const voteResp = await page.request.post(
      `${API_BASE}/resolutions/${resolution.id}/vote`,
      {
        data: {
          owner_id: owner.id,
          unit_id: unit.id,
          vote_choice: "pour",
          voting_power: 100,
          // Story 4.2 (#48) — comment le votant a été authentifié. Son
          // absence rend 422 `VOTE_AUTH_METHOD_REQUIRED` : Art. 3.87 §1er
          // suppose de savoir QUI a voté. `presence` est la modalité d'une
          // AG tenue en salle ; `itsme`/`eid` seraient exigées pour un vote
          // à distance.
          auth_method: "presence",
        },
        headers: { Authorization: `Bearer ${ownerToken}` },
      },
    );
    expect(voteResp.status()).toBe(201);
  });

  test("@happy should list votes for a resolution", async ({ page }) => {
    const { token, meetingId } = await loginAsSyndicWithMeeting(
      page,
      "resolution",
    );
    const timestamp = Date.now();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `List votes test ${timestamp}`,
          description: "Test",
          resolution_type: "ordinary",
          majority_required: "absolute",
          // Rattachée au point 0 de l'ordre du jour : une résolution qui n'y
          // est pas rattachée n'est pas votable (Art. 3.87 § 2 CC, #840).
          agenda_item_index: 0,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(resolutionResp.status()).toBe(201);
    const resolution = await resolutionResp.json();

    const listResp = await page.request.get(
      `${API_BASE}/resolutions/${resolution.id}/votes`,
      { headers: { Authorization: `Bearer ${token}` } },
    );
    expect(listResp.status()).toBe(200);
    const votes = await listResp.json();
    expect(Array.isArray(votes)).toBeTruthy();
  });

  test("@happy should close voting and calculate result", async ({ page }) => {
    const { token, meetingId, buildingId, orgId, acpId, adminToken } =
      await loginAsSyndicWithMeeting(page, "resolution");
    const timestamp = Date.now();

    // Create unit + owner for voting
    // acp_id (pas organization_id) requis sur CreateUnitDto depuis #602.
    const unitResp = await page.request.post(`${API_BASE}/units`, {
      data: {
        acp_id: acpId,
        building_id: buildingId,
        unit_number: `C${timestamp}`,
        floor: 1,
        surface_area: 80.0,
        unit_type: "Apartment",
        quota: 100.0,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const unit = await unitResp.json();

    const ownerResp = await page.request.post(`${API_BASE}/owners`, {
      data: {
        organization_id: orgId,
        first_name: "Close",
        last_name: `Owner${timestamp}`,
        email: `close-owner-${timestamp}@test.com`,
        address: "1 Rue Close",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    const owner = await ownerResp.json();

    const resolutionResp = await page.request.post(
      `${API_BASE}/meetings/${meetingId}/resolutions`,
      {
        data: {
          meeting_id: meetingId,
          title: `Close vote test ${timestamp}`,
          description: "Test clôture vote",
          resolution_type: "ordinary",
          majority_required: "absolute",
          // Rattachée au point 0 de l'ordre du jour : une résolution qui n'y
          // est pas rattachée n'est pas votable (Art. 3.87 § 2 CC, #840).
          agenda_item_index: 0,
        },
        headers: { Authorization: `Bearer ${token}` },
      },
    );

    expect(resolutionResp.status()).toBe(201);
    const resolution = await resolutionResp.json();

    // Cast a vote before closing (required for close to succeed)
    await page.request.post(`${API_BASE}/resolutions/${resolution.id}/vote`, {
      data: {
        owner_id: owner.id,
        unit_id: unit.id,
        vote_choice: "pour",
        voting_power: 100,
      },
      headers: { Authorization: `Bearer ${token}` },
    });

    const closeResp = await page.request.put(
      `${API_BASE}/resolutions/${resolution.id}/close`,
      {
        data: { total_voting_power: 100 },
        headers: { Authorization: `Bearer ${token}` },
      },
    );
    expect(closeResp.status()).toBe(200);

    const closed = await closeResp.json();
    expect(["adopted", "rejected"].includes(closed.status)).toBeTruthy();
  });

  test("@security should require auth for resolutions API", async ({
    page,
  }) => {
    const resp = await page.request.get(`${API_BASE}/resolutions/some-id`);
    expect([401, 403].includes(resp.status())).toBeTruthy();
  });
});
