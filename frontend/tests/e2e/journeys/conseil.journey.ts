/**
 * Parcours de référence — le conseil de copropriété (#816).
 *
 * ── Ce que ce parcours démontre, et qui est du droit ──────────────────────
 *
 * Le conseil de copropriété n'est pas un comité facultatif : il est
 * **obligatoire au-delà de vingt lots** (Art. 577-8/4 du Code civil, cité
 * par `backend/src/domain/copropriete/board_member.rs:37`). En dessous, le
 * serveur refuse l'élection — et c'est un refus juste, pas un défaut.
 *
 * L'immeuble de ce parcours porte donc **vingt-quatre lots**. Ce n'est pas
 * un chiffre décoratif : à douze, l'élection rend 400, et la vidéo
 * montrerait une erreur en prétendant montrer une institution.
 *
 * ── Pourquoi c'est un parcours MULTI-RÔLES ────────────────────────────────
 *
 * Il se joue à deux, et c'est le seul moyen de le montrer honnêtement :
 * le syndic élit, l'élu se connecte. Un seul acteur ne peut pas raconter une
 * élection — il manquerait la moitié du geste, celle où quelqu'un reçoit un
 * mandat qu'il n'a pas décidé.
 *
 * La bascule d'acteur est **racontée à l'écran** (`scene.devenir`), comme la
 * règle 9 de la documentation vivante l'exige.
 *
 * ── Une nuance de vocabulaire qui compte ──────────────────────────────────
 *
 * Les membres du conseil sont des **copropriétaires**, pas nécessairement
 * des utilisateurs de la plateforme — le commentaire du domaine le dit. Un
 * conseil peut donc exister sans qu'aucun de ses membres n'ait de compte.
 * Ce parcours filme le cas où l'élu en a un, parce que c'est le seul qui se
 * filme ; l'autre existe et reste hors champ.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import { adminLogin, seedConformantUnits, ensureAcp } from "../helpers/auth";
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

/** Au-delà de ce seuil, le conseil de copropriété est obligatoire. */
const SEUIL_LEGAL_CONSEIL = 20;

