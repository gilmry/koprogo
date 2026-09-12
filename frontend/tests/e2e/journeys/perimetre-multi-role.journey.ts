/**
 * Parcours de référence — « chacun voit son périmètre, et rien d'autre ».
 *
 * ── Pourquoi celui-ci en premier ──────────────────────────────────────────
 *
 * C'est le parcours le plus court qui démontre ce que KoproGo a de particulier
 * et que le moule Foyer n'a pas à traiter : **la bascule d'acteur**. Le todo du
 * `kit-actix` est mono-utilisateur ; ici le syndic et le copropriétaire se
 * relaient, et ce qu'ils voient diffère.
 *
 * La règle 9 de `CRITICAL.md` l'exige — « pas un seul login pour tout » — mais
 * une règle ne se vérifie pas à la lecture d'un diff. Une vidéo où l'on VOIT
 * l'un se déconnecter et l'autre se connecter la rend opposable d'un regard.
 *
 * ── Ce qu'il ne fait pas ──────────────────────────────────────────────────
 *
 * Il ne joue pas le cycle d'assemblée générale : #780 tient trois verrous
 * cumulatifs et le parcours n'aboutirait pas. Filmer un écran qui casse
 * produirait une documentation périmée le jour de sa livraison — c'est
 * exactement ce que l'épopée T4 dit d'éviter. Le parcours AG viendra quand
 * #780 sera levée.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import { provisionneComptesDuParcours } from "../helpers/auth";

export const perimetreMultiRole: Parcours = {
  slug: "perimetre-multi-role",
  titre: "Chacun voit son périmètre — le syndic, puis le copropriétaire",
  propos:
    "Démontre que deux rôles se relaient sur la même copropriété et que " +
    "l'écran d'accueil de chacun reflète ce qu'il a le droit de voir.",

  /**
   * Une copropriété neuve, son syndic et un copropriétaire qui y détient un
   * lot. Les comptes sont créés par API et **aucune session n'est ouverte** :
   * les deux connexions sont le sujet du parcours, elles se jouent devant la
   * caméra.
   */
  amorcer: async (page) => {
    const comptes = await provisionneComptesDuParcours(page, "vitrine");
    return {
      syndic: comptes.syndic,
      copropriétaire: comptes.coproprietaire,
    };
  },

  etapes: [
    {
      id: "ouverture",
      acteur: "syndic",
      description:
        "KoproGo — gestion de copropriété. Deux rôles vont se relayer : " +
        "le syndic qui administre, puis un copropriétaire qui consulte.",
      action: async (scene) => {
        await scene.aller("/");
      },
    },
    {
      id: "syndic-entre",
      acteur: "syndic",
      description:
        "Le syndic se connecte. C'est le mandataire de l'ACP au sens de " +
        "l'Art. 3.89 : il convoque, préside et exécute.",
      action: async (scene) => {
        await scene.devenir("syndic");
      },
      assertion: async (page) => {
        await expect(page).toHaveURL(/\/syndic/);
      },
    },
    {
      id: "syndic-voit-son-tableau",
      acteur: "syndic",
      description:
        "Son tableau de bord s'ouvre sur les immeubles qu'il gère. " +
        "Il voit l'administration ; il ne voit pas les lots d'un copropriétaire.",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("navigation-menu-gestion")).toBeVisible();
      },
    },
    {
      id: "coproprietaire-prend-la-main",
      acteur: "copropriétaire",
      description:
        "Le syndic se retire. Un copropriétaire se connecte — titulaire " +
        "d'un droit réel, il vote, paie et consulte ses quotes-parts.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
      },
      assertion: async (page) => {
        await expect(page).toHaveURL(/\/owner/);
      },
    },
    {
      id: "coproprietaire-voit-le-sien",
      acteur: "copropriétaire",
      description:
        "Son écran est différent : « Mes lots » et « Communauté », pas " +
        "l'administration. Le cloisonnement se voit, il ne se déduit pas.",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("navigation-menu-communaute"),
        ).toBeVisible();
        await expect(page.getByTestId("navigation-menu-admin")).toHaveCount(0);
      },
    },
    {
      id: "cloture",
      acteur: "copropriétaire",
      description:
        "Parcours terminé. Le même parcours sert de gate E2E (correctness) " +
        "et de preuve de valeur (cette vidéo) — écrit une seule fois.",
      action: async (scene) => {
        await scene.attendreChargement();
      },
    },
  ],
};
