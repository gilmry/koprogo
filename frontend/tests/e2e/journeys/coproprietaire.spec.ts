/**
 * Harnais n°1 — le gate E2E du parcours copropriétaire (#807).
 *
 * Même principe que `parcours.spec.ts` : ce fichier ne redit rien du
 * parcours, il l'importe et le rejoue à la vitesse. Les quatre classes de
 * tests exigées par la story C8.1 vivent ici, chacune vérifiant une facette
 * différente du même parcours :
 *
 *   - @happy    — les dix étapes aboutissent et produisent leur vidéo.
 *   - @negative — les deux points où l'écran est cassé AUJOURD'HUI sont
 *                 caractérisés précisément, pas juste évoqués en prose.
 *   - @edge     — un compte owner tout neuf, sans fiche ni lot, affiche un
 *                 premier écran utilisable — pas une erreur technique.
 *   - @security — la vitrine ne contient aucune adresse email littérale :
 *                 une galerie publiée est un document public.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { coproprietaire } from "./coproprietaire.journey";
import { adminLogin, uiLoginWithRetry } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = coproprietaire;

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
    `@negative ${parcours.slug} — le montant par ACP ne suit pas encore la ` +
      "quote-part du lot (#807)",
    async ({ page }) => {
      test.setTimeout(120_000);
      const comptes = await parcours.amorcer(page);

      const loginResp = await page.request.post(`${API_BASE}/auth/login`, {
        data: {
          email: comptes.copropriétaire!.email,
          password: comptes.copropriétaire!.motDePasse,
        },
      });
      expect(loginResp.ok()).toBeTruthy();
      const { token } = await loginResp.json();

      const duesResp = await page.request.get(
        `${API_BASE}/stats/owner/dues-by-acp`,
        { headers: { Authorization: `Bearer ${token}` } },
      );
      expect(duesResp.ok()).toBeTruthy();
      const dettes: Array<{ acp_name: string; montant: number }> =
        await duesResp.json();

      // Ce que la formule (quotité ÷ tantièmes × charge) rendrait pour un
      // lot à 120/1000 sur une charge de 1200 : 144, pas 1200.
      const erables = dettes.find((d) => d.acp_name.includes("Erables"));
      expect(erables).toBeTruthy();

      // ── L'assertion documente le défaut, elle ne le cache pas ──────────
      //
      // `get_owner_dues_by_acp` (stats_repository_impl.rs) somme
      // `expenses.amount` pour tout immeuble où le copropriétaire détient
      // NE SERAIT-CE QU'UN lot, sans jamais multiplier par sa quotité. Un
      // copropriétaire à 12 % des tantièmes voit donc 100 % de la charge en
      // attente de son immeuble, pas sa part. `montant` vaut ici 1200 (le
      // total de la charge semée par `amorcer()`), alors que la quote-part
      // réelle du lot serait 1200 × 120/1000 = 144.
      //
      // Cette assertion NE VÉRIFIE PAS le comportement souhaité — elle fixe
      // le comportement observé, pour qu'un correctif futur la fasse échouer
      // et force sa mise à jour délibérée (#807). La supprimer sans la
      // comprendre serait exactement ce que `.claude/rules/CRITICAL.md`
      // interdit.
      expect(erables!.montant).toBe(1200);
    },
  );

  test(
    `@negative ${parcours.slug} — payer un appel de fonds reste hors de ` +
      "portée du copropriétaire (#807)",
    async ({ page }) => {
      test.setTimeout(120_000);
      const comptes = await parcours.amorcer(page);
      await uiLoginWithRetry(
        page,
        comptes.copropriétaire!.email,
        comptes.copropriétaire!.motDePasse,
        /\/owner/,
      );

      // Le seul formulaire de paiement connu du contrat gelé vit sur cette
      // route, réservée SYNDIC/ACCOUNTANT (`frontend/src/lib/guards.ts`). Le
      // garde renvoie tout copropriétaire vers `/owner` — c'est ce qui se
      // passe RÉELLEMENT, documenté ici plutôt que supposé.
      await page.goto("/owner-contributions", { waitUntil: "domcontentloaded" });
      await page.waitForURL(/\/owner$/, { timeout: 10_000 });
      await expect(
        page.getByTestId("owner-contribution-payment-form"),
      ).toHaveCount(0);
    },
  );

  test(
    `@negative ${parcours.slug} — prêter un objet appelle une route que le ` +
      "backend ne sert pas (#779)",
    async ({ page }) => {
      test.setTimeout(120_000);
      const comptes = await parcours.amorcer(page);

      const loginResp = await page.request.post(`${API_BASE}/auth/login`, {
        data: {
          email: comptes.copropriétaire!.email,
          password: comptes.copropriétaire!.motDePasse,
        },
      });
      const { token } = await loginResp.json();

      // `frontend/src/lib/api/sharing.ts:170` (`createLoan`) poste ici. Le
      // backend n'expose aucune route `/loans` : il sert le prêt via
      // `POST /shared-objects/{id}/borrow`. Tant que ce désaccord de chemin
      // n'est pas corrigé (#779), cet appel échoue TOUJOURS, quel que soit
      // l'objet demandé — d'où l'absence de corps réaliste ci-dessous.
      const loanResp = await page.request.post(`${API_BASE}/loans`, {
        data: {},
        headers: { Authorization: `Bearer ${token}` },
      });
      expect(loanResp.status()).toBe(404);
    },
  );

  test(
    `@edge ${parcours.slug} — un compte copropriétaire tout neuf affiche un ` +
      "premier écran utilisable",
    async ({ page }) => {
      test.setTimeout(60_000);
      const horodatage = Date.now();
      const email = `coproprietaire-vide-${horodatage}@example.com`;
      const motDePasse = "test123456";

      const adminToken = await adminLogin(page);
      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Coproprietaire Vide Org ${horodatage}`,
          slug: `coproprietaire-vide-${horodatage}`,
          contact_email: email,
          subscription_plan: "professional",
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      const org = await orgResp.json();

      // Ni fiche `owners`, ni lot : c'est exactement l'état d'un compte tout
      // juste créé, avant que le syndic ne l'ait rattaché à quoi que ce
      // soit — l'écran que « personne ne teste et que tout le monde voit ».
      const regResp = await page.request.post(`${API_BASE}/auth/register`, {
        data: {
          email,
          password: motDePasse,
          first_name: "Nouveau",
          last_name: `Coproprietaire${horodatage}`,
          role: "owner",
          organization_id: org.id,
        },
      });
      expect(regResp.ok()).toBeTruthy();

      await uiLoginWithRetry(page, email, motDePasse, /\/owner/);
      await page.goto("/owner", { waitUntil: "networkidle" });

      await expect(page.getByTestId("owner-dashboard")).toBeVisible();
      // Pas d'erreur technique brute affichée à la place de l'accueil (R0-6,
      // cf. `docs/tests/BRIEF-COWORK-recette-5.md` §4 — jamais revérifié
      // depuis, et c'est le premier écran que voit un prospect).
      await expect(page.locator("body")).not.toContainText("panic");
      await expect(page.locator("body")).not.toContainText("undefined");
    },
  );

  test(
    `@security ${parcours.slug} — la narration filmée ne contient aucune ` +
      "adresse email littérale",
    async ({ page }) => {
      test.setTimeout(240_000);
      const comptes = await parcours.amorcer(page);
      const scene = new Scene(page, comptes);

      for (const etape of parcours.etapes) {
        await etape.action(scene);
        if (etape.assertion) await etape.assertion(page);
      }

      // Même détecteur que `garde_documentation_personas.rs`
      // (`aucune_adresse_email_litterale_dans_les_documents_persona`) : une
      // vitrine publiée est un document public, les comptes de recette
      // n'y figurent que par leur prénom fictif (Carine, Sophie).
      const ressembleAUnEmail = (texte: string): boolean =>
        /[a-z0-9._-]+@[a-z0-9.-]+\.[a-z]{2,}/i.test(texte);

      const fautes = scene.narration.filter((c) => ressembleAUnEmail(c.texte));
      expect(
        fautes,
        `Narration contenant une adresse email : ${JSON.stringify(fautes)}`,
      ).toHaveLength(0);
    },
  );
});
