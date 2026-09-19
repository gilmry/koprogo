/**
 * Parcours de référence — le modérateur communauté, et la porte qu'il ne
 * peut pas franchir (#805, #962).
 *
 * ── Ce que ce parcours a découvert ────────────────────────────────────────
 *
 * Le rôle `community.moderator` existe des deux côtés : le backend l'accepte
 * (inscription en 201), et `types.ts:29` le déclare. Son persona le décrit
 * comme « prévu, non construit », avec cette nuance :
 *
 *   « Ne dispose d'aucune action de modération DISTINCTE de celles d'un
 *     owner aujourd'hui. »
 *
 * C'était optimiste. Il ne dispose pas non plus de celles d'un
 * copropriétaire : **il n'atteint aucun module communautaire.**
 *
 *     [RouteGuard] Access denied to /notices for role community.moderator
 *
 * Deux fichiers se contredisent. `permissions.ts:212` annonce
 * « community.moderator → comme owner pour `communaute` » et pilote les
 * MENUS. `guards.ts` pilote l'ACCÈS, et ses sept routes communautaires ne
 * listent que superadmin, syndic, comptable et copropriétaire.
 *
 * Le rôle voit une porte qu'il ne peut pas franchir (#962).
 *
 * ── Pourquoi ce parcours vaut d'être filmé malgré tout ────────────────────
 *
 * Parce que c'est ainsi qu'on l'a su. Aucun compte `community.moderator`
 * n'existait en recette, aucune spec ne s'y connectait — le rôle n'avait
 * jamais été exercé. Écrire son parcours a suffi.
 *
 * Le filmer, c'est donner à voir ce que vit quelqu'un à qui on a donné un
 * rôle que le produit ne sert pas. Et le jour où #962 est tranchée, ce
 * parcours gagne des étapes au lieu d'être réécrit.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import { adminLogin, ensureAcp } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

async function ok<T = any>(
  resp: {
    ok: () => boolean;
    status: () => number;
    text: () => Promise<string>;
    json: () => Promise<any>;
  },
  libelle: string,
): Promise<T> {
  if (!resp.ok()) {
    throw new Error(
      `${libelle} : HTTP ${resp.status()} — ${(await resp.text()).slice(0, 200)}`,
    );
  }
  return (await resp.json()) as T;
}

export const moderation: Parcours = {
  slug: "moderation",
  titre: "Le modérateur communauté, et la porte qu'il ne peut pas franchir",
  propos:
    "Montre ce que vit quelqu'un à qui on a donné un rôle que le produit " +
    "ne sert pas encore. Le compte existe, la connexion aboutit, et chaque " +
    "module communautaire le renvoie à l'accueil.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const email = `moderation-${horodatage}@example.be`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Moderation Org ${horodatage}`,
          slug: `moderation-${horodatage}`,
          contact_email: email,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );
    await ensureAcp(page, org.id, adminToken, "moderation");

    await ok(
      await api.post(`${API_BASE}/auth/register`, {
        data: {
          email,
          password: motDePasse,
          first_name: "Marie",
          last_name: "Moderatrice",
          role: "community.moderator",
          organization_id: org.id,
        },
      }),
      "amorçage:modérateur",
    );

    await page.context().clearCookies();
    return { modérateur: { email, motDePasse } };
  },

  etapes: [
    {
      id: "1-un-role-que-le-serveur-delivre",
      acteur: "modérateur",
      description:
        "KoproGo — le modérateur communauté. Celui qui veille sur les " +
        "échanges entre voisins : le SEL, les annonces, les objets prêtés. " +
        "Le serveur délivre bien ce rôle, et la connexion aboutit.",
      action: async (scene) => {
        await scene.devenir("modérateur");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("home-logo")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-les-annonces-lui-sont-refusees",
      acteur: "modérateur",
      description:
        "Les annonces de la copropriété — ce qu'il est censé modérer. " +
        "L'écran ne s'ouvre pas : il est renvoyé à l'accueil, sans un mot.",
      action: async (scene) => {
        await scene.aller("/notices");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // On DÉCRIT ce qui se passe. La console du navigateur dit
        // « Access denied to /notices for role community.moderator » —
        // `guards.ts` n'autorise ces routes qu'à quatre rôles, et le
        // modérateur n'en fait pas partie, alors que `permissions.ts:212`
        // affirme le contraire (#962).
        await expect(page.getByTestId("notices-list")).toHaveCount(0);
        await expect(page.getByTestId("home-logo")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "3-et-les-sondages-aussi",
      acteur: "modérateur",
      description:
        "Les sondages : même porte, même refus. Ce n'est pas un écran " +
        "cassé, c'est un rôle que deux fichiers décrivent différemment.",
      action: async (scene) => {
        await scene.aller("/polls");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("polls-list")).toHaveCount(0);
      },
    },
    {
      id: "4-ce-que-ca-coute",
      acteur: "modérateur",
      description:
        "Un rôle qu'on attribue et qui ne donne accès à rien coûte plus " +
        "qu'un rôle absent : la personne croit pouvoir agir, et cherche " +
        "l'erreur chez elle.",
      action: async (scene) => {
        await scene.aller("/exchanges");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("exchanges-list")).toHaveCount(0);
      },
    },
  ],
};
