/**
 * Parcours de référence — l'administration de la plateforme (#809).
 *
 * ── Pourquoi ce parcours s'appelle « administration » et non « admin » ────
 *
 * Le persona `admin.md` décrit un administrateur d'organisation, et le
 * frontend le connaît : `permissions.ts:96` pose
 * `ADMIN_ROLES = {"superadmin", "admin"}`.
 *
 * **Le backend ne le connaît pas.** `UserRole::from_str` accepte quinze
 * rôles ; `admin` n'en fait pas partie, et la contrainte de base non plus.
 * Aucun compte ne peut porter ce rôle — ni par inscription, ni par semis
 * (#960).
 *
 * Le `population_recette: 0` du persona n'était donc pas un oubli, c'était
 * une impossibilité. Filmer ce rôle reviendrait à inventer un parcours
 * théorique, exactement ce que #805 (@edge) interdit.
 *
 * Ce parcours filme donc l'administration **telle qu'elle existe** : le
 * superadmin, seul membre réel d'`ADMIN_ROLES`, qui opère au-dessus du
 * multi-tenant plutôt qu'à l'intérieur.
 *
 * ── Ce qu'il démontre ─────────────────────────────────────────────────────
 *
 * La chaîne de provisionnement, dans l'ordre où elle se fait réellement :
 * une organisation, puis l'ACP qu'un syndic retranscrira, puis les comptes.
 * C'est le préalable de tous les autres parcours — sans lui, aucun syndic
 * n'a de copropriété à gérer.
 *
 * Et une propriété qui vaut d'être vue : le superadmin **n'appartient à
 * aucune organisation**. Ce n'est pas une omission mais une contrainte de
 * conception — un compte qui administre la plateforme ne doit pas être
 * partie prenante d'une des organisations qu'il administre.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import { ADMIN_EMAIL, ADMIN_PASSWORD } from "../helpers/identifiants";

/**
 * Ce que le parcours crée à la caméra, et que les étapes suivantes
 * retrouvent.
 *
 * Horodaté : l'étape 3 CHERCHE cette organisation par son nom pour y
 * rattacher l'ACP. Un nom partagé avec une campagne précédente ferait
 * rattacher l'ACP au mauvais cabinet, et l'assertion passerait au vert sans
 * rien prouver de la chaîne.
 */
let nomDuCabinet = "";

