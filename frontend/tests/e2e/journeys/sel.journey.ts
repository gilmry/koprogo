/**
 * Parcours de référence — le SEL : un service rendu, des crédits qui
 * changent de main.
 *
 * ── Ce qu'il démontre, et pourquoi ça compte pour KoproGo ─────────────────
 *
 * Le système d'échange local est la partie du produit qui ne ressemble à
 * aucun logiciel de syndic. Il ne gère pas de l'argent : il tient un compte
 * de temps entre voisins. Une heure de garde d'enfant vaut une heure de
 * bricolage, et le solde de chacun est public.
 *
 * C'est la mission ASBL du projet, et c'est aussi la partie la plus facile à
 * mal juger sur une capture d'écran : une liste d'annonces ressemble à
 * n'importe quelle petite annonce. Ce qui la distingue est invisible à
 * l'arrêt — le crédit qui se déplace quand le service est rendu.
 *
 * ── Le cycle, joué à deux ─────────────────────────────────────────────────
 *
 *     Offered ──(un voisin demande)──> Requested
 *             ──(l'offrant accepte)──> InProgress
 *             ──(l'offrant a rendu)──> Completed   → les crédits bougent
 *                                                  → et on peut noter
 *
 * Chaque transition est réservée à l'UN des deux : le voisin demande,
 * l'offrant accepte et clôture. `ExchangeDetail.svelte:400-430` conditionne
 * chaque bouton à `isProvider`, et les conditions sont exclusives. Un
 * parcours mono-acteur ne pourrait en jouer que la moitié.
 *
 * ── Ce qu'il vérifie de plus que « l'écran se rend » ──────────────────────
 *
 * Que le solde de crédits a CHANGÉ. `credit-balance` est relu après la
 * clôture : un SEL dont les crédits ne bougeraient pas serait un tableau
 * d'affichage déguisé, et l'écran serait exactement le même.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import {
  adminLogin,
  ensureAcp,
  rattacherLeCoproprietaire,
  seedConformantUnits,
} from "../helpers/auth";
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

let monde: { nomDeLImmeuble: string; titreDeLOffre: string } | null = null;

export function mondeDuParcours(): typeof monde {
  return monde;
}

/** Ce que l'offre vaut. Trois heures de bricolage, trois crédits. */
const CREDITS_DE_L_OFFRE = 3;

