/**
 * Parcours de référence — consulter la copropriété, et compter les voix.
 *
 * ── Ce qu'il démontre ─────────────────────────────────────────────────────
 *
 * Une décision prise entre deux assemblées. L'Art. 3.87 du Code civil réserve
 * les votes contraignants à l'assemblée générale ; entre deux AG, un syndic
 * qui veut savoir ce que pensent les copropriétaires n'a rien d'autre qu'un
 * courriel groupé et des réponses éparpillées.
 *
 * Le sondage n'est pas un vote d'AG et ne prétend pas l'être. Il sert à
 * préparer l'ordre du jour : si trente copropriétaires sur trente-deux sont
 * contre le remplacement de l'ascenseur, la résolution n'a pas besoin d'être
 * rédigée. C'est du temps d'assemblée épargné, et de l'argent de syndic.
 *
 * ── Pourquoi ce parcours clique, et ne se contente pas d'ouvrir ───────────
 *
 * Il joue le cycle entier :
 *
 *     Draft ──(publier)──> Active ──(un vote)──> ──(clôturer)──> Closed
 *                                                         └─> résultats
 *
 * Chaque transition est un bouton que quelqu'un presse, et le vote est
 * déposé par une AUTRE personne que celle qui a ouvert le sondage. Un
 * parcours mono-acteur qui voterait dans son propre sondage ne démontrerait
 * rien : c'est la séparation qui fait la valeur.
 *
 * ── Ce qu'il vérifie de plus que « l'écran se rend » ──────────────────────
 *
 * Que le dépouillement compte. `poll-results-section` n'apparaît qu'à la
 * clôture, et le parcours y lit le libellé de l'option choisie par la
 * copropriétaire. Un sondage qui perdrait les voix afficherait exactement le
 * même écran de clôture — c'est le genre de défaut qu'une capture ne montre
 * jamais.
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

/** Le monde posé par l'amorçage, que les étapes doivent connaître. */
let monde: {
  nomDeLImmeuble: string;
  question: string;
  /** Rempli à l'étape 3 : le sondage existe et porte un identifiant. */
  sondageId: string | null;
} | null = null;

/** Ce que la copropriétaire choisit. Le dépouillement doit le retrouver. */
const OPTION_CHOISIE = "Réparer, et provisionner le remplacement";

export function mondeDuParcours(): typeof monde {
  return monde;
}

