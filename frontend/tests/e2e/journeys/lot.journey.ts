/**
 * Parcours de référence — l'acte de base, et le compteur qui devient vert.
 *
 * ── Ce qu'il démontre ─────────────────────────────────────────────────────
 *
 * La conformité d'un immeuble à son acte de base, RENDUE VISIBLE.
 *
 * L'ADR-0010 pose un verrou : avant toute dépense, toute répartition, tout
 * appel de fonds et tout état daté, le serveur recalcule Σ(quotités) et la
 * compare au total déclaré par l'acte de base. Si les deux divergent, il
 * refuse en `422 BUILDING_NOT_CONFORMANT`.
 *
 * `comptable.journey.ts` le nomme « la question de support la plus fréquente
 * du produit ». Un syndic qui reçoit un 422 en saisissant une facture ne fait
 * pas le lien avec un lot qu'il a oublié d'encoder trois semaines plus tôt.
 *
 * Ce parcours filme le seul écran où ce lien se voit : `quotas-total`, qui
 * affiche « 800/1000èmes » en ROUGE, puis « 1000/1000èmes » en VERT dès que
 * le lot manquant est encodé.
 *
 * ── Pourquoi c'est une preuve de valeur, et pas une démonstration ─────────
 *
 * Parce que le compteur change SOUS L'ŒIL du spectateur, après un geste
 * qu'il vient de voir. Une capture du compteur vert ne dirait rien : elle ne
 * montrerait ni d'où il vient, ni ce qui l'a fait bouger.
 *
 * ── L'acte de base n'est pas qu'une somme ─────────────────────────────────
 *
 * Un lot sans propriétaire ne vote pas, ne reçoit pas d'appel de fonds, et
 * ne compte pas au corps électoral d'une consultation — `poll_use_cases.rs`
 * dérive celui-ci des `unit_owners` actifs, et refuse un total nul.
 *
 * Le parcours va donc jusqu'au rattachement : créer le lot ne suffit pas, il
 * faut dire à qui il est.
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

let monde: { buildingId: string; nomDuProprietaire: string } | null = null;

/** Le total déclaré par l'acte de base. */
const ACTE_DE_BASE = 1000;
/** Ce que les deux premiers lots totalisent. Il en manque 200. */
const DEJA_ENCODE = 800;
/** La quotité du lot que le syndic va encoder à la caméra. */
const QUOTITE_DU_LOT_MANQUANT = ACTE_DE_BASE - DEJA_ENCODE;

export function mondeDuParcours(): typeof monde {
  return monde;
}

