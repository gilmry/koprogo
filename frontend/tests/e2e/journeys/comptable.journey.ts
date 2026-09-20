/**
 * Parcours de référence — le comptable, neuf étapes (#808).
 *
 * ── Pourquoi ce parcours-ci ────────────────────────────────────────────────
 *
 * `docs/personas/accountant.md` le dit : c'est **le rôle le mieux cadré du
 * produit**. Son périmètre vient d'une ADR (0052 — `compta`, et rien
 * d'autre), il a deux comptes de recette, et il est le seul rôle conçu pour
 * le bureau d'abord. On ne saisit pas une écriture comptable au doigt.
 *
 * C'est donc le troisième parcours filmé, après le syndic et le
 * copropriétaire, et celui qui coûte le moins d'hypothèses.
 *
 * ── Ce que ce parcours DÉMONTRE, et qui ne se lit pas dans un test ────────
 *
 * **Qui saisit une dépense ne l'approuve pas.** Étape 4 : le comptable
 * soumet la facture, et `approve-button` n'existe pas pour lui. L'assertion
 * porte sur cette ABSENCE, comme une propriété — pas comme une gêne.
 *
 * La documentation affirmait l'inverse jusqu'au 2026-09-17, et le produit la
 * démentait des deux côtés : `check_syndic_role` rend 403 côté API, et
 * `InvoiceWorkflow.svelte:220` ne rend le bouton que pour syndic ou
 * superadmin. Arbitrage du PO : c'est la doc qui avait tort (#942).
 *
 * Une vidéo où l'on voit le bouton ne pas être là vaut mieux qu'un
 * paragraphe qui l'affirme.
 *
 * ── Le verrou de conformité, et pourquoi l'amorçage y prend soin ──────────
 *
 * Quatre étapes du persona portent un 🔒 : dépense, répartition, appel de
 * fonds, état daté. Avant chacune, le serveur recalcule Σ(quotités) et la
 * compare au total déclaré par l'acte de base. Si les deux divergent, il
 * refuse en 422 `BUILDING_NOT_CONFORMANT` (ADR-0010).
 *
 * C'est la question de support la plus fréquente du produit. L'amorçage
 * sème donc des lots CONFORMES — sans quoi ce parcours filmerait un écran
 * d'erreur et prétendrait montrer la comptabilité.
 *
 * ── Ce qui n'est PAS filmé ici, et pourquoi ───────────────────────────────
 *
 * L'état daté (étape 9 du persona) a déjà son parcours dédié dans
 * `SyndicCreationJourneys.spec.ts`, et il appartient au syndic autant qu'au
 * comptable. Le refilmer ici allongerait la vidéo sans rien montrer de neuf
 * sur le rôle.
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

/**
 * Le libellé de la facture que le comptable saisit à l'étape 3.
 *
 * Unique par campagne : l'étape 4 la RETROUVE dans la liste, et un libellé
 * partagé avec une exécution précédente rendrait cette vérification non
 * concluante — elle passerait au vert sur la facture de la veille.
 */
let libelleDeLaFacture = "";