export const sondage: Parcours = {
  slug: "sondage",
  titre: "Consulter la copropriété, et compter les voix",
  propos:
    "Un syndic prépare son assemblée sans convoquer personne : il ouvre un " +
    "sondage, le publie, une copropriétaire vote, il clôture et lit le " +
    "dépouillement. Huit étapes, presque toutes des gestes — on choisit un " +
    "immeuble, on rédige la question, on ajoute une option, on coche, on " +
    "dépose sa voix.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `sondage-syndic-${horodatage}@example.com`;
    const emailCopro = `sondage-owner-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Sondage Org ${horodatage}`,
          slug: `sondage-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "sondage");

    // Le nom porte l'horodatage : `buildings.slug` est unique globalement et
    // le backend le dérive du nom. C'est aussi ce qui permet à l'étape 2 de
    // désigner CET immeuble dans une liste déroulante partagée.
    const nomDeLImmeuble = `Résidence de l'Ascenseur ${horodatage}`;
    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: nomDeLImmeuble,
          address: `${horodatage} Rue de la Cage`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 4,
          total_tantiemes: 1000,
          construction_year: 1975,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );

    await seedConformantUnits(page, adminToken, acpId, immeuble.id, 4, 1000);

    // ── Un SECOND immeuble, et il n'est pas décoratif ─────────────────────
    //
    // `BuildingSelector.svelte:182` n'affiche une liste déroulante que si le
    // syndic gère PLUSIEURS immeubles ; avec un seul, il rend un simple
    // encadré (`building-selected`) et il n'y a rien à choisir.
    //
    // Mesuré au navigateur le 2026-09-20 : l'étape 2 attendait un `<select>`
    // et trouvait du texte. Semer un second immeuble n'est donc pas un
    // contournement — c'est ce qui rend le geste possible, et c'est aussi
    // la situation réelle d'un cabinet syndic, qui n'en gère jamais un seul.
    const secondImmeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: `Résidence des Tilleuls ${horodatage}`,
          address: `${horodatage} Rue des Tilleuls`,
          city: "Brussels",
          postal_code: "1030",
          country: "Belgium",
          total_units: 3,
          total_tantiemes: 1000,
          construction_year: 2004,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:second-immeuble",
    );
    await seedConformantUnits(
      page,
      adminToken,
      acpId,
      secondImmeuble.id,
      3,
      1000,
    );

    const comptes: Array<[string, string, string, string]> = [
      [emailSyndic, "Sophie", "Syndic", "syndic"],
      [emailCopro, "Carine", "Copropriétaire", "owner"],
    ];
    let idDuCompteCopro = "";
    for (const [email, prenom, nom, role] of comptes) {
      const cree = await ok<{ token: string; user?: { id?: string }; id?: string }>(
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
      if (role === "owner") {
        idDuCompteCopro = cree.user?.id ?? cree.id ?? "";
      }
    }

    // Un compte `owner` n'est pas un copropriétaire tant qu'il ne possède
    // rien — et sans `unit_owners`, `POST /polls` refuse en 400 « Total
    // eligible voters must be positive ».
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
      question: `Faut-il remplacer l'ascenseur cette année ? ${horodatage}`,
      sondageId: null,
    };

    await page.context().clearCookies();

    return {
      syndic: { email: emailSyndic, motDePasse },
      copropriétaire: { email: emailCopro, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-le-syndic-ouvre-la-question",
      acteur: "syndic",
      description:
        "L'ascenseur tombe en panne tous les deux mois. Avant de rédiger " +
        "une résolution pour l'assemblée, Sophie veut savoir ce que pensent " +
        "les copropriétaires.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller("/polls/new");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("create-poll-form")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-elle-redige",
      acteur: "syndic",
      description:
        "Elle gère deux immeubles : elle désigne celui dont l'ascenseur " +
        "tombe en panne, pose la question, et bascule sur un choix " +
        "multiple. Entre « oui » et « non », il y a « réparer encore une " +
        "fois » — une consultation qui n'offre que deux cases oriente la " +
        "réponse.",
      action: async (scene) => {
        await scene.choisirQuiContient(
          "building-selector",
          monde!.nomDeLImmeuble,
        );
        await scene.choisir("create-poll-type-select", "multiple_choice");
        await scene.saisir("create-poll-question-input", monde!.question);
        await scene.saisir(
          "create-poll-description-input",
          "L'ascenseur a été immobilisé quatre fois cette année. Le devis " +
            "de remplacement s'élève à 68 000 € hors TVA, à répartir aux " +
            "quotités. Cette consultation prépare l'ordre du jour ; elle ne " +
            "remplace pas le vote en assemblée (Art. 3.87).",
        );
      },
      assertion: async (page) => {
        // Basculer le type vide la liste d'options (`onPollTypeChange`) :
        // l'étape suivante doit donc les saisir. Si des options « Oui / Non »
        // survivaient ici, la consultation partirait avec des choix que
        // personne n'a écrits.
        await expect(page.getByTestId("poll-create-option-text-input")).toHaveCount(
          0,
        );
      },
    },
    {
      id: "3-elle-ecrit-les-choix",
      acteur: "syndic",
      description:
        "Trois options, écrites une par une. C'est le geste que la vitrine " +
        "ne montrait jamais : remplir un champ, presser « ajouter », " +
        "recommencer.",
      action: async (scene) => {
        for (const choix of [
          "Remplacer l'ascenseur cette année",
          OPTION_CHOISIE,
          "Ne rien faire pour l'instant",
        ]) {
          await scene.saisir("create-poll-new-option-input", choix);
          await scene.cliquer("create-poll-add-option-btn");
        }
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("poll-create-option-text-input"),
          "Les options ajoutées ne s'affichent pas : le bouton « ajouter » " +
            "ne retient rien, et la consultation partirait sans choix.",
        ).toHaveCount(3, { timeout: 10000 });
      },
    },
    {
      id: "4-elle-ouvre-la-consultation",
      acteur: "syndic",
      description:
        "Elle fixe la date de clôture et envoie. Le sondage naît en " +
        "brouillon : rien n'est visible des copropriétaires tant qu'elle " +
        "n'a pas publié.",
      action: async (scene) => {
        const dansUneSemaine = new Date(Date.now() + 7 * 24 * 3600 * 1000)
          .toISOString()
          .slice(0, 10);
        await scene.saisir("create-poll-end-date-input", dansUneSemaine);
        await scene.cliquer("create-poll-submit-btn");
        await scene.page.waitForURL(/\/polls\/detail\?id=/, { timeout: 30000 });
        monde!.sondageId = new URL(scene.page.url()).searchParams.get("id");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("poll-detail")).toBeVisible({
          timeout: 20000,
        });
        // En brouillon : le bouton de publication existe, celui de clôture
        // pas encore. Les deux conditions sont exclusives dans
        // `PollDetail.svelte:323-340`, donc voir les deux serait un défaut.
        await expect(
          page.getByTestId("poll-publish-button"),
          "Le sondage créé n'est pas en brouillon : il serait déjà visible " +
            "des copropriétaires, avant relecture.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("poll-close-button")).toHaveCount(0);
      },
    },
    {
      id: "5-elle-publie",
      acteur: "syndic",
      description:
        "Elle relit, publie, et confirme. À partir de cet instant, la " +
        "consultation est ouverte et chaque copropriétaire peut y déposer " +
        "une voix — la question, elle, ne se réécrit plus.",
      action: async (scene) => {
        await scene.cliquer("poll-publish-button");
        // Publier demande confirmation, et c'est justifié : une fois la
        // consultation ouverte, la question ne se réécrit plus sans invalider
        // les voix déjà déposées.
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // `poll-close-button` ⇔ `Active`. Son apparition prouve la
        // transition ; `poll-publish-button` (⇔ `Draft`) doit avoir disparu.
        await expect(
          page.getByTestId("poll-close-button"),
          "La publication n'a rien changé : le sondage reste en brouillon " +
            "et personne ne le verra.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("poll-publish-button")).toHaveCount(0);
      },
    },
    {
      id: "6-la-coproprietaire-trouve-la-consultation",
      acteur: "copropriétaire",
      description:
        "Carine se connecte et trouve la consultation dans sa liste. Elle " +
        "n'a pas eu besoin d'un courriel avec un lien : le produit la lui " +
        "présente.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
        await scene.aller("/polls");
        await scene.attendreChargement();
        await scene.cliquerLePremier("poll-list-detail-link");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("poll-voting-section"),
          "La copropriétaire voit la consultation mais ne peut pas y voter : " +
            "une consultation qu'on ne peut que lire n'en est pas une.",
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "7-elle-depose-sa-voix",
      acteur: "copropriétaire",
      description:
        "Elle choisit la solution intermédiaire — réparer maintenant, " +
        "provisionner le remplacement — et dépose sa voix.",
      action: async (scene) => {
        await scene.page
          .getByTestId("poll-detail-option-input")
          .nth(1)
          .check();
        await scene.cliquer("poll-vote-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le bulletin déposé, l'urne se referme pour elle : `canVote()`
        // devient faux dès `hasVoted`. Un écran qui laisserait le bouton
        // actif inviterait à voter deux fois.
        await expect(
          page.getByTestId("poll-vote-button"),
          "Le bouton de vote reste actif après le dépôt : rien n'empêche " +
            "visuellement de voter deux fois.",
        ).toHaveCount(0, { timeout: 20000 });
      },
    },
    {
      id: "8-le-depouillement",
      acteur: "syndic",
      description:
        "Sophie clôture et lit le dépouillement. La voix de Carine y est, " +
        "avec son libellé — c'est ce résultat qui décidera si une " +
        "résolution part à l'ordre du jour.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller(`/polls/detail?id=${monde!.sondageId}`);
        await scene.attendreChargement();
        await scene.cliquer("poll-close-button");
        await scene.cliquer("confirm-dialog-confirm");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("poll-results-section"),
          "La clôture n'affiche aucun dépouillement : le sondage aura " +
            "collecté des voix que personne ne peut lire.",
        ).toBeVisible({ timeout: 20000 });
        // Et la voix déposée s'y retrouve. Sans cette ligne, un sondage qui
        // perdrait tous ses bulletins afficherait le même écran de clôture.
        await expect(
          page.getByTestId("poll-results-section"),
          "Le dépouillement ne mentionne pas l'option choisie : les voix " +
            "sont comptées ailleurs, ou pas comptées du tout.",
        ).toContainText(OPTION_CHOISIE);
      },
    },
  ],
};
