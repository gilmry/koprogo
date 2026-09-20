/**
 * SCENARIO: Administration de la plateforme par le SuperAdmin
 *
 * Documentation Vivante — video exploitable pour YouTube.
 * Montre le parcours complet d'un administrateur :
 *   1. Connexion via le formulaire login
 *   2. Navigation vers la liste des Organisations
 *   3. Consultation de la liste des organisations
 *   4. Navigation vers la liste des Immeubles
 *   5. Consultation de la liste des immeubles
 *   6. Navigation vers la liste des Utilisateurs
 *   7. Consultation de la liste des utilisateurs
 *   8. Pause finale sur le tableau de bord admin
 *
 * Duree video attendue : ~40-50 secondes (rythme humain)
 */
import { test, expect } from "@playwright/test";
import { ADMIN_PASSWORD } from "../helpers/identifiants";
import { amorce } from "../helpers/amorcage";
import {
  humanLogin,
  humanClick,
  humanGoto,
  waitForSpinner,
  stepPause,
  finalPause,
  PACE,
} from "../helpers/video-pace";

import { API_BASE } from "../helpers/adresses";

test.describe("Scenario: Le SuperAdmin explore la plateforme", () => {
  test.setTimeout(120_000);

  let seedData: any;

  test.beforeAll(async ({ request }) => {
    // 1. Login admin
    const adminResp = await request.post(`${API_BASE}/auth/login`, {
      data: { email: "admin@koprogo.com", password: ADMIN_PASSWORD },
    });
    const admin = await amorce(adminResp, "POST /auth/login");
    const adminHeaders = { Authorization: `Bearer ${admin.token}` };

    // 2. Seed the world (creates orgs, buildings, users — rich data for admin to explore)
    const seedResp = await request.post(`${API_BASE}/seed/scenario/world`, {
      headers: adminHeaders,
    });
    if (!seedResp.ok()) {
      console.log("Seed world already exists, continuing...");
    } else {
      seedData = await seedResp.json();
      seedData = seedData.data;
    }
  });

  // PAS de suppression du monde de scénario.
  //
  // Il est PARTAGÉ : huit fichiers le sèment, dix emploient ses comptes, et
  // ce même bloc de teardown était copié dans QUATORZE d'entre eux. Chacun
  // détruisait donc la précondition des autres.
  //
  // Le semer coûte 44 s, le supprimer une seconde. Pendant une campagne, la
  // spec suivante devait le reconstruire contre un plafond de requête de
  // 10 s : elle échouait, et ses tests en série étaient sautés. C'est ce qui
  // rendait `AccessibiliteEcransAuthentifies` inexécutable, alors que son
  // propre `beforeAll` est tolérant et dit même que le monde « peut déjà
  // exister, semé par un autre fichier de la même campagne ».
  //
  // Le monde est un scénario FIXE (« Résidence du Parc Royal ») : le laisser
  // en place n'accumule rien. Le nettoyage, si on en veut un, appartient à un
  // `globalTeardown` — c'est-à-dire à quelqu'un qui possède le fixture.
  // Cf. #942 et #876.

  test("@happy Le SuperAdmin consulte organisations, immeubles et utilisateurs", async ({
    page,
  }) => {
    // ============================================================
    // ETAPE 1 : Connexion (visible dans la video)
    // ============================================================
    await humanLogin(page, "admin@koprogo.com", ADMIN_PASSWORD);
    await stepPause(page);

    // ============================================================
    // ETAPE 2 : Navigation vers les Organisations
    // ============================================================
    await humanClick(page, "nav-link-admin-organizations");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Organisations est chargee
    await expect(page.getByTestId("organizations-table-body")).toBeVisible({
      timeout: 15000,
    });

    // Verifier qu'au moins une organisation apparait
    await expect(page.getByTestId("organization-row").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 3 : Navigation vers les Immeubles
    // ============================================================
    //
    // Par le tableau de bord, pas par la barre laterale.
    //
    // `canSee` (permissions.ts) ne montre les menus METIER — gestion,
    // compta, gouvernance, communaute, ticketing — a un superadmin QUE
    // s'il a selectionne un immeuble (« mode in-context »). Sans
    // selection, il est en mode plateforme et ne voit que le menu `admin`.
    // `nav-link-buildings` n'existe donc pas dans son DOM, et le scenario
    // attendait trente secondes un lien que la refonte a rendu
    // conditionnel.
    //
    // Le chemin prevu est la tuile `admin-buildings-tile` du tableau de
    // bord admin. On y retourne d'abord, ce qui est aussi le geste reel :
    // un superadmin revient a son tableau de bord entre deux ecrans.
    await humanClick(page, "nav-link-admin");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);
    await humanClick(page, "admin-buildings-tile");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Immeubles est chargee
    await expect(page.locator("main h1").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 4 : Navigation vers les Utilisateurs
    // ============================================================
    await humanClick(page, "nav-link-admin-users");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    // Verifier que la page Utilisateurs est chargee
    await expect(page.getByTestId("users-table-body")).toBeVisible({
      timeout: 15000,
    });

    // Verifier qu'au moins un utilisateur apparait
    await expect(page.getByTestId("user-row").first()).toBeVisible({
      timeout: 10000,
    });

    await stepPause(page);

    // ============================================================
    // ETAPE 5 : Retour au tableau de bord admin
    // ============================================================
    await humanGoto(page, "/admin");
    await waitForSpinner(page);
    await page.waitForTimeout(PACE.AFTER_NAVIGATION);

    await stepPause(page);

    // ============================================================
    // FIN : Pause finale pour que la video montre le resultat
    // ============================================================
    await finalPause(page);
  });
});
