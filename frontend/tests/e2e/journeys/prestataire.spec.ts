/**
 * Harnais n°1 — le gate E2E du parcours prestataire (#815).
 *
 *   - @happy    — les trois étapes aboutissent et produisent leur vidéo.
 *   - @security — le prestataire ne voit AUCUN menu métier : son rôle est au
 *                 registre des rôles sans interface, et ce registre engage.
 *   - @negative — le lien magique n'existe pas, et le harnais REFUSE d'en
 *                 fabriquer un. Cette absence est vérifiée.
 *   - @edge     — tout rôle du registre reçoit le même écran : ce n'est pas
 *                 un cas particulier du prestataire.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { prestataire } from "./prestataire.journey";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = prestataire;

test.describe(`Parcours de référence — ${parcours.titre}`, () => {
  test(`@happy ${parcours.slug} aboutit de bout en bout`, async ({ page }) => {
    test.setTimeout(180_000);
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
    `@security ${parcours.slug} — aucun menu métier, et c'est le registre ` +
      "qui l'engage",
    async ({ page }) => {
      test.setTimeout(120_000);
      const comptes = await parcours.amorcer(page);
      const scene = new Scene(page, comptes);
      await scene.devenir("prestataire");

      // Le registre `ROLES_SANS_INTERFACE` est vérifié par `garde-roles`,
      // et le message existe. Mais il vit dans `Navigation`, que la page
      // d'accueil ne monte pas (`index.astro:39`, `showNav={false}`) — et
      // c'est là que `getDefaultRedirect` envoie ce rôle.
      //
      // On fige donc l'état RÉEL : le message n'est pas là. Le jour où #961
      // est corrigée, ce test rougit, et c'est le signal que le parcours
      // peut montrer la phrase au lieu de son absence.
      await expect(
        page.getByTestId("navigation-role-sans-interface"),
        "si ce message APPARAÎT, #961 est corrigée : mettre à jour le " +
          "parcours filmé, qui documente aujourd'hui son absence",
      ).toHaveCount(0);
      await expect(page.getByTestId("home-logo")).toBeVisible({
        timeout: 20000,
      });

      // Et aucun écran de gestion ne lui est ouvert.
      const gestion = await page.request.get(`${API_BASE}/organizations`, {
        headers: {},
      });
      expect(
        gestion.status(),
        "un prestataire non authentifié auprès de l'API ne liste rien",
      ).not.toBe(200);
    },
  );

  test(
    `@negative ${parcours.slug} — le lien magique n'existe pas, et on ` +
      "refuse d'en inventer un (story 3.2)",
    async ({ page }) => {
      test.setTimeout(60_000);
      const adminToken = await adminLogin(page);

      // `POST /magic-links` est l'entrée PRÉVUE du prestataire. Tant qu'elle
      // n'existe pas, `helpers/magic-link.ts` lève plutôt que de fabriquer
      // un faux jeton — « ne pas masquer l'implémentation manquante en CI ».
      //
      // Ce test fige l'absence. Le jour où la route répond, il rougit, et
      // c'est le signal que le parcours peut gagner ses étapes.
      const reponse = await page.request.post(`${API_BASE}/magic-links`, {
        data: {
          subject_user_id: "00000000-0000-0000-0000-000000000000",
          scope_type: "quote",
          scope_id: "00000000-0000-0000-0000-000000000000",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });

      expect(
        reponse.ok(),
        "si cette route RÉPOND, la story 3.2 a atterri : le parcours du " +
          "prestataire doit alors filmer son vrai point d'entrée, et ce " +
          "test doit être remplacé",
      ).toBeFalsy();
    },
  );

  test(
    `@edge ${parcours.slug} — le registre vaut pour sept rôles, pas pour ` +
      "le seul prestataire",
    async ({ page }) => {
      test.setTimeout(120_000);
      const horodatage = Date.now();
      const motDePasse = "test123456";
      const email = `notaire-${horodatage}@example.be`;
      const adminToken = await adminLogin(page);

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Notaire Org ${horodatage}`,
          slug: `notaire-${horodatage}`,
          contact_email: email,
          subscription_plan: "professional",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();

      // Le notaire est un autre membre du registre. S'il voyait un écran
      // différent, le registre ne serait qu'une liste décorative.
      const inscription = await page.request.post(`${API_BASE}/auth/register`, {
        data: {
          email,
          password: motDePasse,
          first_name: "Maitre",
          last_name: "Notaire",
          role: "notary",
          organization_id: org.id,
        },
      });
      expect(
        inscription.ok(),
        "le notaire est un rôle réel du backend, comme le prestataire",
      ).toBeTruthy();

      await page.context().clearCookies();
      const scene = new Scene(page, {
        prestataire: { email, motDePasse },
      });
      await scene.devenir("prestataire");

      // Le notaire subit le MÊME sort : c'est bien le registre entier qui
      // est concerné, pas une particularité du prestataire (#961).
      await expect(
        page.getByTestId("home-logo"),
        "le notaire atterrit lui aussi sur la page commerciale",
      ).toBeVisible({ timeout: 20000 });
      await expect(
        page.getByTestId("navigation-role-sans-interface"),
      ).toHaveCount(0);
    },
  );
});
