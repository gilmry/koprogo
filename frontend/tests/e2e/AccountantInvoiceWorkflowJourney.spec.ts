import { test, expect } from "@playwright/test";
import { confirmerSiDemande } from "./helpers/amorcage";
import { loginAsSyndicWithBuilding } from "./helpers/auth";
import { failOnPageErrors } from "./helpers/pageErrors";

import { API_BASE } from "./helpers/adresses";

test.describe("Comptable — Workflow factures, cycle de vie rempli jusqu'au bout", () => {
  test.beforeEach(async ({ page }) => failOnPageErrors(page));

  test("soumet, approuve puis marque payée une dépense (draft → pending_approval → approved → paid)", async ({
    page,
  }) => {
    const { token, buildingId } = await loginAsSyndicWithBuilding(
      page,
      "journey-invoice-wf",
    );
    const description = `Entretien chaudière ${Date.now()}`;

    const expenseResp = await page.request.post(`${API_BASE}/expenses`, {
      data: {
        building_id: buildingId,
        category: "Maintenance",
        description,
        amount: 800.0,
        expense_date: new Date().toISOString(),
      },
      headers: { Authorization: `Bearer ${token}` },
    });
    expect(expenseResp.status()).toBe(201);

    await page.goto("/invoice-workflow", { waitUntil: "networkidle" });
    // Plus de dialogue natif depuis #844 : les confirmations sont des
    // modales, et un navigateur piloté ne les supprime pas. Le gestionnaire
    // ci-dessous ne se déclenchait plus, et le geste restait en suspens.
    // `confirmerSiDemande` est appelée après chaque clic qui en demande une.

    const card = page
      .getByTestId("invoice-card")
      .filter({ hasText: description });
    await expect(card).toBeVisible();

    // L'attente est ARMÉE avant le clic, mais la confirmation vient entre les
    // deux. Le `Promise.all` d'avant ne laissait aucune place à ce geste : le
    // clic ouvrait la modale, la requête ne partait pas, et `waitForResponse`
    // expirait au bout de dix secondes en accusant le réseau.
    const attenteSoumission = page.waitForResponse(
      (r) => r.url().includes("/submit") && r.request().method() === "PUT",
    );
    await card.getByTestId("submit-approval-button").click();
    await confirmerSiDemande(page);
    const submitResp = await attenteSoumission;
    expect(submitResp.status()).toBe(200);
    await expect(card.getByTestId("approve-button")).toBeVisible();

    await card.getByTestId("approve-button").click();
    const approveModal = page.locator(".modal-footer").filter({
      has: page.getByRole("button", { name: "Approuver" }),
    });
    const [approveResp] = await Promise.all([
      page.waitForResponse(
        (r) => r.url().includes("/approve") && r.request().method() === "PUT",
      ),
      approveModal.getByRole("button", { name: "Approuver" }).click(),
    ]);
    expect(approveResp.status()).toBe(200);
    await expect(card.getByTestId("mark-paid-button")).toBeVisible();

    // Troisième confirmation du parcours, et la dernière qui manquait.
    //
    // La capture du run 34377060293 montre la modale OUVERTE — « Annuler »,
    // « Confirmer » — au-dessus d'une carte qui propose « Marquer comme
    // payée ». La soumission et l'approbation avaient abouti ; c'est le
    // paiement qui restait en suspens.
    //
    // Comme pour la soumission : l'attente est armée AVANT le clic, la
    // confirmation vient entre les deux. Un `Promise.all` ne laisse aucune
    // place à ce geste.
    const attentePaiement = page.waitForResponse(
      (r) => r.url().includes("/mark-paid") && r.request().method() === "PUT",
    );
    await card.getByTestId("mark-paid-button").click();
    await confirmerSiDemande(page);
    const paidResp = await attentePaiement;
    expect(paidResp.status()).toBe(200);
  });
});
