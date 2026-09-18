/**
 * Harnais n°1 — le gate E2E du parcours d'administration (#809).
 *
 * Les quatre classes :
 *
 *   - @happy    — les cinq étapes aboutissent et produisent leur vidéo.
 *   - @security — le rôle `admin` que le frontend croit connaître n'existe
 *                 pas côté backend : aucun compte ne peut le porter (#960).
 *                 Cette absence est vérifiée, pas supposée.
 *   - @negative — un compte NON administrateur n'atteint pas ces écrans.
 *   - @edge     — le superadmin n'appartient à aucune organisation, et c'est
 *                 une contrainte de conception, pas une omission.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { administration } from "./administration.journey";
import { ADMIN_EMAIL, ADMIN_PASSWORD } from "../helpers/identifiants";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = administration;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(240_000);
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
    `@security ${parcours.slug} — le rôle « admin » ne peut être porté par ` +
      "aucun compte (#960)",
    async ({ page }) => {
      test.setTimeout(60_000);
      const horodatage = Date.now();

      // `permissions.ts:96` pose ADMIN_ROLES = {"superadmin", "admin"}.
      // `UserRole::from_str` n'accepte pas "admin". Ce test fige l'écart
      // plutôt que de le laisser se découvrir en filmant.
      const inscription = await page.request.post(`${API_BASE}/auth/register`, {
        data: {
          email: `faux-admin-${horodatage}@example.com`,
          password: "SecurePass123!",
          first_name: "Faux",
          last_name: "Admin",
          role: "admin",
        },
      });

      expect(
        inscription.ok(),
        "si cette inscription RÉUSSIT, le rôle `admin` est devenu réel et " +
          "#960 doit être relu : le persona, la vitrine et ADMIN_ROLES " +
          "doivent alors être mis d'accord",
      ).toBeFalsy();
    },
  );

  test(
    `@negative ${parcours.slug} — un compte sans privilège n'atteint pas ` +
      "l'administration",
    async ({ page }) => {
      test.setTimeout(120_000);
      const horodatage = Date.now();
      const motDePasse = "test123456";
      const email = `non-admin-${horodatage}@example.com`;

      const adminToken = await adminLogin(page);
      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Non Admin ${horodatage}`,
          slug: `non-admin-${horodatage}`,
          contact_email: email,
          subscription_plan: "professional",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();

      await page.request.post(`${API_BASE}/auth/register`, {
        data: {
          email,
          password: motDePasse,
          first_name: "Simple",
          last_name: "Copropriétaire",
          role: "owner",
          organization_id: org.id,
        },
      });

      const connexion = await page.request.post(`${API_BASE}/auth/login`, {
        data: { email, password: motDePasse },
      });
      expect(connexion.ok()).toBeTruthy();
      const { token } = await connexion.json();

      // La liste de TOUTES les organisations est le geste le plus
      // révélateur : un copropriétaire qui l'obtiendrait verrait les
      // cabinets concurrents de son syndic.
      const fuite = await page.request.get(`${API_BASE}/organizations`, {
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(
        fuite.status(),
        "un copropriétaire ne doit pas lister les organisations de " +
          "l'instance entière",
      ).toBe(403);
    },
  );

  test(
    `@edge ${parcours.slug} — le superadmin n'appartient à aucune ` +
      "organisation, par conception",
    async ({ page }) => {
      test.setTimeout(60_000);

      const connexion = await page.request.post(`${API_BASE}/auth/login`, {
        data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
      });
      expect(connexion.ok()).toBeTruthy();
      const { user } = await connexion.json();

      // `docs/personas/superadmin.md` : « c'est une contrainte de
      // conception, pas une omission — un compte qui gère la plateforme ne
      // doit pas être également partie prenante d'une des organisations
      // qu'il administre. »
      expect(
        user.organization_id ?? null,
        "le superadmin rattaché à une organisation serait juge et partie",
      ).toBeNull();
    },
  );
});
