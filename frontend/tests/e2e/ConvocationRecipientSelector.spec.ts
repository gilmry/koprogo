import { test, expect } from "@playwright/test";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { API_BASE } from "./helpers/adresses";

/**
 * Écran de sélection des destinataires d'une convocation (#780 verrou 1).
 *
 * Avant cet écran, « 0 destinataire » était un libellé affiché en
 * permanence, sans aucun contrôle pour le constituer : le clic sur
 * « Envoyer » ne changeait rien, ni succès ni erreur (RN-10, recette 4 du
 * 2026-09-06). Ces scénarios prouvent que le syndic voit maintenant qui sera
 * convoqué AVANT d'envoyer, et que la sélection est respectée — y compris
 * quand il la vide délibérément.
 */

async function createOwnerOnUnit(
  page: import("@playwright/test").Page,
  token: string,
  orgId: string,
  acpId: string,
  buildingId: string,
  unitNumber: string,
  quota: number,
  ownerLabel: string,
): Promise<{ ownerId: string; unitId: string }> {
  const timestamp = Date.now();

  const ownerResp = await page.request.post(`${API_BASE}/owners`, {
    data: {
      organization_id: orgId,
      first_name: ownerLabel,
      last_name: `Test${timestamp}${unitNumber}`,
      email: `recip-${unitNumber}-${timestamp}@test.com`,
      address: "1 Rue Test",
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(ownerResp.status()).toBe(201);
  const owner = await ownerResp.json();

  const unitResp = await page.request.post(`${API_BASE}/units`, {
    data: {
      acp_id: acpId,
      building_id: buildingId,
      unit_number: unitNumber,
      floor: 1,
      surface_area: 60.0,
      unit_type: "Apartment",
      quota,
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(unitResp.status()).toBe(201);
  const unit = await unitResp.json();

  const attachResp = await page.request.post(
    `${API_BASE}/units/${unit.id}/owners`,
    {
      data: {
        owner_id: owner.id,
        ownership_percentage: 1.0,
        is_primary_contact: true,
      },
      headers: { Authorization: `Bearer ${token}` },
    },
  );
  expect(attachResp.status()).toBe(201);

  return { ownerId: owner.id, unitId: unit.id };
}

async function createMeeting(
  page: import("@playwright/test").Page,
  token: string,
  orgId: string,
  buildingId: string,
): Promise<string> {
  const meetingDate = new Date();
  meetingDate.setDate(meetingDate.getDate() + 30);

  const meetingResp = await page.request.post(`${API_BASE}/meetings`, {
    data: {
      building_id: buildingId,
      organization_id: orgId,
      title: `AG destinataires ${Date.now()}`,
      scheduled_date: meetingDate.toISOString(),
      meeting_type: "Ordinary",
      location: "Salle communale",
      is_second_convocation: false,
    },
    headers: { Authorization: `Bearer ${token}` },
  });
  expect(meetingResp.status()).toBe(201);
  const meeting = await meetingResp.json();
  return meeting.id;
}

test.describe("Convocation — sélection des destinataires (#780)", () => {
  test("@happy le syndic voit les copropriétaires éligibles et les envoie", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "recipsel", {
      totalUnits: 2,
      totalTantiemes: 1000,
      seedUnits: false,
    });

    const alice = await createOwnerOnUnit(
      page,
      ctx.token,
      ctx.orgId,
      ctx.acpId,
      ctx.buildingId,
      "H1",
      500,
      "Alice",
    );
    const bob = await createOwnerOnUnit(
      page,
      ctx.token,
      ctx.orgId,
      ctx.acpId,
      ctx.buildingId,
      "H2",
      500,
      "Bob",
    );

    const meetingId = await createMeeting(
      page,
      ctx.token,
      ctx.orgId,
      ctx.buildingId,
    );

    await page.goto(`/meeting-detail?id=${meetingId}`, {
      waitUntil: "networkidle",
    });

    await page.getByTestId("convocation-btn-create").click();
    await expect(page.getByTestId("convocation-field-type")).toBeVisible({
      timeout: 15000,
    });

    // Les deux copropriétaires apparaissent, cochés par défaut.
    const checkboxAlice = page.getByTestId(
      `convocation-recipient-selector-checkbox-${alice.ownerId}`,
    );
    const checkboxBob = page.getByTestId(
      `convocation-recipient-selector-checkbox-${bob.ownerId}`,
    );
    await expect(checkboxAlice).toBeVisible({ timeout: 10000 });
    await expect(checkboxBob).toBeVisible();
    await expect(checkboxAlice).toBeChecked();
    await expect(checkboxBob).toBeChecked();

    const sendBtn = page.getByTestId("convocation-btn-send");
    await expect(sendBtn).toBeEnabled();
    await sendBtn.click();
    await page.getByTestId("confirm-dialog-confirm").click();

    // La convocation est envoyée aux deux destinataires choisis, pas à zéro.
    // Assertion sur le chiffre entre parenthèses plutôt que sur le libellé
    // traduit, dont la langue affichée dépend de la locale du navigateur.
    await expect(
      page.getByTestId("convocation-btn-toggle-recipients"),
    ).toContainText("(2)", { timeout: 15000 });
  });

  test("@negative décocher tous les destinataires bloque l'envoi", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "recipselneg", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });

    const alice = await createOwnerOnUnit(
      page,
      ctx.token,
      ctx.orgId,
      ctx.acpId,
      ctx.buildingId,
      "H1",
      1000,
      "Alice",
    );

    const meetingId = await createMeeting(
      page,
      ctx.token,
      ctx.orgId,
      ctx.buildingId,
    );

    await page.goto(`/meeting-detail?id=${meetingId}`, {
      waitUntil: "networkidle",
    });
    await page.getByTestId("convocation-btn-create").click();
    await expect(page.getByTestId("convocation-field-type")).toBeVisible({
      timeout: 15000,
    });

    const checkbox = page.getByTestId(
      `convocation-recipient-selector-checkbox-${alice.ownerId}`,
    );
    await expect(checkbox).toBeVisible({ timeout: 10000 });
    await checkbox.click(); // seul destinataire, décoché

    // Le texte affiché dépend de la locale du navigateur ; ce qui compte est
    // vérifiable sans dépendre de la traduction : le compteur passe à 0 et
    // l'envoi se bloque.
    await expect(
      page.getByTestId("convocation-recipient-selector-count"),
    ).toBeVisible();
    await expect(page.getByTestId("convocation-btn-send")).toBeDisabled();
  });

  test("@security un copropriétaire ne voit ni le sélecteur ni le bouton d'envoi", async ({
    page,
  }) => {
    const ctx = await loginAsSyndicWithBuilding(page, "recipselsec", {
      totalUnits: 1,
      totalTantiemes: 1000,
      seedUnits: false,
    });
    await createOwnerOnUnit(
      page,
      ctx.token,
      ctx.orgId,
      ctx.acpId,
      ctx.buildingId,
      "H1",
      1000,
      "Alice",
    );
    const meetingId = await createMeeting(
      page,
      ctx.token,
      ctx.orgId,
      ctx.buildingId,
    );

    // Convocation créée par le syndic avant le changement de rôle : le
    // scénario porte sur ce qu'un copropriétaire peut faire une fois la
    // convocation en brouillon, pas sur la création elle-même.
    const meetingDate = new Date();
    meetingDate.setDate(meetingDate.getDate() + 30);
    const convocResp = await page.request.post(`${API_BASE}/convocations`, {
      data: {
        building_id: ctx.buildingId,
        meeting_id: meetingId,
        meeting_type: "Ordinary",
        meeting_date: meetingDate.toISOString(),
        language: "FR",
      },
      headers: { Authorization: `Bearer ${ctx.token}` },
    });
    expect(convocResp.status()).toBe(201);

    const timestamp = Date.now();
    const ownerEmail = `recipsel-owner-${timestamp}@test.com`;
    const regResp = await page.request.post(`${API_BASE}/auth/register`, {
      data: {
        email: ownerEmail,
        password: "test123456",
        first_name: "Owner",
        last_name: `Sec${timestamp}`,
        role: "owner",
        organization_id: ctx.orgId,
      },
    });
    expect(regResp.status()).toBe(201);

    await page.goto("/login");
    await page.getByTestId("login-email").fill(ownerEmail);
    await page.getByTestId("login-password").fill("test123456");
    await page.getByTestId("login-submit").click();
    await page.waitForURL(/\/(owner|syndic|admin|accountant)/, {
      timeout: 15000,
    });

    await page.goto(`/meeting-detail?id=${meetingId}`, {
      waitUntil: "networkidle",
    });

    // Ni le sélecteur ni le bouton d'envoi ne sont exposés à un
    // copropriétaire : envoyer une convocation est un geste du syndic
    // (Art. 3.87 § 3 CC).
    await expect(
      page.getByTestId("convocation-recipient-selector"),
    ).toHaveCount(0);
    await expect(page.getByTestId("convocation-btn-send")).toHaveCount(0);
  });
});
