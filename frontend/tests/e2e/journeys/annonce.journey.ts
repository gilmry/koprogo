/**
 * Parcours de référence — le panneau d'affichage, et qui le lit.
 *
 * ── Ce qu'il démontre ─────────────────────────────────────────────────────
 *
 * L'information descendante d'une copropriété. Aujourd'hui, elle tient à une
 * feuille scotchée près des boîtes aux lettres : celui qui rentre par le
 * parking ne la voit pas, celle qui part en congé la rate, et personne ne
 * sait qui l'a lue.
 *
 * Le parcours filme le remplacement : le syndic publie, et une
 * copropriétaire RETROUVE l'annonce par une recherche — sans avoir reçu de
 * courriel, sans qu'on lui ait donné de lien.
 *
 * ── Pourquoi la recherche fait partie du parcours ─────────────────────────
 *
 * Parce qu'un panneau d'affichage qu'on ne peut pas fouiller est un panneau
 * d'affichage qui se périme. La question réelle d'un copropriétaire n'est pas
 * « qu'y a-t-il de neuf » mais « qu'avait-on dit sur l'ascenseur ». Une liste
 * sans recherche ne répond qu'à la première.
 *
 * Le geste est donc filmé : taper dans le champ, voir la liste se réduire à
 * une ligne, ouvrir celle-là. Une capture d'écran de la liste complète
 * n'aurait rien montré de cette capacité.
 *
 * ── Ce qu'il vérifie de plus que « l'écran se rend » ──────────────────────
 *
 * Que l'annonce lue par la copropriétaire porte le TEXTE écrit par le
 * syndic — pas seulement son titre. Une annonce dont le corps se perdrait
 * afficherait le même écran de détail, avec le même titre.
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

let monde: { nomDeLImmeuble: string; titre: string } | null = null;

/**
 * Le corps de l'annonce. La copropriétaire doit le retrouver MOT POUR MOT :
 * c'est ce qui distingue « l'écran de détail se rend » de « l'annonce est
 * arrivée entière ».
 */
const CORPS_DE_L_ANNONCE =
  "L'ascenseur sera à l'arrêt du lundi 5 au vendredi 9 octobre pour le " +
  "remplacement des câbles de traction. Un monte-charge provisoire sera " +
  "installé dans la cage d'escalier pour les personnes à mobilité réduite. " +
  "Merci de signaler au syndic tout besoin d'assistance avant le 1er octobre.";

export function mondeDuParcours(): typeof monde {
  return monde;
}

