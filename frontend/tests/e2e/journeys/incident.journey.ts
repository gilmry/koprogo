/**
 * Parcours de référence — un incident signalé, reçu, confié, clos.
 *
 * ── Pourquoi ce parcours existe ───────────────────────────────────────────
 *
 * Arbitrage du PO, le 2026-09-20 : « la vitrine se consacre sur les écrans,
 * parfois elle dit *sans cliquer* / *pour avoir testé l'interface* ».
 *
 * Le reproche était mesurable. Sur les sept parcours métier filmés à cette
 * date, 43 étapes produisaient **7 clics, 3 saisies et 2 sélections** — et
 * 25 navigations. La vitrine ouvrait des écrans et constatait qu'un bouton
 * s'y trouvait. Un bouton visible n'est pas un bouton qui marche, et une
 * galerie de captures n'est pas une preuve de valeur.
 *
 * Ce parcours-ci ne navigue que pour arriver quelque part. Tout le reste est
 * un geste : on remplit, on choisit, on coche, on envoie, on cherche, on
 * ouvre, on assigne, on résout, on clôture.
 *
 * ── Ce qu'il DÉMONTRE ─────────────────────────────────────────────────────
 *
 * Le cycle de vie complet d'un incident, à travers deux rôles :
 *
 *     Open ──(le syndic assigne)──> InProgress ──(résolu)──> Resolved
 *                                                    ──(clôturé)──> Closed
 *
 * C'est le cas d'usage qui justifie un logiciel de syndic à lui seul : une
 * copropriétaire constate une fuite, elle la signale, et quelqu'un s'en
 * occupe. Tant que ce cycle n'est pas filmé de bout en bout, le produit
 * n'a pas montré ce pour quoi on l'achète.
 *
 * ── Comment la transition d'état est prouvée ──────────────────────────────
 *
 * `TicketStatusBadge.svelte` ne porte **aucun** `data-testid` : on ne peut
 * pas lire le statut directement. Mais `TicketDetail.svelte` conditionne
 * chaque bouton d'action à un statut précis, et ces conditions sont exclusives :
 *
 *     ticket-assign-btn   ⇔ canManage && Open
 *     ticket-resolve-btn  ⇔ (contractor || canManage) && InProgress
 *     ticket-close-btn    ⇔ canManage && Resolved
 *     ticket-reopen-btn   ⇔ Closed || Cancelled
 *
 * L'apparition de `ticket-resolve-btn` PROUVE donc le passage à `InProgress`,
 * et ainsi de suite. C'est une preuve indirecte, et elle vaut mieux qu'une
 * lecture de texte traduit : elle porte sur ce que l'utilisateur peut FAIRE,
 * qui est la seule chose que l'état gouverne vraiment.
 *
 * ── Ce qu'il ne filme pas, et pourquoi ────────────────────────────────────
 *
 * Ce n'est pas le prestataire qui résout. `permissions.ts:148` range
 * `contractor` dans `ROLES_SANS_INTERFACE` : il n'a pas d'écran, et son point
 * d'entrée prévu — le lien magique — n'existe pas (story 3.2, cf.
 * `prestataire.journey.ts`). Le syndic enregistre donc la résolution.
 *
 * Filmer un prestataire qui clique serait filmer un monde qui n'existe pas.
 * Le jour où la story 3.2 atterrit, ce parcours gagne un acteur ; il n'aura
 * pas à être réécrit.
 */
import { expect } from "@playwright/test";
import type { Parcours } from "./parcours";
import { adminLogin, ensureAcp, seedConformantUnits } from "../helpers/auth";
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

/**
 * Le monde que `amorcer()` pose et dont les étapes ont besoin.
 *
 * Même raison que dans `coproprietaire.journey.ts` : `Parcours.amorcer()` ne
 * rend que des `Comptes`, et un identifiant d'immeuble n'en est pas un. Une
 * fermeture de module suffit — amorçage et étapes tournent dans le même
 * process, séquentiellement, pour un parcours à la fois.
 */
let monde: {
  buildingId: string;
  titreDuSignalement: string;
  /** Rempli à l'étape 6, quand le serveur a donné son identifiant au ticket. */
  ticketId: string | null;
} | null = null;

/**
 * Ce que le parcours a créé, pour les tests qui l'éprouvent SANS le rejouer.
 *
 * `incident.spec.ts` s'en sert pour deux questions que le parcours filmé ne
 * pose pas : un copropriétaire d'une autre organisation voit-il ce
 * signalement, et le formulaire refuse-t-il une date future ? Les deux
 * demandent le monde du parcours, aucune ne demande de le refilmer.
 *
 * Rend `null` avant le premier `amorcer()` — un appelant qui l'oublierait
 * doit le voir, pas hériter d'un identifiant vide qui ferait passer son test
 * au vert pour la mauvaise raison.
 */
