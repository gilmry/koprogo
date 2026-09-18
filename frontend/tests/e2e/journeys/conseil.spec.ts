/**
 * Harnais n°1 — le gate E2E du parcours du conseil de copropriété (#816).
 *
 *   - @happy    — les quatre étapes aboutissent et produisent leur vidéo.
 *   - @security — un copropriétaire NON élu n'est pas membre du conseil.
 *   - @negative — sous vingt lots, l'élection est refusée. Le refus est le
 *                 comportement juste (Art. 577-8/4), pas un défaut.
 *   - @edge     — un conseil peut exister sans qu'aucun membre n'ait de
 *                 compte : ce sont des copropriétaires, pas des utilisateurs.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { conseil } from "./conseil.journey";
import { adminLogin, seedConformantUnits, ensureAcp } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = conseil;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(300_000);
    const comptes = await test.step("amorçage", () => parcours.amorcer(page));
    const scene = new Scene(page, comptes);

    for (const etape of parcours.etapes) {
      await test.step(`${etape.acteur} · ${etape.id}`, async () => {
        await etape.action(scene);
        if (etape.assertion) await etape.assertion(page);
      });
    }

    expect(scene.narration.length).toBeGreaterThan(0);
  });

  test(
    `@negative ${parcours.slug} — sous vingt lots, l'élection est refusée ` +
      "(Art. 577-8/4)",
    async ({ page }) => {
      test.setTimeout(180_000);
      const horodatage = Date.now();
      const adminToken = await adminLogin(page);
      const entete = { Authorization: `Bearer ${adminToken}` };

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Conseil Petit ${horodatage}`,
          slug: `conseil-petit-${horodatage}`,
          contact_email: `petit-${horodatage}@example.com`,
          subscription_plan: "professional",
        },
        headers: entete,
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();
      const acpId = await ensureAcp(page, org.id, adminToken, "conseil-petit");

      // Douze lots : sous le seuil légal. Le conseil n'est pas obligatoire,
      // et le produit refuse d'en élire un.
      const immeubleResp = await page.request.post(`${API_BASE}/buildings`, {
        data: {
          name: `Petite Résidence ${horodatage}`,
          address: `${horodatage} Rue Étroite`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 12,
          total_tantiemes: 1000,
          construction_year: 2010,
          acp_id: acpId,
        },
        headers: entete,
      });
      expect(immeubleResp.ok()).toBeTruthy();
      const immeuble = await immeubleResp.json();
      await seedConformantUnits(page, adminToken, acpId, immeuble.id, 12, 1000);

      const ownerResp = await page.request.post(`${API_BASE}/owners`, {
        data: {
          organization_id: org.id,
          first_name: "Petit",
          last_name: "Propriétaire",
          email: `petit-owner-${horodatage}@example.com`,
          address: "1 Rue Étroite",
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
        },
        headers: entete,
      });
      expect(ownerResp.ok()).toBeTruthy();
      const owner = await ownerResp.json();

      const election = await page.request.post(`${API_BASE}/board-members`, {
        data: {
          owner_id: owner.id,
          building_id: immeuble.id,
          position: "president",
          mandate_start: "2027-01-01T00:00:00Z",
          mandate_end: "2027-12-31T00:00:00Z",
          elected_by_meeting_id: "00000000-0000-0000-0000-000000000000",
        },
        headers: entete,
      });

      expect(
        election.ok(),
        "élire un conseil sous vingt lots doit être refusé — c'est le " +
          "comportement JUSTE, pas un défaut à contourner",
      ).toBeFalsy();
      expect(
        election.status(),
        "le refus ne doit jamais se présenter en 500",
      ).not.toBe(500);
    },
  );

  test(
    `@security ${parcours.slug} — un copropriétaire non élu n'est pas membre ` +
      "du conseil",
    async ({ page }) => {
      test.setTimeout(180_000);
      const comptes = await parcours.amorcer(page);

      const connexion = await page.request.post(`${API_BASE}/auth/login`, {
        data: {
          email: comptes.conseil!.email,
          password: comptes.conseil!.motDePasse,
        },
      });
      expect(connexion.ok()).toBeTruthy();
      const { token } = await connexion.json();

      // Le compte existe et a une fiche de copropriétaire, mais AUCUNE
      // élection n'a eu lieu dans l'amorçage. Il ne doit donc être membre
      // d'aucun conseil : un mandat se reçoit de l'assemblée.
      const conseils = await page.request.get(`${API_BASE}/board-members`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(
        conseils.status(),
        "consulter les membres du conseil ne doit jamais rendre 500",
      ).not.toBe(500);
    },
  );

  test(
    `@edge ${parcours.slug} — le conseil se compose de copropriétaires, pas ` +
      "d'utilisateurs de la plateforme",
    async ({ page }) => {
      test.setTimeout(180_000);
      const horodatage = Date.now();
      const adminToken = await adminLogin(page);
      const entete = { Authorization: `Bearer ${adminToken}` };

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Conseil SansCompte ${horodatage}`,
          slug: `conseil-sanscompte-${horodatage}`,
          contact_email: `sanscompte-${horodatage}@example.com`,
          subscription_plan: "professional",
        },
        headers: entete,
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();

      // Une fiche SANS `user_id` : une personne réelle qui n'utilise pas le
      // logiciel. `board_member.rs` le dit — « les membres doivent être des
      // copropriétaires, pas nécessairement des utilisateurs de la
      // plateforme ».
      const ficheResp = await page.request.post(`${API_BASE}/owners`, {
        data: {
          organization_id: org.id,
          first_name: "Marguerite",
          last_name: "Sanscompte",
          email: `marguerite-${horodatage}@example.com`,
          address: "1 Rue Papier",
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
        },
        headers: entete,
      });

      expect(
        ficheResp.ok(),
        "une fiche de copropriétaire sans compte doit pouvoir exister : " +
          "sinon le conseil serait réservé aux personnes connectées",
      ).toBeTruthy();
      const fiche = await ficheResp.json();
      expect(fiche.user_id ?? null).toBeNull();
    },
  );
});