export const sel: Parcours = {
  slug: "sel",
  titre: "Le SEL : un service rendu, des crédits qui changent de main",
  propos:
    "La partie du produit qui ne ressemble à aucun logiciel de syndic. " +
    "Une copropriétaire offre trois heures de bricolage, un voisin les " +
    "demande, elle accepte, elle rend le service, il la note — et les " +
    "crédits se déplacent. Ce déplacement est invisible sur une capture : " +
    "c'est pourtant tout ce qui distingue un SEL d'un panneau de petites " +
    "annonces.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `sel-syndic-${horodatage}@example.com`;
    const emailOffrante = `sel-offrante-${horodatage}@example.com`;
    const emailVoisin = `sel-voisin-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `SEL Org ${horodatage}`,
          slug: `sel-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "sel");

    const nomDeLImmeuble = `Résidence du Partage ${horodatage}`;
    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: nomDeLImmeuble,
          address: `${horodatage} Rue de l'Entraide`,
          city: "Brussels",
          postal_code: "1030",
          country: "Belgium",
          total_units: 4,
          total_tantiemes: 1000,
          construction_year: 1992,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );
    await seedConformantUnits(page, adminToken, acpId, immeuble.id, 4, 1000);

    const identites: Array<[string, string, string, string]> = [
      [emailSyndic, "Sophie", "Syndic", "syndic"],
      [emailOffrante, "Fatima", "Benali", "owner"],
      [emailVoisin, "Luc", "Moreau", "owner"],
    ];
    const comptes: Record<string, string> = {};
    for (const [email, prenom, nom, role] of identites) {
      const cree = await ok<{ user?: { id?: string }; id?: string }>(
        await api.post(`${API_BASE}/auth/register`, {
          data: {
            email,
            password: motDePasse,
            first_name: prenom,
            last_name: nom,
            role,
            organization_id: org.id,
          },
        }),
        `amorçage:${email}`,
      );
      comptes[email] = cree.user?.id ?? cree.id ?? "";
    }

    // Deux copropriétaires RÉELS, chacun sur son lot. Un SEL entre deux
    // comptes qui ne possèdent rien n'existerait pas : `/exchanges` ne
    // listerait aucun immeuble, et le solde de crédits n'aurait pas de
    // titulaire.
    await rattacherLeCoproprietaire(page, adminToken, {
      orgId: org.id,
      buildingId: immeuble.id,
      userId: comptes[emailOffrante],
      email: emailOffrante,
      firstName: "Fatima",
      lastName: "Benali",
      lotIndex: 0,
    });
    await rattacherLeCoproprietaire(page, adminToken, {
      orgId: org.id,
      buildingId: immeuble.id,
      userId: comptes[emailVoisin],
      email: emailVoisin,
      firstName: "Luc",
      lastName: "Moreau",
      lotIndex: 1,
    });

    monde = {
      nomDeLImmeuble,
      titreDeLOffre: `Montage de meubles et petit bricolage ${horodatage}`,
    };

    await page.context().clearCookies();

    return {
      // Fatima offre : c'est elle la « copropriétaire » du parcours.
      copropriétaire: { email: emailOffrante, motDePasse },
      // Luc demande. Faute d'un rôle « voisin » dans la typologie des
      // acteurs, il emprunte celui de « conseil » — les deux sont des
      // copropriétaires, et la bascule reste visible à l'écran.
      conseil: { email: emailVoisin, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-fatima-offre-du-temps",
      acteur: "copropriétaire",
      description:
        "Fatima sait monter des meubles. Elle n'en fait pas commerce : " +
        "elle offre trois heures à ses voisins, contre trois crédits — " +
        "trois heures qu'un voisin lui rendra un jour, en autre chose.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
        await scene.aller("/exchanges/new");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("create-exchange-form")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-elle-decrit-son-offre",
      acteur: "copropriétaire",
      description:
        "Elle décrit ce qu'elle propose et ce que ça vaut. Le prix est en " +
        "TEMPS, pas en euros : c'est la règle du SEL, et c'est elle qui " +
        "met le retraité et le cadre sur le même pied.",
      action: async (scene) => {
        await scene.saisir("exchange-title-input", monde!.titreDeLOffre);
        await scene.saisir(
          "exchange-description-input",
          "Montage de meubles en kit, pose d'étagères, petites réparations. " +
            "J'ai l'outillage. Disponible en soirée et le samedi matin.",
        );
        await scene.saisir(
          "exchange-credits-input",
          String(CREDITS_DE_L_OFFRE),
        );
      },
      assertion: async (page) => {
        await expect(page.getByTestId("exchange-submit-btn")).toBeEnabled();
      },
    },
    {
      id: "3-l-offre-est-publiee",
      acteur: "copropriétaire",
      description:
        "Elle publie. L'offre rejoint le SEL de l'immeuble, et son fil de " +
        "vie commence : offerte, demandée, rendue.",
      action: async (scene) => {
        await scene.cliquer("exchange-submit-btn");
        await scene.page.waitForURL(/\/exchange-detail\?id=/, {
          timeout: 30000,
        });
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("exchange-detail")).toBeVisible({
          timeout: 20000,
        });
        await expect(page.getByTestId("exchange-detail-header")).toContainText(
          monde!.titreDeLOffre,
        );
        // L'offrante ne peut pas demander son propre service :
        // `ExchangeDetail.svelte:400` exige `!isProvider`. C'est ce qui
        // empêche un compte de se créditer tout seul.
        await expect(
          page.getByTestId("exchange-request-btn"),
          "L'offrante peut demander son propre service : un compte " +
            "pourrait se créditer sans que personne ne rende rien.",
        ).toHaveCount(0);
      },
    },
    {
      id: "4-un-voisin-demande",
      acteur: "conseil",
      description:
        "Luc a une bibliothèque en pièces détachées depuis trois mois. Il " +
        "trouve l'offre dans le SEL de l'immeuble, la demande, et " +
        "confirme — demander un service, c'est s'engager à recevoir " +
        "quelqu'un chez soi.",
      action: async (scene) => {
        await scene.devenir("conseil");
        await scene.aller("/exchanges");
        await scene.attendreChargement();
        await scene.saisir("exchange-search-input", "bricolage");
        await scene.attendreChargement();
        await scene.cliquerLePremier("exchange-view-btn");
        await scene.attendreChargement();
        await scene.cliquer("exchange-request-btn");
        // Chaque transition du SEL demande confirmation. C'est justifié :
        // demander un service engage Luc à recevoir quelqu'un chez lui, et
        // l'acceptation engage Fatima à y aller.
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le bouton « demander » a disparu : l'échange n'est plus `Offered`.
        // Les conditions de `ExchangeDetail.svelte` sont exclusives, donc
        // son absence prouve la transition.
        await expect(
          page.getByTestId("exchange-request-btn"),
          "La demande n'a rien changé : l'offre reste disponible et Luc " +
            "peut la redemander indéfiniment.",
        ).toHaveCount(0, { timeout: 20000 });
      },
    },
    {
      id: "5-fatima-accepte-et-rend-le-service",
      acteur: "copropriétaire",
      description:
        "Fatima accepte, monte la bibliothèque, et marque le service " +
        "rendu. C'est à cet instant que les crédits se déplacent — pas à " +
        "la demande, pas à l'acceptation.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
        await scene.aller("/exchanges");
        await scene.attendreChargement();
        await scene.saisir("exchange-search-input", "bricolage");
        await scene.attendreChargement();
        await scene.cliquerLePremier("exchange-view-btn");
        await scene.attendreChargement();
        await scene.cliquer("exchange-start-btn");
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
        await scene.cliquer("exchange-complete-btn");
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Ni démarrer ni clôturer ne sont plus offerts : l'échange est
        // `Completed`, un état terminal.
        await expect(
          page.getByTestId("exchange-complete-btn"),
          "Le service marqué rendu reste clôturable : rien n'empêche de " +
            "créditer deux fois le même échange.",
        ).toHaveCount(0, { timeout: 20000 });
        await expect(page.getByTestId("exchange-start-btn")).toHaveCount(0);
      },
    },
    {
      id: "6-le-credit-a-change-de-main",
      acteur: "copropriétaire",
      description:
        "Et voilà ce qui distingue un SEL d'un panneau de petites " +
        "annonces : Fatima a désormais trois crédits de plus. Elle pourra " +
        "demander trois heures à quelqu'un d'autre, sans jamais avoir " +
        "parlé d'argent.",
      action: async (scene) => {
        await scene.aller("/exchanges");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le solde EXISTE et se rend. Sans cet écran, le crédit serait une
        // ligne de base que personne ne voit — et le SEL, une liste
        // d'annonces comme une autre.
        await expect(
          page.getByTestId("credit-balance"),
          "Le solde de crédits ne s'affiche pas : le SEL n'a plus de " +
            "monnaie visible, et rien ne distingue un service rendu d'un " +
            "service promis.",
        ).toBeVisible({ timeout: 20000 });
        // Les crédits gagnés apparaissent. On cherche le nombre, pas un
        // libellé traduit : c'est la valeur qui porte la preuve.
        await expect(
          page.getByTestId("credit-balance"),
          `Le solde ne porte pas les ${CREDITS_DE_L_OFFRE} crédits du ` +
            "service rendu : la clôture a l'air d'aboutir sans rien créditer.",
        ).toContainText(String(CREDITS_DE_L_OFFRE), { timeout: 20000 });
      },
    },
  ],
};
