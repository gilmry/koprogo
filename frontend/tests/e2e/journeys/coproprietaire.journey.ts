/**
 * Parcours de référence — le copropriétaire, dix étapes (#807).
 *
 * ── Pourquoi ce parcours-ci ────────────────────────────────────────────────
 *
 * Le rôle `owner` est le plus peuplé en base (5 comptes de recette) et le
 * seul que cinq recettes navigateur n'ont jamais éprouvé de bout en bout,
 * faute d'un mot de passe qui fonctionne (cf. `docs/personas/owner.md`).
 * Les dix étapes ci-dessous reprennent, dans l'ordre, le tableau de l'issue
 * #807 : de la connexion au paiement d'un appel de fonds, en passant par le
 * vote et les modules communautaires.
 *
 * ── Ce que ce parcours tranche ─────────────────────────────────────────────
 *
 * **Le multi-ACP.** Le copropriétaire amorcé ici détient un lot MINORITAIRE
 * (12 % et 20 % des tantièmes) dans deux copropriétés distinctes. La règle
 * produit — « agréger pour informer, séparer pour agir » — est vérifiée à
 * l'étape 2 : un total consolidé pour savoir, un bouton par ACP pour agir.
 *
 * **#781** (« le copropriétaire peut-il faire ce que le syndic ne peut pas »)
 * est tranchée par l'observation à l'étape 10 : offrir une compétence
 * réussit pour le copropriétaire, exactement le comportement que la revue
 * de design attendait.
 *
 * ── Ce que ce parcours documente comme rouge ──────────────────────────────
 *
 * Deux étapes atteignent un écran **cassé aujourd'hui**, et l'assertion porte
 * sur le comportement OBSERVÉ (pas sur le comportement souhaité) — la règle
 * du témoin : on ne supprime ni n'invente une assertion, on décrit ce qui se
 * passe réellement, avec l'issue qui le ferme.
 *
 *   - Étape 5 (payer un appel de fonds) — le bouton « Payer » de l'étape 2
 *     mène à un écran de LECTURE SEULE ; le formulaire de paiement existe
 *     ailleurs (`/owner-contributions`) mais cette route est réservée au
 *     syndic et au comptable (#807).
 *
 * Un second point rouge sur les modules communautaires — prêter un objet —
 * n'est PAS filmé dans ce parcours : il vit dans un test `@negative` dédié de
 * `coproprietaire.spec.ts`, parce qu'il caractérise un appel d'API
 * (`sharing.ts` appelle `POST /loans`, une route que le backend ne sert pas —
 * il expose `/shared-objects/{id}/borrow`, #779) plutôt qu'un geste
 * d'interface qui aurait sa place dans la vitrine.
 */
import { expect } from "@playwright/test";
import type { APIRequestContext } from "@playwright/test";
import type { Parcours } from "./parcours";
import { adminLogin } from "../helpers/auth";
import { API_BASE } from "../helpers/adresses";

async function assertOk<T = any>(
  resp: {
    status: () => number;
    ok: () => boolean;
    text: () => Promise<string>;
    json: () => Promise<any>;
  },
  label: string,
): Promise<T> {
  if (!resp.ok()) {
    throw new Error(
      `${label}: HTTP ${resp.status()} — ${(await resp.text()).slice(0, 200)}`,
    );
  }
  return (await resp.json()) as T;
}