export const administration: Parcours = {
  slug: "administration",
  titre: "L'administration de la plateforme, au-dessus du multi-tenant",
  propos:
    "Montre la chaîne de provisionnement — organisation, ACP, comptes — qui " +
    "précède tous les autres parcours. Et la contrainte qui la borne : le " +
    "superadmin n'appartient à aucune organisation.",

  amorcer: async (page) => {
    // Aucun monde à semer : ce parcours filme le rôle qui CRÉE les mondes.
    // Ses identifiants viennent de l'environnement, jamais d'ici — le mot
    // de passe du superadministrateur tourne au déploiement.
    await page.context().clearCookies();
    nomDuCabinet = `Cabinet Verhaeren ${Date.now()}`;
    return {
      administrateur: { email: ADMIN_EMAIL, motDePasse: ADMIN_PASSWORD },
    };
  },

  etapes: [
    {
      id: "1-connexion",
      acteur: "administrateur",
      description:
        "KoproGo — l'administration de la plateforme. C'est le seul rôle " +
        "qui opère AU-DESSUS du multi-tenant : il crée les organisations " +
        "plutôt que d'appartenir à l'une d'elles.",
      action: async (scene) => {
        await scene.devenir("administrateur");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("admin-dashboard")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-les-organisations",
      acteur: "administrateur",
      description:
        "Les organisations : chaque cabinet de syndic en est une, et " +
        "chacune est étanche aux autres. C'est le premier maillon de la " +
        "chaîne — sans organisation, aucune copropriété.",
      action: async (scene) => {
        await scene.aller("/admin/organizations");
        await scene.attendreChargement();
        // L'étape se contentait d'atteindre l'écran et de constater qu'un
        // bouton « créer » s'affichait (#974). Elle CRÉE désormais : c'est
        // le premier maillon de la chaîne, et le narrer sans le faire
        // laissait le maillon non vérifié.
        await scene.cliquer("create-organization-button");
        await scene.attendreChargement();
        await scene.saisir("organization-name-input", nomDuCabinet);
        await scene.saisir(
          "organization-email-input",
          `contact-${Date.now()}@verhaeren.be`,
        );
        await scene.cliquer("organization-submit-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // La modale s'est refermée ET le cabinet est dans la table.
        // Vérifier seulement la fermeture ne distinguerait pas une création
        // d'une annulation.
        await expect(page.getByTestId("organization-form")).toHaveCount(0, {
          timeout: 20000,
        });
        await expect(
          page.getByTestId("organization-row").filter({
            hasText: nomDuCabinet,
          }),
          "Le cabinet créé n'apparaît pas dans la table : sans lui, aucune " +
            "copropriété n'a de syndic, et tous les autres parcours " +
            "s'arrêtent avant de commencer.",
        ).toHaveCount(1, { timeout: 20000 });
      },
    },
    {
      id: "3-les-copropriétés",
      acteur: "administrateur",
      description:
        "Les ACP — les associations de copropriétaires. Le superadmin les " +
        "provisionne ; c'est le syndic mandaté qui retranscrira ensuite " +
        "leur acte de base. Provisionner n'est pas gérer.",
      action: async (scene) => {
        await scene.aller("/admin/acps");
        await scene.attendreChargement();
        await scene.cliquer("acp-create-toggle");
        await scene.saisir("acp-form-name", `Les Peupliers ${Date.now()}`);
        // On CHERCHE le cabinet créé à l'étape 2, puis on l'attache. La
        // liste des organisations est plafonnée côté serveur : sans la
        // recherche, un cabinet créé à l'instant peut ne pas y figurer, et
        // l'ACP se retrouverait « en autogestion » alors que la narration
        // annonce un syndic mandaté.
        await scene.saisir("acp-form-org-search", nomDuCabinet);
        await scene.attendreChargement();
        await scene.choisirQuiContient("acp-form-org-id", nomDuCabinet);
        await scene.saisir("acp-form-street", "12 Avenue des Peupliers");
        await scene.saisir("acp-form-postal", "1180");
        await scene.saisir("acp-form-city", "Uccle");
        await scene.cliquer("acp-form-submit");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("acp-create-form")).toHaveCount(0, {
          timeout: 20000,
        });
        // La copropriété est provisionnée ET porte le nom de son syndic
        // mandaté. C'est ce rattachement qui fait la différence entre une
        // ACP en autogestion et une ACP gérée — et il ne se voit nulle part
        // ailleurs.
        await expect(
          page.getByTestId("acps-table"),
          "La copropriété créée n'apparaît pas, ou n'est pas rattachée au " +
            "cabinet qui vient de la recevoir en mandat.",
        ).toContainText(nomDuCabinet, { timeout: 20000 });
      },
    },
    {
      id: "4-les-comptes",
      acteur: "administrateur",
      description:
        "Les comptes, et les rôles qu'ils portent. Chaque rôle ouvre un " +
        "périmètre différent : le comptable ne voit pas ce que voit le " +
        "syndic, et aucun des deux ne voit les autres copropriétés.",
      action: async (scene) => {
        await scene.aller("/admin/users");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("user-search-input")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "5-le-registre-rgpd",
      acteur: "administrateur",
      description:
        "Le registre RGPD : exporter ce qu'on détient d'une personne, ou " +
        "l'effacer. Chaque geste y est journalisé — c'est une obligation, " +
        "pas une commodité.",
      action: async (scene) => {
        await scene.aller("/admin/gdpr");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("admin-gdpr-panel")).toBeVisible({
          timeout: 20000,
        });
      },
    },
  ],
};