export const annonce: Parcours = {
  slug: "annonce",
  titre: "Le panneau d'affichage, et qui le lit",
  propos:
    "Remplace la feuille scotchée près des boîtes aux lettres. Le syndic " +
    "rédige une annonce de travaux et la publie ; une copropriétaire la " +
    "retrouve en cherchant « ascenseur », l'ouvre et lit le texte entier. " +
    "Puis le syndic archive : une annonce périmée cesse d'encombrer le " +
    "panneau sans disparaître de l'historique.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `annonce-syndic-${horodatage}@example.com`;
    const emailCopro = `annonce-owner-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Annonce Org ${horodatage}`,
          slug: `annonce-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "annonce");

    const nomDeLImmeuble = `Résidence du Panneau ${horodatage}`;
    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: nomDeLImmeuble,
          address: `${horodatage} Rue de l'Affiche`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 5,
          total_tantiemes: 1000,
          construction_year: 1988,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );
    await seedConformantUnits(page, adminToken, acpId, immeuble.id, 5, 1000);

    let idDuCompteCopro = "";
    for (const [email, prenom, nom, role] of [
      [emailSyndic, "Sophie", "Syndic", "syndic"],
      [emailCopro, "Carine", "Copropriétaire", "owner"],
    ] as const) {
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
        `amorçage:${role}`,
      );
      if (role === "owner") idDuCompteCopro = cree.user?.id ?? cree.id ?? "";
    }

    // Un compte `owner` n'est pas un copropriétaire tant qu'il ne possède
    // rien : le sélecteur d'immeuble de `/notices` ne liste que les immeubles
    // où le compte a un lot, et il répondait « Aucun immeuble trouvé ».
    await rattacherLeCoproprietaire(page, adminToken, {
      orgId: org.id,
      buildingId: immeuble.id,
      userId: idDuCompteCopro,
      email: emailCopro,
      firstName: "Carine",
      lastName: "Copropriétaire",
    });

    monde = {
      nomDeLImmeuble,
      // « ascenseur » est le mot que la copropriétaire cherchera. L'horodatage
      // garantit qu'elle ne retombe pas sur l'annonce d'une campagne
      // précédente, ce qui ferait passer l'étape 5 au vert sans rien prouver.
      titre: `Travaux ascenseur ${horodatage}`,
    };

    await page.context().clearCookies();

    return {
      syndic: { email: emailSyndic, motDePasse },
      copropriétaire: { email: emailCopro, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-le-syndic-ouvre-le-panneau",
      acteur: "syndic",
      description:
        "Les câbles de l'ascenseur doivent être remplacés. Avant, Sophie " +
        "imprimait une feuille et la scotchait près des boîtes aux lettres.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller("/notices");
        await scene.attendreChargement();
        await scene.cliquer("notices-create-btn");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("notice-create-form"),
          "Le bouton « créer une annonce » n'ouvre pas de formulaire : la " +
            "capacité est affichée sans être atteignable.",
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "2-elle-redige",
      acteur: "syndic",
      description:
        "Elle qualifie : une annonce d'entretien, pas une petite annonce. " +
        "Le classement n'est pas décoratif — c'est lui qui permettra à " +
        "quelqu'un de filtrer six mois plus tard.",
      action: async (scene) => {
        await scene.choisir("notice-create-type-select", "Announcement");
        await scene.choisir("notice-create-category-select", "Maintenance");
        await scene.saisir("notice-title-input", monde!.titre);
        await scene.saisir("notice-content-input", CORPS_DE_L_ANNONCE);
      },
      assertion: async (page) => {
        await expect(page.getByTestId("notice-submit-btn")).toBeEnabled();
      },
    },
    {
      id: "3-elle-enregistre-un-brouillon",
      acteur: "syndic",
      description:
        "Elle enregistre. L'annonce existe, mais en BROUILLON : elle n'est " +
        "encore visible que d'elle. Une note diffusée à trente foyers ne se " +
        "reprend pas — on relit d'abord.",
      action: async (scene) => {
        await scene.cliquer("notice-submit-btn");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // La modale s'est refermée ET l'annonce est dans la liste. Vérifier
        // seulement la fermeture ne distinguerait pas un enregistrement
        // d'une annulation.
        await expect(page.getByTestId("notice-create-form")).toHaveCount(0, {
          timeout: 20000,
        });
        await expect(
          page.getByTestId("notice-list-row").filter({ hasText: monde!.titre }),
          "L'annonce enregistrée n'apparaît pas dans la liste de son " +
            "auteure : l'envoi a l'air d'aboutir sans rien enregistrer.",
        ).toHaveCount(1, { timeout: 20000 });
      },
    },
    {
      id: "3bis-elle-relit-et-diffuse",
      acteur: "syndic",
      description:
        "Elle ouvre son brouillon, le relit, et le publie. C'est ce geste " +
        "qui le fait passer du bureau du syndic au hall d'entrée.",
      action: async (scene) => {
        await scene.cliquerLePremier("notice-list-detail-link");
        await scene.attendreChargement();
        await scene.cliquer("notice-publish-btn");
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // ── Ce que cette assertion a coûté à découvrir ───────────────────
        //
        // `notice-publish-btn` n'existait pas avant ce parcours. Le backend
        // servait `POST /notices/{id}/publish` depuis toujours ; le
        // frontend ne l'appelait JAMAIS, et `Notice::new` crée en `Draft`,
        // « not visible to others ».
        //
        // Toute annonce rédigée au formulaire restait donc invisible des
        // copropriétaires, pour toujours. Le syndic la voyait dans sa
        // propre liste — il en est l'auteur — et concluait que
        // l'information était passée (#978).
        //
        // C'est le changement d'acteur de l'étape 4 qui l'a révélé : tant
        // qu'un seul compte regardait, tout paraissait normal.
        //
        // Le bouton disparaît une fois l'annonce publiée : les deux états
        // sont exclusifs, et republier n'aurait aucun sens.
        await expect(
          page.getByTestId("notice-publish-btn"),
          "La publication n'a rien changé : l'annonce reste un brouillon " +
            "que personne ne verra.",
        ).toHaveCount(0, { timeout: 20000 });
      },
    },
    {
      id: "4-la-coproprietaire-cherche",
      acteur: "copropriétaire",
      description:
        "Carine se demande ce qui avait été dit sur l'ascenseur. Elle " +
        "cherche le mot — elle ne fait pas défiler six mois d'annonces.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
        await scene.aller("/notices");
        await scene.attendreChargement();
        await scene.saisir("notice-list-search-input", "ascenseur");
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("notice-list-row"),
          "La recherche ne réduit pas la liste à l'annonce cherchée : un " +
            "panneau qu'on ne peut pas fouiller se périme.",
        ).toHaveCount(1, { timeout: 20000 });
        await expect(page.getByTestId("notice-list-row")).toContainText(
          monde!.titre,
        );
      },
    },
    {
      id: "5-elle-lit",
      acteur: "copropriétaire",
      description:
        "Elle ouvre et lit : les dates, le monte-charge provisoire, et à " +
        "qui signaler un besoin d'assistance. L'information complète, pas " +
        "un titre.",
      action: async (scene) => {
        await scene.cliquerLePremier("notice-list-detail-link");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("notice-detail")).toBeVisible({
          timeout: 20000,
        });
        // LE point de cette étape : le corps du texte est arrivé entier. Une
        // annonce dont le contenu se perdrait afficherait le même écran, avec
        // le même titre — et personne ne s'en apercevrait avant les travaux.
        await expect(
          page.getByTestId("notice-detail"),
          "L'annonce s'ouvre mais son texte ne s'y trouve pas : le titre a " +
            "voyagé, le contenu non.",
        ).toContainText("monte-charge provisoire");
      },
    },
    {
      id: "6-le-syndic-archive",
      acteur: "syndic",
      description:
        "Les travaux sont finis. Sophie archive, et confirme : l'annonce " +
        "quitte le panneau sans quitter l'historique. Supprimer effacerait " +
        "la preuve que l'information a bien été donnée.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller("/notices");
        await scene.attendreChargement();
        await scene.saisir("notice-list-search-input", "ascenseur");
        await scene.cliquerLePremier("notice-list-detail-link");
        await scene.attendreChargement();
        await scene.cliquer("notice-archive-btn");
        await scene.cliquer("confirm-dialog-confirm");
        await scene.page.waitForURL(/\/notices\/?$/, { timeout: 20000 });
        await scene.attendreChargement();
        await scene.saisir("notice-list-search-input", "ascenseur");
      },
      assertion: async (page) => {
        // ── Ce que l'archivage doit prouver, et où ───────────────────────
        //
        // `executer_archiver` redirige vers `/notices` : l'assertion ne peut
        // pas porter sur l'écran de détail, qui n'est plus là. C'est mieux
        // ainsi — ce qui compte n'est pas l'état d'un bouton, mais que
        // l'annonce ait QUITTÉ le panneau.
        //
        // Le filtre par défaut de la liste vaut « Publiées uniquement ». Une
        // annonce archivée n'y figure donc plus, alors qu'elle y était à
        // l'étape 4. La recherche est rejouée à l'identique : seule la
        // réponse a changé.
        await expect(
          page.getByTestId("notice-list-row"),
          "L'annonce archivée encombre toujours le panneau : l'archivage " +
            "affiche un succès sans rien retirer.",
        ).toHaveCount(0, { timeout: 20000 });
      },
    },
  ],
};
