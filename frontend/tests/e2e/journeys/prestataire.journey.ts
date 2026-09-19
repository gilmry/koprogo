/**
 * Parcours de référence — le prestataire, et l'écran qu'il n'a pas (#815).
 *
 * ── Ce que ce parcours filme, et pourquoi ce n'est pas un pis-aller ───────
 *
 * Le prestataire est un rôle **réel** : `UserRole::Contractor` existe, la
 * contrainte de base l'accepte, un compte s'inscrit en 201. Ce qu'il n'a
 * pas, c'est un écran — et ce n'est pas un oubli.
 *
 * `permissions.ts:148` tient un **registre des rôles sans interface** :
 *
 *     contractor · lawyer · notary · amo · architect · bet · warden
 *
 * et `garde-roles` exige que chaque rôle du backend voie au moins un menu
 * OU figure dans ce registre. Un rôle ajouté côté serveur et oublié côté
 * interface fait échouer un test, « au lieu d'offrir un écran vide à un
 * utilisateur ». Ceux du registre reçoivent une phrase — « ceux-là méritent
 * une phrase, pas un vide » (`Navigation.svelte:110`).
 *
 * **C'est cette phrase que le parcours montre.** Un produit honnête dit à
 * quelqu'un qu'il n'a rien à faire ici, plutôt que de lui servir une barre
 * de navigation vide où il cherchera dix minutes.
 *
 * ── Ce qu'il ne filme PAS, et quand ça changera ───────────────────────────
 *
 * Le point d'entrée prévu du prestataire est un **lien magique** : le syndic
 * lui envoie une URL portant un jeton de portée, sans compte ni mot de
 * passe. `POST /magic-links` n'existe pas encore — story 3.2 — et
 * `helpers/magic-link.ts:85` LÈVE plutôt que de fabriquer un faux jeton :
 *
 *     « Do NOT silently fake a token — that would mask the missing
 *       implementation in CI. »
 *
 * Ce parcours filme donc l'état d'aujourd'hui, pas celui qu'on espère. Le
 * jour où la story 3.2 atterrit, il gagnera des étapes ; il n'aura pas à
 * être réécrit.
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

export const prestataire: Parcours = {
  slug: "prestataire",
  titre: "Le prestataire : un rôle réel, et un écran qu'il n'a pas encore",
  propos:
    "Montre ce qu'un produit honnête fait d'un rôle qu'il connaît sans " +
    "encore le servir : il le dit. Le prestataire s'inscrit, se connecte, " +
    "et reçoit une phrase plutôt qu'une barre de navigation vide.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailPrestataire = `prestataire-${horodatage}@plomberie-dupont.be`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Prestataire Org ${horodatage}`,
          slug: `prestataire-${horodatage}`,
          contact_email: emailPrestataire,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );
    await ensureAcp(page, org.id, adminToken, "prestataire");

    await ok(
      await api.post(`${API_BASE}/auth/register`, {
        data: {
          email: emailPrestataire,
          password: motDePasse,
          first_name: "Michel",
          last_name: "Dupont",
          role: "contractor",
          organization_id: org.id,
        },
      }),
      "amorçage:prestataire",
    );

    await page.context().clearCookies();
    return { prestataire: { email: emailPrestataire, motDePasse } };
  },

  etapes: [
    {
      id: "1-un-role-que-le-serveur-connait",
      acteur: "prestataire",
      description:
        "KoproGo — le prestataire. Le plombier, l'électricien, celui qui " +
        "vient réparer. Le serveur connaît ce rôle : le compte existe et " +
        "la connexion aboutit.",
      action: async (scene) => {
        await scene.devenir("prestataire");
      },
      assertion: async (page) => {
        // Il est authentifié — ce n'est pas un refus d'accès. Mais il
        // atterrit sur la page d'ACCUEIL COMMERCIALE, pas sur un écran de
        // travail : `getDefaultRedirect` renvoie `/` pour les onze rôles
        // qu'il ne connaît pas (`guards.ts:129`).
        await expect(page.getByTestId("home-logo")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-et-le-produit-le-lui-dit",
      acteur: "prestataire",
      description:
        "Et là, rien ne lui est dit. La page lui propose de se connecter et " +
        "de s'inscrire — alors qu'il vient de le faire. Le produit se " +
        "comporte comme s'il était un visiteur.",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // ── Ce que cette assertion DÉCRIT, et ce qu'elle refuse d'inventer
        //
        // Le produit a tout prévu : `permissions.ts:148` tient un registre
        // des rôles sans interface, `garde-roles` exige que tout rôle du
        // backend y figure ou voie un menu, et `Navigation.svelte:544`
        // porte le message — « ceux-là méritent une phrase, pas un vide ».
        //
        // Mais `index.astro:39` pose `showNav={false}`, et c'est là que
        // `getDefaultRedirect` les envoie. **Le message est inatteignable
        // pour les sept rôles pour lesquels il a été écrit** (#961).
        //
        // On filme donc ce qui se passe : un utilisateur connecté à qui la
        // page propose de se connecter. Écrire l'assertion qu'on souhaite
        // ferait mentir la vitrine — et c'est précisément le genre de
        // défaut qu'elle existe pour montrer.
        await expect(
          page.getByTestId("navigation-role-sans-interface"),
        ).toHaveCount(0);
        await expect(page.getByTestId("home-login-nav")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "3-ce-qui-viendra",
      acteur: "prestataire",
      description:
        "Son entrée prévue n'est pas un mot de passe : c'est un lien que " +
        "le syndic lui envoie, portant sa seule mission. Le serveur ne " +
        "sait pas encore l'émettre — et le harnais refuse d'en inventer un.",
      action: async (scene) => {
        await scene.aller("/profile");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Son profil existe : c'est bien un utilisateur à part entière,
        // pas un compte dégradé.
        await expect(page.locator("body")).not.toContainText("Internal Server");
      },
    },
  ],
};