export function mondeDuParcours(): {
  buildingId: string;
  titreDuSignalement: string;
  ticketId: string | null;
} | null {
  return monde;
}

/** La veille — `ticket-create-incident-date-input` refuse une date future. */
function hier(): string {
  const d = new Date(Date.now() - 24 * 3600 * 1000);
  return d.toISOString().slice(0, 10);
}

export const incident: Parcours = {
  slug: "incident",
  titre: "Un incident signalé, reçu, confié, clos",
  propos:
    "Le cycle de vie complet d'un incident, de la copropriétaire qui " +
    "constate une fuite au syndic qui clôture le dossier. Dix étapes, " +
    "toutes des gestes : on remplit le formulaire, on l'envoie, on cherche " +
    "le ticket dans la liste, on l'ouvre, on l'assigne, on le résout, on le " +
    "clôture. C'est le cas d'usage qui justifie un logiciel de syndic.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `incident-syndic-${horodatage}@example.com`;
    const emailCopro = `incident-owner-${horodatage}@example.com`;
    const emailPrestataire = `incident-plombier-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Incident Org ${horodatage}`,
          slug: `incident-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "incident");

    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          // L'horodatage est indispensable : `buildings.slug` est unique
          // GLOBALEMENT et le backend le dérive du nom. Sans lui, la deuxième
          // campagne échoue sur `buildings_slug_key` (constaté sur le
          // parcours copropriétaire, quatre échecs de ce seul fait).
          name: `Résidence du Parc ${horodatage}`,
          address: `${horodatage} Avenue des Tilleuls`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 6,
          total_tantiemes: 1000,
          construction_year: 1998,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );

    // Des lots conformes à l'acte de base. Ce parcours ne calcule aucune
    // charge, mais le tableau de bord du syndic en calcule, et un immeuble
    // non conforme l'ouvrirait sur un 422 (ADR-0010).
    await seedConformantUnits(page, adminToken, acpId, immeuble.id, 6, 1000);

    for (const [email, prenom, nom, role] of [
      [emailSyndic, "Sophie", "Syndic", "syndic"],
      [emailCopro, "Carine", "Copropriétaire", "owner"],
      // Le plombier. Il n'a pas d'écran (`ROLES_SANS_INTERFACE`), mais il
      // existe côté serveur — et c'est lui que `GET /tickets/assignable-users`
      // proposera au syndic à l'étape 8. Sans ce compte, la liste
      // d'assignation serait vide et l'étape filmerait un `<select>` sans
      // option, ce qui ressemblerait à une panne.
      [emailPrestataire, "Pierre", "Plombier", "contractor"],
    ] as const) {
      await ok(
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
    }

    monde = {
      buildingId: immeuble.id,
      // Unique : le syndic le cherchera à l'étape 6, et un titre partagé
      // avec une campagne précédente rendrait cette recherche non
      // concluante.
      titreDuSignalement: `Fuite au plafond du hall ${horodatage}`,
      ticketId: null,
    };

    // Le contexte de l'admin doit partir avant la première image : `injectAuth`
    // survit à un `clearCookies` et le parcours filmerait quelqu'un qu'il n'a
    // pas connecté.
    await page.context().clearCookies();

    return {
      syndic: { email: emailSyndic, motDePasse },
      copropriétaire: { email: emailCopro, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-la-coproprietaire-se-connecte",
      acteur: "copropriétaire",
      description:
        "Carine rentre chez elle et trouve une flaque dans le hall. Elle " +
        "se connecte à KoproGo pour le signaler — pas pour téléphoner au " +
        "syndic un vendredi à 19 h.",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
      },
    },
    {
      id: "2-le-formulaire-de-signalement",
      acteur: "copropriétaire",
      description:
        "Le formulaire de signalement, pour son immeuble. Il s'ouvre sur " +
        "une demande simple ; c'est elle qui décidera d'en faire une plainte.",
      action: async (scene) => {
        await scene.aller(`/tickets/new?buildingId=${monde!.buildingId}`);
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("ticket-create-form-element"),
          "Le formulaire ne se rend pas : sans lui, rien de ce parcours " +
            "n'est démontrable.",
        ).toBeVisible({ timeout: 20000 });
        // Les champs de plainte sont conditionnels : ils ne doivent PAS être
        // là avant qu'elle ne choisisse. C'est ce que l'étape 3 va révéler.
        await expect(
          page.getByTestId("ticket-create-incident-date-input"),
        ).toHaveCount(0);
      },
    },
    {
      id: "3-elle-en-fait-une-plainte",
      acteur: "copropriétaire",
      description:
        "Elle bascule sur « plainte » : la fuite dure depuis une semaine et " +
        "elle veut que ce soit consigné. Le formulaire s'ouvre alors sur la " +
        "gravité, la date de l'incident, les preuves et les témoins.",
      action: async (scene) => {
        await scene.choisir("ticket-create-kind-select", "complaint");
      },
      assertion: async (page) => {
        // La section conditionnelle est apparue. C'est un geste qui CHANGE
        // l'écran — le genre de chose qu'aucune capture ne montre.
        await expect(
          page.getByTestId("ticket-create-incident-date-input"),
          "Basculer sur « plainte » n'ouvre pas la section correspondante : " +
            "le champ `kind` est décoratif.",
        ).toBeVisible({ timeout: 10000 });
        await expect(
          page.getByTestId("ticket-severity-radio-high"),
        ).toBeVisible();
      },
    },
    {
      id: "4-elle-decrit-ce-qu-elle-a-vu",
      acteur: "copropriétaire",
      description:
        "Elle décrit : où, depuis quand, ce qu'elle a constaté. Le titre " +
        "fait au moins cinq caractères, la description au moins vingt — le " +
        "produit refuse les signalements qu'un syndic ne pourrait pas traiter.",
      action: async (scene) => {
        await scene.saisir(
          "ticket-create-title-input",
          monde!.titreDuSignalement,
        );
        await scene.saisir(
          "ticket-create-description-textarea",
          "Une flaque se forme chaque jour sous la trappe technique du hall " +
            "d'entrée, côté boîtes aux lettres. Le plafond est taché sur " +
            "environ un mètre carré et la peinture cloque. Cela dure depuis " +
            "une semaine environ.",
        );
      },
      assertion: async (page) => {
        // Le compteur de description est la seule chose qui dise à
        // l'utilisatrice si elle a assez écrit. S'il ne bouge pas, elle
        // découvre le refus au moment d'envoyer.
        await expect(
          page.getByTestId("ticket-create-description-counter"),
        ).toBeVisible();
      },
    },
    {
      id: "5-elle-qualifie",
      acteur: "copropriétaire",
      description:
        "Plomberie, priorité haute, gravité élevée, constaté hier. Cette " +
        "qualification n'est pas cosmétique : la priorité fixe le délai de " +
        "réponse que le syndic s'engage à tenir.",
      action: async (scene) => {
        await scene.choisir("ticket-create-category-select", "Plumbing");
        await scene.choisir("ticket-create-priority-select", "High");
        await scene.cliquer("ticket-severity-radio-high");
        await scene.saisir("ticket-create-incident-date-input", hier());
      },
      assertion: async (page) => {
        // Aucune preuve jointe : le produit le SIGNALE plutôt que de refuser.
        // Une copropriétaire qui n'a pas pensé à photographier ne doit pas
        // être empêchée de signaler une fuite.
        await expect(
          page.getByTestId("ticket-create-evidence-warning"),
          "Une plainte sans preuve ni témoin devrait porter un avertissement " +
            "visible — sans quoi personne ne sait qu'il manque quelque chose.",
        ).toBeVisible({ timeout: 10000 });
        // Et la date de l'incident est acceptée : hier n'est pas dans le futur.
        await expect(
          page.getByTestId("ticket-create-incident-date-error"),
        ).toHaveCount(0);
      },
    },
    {
      id: "6-elle-envoie",
      acteur: "copropriétaire",
      description:
        "Elle envoie. C'est ici que tout se joue : le signalement existe " +
        "désormais, il porte son nom, et il attend quelqu'un.",
      action: async (scene) => {
        await scene.cliquer("ticket-create-submit");
        await scene.page.waitForURL(/\/ticket-detail\?id=/, {
          timeout: 20000,
        });
        // L'identifiant que le serveur vient d'attribuer. On le retient pour
        // les tests qui éprouvent le cloisonnement sans rejouer le parcours.
        monde!.ticketId = new URL(scene.page.url()).searchParams.get("id");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // LA preuve de ce parcours : le ticket a été créé, le serveur lui a
        // donné un identifiant, et l'écran de détail le rend. Une assertion
        // sur le formulaire (« pas d'erreur affichée ») n'aurait rien prouvé
        // — un formulaire qui n'envoie rien n'affiche pas d'erreur non plus.
        await expect(page.getByTestId("ticket-detail")).toBeVisible({
          timeout: 20000,
        });
        await expect(page.getByTestId("ticket-detail-title")).toHaveText(
          monde!.titreDuSignalement,
        );
      },
    },
    {
      id: "7-le-syndic-le-trouve",
      acteur: "syndic",
      description:
        "Lundi matin. Sophie ouvre la liste des incidents de l'immeuble et " +
        "cherche celui-là. Elle n'a pas été prévenue par téléphone : le " +
        "signalement l'attendait.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller(`/tickets?building_id=${monde!.buildingId}`);
        await scene.attendreChargement();
        // Un vrai geste de recherche, pas un lien direct : c'est ce qu'une
        // syndic fait quand sa liste en contient trente.
        await scene.saisir("ticket-search-input", monde!.titreDuSignalement);
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("ticket-row"),
          "La recherche ne ramène pas le signalement : soit le syndic ne " +
            "voit pas les tickets de son immeuble, soit le filtre ne filtre " +
            "rien. Les deux sont des défauts.",
        ).toHaveCount(1, { timeout: 20000 });
        await expect(page.getByTestId("ticket-row")).toContainText(
          monde!.titreDuSignalement,
        );
      },
    },
    {
      id: "8-elle-le-confie-au-plombier",
      acteur: "syndic",
      description:
        "Elle ouvre le dossier et le confie à Pierre, le plombier. " +
        "L'incident passe « en cours » — et le bouton « assigner » " +
        "disparaît, parce qu'il n'y a plus rien à assigner.",
      action: async (scene) => {
        await scene.cliquerLePremier("ticket-row");
        await scene.attendreChargement();
        await scene.cliquer("ticket-assign-btn");
        // Par le NOM, pas par l'index : `GET /tickets/assignable-users` rend
        // aussi les syndics et les membres du conseil, dans un ordre que rien
        // ne garantit. Le parcours narre qu'elle le confie au plombier — il
        // doit le confier au plombier.
        await scene.choisirQuiContient("ticket-assignee-select", "Plombier");
        await scene.cliquer("ticket-assign-submit-btn");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // `ticket-resolve-btn` n'existe QUE si le statut est `InProgress`
        // (TicketDetail.svelte:208). Son apparition prouve la transition,
        // et `ticket-assign-btn` (⇔ `Open`) doit avoir disparu : les deux
        // conditions sont exclusives, donc voir les deux serait un défaut.
        await expect(
          page.getByTestId("ticket-resolve-btn"),
          "L'assignation n'a pas fait passer l'incident « en cours » : le " +
            "geste a l'air d'aboutir sans rien changer.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("ticket-assign-btn")).toHaveCount(0);
        // Et le dossier porte bien le nom de qui l'a signalé : c'est ce qui
        // permet au syndic de rappeler la bonne personne.
        await expect(page.getByTestId("ticket-detail-metadata")).toContainText(
          "Carine",
        );
      },
    },
    {
      id: "9-la-fuite-est-reparee",
      acteur: "syndic",
      description:
        "Le plombier est passé. Sophie consigne ce qui a été fait — le " +
        "produit le lui demande, parce qu'une plainte close sans explication " +
        "ne vaut pas mieux qu'une ligne effacée d'un tableur.",
      action: async (scene) => {
        await scene.cliquer("ticket-resolve-btn");
        await scene.saisir(
          "ticket-transition-motif",
          "Joint de la colonne d'eau froide remplacé au niveau de la trappe " +
            "technique. Plafond asséché, reprise de peinture prévue à la " +
            "prochaine campagne d'entretien.",
        );
        await scene.cliquer("ticket-transition-submit");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // La demande de motif s'est refermée : la transition a abouti.
        await expect(page.getByTestId("ticket-transition-form")).toHaveCount(
          0,
          {
            timeout: 20000,
          },
        );
        // `ticket-close-btn` ⇔ `Resolved`. C'est ici que le parcours est
        // tombé au rouge la première fois : `ticketsApi.resolve()` envoyait
        // un corps vide à un serveur qui exige `resolution_notes`, le bouton
        // affichait un toast d'échec, et l'écran restait identique (#977).
        await expect(
          page.getByTestId("ticket-close-btn"),
          "L'incident n'est pas passé « résolu » : le bouton de clôture " +
            "n'apparaît pas.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("ticket-resolve-btn")).toHaveCount(0);
      },
    },
    {
      id: "10-le-dossier-se-ferme",
      acteur: "syndic",
      description:
        "Elle clôture. Le dossier reste consultable et rouvrable — un " +
        "incident clos n'est pas un incident effacé, et c'est ce qui permet " +
        "d'y revenir si la fuite recommence. La clôture, elle, ne demande " +
        "rien de plus : ce qui devait être écrit l'a été à l'étape " +
        "précédente.",
      action: async (scene) => {
        await scene.cliquer("ticket-close-btn");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // `ticket-reopen-btn` ⇔ `Closed || Cancelled`. Le cycle est complet.
        await expect(
          page.getByTestId("ticket-reopen-btn"),
          "L'incident clos ne peut pas être rouvert : une fuite qui " +
            "recommence obligerait à créer un second dossier, sans lien avec " +
            "le premier.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("ticket-detail-title")).toHaveText(
          monde!.titreDuSignalement,
        );
      },
    },
  ],
};