export const conseil: Parcours = {
  slug: "conseil",
  titre: "Le conseil de copropriété, obligatoire au-delà de vingt lots",
  propos:
    "Montre une élection à deux voix : le syndic élit un copropriétaire au " +
    "conseil, et l'élu découvre son mandat. Le conseil surveille le syndic " +
    "— ce n'est pas un comité de plus, c'est un contre-pouvoir que la loi " +
    "impose aux grandes copropriétés.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `conseil-syndic-${horodatage}@example.com`;
    const emailElu = `conseil-elu-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Conseil Org ${horodatage}`,
          slug: `conseil-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "conseil");

    // Vingt-quatre lots : au-dessus du seuil légal, sans quoi l'élection
    // est refusée en 400 — à raison.
    const nombreDeLots = SEUIL_LEGAL_CONSEIL + 4;
    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: `Grande Résidence ${horodatage}`,
          address: `${horodatage} Avenue du Conseil`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: nombreDeLots,
          total_tantiemes: 1000,
          construction_year: 2010,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );

    await seedConformantUnits(
      page,
      adminToken,
      acpId,
      immeuble.id,
      nombreDeLots,
      1000,
    );

    const eluUser = await ok<any>(
      await api.post(`${API_BASE}/auth/register`, {
        data: {
          email: emailElu,
          password: motDePasse,
          first_name: "Alice",
          last_name: "Dubois",
          role: "owner",
          organization_id: org.id,
        },
      }),
      "amorçage:élu",
    );
    await ok(
      await api.post(`${API_BASE}/auth/register`, {
        data: {
          email: emailSyndic,
          password: motDePasse,
          first_name: "François",
          last_name: "Leroy",
          role: "syndic",
          organization_id: org.id,
        },
      }),
      "amorçage:syndic",
    );

    // La fiche de copropriétaire, LIÉE au compte : le conseil se compose de
    // copropriétaires, et c'est la fiche qui porte cette qualité — pas le
    // compte.
    const eluUserId = eluUser.user?.id ?? eluUser.id ?? eluUser.user_id;
    await ok(
      await api.post(`${API_BASE}/owners`, {
        data: {
          organization_id: org.id,
          first_name: "Alice",
          last_name: "Dubois",
          email: emailElu,
          address: "1 Avenue du Conseil",
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          user_id: eluUserId,
        },
        headers: entete,
      }),
      "amorçage:fiche du copropriétaire",
    );

    // L'assemblée qui élira : un conseil ne se nomme pas, il s'élit.
    //
    // Créée par le SYNDIC, pas par l'administrateur. Le superadmin
    // n'appartient à aucune organisation — c'est une contrainte de
    // conception (`docs/personas/superadmin.md`), et le serveur la fait
    // respecter : il rend `401 User does not belong to an organization`.
    //
    // Le refus est juste, et il est même rassurant : un compte qui gère la
    // plateforme ne doit pas pouvoir convoquer une assemblée chez un de ses
    // clients.
    const jetonSyndic = (
      await ok<{ token: string }>(
        await api.post(`${API_BASE}/auth/login`, {
          data: { email: emailSyndic, password: motDePasse },
        }),
        "amorçage:connexion du syndic",
      )
    ).token;

    await ok(
      await api.post(`${API_BASE}/meetings`, {
        data: {
          building_id: immeuble.id,
          organization_id: org.id,
          title: `Assemblée générale ordinaire ${horodatage}`,
          scheduled_date: new Date(Date.now() + 30 * 86400000).toISOString(),
          meeting_type: "Ordinary",
          location: "Salle commune",
          is_second_convocation: true,
        },
        headers: { Authorization: `Bearer ${jetonSyndic}` },
      }),
      "amorçage:assemblée",
    );

    await page.context().clearCookies();

    return {
      syndic: { email: emailSyndic, motDePasse },
      conseil: { email: emailElu, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-le-syndic-ouvre-le-conseil",
      acteur: "syndic",
      description:
        "KoproGo — le conseil de copropriété. Au-delà de vingt lots, la loi " +
        "belge l'impose : un contre-pouvoir élu qui surveille le syndic.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller("/syndic/board-members");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // `/syndic/board-members` monte `BoardManagement`, pas
        // `BoardMemberList` — ce dernier sert au tableau du conseil. Deux
        // composants, deux écrans, et j'ai visé le mauvais au premier jet.
        await expect(page.getByTestId("board-management")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-elire-un-membre",
      acteur: "syndic",
      description:
        "Élire un membre. Le syndic propose, mais c'est l'assemblée qui " +
        "élit — le formulaire demande d'ailleurs par QUELLE assemblée le " +
        "mandat a été conféré.",
      action: async (scene) => {
        await scene.attendreChargement();
        // ── Pourquoi cette étape va maintenant jusqu'au bout ─────────────
        //
        // Elle ouvrait le formulaire et s'arrêtait là, au motif que
        // `SyndicCreationJourneys` exerce déjà l'élection. C'était vrai, et
        // c'était le mauvais argument : le gate couvrait l'élection, la
        // VITRINE montrait un formulaire ouvert sous un titre qui annonçait
        // une élection. L'étape suivante s'appelle « l'élue découvre son
        // mandat » — elle ne pouvait découvrir aucun mandat, puisque
        // personne ne l'avait élue (#974).
        await scene.cliquer("board-elect-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le sélecteur d'immeuble est la porte de l'élection : on élit au
        // conseil D'UNE copropriété, jamais dans l'absolu.
        await expect(page.getByTestId("board-building-select")).toBeVisible({
          timeout: 20000,
        });
        await expect(
          page.getByTestId("board-elect-form"),
          "Le bouton d'élection est visible mais n'ouvre aucun formulaire : " +
            "la capacité est affichée sans être atteignable.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("board-elect-meeting")).toBeVisible();
      },
    },
    {
      id: "2bis-le-mandat-est-confere",
      acteur: "syndic",
      description:
        "Il désigne Alice Dubois comme présidente, cite l'assemblée qui l'a " +
        "élue, et borne le mandat à UN AN. Ce n'est pas un réglage : " +
        "l'Art. 3.90 du Code civil fixe la durée, et le serveur refuse tout " +
        "ce qui s'en écarte de plus de deux mois.",
      action: async (scene) => {
        await scene.choisirQuiContient("board-elect-owner", "Dubois");
        await scene.choisir("board-elect-position", "president");
        // L'assemblée qui confère : la seule de cet immeuble, semée à
        // l'amorçage. C'est le champ qui matérialise « le syndic propose,
        // l'assemblée élit ».
        await scene.page
          .getByTestId("board-elect-meeting")
          .selectOption({ index: 1 });
        // ── Un an, et pas trois ──────────────────────────────────────────
        //
        // Ma première version posait trois ans, en citant l'Art. 3.88. Le
        // produit m'a corrigé : `BoardMember::new` exige une durée de
        // 330 à 395 jours et cite l'Art. 3.90 CC, « conseil de
        // copropriété ». La contrainte de base `mandate_duration_one_year`
        // dit la même chose.
        //
        // L'échec était SILENCIEUX : le formulaire accepte les deux dates,
        // le serveur rend 400, et le refus passe par un toast qui s'efface.
        // Rien dans le formulaire n'annonce la règle (#980).
        const aujourdHui = new Date();
        const dansUnAn = new Date(aujourdHui);
        dansUnAn.setFullYear(dansUnAn.getFullYear() + 1);
        await scene.saisir(
          "board-elect-mandate-start",
          aujourdHui.toISOString().slice(0, 10),
        );
        await scene.saisir(
          "board-elect-mandate-end",
          dansUnAn.toISOString().slice(0, 10),
        );
        await scene.cliquer("board-elect-submit-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le formulaire s'est refermé ET le conseil compte un membre.
        // Vérifier seulement la fermeture ne distinguerait pas une élection
        // d'une annulation.
        await expect(page.getByTestId("board-elect-form")).toHaveCount(0, {
          timeout: 20000,
        });
        await expect(
          page.getByTestId("board-member-remove-button"),
          "L'élection a l'air d'aboutir mais le conseil reste vide : " +
            "l'étape suivante montrerait une élue sans mandat.",
        ).toHaveCount(1, { timeout: 20000 });
        // ── Ce que la carte du membre montre, et ce qu'elle devrait ──────
        //
        // Le mandat est bien conféré : poste, dates et jours restants s'y
        // lisent. Mais l'élue y figure sous son UUID —
        // « ID du copropriétaire: 553e245c-… » — et non sous son nom
        // (`BoardManagement.svelte:300`).
        //
        // L'assertion porte donc sur ce qui EST vrai, avec l'issue qui le
        // ferme (#980). Écrire `toContainText("Dubois")` ferait échouer le
        // parcours sur un défaut d'affichage qu'il n'est pas là pour
        // corriger, et une assertion qu'on souhaite ferait mentir la
        // vitrine.
        await expect(page.getByTestId("board-management")).toContainText(
          "Président",
        );
        await expect(page.getByTestId("board-management")).toContainText("365");
      },
    },
    {
      id: "3-l-elu-decouvre-son-mandat",
      acteur: "conseil",
      description:
        "Au tour de l'élue. Elle n'a rien demandé : le mandat vient d'être " +
        "conféré sous nos yeux, et elle le reçoit. C'est ce qui distingue " +
        "le conseil d'un comité de volontaires.",
      action: async (scene) => {
        await scene.devenir("conseil");
      },
      assertion: async (page) => {
        // Elle arrive sur SON écran de copropriétaire : le mandat ne change
        // pas ce qu'elle est, il ajoute ce qu'elle surveille.
        await expect(page.getByTestId("owner-dashboard")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "4-le-tableau-du-conseil",
      acteur: "conseil",
      description:
        "Le tableau du conseil : les décisions de l'assemblée et leur " +
        "suivi. Surveiller le syndic, c'est d'abord savoir ce qui a été " +
        "décidé et ce qui ne l'a pas encore été fait.",
      action: async (scene) => {
        await scene.aller("/board-dashboard");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("board-dashboard")).toBeVisible({
          timeout: 20000,
        });
      },
    },
  ],
};