/** Une ACP conforme à deux lots : celui du copropriétaire (minoritaire) et un second, non lié, qui complète les tantièmes. */
async function creerAcpAvecLotMinoritaire(
  api: APIRequestContext,
  adminToken: string,
  orgId: string,
  nom: string,
  quotaProprietaire: number,
): Promise<{ acpId: string; buildingId: string; uniteId: string }> {
  const horodatage = Date.now();

  const acpResp = await api.post(`${API_BASE}/acps`, {
    data: {
      organization_id: orgId,
      name: `${nom} ${horodatage}`,
      address_street: `${horodatage} Rue Test`,
      address_postal_code: "1000",
      address_city: "Brussels",
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const acp = await assertOk<{ id: string }>(acpResp, `seed:acp:${nom}`);

  const buildingResp = await api.post(`${API_BASE}/buildings`, {
    data: {
      // L'horodatage est INDISPENSABLE ici, comme sur l'ACP juste au-dessus.
      // `buildings.slug` est unique GLOBALEMENT et le backend le dérive du
      // nom : sans lui, « Les Erables — immeuble » produit le même slug à
      // chaque campagne, et ce parcours échoue dès la deuxième exécution sur
      // `buildings_slug_key`. Mesuré le 2026-09-16 — 4 échecs de ce seul fait.
      name: `${nom} — immeuble ${horodatage}`,
      address: `${horodatage} Rue Test`,
      city: "Brussels",
      postal_code: "1000",
      country: "Belgium",
      total_units: 2,
      total_tantiemes: 1000,
      construction_year: 2010,
      acp_id: acp.id,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const building = await assertOk<{ id: string }>(
    buildingResp,
    `seed:building:${nom}`,
  );

  // Le lot du copropriétaire — minoritaire, pour que l'étape 3 puisse
  // vérifier si le montant affiché suit vraiment sa quote-part.
  const uniteResp = await api.post(`${API_BASE}/units`, {
    data: {
      acp_id: acp.id,
      building_id: building.id,
      unit_number: "1A",
      floor: 0,
      surface_area: 65,
      unit_type: "Apartment",
      quota: quotaProprietaire,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  const unite = await assertOk<{ id: string }>(uniteResp, `seed:unit:${nom}`);

  // Le second lot, non lié à personne dans ce parcours : uniquement là pour
  // que SUM(quota) == total_tantiemes (immeuble conforme, cf. Track H
  // Story H2 — un immeuble non conforme bloque tout calcul de charge en 422).
  const secondUniteResp = await api.post(`${API_BASE}/units`, {
    data: {
      acp_id: acp.id,
      building_id: building.id,
      unit_number: "2A",
      floor: 0,
      surface_area: 65,
      unit_type: "Apartment",
      quota: 1000 - quotaProprietaire,
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  await assertOk(secondUniteResp, `seed:unit-2:${nom}`);

  return { acpId: acp.id, buildingId: building.id, uniteId: unite.id };
}

/** Une charge en attente sur l'immeuble, pour peupler « Mes dépenses » et le tableau des dettes par ACP. */
async function creerChargeEnAttente(
  api: APIRequestContext,
  adminToken: string,
  buildingId: string,
  description: string,
  montant: number,
): Promise<void> {
  const resp = await api.post(`${API_BASE}/expenses`, {
    data: {
      building_id: buildingId,
      category: "Maintenance",
      description,
      amount: montant,
      expense_date: new Date().toISOString(),
    },
    headers: { Authorization: `Bearer ${adminToken}` },
  });
  await assertOk(resp, `seed:expense:${description}`);
}

/**
 * L'état du monde que `amorcer()` construit et que les étapes 7-9 doivent
 * connaître pour naviguer vers LA bonne réunion.
 *
 * `Parcours.amorcer()` ne rend que des `Comptes` (des identifiants de
 * connexion) : ce n'en est pas un, donc on ne force pas ces champs dedans.
 * Une fermeture de module suffit — `amorcer` et les `etapes` du même fichier
 * s'exécutent dans le même process, séquentiellement, pour un seul parcours
 * à la fois (cf. `parcours.spec.ts`, qui les rejoue l'un après l'autre).
 */
let mondeSeme: {
  meetingId: string;
  resolutionId: string;
  uniteErablesId: string;
  buildingErablesId: string;
} | null = null;

export const coproprietaire: Parcours = {
  slug: "coproprietaire",
  titre:
    "Le copropriétaire, dix étapes — de la connexion au paiement d'un appel de fonds",
  propos:
    "Démontre le parcours du rôle le plus peuplé et le moins éprouvé du " +
    "produit (#807) : ses lots agrégés sur deux ACP, le vote en assemblée, " +
    "et les deux points où l'écran casse aujourd'hui (#807, #779).",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `coproprietaire-syndic-${horodatage}@example.com`;
    const emailOwner = `coproprietaire-owner-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);

    const orgResp = await api.post(`${API_BASE}/organizations`, {
      data: {
        name: `Coproprietaire Org ${horodatage}`,
        slug: `coproprietaire-${horodatage}`,
        contact_email: emailSyndic,
        subscription_plan: "professional",
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const org = await assertOk<{ id: string }>(orgResp, "seed:org");

    // Deux ACP, sous la même organisation (cabinet syndic) : c'est le cas
    // réel que `Owner.organization_id` unique permet — cf. la recherche
    // menée pour #807. Un lot minoritaire dans chacune, pour que l'agrégat
    // « ce que je dois » ne se réduise pas trivialement au 100 % d'un lot
    // unique.
    const acpErables = await creerAcpAvecLotMinoritaire(
      api,
      adminToken,
      org.id,
      "Les Erables",
      120, // 12 % des tantièmes
    );
    const acpGlycines = await creerAcpAvecLotMinoritaire(
      api,
      adminToken,
      org.id,
      "Les Glycines",
      200, // 20 % des tantièmes
    );

    // Montants volontairement ronds et très supérieurs à la quote-part
    // réelle du copropriétaire (144 € et 90 €) : si l'écran affichait un
    // jour le montant PRORATÉ, la différence avec ces totaux serait
    // immédiatement visible sans recalcul.
    await creerChargeEnAttente(
      api,
      adminToken,
      acpErables.buildingId,
      "Entretien toiture",
      1200,
    );
    await creerChargeEnAttente(
      api,
      adminToken,
      acpGlycines.buildingId,
      "Ravalement façade",
      450,
    );

    const syndicResp = await api.post(`${API_BASE}/auth/register`, {
      data: {
        email: emailSyndic,
        password: motDePasse,
        first_name: "Sophie",
        last_name: "Syndic",
        role: "syndic",
        organization_id: org.id,
      },
    });
    const syndic = await assertOk<{ token: string }>(syndicResp, "seed:syndic");
    const syndicToken = syndic.token;

    const ownerUserResp = await api.post(`${API_BASE}/auth/register`, {
      data: {
        email: emailOwner,
        password: motDePasse,
        first_name: "Carine",
        last_name: "Copropriétaire",
        role: "owner",
        organization_id: org.id,
      },
    });
    const ownerUser = await assertOk<{
      user?: { id?: string };
      id?: string;
      user_id?: string;
    }>(ownerUserResp, "seed:owner-user");
    const ownerUserId =
      ownerUser.user?.id || ownerUser.id || ownerUser.user_id || "";

    const ficheResp = await api.post(`${API_BASE}/owners`, {
      data: {
        organization_id: org.id,
        first_name: "Carine",
        last_name: "Copropriétaire",
        email: emailOwner,
        address: "1 Rue Test",
        city: "Brussels",
        postal_code: "1000",
        country: "Belgium",
        user_id: ownerUserId,
      },
      headers: { Authorization: `Bearer ${adminToken}` },
    });
    const fiche = await assertOk<{ id: string }>(ficheResp, "seed:fiche");

    // Lié dans les DEUX ACP — c'est le cas multi-ACP que l'étape 2 vérifie.
    for (const { uniteId } of [acpErables, acpGlycines]) {
      const linkResp = await api.post(`${API_BASE}/units/${uniteId}/owners`, {
        data: {
          owner_id: fiche.id,
          ownership_percentage: 1.0,
          is_primary_contact: true,
        },
        headers: { Authorization: `Bearer ${adminToken}` },
      });
      await assertOk(linkResp, `seed:link-unit:${uniteId}`);
    }

    // Une assemblée, sur l'ACP des Erables uniquement : convocation, ordre du
    // jour, résolution — de quoi jouer les étapes 6 à 9 avec un vrai vote.
    const dateReunion = new Date();
    dateReunion.setDate(dateReunion.getDate() + 30);
    const meetingResp = await api.post(`${API_BASE}/meetings`, {
      data: {
        building_id: acpErables.buildingId,
        organization_id: org.id,
        title: `AG Les Erables ${horodatage}`,
        scheduled_date: dateReunion.toISOString(),
        meeting_type: "Ordinary",
        location: "Salle communale",
        is_second_convocation: true,
      },
      headers: { Authorization: `Bearer ${syndicToken}` },
    });
    const meeting = await assertOk<{ id: string }>(meetingResp, "seed:meeting");

    const agendaResp = await api.post(
      `${API_BASE}/meetings/${meeting.id}/agenda`,
      {
        data: { item: "Approbation des comptes" },
        headers: { Authorization: `Bearer ${syndicToken}` },
      },
    );
    await assertOk(agendaResp, "seed:agenda");

    const resolutionResp = await api.post(
      `${API_BASE}/meetings/${meeting.id}/resolutions`,
      {
        data: {
          meeting_id: meeting.id,
          title: `Approbation des comptes ${horodatage}`,
          description: "Approbation des comptes de l'exercice écoulé",
          resolution_type: "ordinary",
          majority_required: "absolute",
          agenda_item_index: 0,
        },
        headers: { Authorization: `Bearer ${syndicToken}` },
      },
    );
    const resolution = await assertOk<{ id: string }>(
      resolutionResp,
      "seed:resolution",
    );

    const convocResp = await api.post(`${API_BASE}/convocations`, {
      data: {
        meeting_id: meeting.id,
        building_id: acpErables.buildingId,
        meeting_type: "Ordinary",
        meeting_date: dateReunion.toISOString(),
        language: "FR",
      },
      headers: { Authorization: `Bearer ${syndicToken}` },
    });
    const convocation = await assertOk<{ id: string }>(
      convocResp,
      "seed:convocation",
    );

    // Recipients omis à dessein : le serveur convoque alors tous les
    // copropriétaires actifs de l'immeuble (cf. commentaire de
    // `SendConvocationRequest`), notre copropriétaire y compris.
    const sendResp = await api.post(
      `${API_BASE}/convocations/${convocation.id}/send`,
      {
        data: {},
        headers: { Authorization: `Bearer ${syndicToken}` },
      },
    );
    await assertOk(sendResp, "seed:convocation-send");

    mondeSeme = {
      meetingId: meeting.id,
      resolutionId: resolution.id,
      uniteErablesId: acpErables.uniteId,
      buildingErablesId: acpErables.buildingId,
    };

    // Le contexte redevient anonyme : la première image du parcours doit
    // être celle d'un visiteur non connecté (cf. perimetre-multi-role.journey.ts).
    await page.context().clearCookies();

    return {
      syndic: { email: emailSyndic, motDePasse },
      copropriétaire: { email: emailOwner, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-connexion",
      acteur: "copropriétaire",
      description:
        "KoproGo — le parcours du copropriétaire, le rôle le plus nombreux " +
        "en base et le seul que cinq recettes n'ont jamais éprouvé (#807).",
      action: async (scene) => {
        await scene.devenir("copropriétaire");
      },
      assertion: async (page) => {
        await expect(page).toHaveURL(/\/owner/);
        await expect(page.getByTestId("owner-dashboard")).toBeVisible();
      },
    },
    {
      id: "2-lots-et-quotes-parts",
      acteur: "copropriétaire",
      description:
        "Ses lots, dans deux copropriétés distinctes. Le total consolidé " +
        "informe ; le paiement, lui, se fera ACP par ACP — chacune a son " +
        "propre compte bancaire (Art. 3.86 § 1er et § 3).",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("dettes-par-acp")).toBeVisible();
        await expect(page.getByTestId("dette-acp")).toHaveCount(2);
        await expect(page.getByTestId("dettes-par-acp-total")).toBeVisible();
        await expect(
          page.getByTestId("dettes-par-acp-explication"),
        ).toBeVisible();
      },
    },
    {
      id: "3-comprendre-la-quote-part",
      acteur: "copropriétaire",
      description:
        "Sa quote-part se calcule ainsi : quotité du lot ÷ total des " +
        "tantièmes de l'immeuble (1000, Art. 3.85 § 1er al. 2) × montant " +
        "de la charge. Un lot à 120/1000 doit 12 % d'une charge, pas 100 %.",
      action: async (scene) => {
        // Pas de nouvelle navigation : l'écran qui porte cette explication
        // est celui de l'étape précédente. Cette étape existe pour porter
        // sa propre narration, horodatée séparément dans la vitrine.
        await scene.attendreChargement();
      },
      // Pas d'assertion « heureuse » ici à dessein : le test @negative
      // dédié (`coproprietaire.spec.ts`) vérifie le montant affiché et
      // documente qu'il ne suit PAS encore cette formule (#807).
    },
    {
      id: "4-charges-imputees",
      acteur: "copropriétaire",
      description:
        "Les charges de son immeuble, en lecture seule — pas de bouton " +
        "« Créer », réservé au syndic et au comptable.",
      action: async (scene) => {
        await scene.aller("/owner/expenses");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("owner-expenses")).toBeVisible();
        await expect(page.getByTestId("create-button")).toHaveCount(0);
      },
    },
    {
      id: "5-payer-un-appel-de-fonds",
      acteur: "copropriétaire",
      description:
        "Il tente de payer. Le bouton « Payer » de son tableau de bord " +
        "mène à un historique en lecture seule : aucun formulaire de " +
        "paiement ne s'y trouve. C'est une étape rouge, documentée (#807).",
      action: async (scene) => {
        await scene.aller("/owner/payments");
        // La route où le formulaire de paiement existe réellement le
        // refuse : le garde de rôle renvoie le copropriétaire vers `/owner`
        // (`getDefaultRedirect`, `frontend/src/components/RouteGuard.svelte`).
        await scene.aller("/owner-contributions");
      },
      assertion: async (page) => {
        // Ce que l'écran fait vraiment aujourd'hui : la navigation ci-dessus
        // aboutit sur `/owner`, pas sur `/owner-contributions` — et même sur
        // `/owner/payments`, aucun formulaire de paiement n'est présent. Le
        // seul connu du contrat gelé, `owner-contribution-payment-form`, vit
        // sur la route qui vient de refuser.
        await expect(page).toHaveURL(/\/owner$/);
        await expect(
          page.getByTestId("owner-contribution-payment-form"),
        ).toHaveCount(0);
      },
    },
    {
      id: "6-convocation-recue",
      acteur: "copropriétaire",
      description:
        "La convocation à l'assemblée des Erables, envoyée par le syndic " +
        "(#784 côté envoi). Il la lit depuis son propre compte.",
      action: async (scene) => {
        await scene.aller("/convocations");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("convocations-list")).toBeVisible();
      },
    },
    {
      id: "7-voter-la-resolution",
      acteur: "copropriétaire",
      description:
        "Il vote « pour » l'approbation des comptes, avec le lot dont il " +
        "est titulaire. Le mécanisme de procuration existe pour un " +
        "copropriétaire absent (`vote-proxy-input`) — vérifié séparément.",
      action: async (scene) => {
        if (!mondeSeme) {
          throw new Error(
            "coproprietaire.journey: mondeSeme absent — amorcer() n'a pas " +
              "encore tourné, ou a tourné dans un autre process.",
          );
        }
        await scene.aller(`/meeting-detail?id=${mondeSeme.meetingId}`);
        await scene.attendreChargement();
        await scene.choisir("vote-unit-select", mondeSeme.uniteErablesId);
        await scene.cliquer("vote-btn-pour");
        await scene.cliquer("resolution-vote-submit-button");
      },
    },
    {
      id: "8-lire-le-resultat",
      acteur: "copropriétaire",
      description:
        "Une fois le vote clôturé par le syndic, le décompte est visible " +
        "et plafonné correctement — vérifié par la suite de tests des " +
        "résolutions (`Resolutions.spec.ts`).",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("vote-progress-pour")).toBeVisible();
      },
    },
    {
      id: "9-consulter-le-proces-verbal",
      acteur: "copropriétaire",
      description:
        "Les documents de la réunion, procès-verbal compris, sur ce même " +
        "écran — accessible au copropriétaire (`guards.ts` : SYNDIC + OWNER).",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("meeting-documents-list")).toBeVisible();
      },
    },
    {
      id: "10-modules-communautaires",
      acteur: "copropriétaire",
      description:
        "Offrir une compétence — ce que le syndic ne peut pas faire en son " +
        "nom (#781, tranchée ici par l'observation : ça marche, pour le " +
        "bon acteur).",
      action: async (scene) => {
        if (!mondeSeme) {
          throw new Error(
            "coproprietaire.journey: mondeSeme absent — amorcer() n'a pas " +
              "encore tourné, ou a tourné dans un autre process.",
          );
        }
        await scene.aller("/skills");
        // Deux immeubles liés à ce copropriétaire (le cas multi-ACP de
        // l'étape 2) : `BuildingSelector` rend un <select>, pas un résumé
        // auto-sélectionné. Le bouton de création reste inerte tant
        // qu'aucun immeuble n'est choisi.
        await scene.choisir("building-selector", mondeSeme.buildingErablesId);
        await scene.cliquer("create-offer-button");
        await scene.saisir(
          "skill-create-name-input",
          "Petits travaux de plomberie",
        );
        await scene.saisir(
          "skill-create-description-textarea",
          "Dépannage robinetterie, sur rendez-vous.",
        );
        await scene.cliquer("submit-skill-offer-button");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("skill-offer-create-form")).toHaveCount(
          0,
        );
      },
    },
  ],
};
