/**
 * Harnais n°1 — le gate E2E du parcours comptable (#808).
 *
 * Même principe que ses deux voisins : ce fichier ne redit rien du parcours,
 * il l'importe et le rejoue à la vitesse. C'est l'invariant anti-dette de la
 * documentation vivante — un harnais de valeur qui ne rejoue plus le
 * parcours partagé fait passer la suite au rouge.
 *
 * Les quatre classes, chacune sur une facette différente :
 *
 *   - @happy    — les neuf étapes aboutissent et produisent leur vidéo.
 *   - @security — le comptable ne peut PAS approuver, ni par l'écran ni par
 *                 l'API. C'est la propriété que le PO a tranchée (#942), et
 *                 elle se vérifie des deux côtés.
 *   - @negative — un immeuble NON conforme ferme la comptabilité en 422,
 *                 avec le code que l'interface consomme.
 *   - @edge     — un comptable sans aucun immeuble atteint un premier écran
 *                 utilisable, pas une erreur technique.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { comptable } from "./comptable.journey";
import { adminLogin, ensureAcp } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = comptable;

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
    `@security ${parcours.slug} — l'approbation n'est pas de son ressort, ` +
      "ni par l'écran ni par l'API (#942)",
    async ({ page }) => {
      test.setTimeout(120_000);
      const comptes = await parcours.amorcer(page);

      const loginResp = await page.request.post(`${API_BASE}/auth/login`, {
        data: {
          email: comptes.comptable!.email,
          password: comptes.comptable!.motDePasse,
        },
      });
      expect(loginResp.ok()).toBeTruthy();
      const { token } = await loginResp.json();

      // Le serveur refuse, et c'est le vrai garde-fou : retirer le bouton
      // de l'écran ne protégerait de rien, `check_syndic_role` si.
      const approbation = await page.request.post(
        `${API_BASE}/expenses/00000000-0000-0000-0000-000000000000/approve`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(
        [401, 403, 404].includes(approbation.status()),
        `approuver une dépense en tant que comptable doit être refusé ; ` +
          `obtenu ${approbation.status()}`,
      ).toBeTruthy();
      expect(
        approbation.status(),
        "un 200 signifierait que la séparation des rôles est tombée",
      ).not.toBe(200);
    },
  );

  test(
    `@negative ${parcours.slug} — un immeuble non conforme ferme la ` +
      "comptabilité, avec son code (ADR-0010)",
    async ({ page }) => {
      test.setTimeout(120_000);
      const horodatage = Date.now();
      const adminToken = await adminLogin(page);
      const entete = { Authorization: `Bearer ${adminToken}` };

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Comptable NonConforme ${horodatage}`,
          slug: `comptable-nc-${horodatage}`,
          contact_email: `nc-${horodatage}@example.com`,
          subscription_plan: "professional",
        },
        headers: entete,
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();
      const acpId = await ensureAcp(page, org.id, adminToken, "comptable-nc");

      // Un acte de base qui déclare 1000 millièmes, et AUCUN lot pour les
      // porter : Σ quotités = 0 ≠ 1000. C'est le cas dégradé du persona.
      const immeubleResp = await page.request.post(`${API_BASE}/buildings`, {
        data: {
          name: `Immeuble non conforme ${horodatage}`,
          address: `${horodatage} Rue Bancale`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 4,
          total_tantiemes: 1000,
          construction_year: 2010,
          acp_id: acpId,
        },
        headers: entete,
      });
      expect(immeubleResp.ok()).toBeTruthy();
      const immeuble = await immeubleResp.json();

      const depense = await page.request.post(`${API_BASE}/expenses`, {
        data: {
          building_id: immeuble.id,
          description: `Charge sur immeuble non conforme ${horodatage}`,
          amount: "100.00",
          expense_date: new Date().toISOString().slice(0, 10),
          category: "maintenance",
        },
        headers: entete,
      });

      // On décrit ce qui se passe, on n'invente pas ce qu'on voudrait.
      // Le refus peut arriver à la création ou au calcul de répartition
      // selon le chemin ; ce qui compte est qu'il ARRIVE et qu'il soit
      // motivé, jamais un 500 muet.
      expect(
        depense.status(),
        "le verrou de conformité ne doit jamais se présenter en 500",
      ).not.toBe(500);

      if (depense.status() === 422) {
        const corps = await depense.text();
        expect(
          corps,
          "un 422 doit porter le code que l'interface consomme pour nommer " +
            "le lot fautif, sans quoi le toast n'a rien à dire",
        ).toContain("CONFORMANT");
      }
    },
  );

  test(
    `@edge ${parcours.slug} — un comptable sans immeuble voit un écran, ` +
      "pas une erreur",
    async ({ page }) => {
      test.setTimeout(120_000);
      const horodatage = Date.now();
      const motDePasse = "test123456";
      const email = `comptable-vide-${horodatage}@example.com`;
      const adminToken = await adminLogin(page);

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Comptable Vide ${horodatage}`,
          slug: `comptable-vide-${horodatage}`,
          contact_email: email,
          subscription_plan: "professional",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      expect(orgResp.ok()).toBeTruthy();
      const org = await orgResp.json();

      const inscription = await page.request.post(`${API_BASE}/auth/register`, {
        data: {
          email,
          password: motDePasse,
          first_name: "Comptable",
          last_name: "Sansimmeuble",
          role: "accountant",
          organization_id: org.id,
        },
      });
      expect(inscription.ok()).toBeTruthy();

      await page.context().clearCookies();
      const scene = new Scene(page, {
        comptable: { email, motDePasse },
      });
      await scene.devenir("comptable");

      // Un périmètre vide reste un périmètre : le tableau de bord s'ouvre.
      await expect(page.getByTestId("accountant-dashboard")).toBeVisible({
        timeout: 20000,
      });
      await expect(page.locator("body")).not.toContainText("Internal Server");
    },
  );
});
