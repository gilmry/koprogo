/**
 * Harnais n°1 — le gate E2E du parcours de modération (#805, #962).
 *
 *   - @happy    — les quatre étapes aboutissent et produisent leur vidéo.
 *   - @security — les SEPT routes communautaires le refusent, pas seulement
 *                 celles que le parcours visite.
 *   - @negative — `permissions.ts` et `guards.ts` se contredisent, et cet
 *                 écart est figé plutôt que découvert une seconde fois.
 *   - @edge     — le backend, lui, délivre bien ce rôle : l'écart est
 *                 entièrement côté interface.
 */
import { test, expect } from "@playwright/test";
import { Scene } from "./scene";
import { moderation } from "./moderation.journey";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

const parcours = moderation;

/** Les sept routes que `permissions.ts` promet au groupe `communaute`. */
const ROUTES_COMMUNAUTAIRES = [
  "/exchanges",
  "/polls",
  "/notices",
  "/bookings",
  "/sharing",
  "/skills",
  "/energy-campaigns",
] as const;

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
    `@security ${parcours.slug} — les SEPT routes communautaires le ` +
      "refusent, pas seulement trois (#962)",
    async ({ page }) => {
      test.setTimeout(240_000);
      const comptes = await parcours.amorcer(page);
      const scene = new Scene(page, comptes);
      await scene.devenir("modérateur");

      const refusees: string[] = [];
      for (const route of ROUTES_COMMUNAUTAIRES) {
        await page.goto(route);
        await page.waitForTimeout(2500);
        if (!page.url().includes(route)) refusees.push(route);
      }

      // On fige l'ÉTENDUE de l'écart, pas seulement son existence. Le
      // parcours filmé n'en visite que trois ; si les quatre autres se
      // comportaient différemment, le diagnostic de #962 serait faux.
      expect(
        refusees.sort(),
        "les sept routes du groupe `communaute` doivent se comporter de la " +
          "même façon : c'est ce qui prouve que le défaut est dans la table " +
          "de `guards.ts`, et non dans un écran particulier",
      ).toEqual([...ROUTES_COMMUNAUTAIRES].sort());
    },
  );

  test(
    `@negative ${parcours.slug} — l'écart entre permissions.ts et guards.ts ` +
      "est figé, pas redécouvert",
    async () => {
      const { readFileSync } = await import("node:fs");
      const { join } = await import("node:path");

      const permissions = readFileSync(
        join(process.cwd(), "src", "lib", "auth", "permissions.ts"),
        "utf8",
      );
      const guards = readFileSync(
        join(process.cwd(), "src", "lib", "guards.ts"),
        "utf8",
      );

      // `permissions.ts` promet le groupe `communaute` au modérateur…
      expect(
        permissions,
        "si cette promesse disparaît, #962 a été tranchée dans le sens " +
          "« le rôle n'est pas prêt » — mettre à jour le parcours filmé",
      ).toContain("community.moderator");

      // …et `guards.ts` ne le laisse pas passer.
      const blocNotices = guards.slice(
        guards.indexOf('"/notices": ['),
        guards.indexOf('"/notices": [') + 200,
      );
      expect(
        blocNotices.includes("COMMUNITY_MODERATOR"),
        "si le modérateur apparaît ici, #962 a été tranchée dans l'autre " +
          "sens : le parcours filmé doit montrer les modules, pas leur refus",
      ).toBe(false);
    },
  );

  test(
    `@edge ${parcours.slug} — le backend délivre bien ce rôle : l'écart est ` +
      "entièrement côté interface",
    async ({ page }) => {
      test.setTimeout(120_000);
      const horodatage = Date.now();
      const email = `mod-backend-${horodatage}@example.be`;
      const adminToken = await adminLogin(page);

      const orgResp = await page.request.post(`${API_BASE}/organizations`, {
        data: {
          name: `Mod Backend ${horodatage}`,
          slug: `mod-backend-${horodatage}`,
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
          password: "test123456",
          first_name: "Marie",
          last_name: "Moderatrice",
          role: "community.moderator",
          organization_id: org.id,
        },
      });

      expect(
        inscription.ok(),
        "le serveur accepte ce rôle : ce n'est pas un rôle fantôme comme " +
          "`admin` (#960), c'est un rôle réel qu'une table d'interface oublie",
      ).toBeTruthy();

      const corps = await inscription.json();
      expect(corps.user?.role ?? corps.role).toBe("community.moderator");
    },
  );
});