export const comptable: Parcours = {
  slug: "comptable",
  titre: "Le comptable tient les comptes, et n'approuve pas ce qu'il saisit",
  propos:
    "Montre le rôle le mieux cadré du produit : de la saisie d'une dépense " +
    "au bilan, en passant par l'écriture en partie double et l'appel de " +
    "fonds. Et démontre la séparation des rôles — le bouton « approuver » " +
    "n'existe pas pour lui.",

  amorcer: async (page) => {
    const api = page.request;
    const horodatage = Date.now();
    const motDePasse = "test123456";
    const emailComptable = `comptable-${horodatage}@cabinet-vdb.be`;
    const emailSyndic = `comptable-syndic-${horodatage}@example.com`;

    const adminToken = await adminLogin(page);
    const entete = { Authorization: `Bearer ${adminToken}` };

    const org = await ok<{ id: string }>(
      await api.post(`${API_BASE}/organizations`, {
        data: {
          name: `Comptable Org ${horodatage}`,
          slug: `comptable-${horodatage}`,
          contact_email: emailComptable,
          subscription_plan: "professional",
        },
        headers: entete,
      }),
      "amorçage:organisation",
    );

    const acpId = await ensureAcp(page, org.id, adminToken, "comptable");

    const immeuble = await ok<{ id: string }>(
      await api.post(`${API_BASE}/buildings`, {
        data: {
          name: `Résidence des Comptes ${horodatage}`,
          address: `${horodatage} Rue du Bilan`,
          city: "Brussels",
          postal_code: "1000",
          country: "Belgium",
          total_units: 8,
          total_tantiemes: 1000,
          construction_year: 2010,
          acp_id: acpId,
        },
        headers: entete,
      }),
      "amorçage:immeuble",
    );

    // Des lots dont les quotités somment EXACTEMENT à l'acte de base.
    // Sans eux, les quatre étapes 🔒 rendraient 422 et ce parcours
    // filmerait un toast d'erreur en prétendant montrer la comptabilité.
    await seedConformantUnits(page, adminToken, acpId, immeuble.id, 8, 1000);

    for (const [email, prenom, nom, role] of [
      [emailComptable, "Gisèle", "Vandenberghe", "accountant"],
      [emailSyndic, "François", "Leroy", "syndic"],
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

    // Le contexte de l'admin doit partir avant que la scène ouvre la
    // première session à la caméra : `injectAuth` survit à un
    // `clearCookies`, et le parcours filmerait quelqu'un qu'il n'a pas
    // connecté.
    await page.context().clearCookies();

    libelleDeLaFacture = `Entretien de la chaudière ${horodatage}`;

    return {
      comptable: { email: emailComptable, motDePasse },
      syndic: { email: emailSyndic, motDePasse },
    };
  },

  etapes: [
    {
      id: "1-connexion",
      acteur: "comptable",
      description:
        "KoproGo — le parcours du comptable. Il tient les comptes de la " +
        "copropriété sans en être membre : écritures, PCMN, rapports.",
      action: async (scene) => {
        await scene.devenir("comptable");
      },
      assertion: async (page) => {
        await expect(page.getByTestId("accountant-dashboard")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "2-son-perimetre",
      acteur: "comptable",
      description:
        "Il arrive sur SON périmètre, jamais sur un tableau de bord " +
        "générique. Immeubles, dépenses, factures, rapports : l'ADR-0052 " +
        "lui donne « compta », et rien d'autre.",
      action: async (scene) => {
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("accountant-expenses-tile"),
        ).toBeVisible();
        await expect(page.getByTestId("accountant-reports-tile")).toBeVisible();
      },
    },
    {
      id: "3-saisir-une-depense",
      acteur: "comptable",
      description:
        "Saisir une dépense. Le serveur vérifie d'abord que l'immeuble est " +
        "conforme à son acte de base : Σ des quotités contre le total " +
        "déclaré. C'est la question de support la plus fréquente du produit.",
      action: async (scene) => {
        await scene.aller("/expenses");
        await scene.attendreChargement();
        // L'étape s'intitule « Saisir une dépense ». Elle se contentait
        // d'atteindre l'écran et de constater qu'un bouton s'affichait : la
        // narration promettait un acte que le parcours n'accomplissait pas
        // (#974). Elle le SAISIT désormais, en entier.
        await scene.cliquer("create-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // Le formulaire est ouvert, donc fermable : deux ancres qui
        // n'existent QUE dans la modale de saisie.
        await expect(
          page.getByTestId("expense-form-cancel-button"),
          "Le bouton « créer » est visible mais n'ouvre rien : la capacité " +
            "est affichée sans être atteignable.",
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "3bis-il-remplit-la-facture",
      acteur: "comptable",
      description:
        "Il remplit : l'immeuble, le libellé, le montant hors TVA, le taux " +
        "de 21 % et la date de facture. Le formulaire calcule le TTC — le " +
        "comptable ne ressaisit pas ce que la machine sait faire.",
      action: async (scene) => {
        await scene.choisirQuiContient("building-select", "Résidence des Comptes");
        await scene.saisir("description-input", libelleDeLaFacture);
        await scene.saisir("amount-input", "1450");
        await scene.choisir("vat-rate-select", "21.00");
        await scene.saisir(
          "invoice-date-input",
          new Date().toISOString().slice(0, 10),
        );
      },
      assertion: async (page) => {
        // Le formulaire accepte la saisie : aucun message d'erreur, et le
        // bouton d'envoi est atteignable. Une modale qui refuserait sans le
        // dire produirait exactement l'écran « crédible et faux » que la
        // recette des cahiers des charges a déjà rencontré (#968).
        await expect(page.getByTestId("submit-button")).toBeEnabled();
      },
    },
    {
      id: "3ter-la-facture-existe",
      acteur: "comptable",
      description:
        "Il envoie. La facture rejoint le registre des dépenses de la " +
        "copropriété — c'est ce document que l'assemblée pourra consulter, " +
        "et qui pèsera sur l'appel de fonds.",
      action: async (scene) => {
        await scene.cliquer("submit-button");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // LA preuve : la modale s'est refermée ET la facture est dans la
        // liste. Vérifier seulement la fermeture ne distinguerait pas un
        // envoi réussi d'une annulation.
        await expect(page.getByTestId("expense-form-cancel-button")).toHaveCount(
          0,
          { timeout: 20000 },
        );
        await expect(
          page.getByTestId("expense-card").filter({
            hasText: libelleDeLaFacture,
          }),
          "La facture saisie n'apparaît pas dans le registre : l'envoi a " +
            "l'air d'aboutir sans rien enregistrer.",
        ).toHaveCount(1, { timeout: 20000 });
      },
    },
    {
      id: "4-il-soumet-il-napprouve-pas",
      acteur: "comptable",
      description:
        "Le workflow de la facture. Il la saisit et la soumet — puis il " +
        "s'arrête. Qui saisit une dépense ne l'approuve pas : c'est le " +
        "syndic qui approuve, et le bouton n'existe pas pour le comptable.",
      action: async (scene) => {
        await scene.aller("/invoice-workflow");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // L'ABSENCE est la démonstration. `InvoiceWorkflow.svelte:220` ne
        // rend `approve-button` que pour syndic ou superadmin, et
        // `check_syndic_role` rend 403 côté API. La documentation affirmait
        // l'inverse jusqu'au 2026-09-17 ; c'est elle qui avait tort (#942).
        await expect(page.getByTestId("approve-button")).toHaveCount(0);
      },
    },
    {
      id: "5-ecriture-en-partie-double",
      acteur: "comptable",
      description:
        "L'écriture au journal, en partie double, sur le plan comptable " +
        "belge PCMN — conforme à l'arrêté royal du 12 juillet 2012. " +
        "L'écran demande d'abord de choisir un immeuble : le périmètre ne " +
        "survit pas à la navigation (#841).",
      action: async (scene) => {
        await scene.aller("/journal-entries");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        // ── Ce que cette assertion décrit, et pourquoi elle n'est pas
        //    celle qu'on aurait voulu écrire ────────────────────────────
        //
        // Le persona annonçait l'ancre `journal-entry-form`. Elle n'existe
        // pas : `JournalEntryForm.svelte:181` expose
        // `journal-entry-panel` — renommée parce que l'ancienne résolvait
        // DEUX éléments. Le document n'avait pas suivi.
        //
        // Et l'écran ne montre pas le formulaire : il montre « choisissez
        // un immeuble », parce que le périmètre est nul au chargement de
        // chacun des douze écrans qui le lisent (#841).
        //
        // On décrit donc ce qui se passe, avec l'issue qui le ferme. Écrire
        // l'assertion qu'on souhaite ferait mentir la vitrine, et une
        // vitrine qui ment est pire qu'une absence de vitrine.
        await expect(
          page.getByTestId("journal-entries-no-building"),
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "6-le-budget",
      acteur: "comptable",
      description:
        "Le budget de l'exercice, en brouillon. Il le prépare ; c'est " +
        "l'assemblée qui l'adopte. Le comptable propose, il ne décide pas.",
      action: async (scene) => {
        await scene.aller("/budgets");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("create-budget-button")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "7-repartir-par-tantiemes",
      acteur: "comptable",
      description:
        "Répartir une charge entre les lots, au prorata des quotités de " +
        "l'acte de base. C'est le calcul que l'Art. 3.86 impose, et celui " +
        "que le verrou de conformité protège.",
      action: async (scene) => {
        await scene.aller("/expenses");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(page.getByTestId("expenses-list")).toBeVisible({
          timeout: 20000,
        });
      },
    },
    {
      id: "8-appel-de-fonds",
      acteur: "comptable",
      description:
        "L'appel de fonds collectif : ce que chaque copropriétaire devra " +
        "verser, ventilé par ses quotités. Rien n'est envoyé tant que " +
        "l'immeuble n'est pas conforme.",
      action: async (scene) => {
        await scene.aller("/call-for-funds");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("call-for-funds-create-button"),
        ).toBeVisible({ timeout: 20000 });
      },
    },
    {
      id: "9-le-bilan",
      acteur: "comptable",
      description:
        "Et le bilan, pour finir : l'actif d'un côté, le passif de " +
        "l'autre, et l'équilibre entre les deux. C'est ce document que " +
        "l'assemblée générale regardera.",
      action: async (scene) => {
        await scene.aller("/reports");
        await scene.attendreChargement();
      },
      assertion: async (page) => {
        await expect(
          page.getByTestId("financial-reports-generate"),
        ).toBeVisible({ timeout: 20000 });
      },
    },
  ],
};