export const lot: Parcours = {
  slug: "lot",
  titre: "L'acte de base, et le compteur qui devient vert",
  propos:
    "La question de support la plus fréquente du produit, filmée. Un " +
    "immeuble dont les quotités ne somment pas à l'acte de base bloque " +
    "toute dépense en 422, et rien ne dit au syndic pourquoi. Ici le " +
    "compteur affiche 800/1000 en rouge ; le syndic encode le lot " +
    "manquant, il passe à 1000/1000 en vert, et le lot reçoit son " +
    "propriétaire — sans qui il ne vote ni ne paie.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailSyndic = `lot-syndic-${horodatage}@example.com`;
    const emailCopro = `lot-owner-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Lot Org ${horodatage}`,
          slug: `lot-${horodatage}`,
          contact_email: emailSyndic,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "lot");

    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: `Résidence de l'Acte ${horodatage}`,
          address: `${horodatage} Rue du Notaire`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          // TROIS lots à l'acte de base, dont un seul sera encodé à la
          // caméra. C'est tout le sujet du parcours.
          total_units: 3,
          total_tantiemes: ACTE_DE_BASE,
          construction_year: 1962,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );

    // ── Un immeuble DÉLIBÉRÉMENT non conforme ────────────────────────────
    //
    // Deux lots sur trois, 800 quotités sur 1000. C'est l'état réel d'un
    // immeuble en cours d'encodage — et celui qui produit le 422 que le
    // syndic ne sait pas interpréter.
    //
    // `seedConformantUnits` fait l'inverse : il ferme la somme. Ce parcours
    // ne peut donc pas s'en servir, et c'est le seul de la vitrine dans ce
    // cas.
    for (const [numero, quotite, etage] of [
      ["1A", 400, 0],
      ["2A", 400, 1],
    ] as const) {
      await ok(
        await api.post(`${API_BASE}/units`, {
          data: {
            acp_id: acpId,
            building_id: immeuble.id,
            unit_number: numero,
            floor: etage,
            surface_area: 85,
            unit_type: "Apartment",
            quota: quotite,
          },
          headers: entete,
        }),
        `amorçage:lot-${numero}`,
      );
    }

    await ok(
      await api.post(`${API_BASE}/auth/register`, {
        data: {
          email: emailSyndic,
          password: motDePasse,
          first_name: "Sophie",
          last_name: "Syndic",
          role: "syndic",
          organization_id: org.id,
        },
      }),
      "amorçage:syndic",
    );

    // La fiche du futur propriétaire du lot manquant. Elle existe AVANT le
    // lot : c'est l'ordre réel — le notaire enregistre l'acquéreur, le
    // syndic encode ensuite le lot et fait le lien.
    const nomDuProprietaire = `Delcourt${horodatage}`;
    await ok(
      await api.post(`${API_BASE}/owners`, {
        data: {
          organization_id: org.id,
          first_name: "Anne",
          last_name: nomDuProprietaire,
          email: emailCopro,
          address: "3 Rue du Notaire",
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
        },
        headers: entete,
      }),
      "amorçage:fiche-proprietaire",
    );

    monde = { buildingId: immeuble.id, nomDuProprietaire };

    await page.context().clearCookies();

    return { syndic: { email: emailSyndic, motDePasse } };
  },

  etapes: [
    {
      id: "1-le-compteur-est-rouge",
      acteur: "syndic",
      description:
        "Sophie reprend un immeuble dont l'encodage n'a jamais été fini. " +
        "Le compteur de quotités le dit en un coup d'œil : 800 sur 1000. " +
        "Tant qu'il est rouge, toute dépense sera refusée.",
      action: async (scene) => {
        await scene.devenir("syndic");
        await scene.aller(`/building-detail?id=${monde!.buildingId}`);
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("quotas-total"),
          "Le compteur de quotités ne s'affiche pas : le syndic n'a aucun " +
            "moyen de savoir pourquoi ses dépenses seront refusées.",
        ).toBeVisible({ timeout: 20000 });
        await expect(page.getByTestId("quotas-total")).toHaveText(
          `${DEJA_ENCODE}/${ACTE_DE_BASE}èmes`,
        );
        // Rouge, pas vert. La couleur EST le message : c'est elle qui
        // distingue « en cours d'encodage » de « prêt à l'emploi ».
        await expect(
          page.getByTestId("quotas-total"),
          "Un immeuble non conforme s'affiche comme un immeuble conforme.",
        ).toHaveClass(/text-red-600/);
      },
    },
    {
      id: "2-elle-encode-le-lot-manquant",
      acteur: "syndic",
      description:
        "Elle ouvre le formulaire et encode le troisième lot : le numéro, " +
        "le type, l'étage, la surface, et surtout la quotité — 200èmes, " +
        "telle que l'acte de base la fixe.",
      action: async (scene) => {
        await scene.cliquer("unit-add-button");
        await scene.attendreChargement();
        await scene.saisir("unit-create-number-input", "3A");
        await scene.choisir("unit-create-type-select", "Apartment");
        await scene.saisir("unit-create-floor-input", "2");
        await scene.saisir("unit-create-surface-input", "78");
        await scene.saisir(
          "unit-create-quota-input",
          String(QUOTITE_DU_LOT_MANQUANT),
        );
      },
      assertion: async (page) => {
        await expect(page.getByTestId("unit-create-submit")).toBeEnabled();
      },
    },
    {
      id: "3-le-compteur-devient-vert",
      acteur: "syndic",
      description:
        "Elle enregistre. Le compteur passe à 1000 sur 1000 et devient " +
        "vert : l'immeuble est conforme à son acte de base, et le verrou " +
        "qui bloquait les dépenses vient de s'ouvrir.",
      action: async (scene) => {
        await scene.cliquer("unit-create-submit");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("unit-create-form")).toHaveCount(0, {
          timeout: 20000,
        });
        // LE moment du parcours. Le compteur a changé sous l'œil du
        // spectateur, après un geste qu'il vient de voir.
        await expect(
          page.getByTestId("quotas-total"),
          "Le compteur n'a pas suivi la création du lot : le syndic ne " +
            "saura pas que l'immeuble est devenu conforme, et continuera de " +
            "chercher la cause de ses 422.",
        ).toHaveText(`${ACTE_DE_BASE}/${ACTE_DE_BASE}èmes`, {
          timeout: 20000,
        });
        await expect(page.getByTestId("quotas-total")).toHaveClass(
          /text-green-600/,
        );
      },
    },
    {
      id: "4-a-qui-est-ce-lot",
      acteur: "syndic",
      description:
        "Un lot conforme mais sans propriétaire ne vote pas et ne reçoit " +
        "aucun appel de fonds. Sophie déplie le lot et cherche " +
        "l'acquéreur que le notaire a enregistré.",
      action: async (scene) => {
        await scene.cliquerLePremier("toggle-unit-owners");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("unit-owners-panel").first(),
          "Le lot ne se déplie pas : impossible de savoir à qui il est, ni " +
            "de le rattacher.",
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "5-le-lot-trouve-son-proprietaire",
      acteur: "syndic",
      description:
        "Elle rattache Anne Delcourt, en pleine propriété. Le lot cesse " +
        "d'être une ligne comptable : il devient la part de quelqu'un.",
      action: async (scene) => {
        await scene.cliquerLePremier("add-owner-button-empty");
        await scene.attendreChargement();
        await scene.saisir(
          "unit-owner-add-search-input",
          monde!.nomDuProprietaire,
        );
        await scene.choisirQuiContient(
          "unit-owner-add-owner-select",
          monde!.nomDuProprietaire,
        );
        await scene.saisir("unit-owner-add-percentage-input", "100");
        await scene.cliquer("unit-owner-add-submit");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("unit-owner-add-form")).toHaveCount(0, {
          timeout: 20000,
        });
        // Le propriétaire apparaît dans le panneau du lot. Vérifier
        // seulement la fermeture du formulaire ne distinguerait pas un
        // rattachement d'une annulation.
        await expect(
          page.getByTestId("owner-row").filter({
            hasText: monde!.nomDuProprietaire,
          }),
          "Le rattachement a l'air d'aboutir mais le lot reste sans " +
            "propriétaire : il ne votera pas et ne recevra aucun appel de " +
            "fonds.",
        ).toHaveCount(1, { timeout: 20000 });
      },
    },
  ],
};
