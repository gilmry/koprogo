import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, extname, relative } from "node:path";

/**
 * Garde-fou : un `data-testid` partagé entre plusieurs fichiers reste
 * volontaire, pas accidentel (#868).
 *
 * ── Le fait déclencheur ─────────────────────────────────────────────────
 *
 * `components/BuildingSelector.svelte` (monté par page, 14 pages) et
 * `components/global/BuildingSelector.svelte` (monté dans la barre de
 * contexte, sur CES MÊMES pages pour syndic/accountant/superadmin)
 * partageaient `building-selector-empty`. Les deux pouvaient rendre leur
 * état vide en même temps → `getByTestId` levait une strict-mode violation.
 *
 * Ce ne sont pas deux instances d'un même composant : ce sont deux widgets
 * distincts qui prétendaient au même nom. Le symptôme (l'ancre) n'était que
 * la partie visible : le vrai risque, c'est qu'un ancrage partagé entre deux
 * composants CO-MONTABLES sur le même écran rend un test ambigu, voire
 * masque une règle produit (cf. `building-selector-403`, qui n'existe que
 * dans le composant global — un test qui ciblerait `-empty` sans savoir
 * lequel des deux il atteint pourrait rater l'absence du refus 403).
 *
 * ── Ce que ce fichier fait ───────────────────────────────────────────────
 *
 * Il mesure, comme `garde-data-testid.test.ts`, tous les `data-testid`
 * littéraux (hors préfixes dynamiques `-{id}`) qui apparaissent dans PLUS
 * D'UN FICHIER de production. Pour chacun, deux issues possibles :
 *
 * 1. **Légitime** — état générique (`loading-spinner`, `cancel-button`...)
 *    ciblé par les recettes DANS un conteneur, ou composants qui ne sont
 *    jamais rendus sur le même écran (routes distinctes). Documenté ici.
 * 2. **Fautif** — composants co-montables sur le même écran. Corrigé au
 *    moment de la découverte (cf. commit #868) : un des deux ancrages est
 *    renommé, avec la raison écrite dans le composant lui-même.
 *
 * `building-selector-empty` (BuildingSelector), `delete-inspection-button`
 * (InspectionList/InspectionDetail) et `cancel-button`
 * (ExpenseDetail/ExpenseDocuments, partiellement) sont sortis de la liste
 * mesurée ci-dessous PARCE QU'ils ont été corrigés — s'ils y réapparaissent,
 * c'est qu'une régression a réintroduit la collision.
 *
 * Toute NOUVELLE entrée mesurée qui n'est pas dans `CLASSIFICATION`
 * fait échouer ce test : elle doit être classée à la main, avec sa raison,
 * pas silencieusement tolérée. C'est la même discipline que
 * `garde-data-testid.test.ts` applique aux ancrages qui disparaissent —
 * ici elle s'applique aux ancrages qui se DUPLIQUENT.
 */

const RACINE = join(process.cwd(), "src");
const EXTENSIONS = new Set([".svelte", ".astro"]);

function fichiersDeGabarit(repertoire: string): string[] {
  const trouves: string[] = [];
  for (const entree of readdirSync(repertoire)) {
    if (entree === "node_modules" || entree.startsWith(".")) continue;
    const chemin = join(repertoire, entree);
    if (statSync(chemin).isDirectory()) {
      trouves.push(...fichiersDeGabarit(chemin));
    } else if (EXTENSIONS.has(extname(entree)) && !entree.includes(".test.")) {
      trouves.push(chemin);
    }
  }
  return trouves;
}

/** Pour chaque data-testid littéral (pas de `{...}`), l'ensemble des fichiers qui le posent. */
function fichiersParTestid(): Map<string, Set<string>> {
  const parTestid = new Map<string, Set<string>>();
  for (const chemin of fichiersDeGabarit(RACINE)) {
    const texte = readFileSync(chemin, "utf8");
    const relatif = relative(RACINE, chemin).split("\\").join("/");
    for (const m of texte.matchAll(/(?:data-testid|testId)="([^"]+)"/g)) {
      const valeur = m[1];
      if (valeur.includes("{")) continue; // préfixe dynamique — hors sujet ici
      if (!parTestid.has(valeur)) parTestid.set(valeur, new Set());
      parTestid.get(valeur)!.add(relatif);
    }
  }
  return parTestid;
}

interface Classification {
  fichiers: string[];
  raison: string;
}

/**
 * Chaque entrée = un ancrage mesuré dans plus d'un fichier au 2026-09-14,
 * avec la liste des fichiers ATTENDUE (triée) et la raison pour laquelle
 * ce n'est PAS un bug — vérifiée en co-montage réel, pas supposée.
 */
const CLASSIFICATION: Record<string, Classification> = {
  "loading-spinner": {
    fichiers: [
      "components/BuildingSelector.svelte",
      "components/ExpenseList.svelte",
      "components/InvoiceList.svelte",
      "components/tickets/TicketCreateModal.svelte",
      "components/tickets/TicketList.svelte",
      "components/tickets/TicketStatistics.svelte",
    ],
    raison:
      "État de chargement générique. `TicketList` et `TicketStatistics` sont co-montés sur pages/tickets.astro : une recette doit scoper par conteneur (ex. le panneau stats) plutôt que cibler `loading-spinner` nu sur cette page. `InvoiceList.svelte` n'est mounté nulle part (code mort) — aucun risque d'exécution, seule la chaîne de caractères existe encore.",
  },
  "cancel-button": {
    fichiers: ["components/ExpenseDetail.svelte", "components/InvoiceForm.svelte"],
    raison:
      "Deux formulaires distincts, jamais co-montés (InvoiceForm vit dans ExpenseList, pas dans ExpenseDetail). Le troisième porteur, ExpenseDocuments (co-monté DANS ExpenseDetail), a été renommé `expense-documents-cancel-button` — c'était le vrai doublon (#868).",
  },
  "mark-paid-button": {
    fichiers: ["components/ExpenseDetail.svelte", "components/InvoiceWorkflow.svelte"],
    raison:
      "pages/expense-detail.astro et pages/invoice-workflow.astro sont deux routes distinctes, jamais rendues ensemble.",
  },
  "status-badge": {
    fichiers: ["components/ExpenseDetail.svelte", "components/ExpenseList.svelte"],
    raison:
      "pages/expense-detail.astro (détail) et pages/expenses.astro / BuildingDetail.svelte (liste) sont des routes distinctes.",
  },
  "refresh-button": {
    fichiers: ["components/InvoiceList.svelte", "components/InvoiceWorkflow.svelte"],
    raison:
      "InvoiceList.svelte n'est importé par aucune page (code mort) ; seul InvoiceWorkflow est vivant sur pages/invoice-workflow.astro.",
  },
  "payment-status-filter": {
    fichiers: ["components/InvoiceWorkflow.svelte", "components/payments/PaymentList.svelte"],
    raison:
      "pages/invoice-workflow.astro et pages/owner/payments.astro sont deux routes distinctes.",
  },
  "meeting-status-badge": {
    fichiers: ["components/MeetingDetail.svelte", "components/MeetingList.svelte"],
    raison:
      "pages/meeting-detail.astro (détail) et pages/meetings.astro / BuildingDetail.svelte (liste) sont des routes distinctes.",
  },
  "exchange-request-btn": {
    fichiers: [
      "components/local-exchanges/ExchangeDetail.svelte",
      "components/local-exchanges/ExchangeList.svelte",
    ],
    raison:
      "pages/exchange-detail.astro et pages/exchanges.astro sont deux routes distinctes.",
  },
  "ticket-cancel-btn": {
    fichiers: [
      "components/tickets/TicketCreateModal.svelte",
      "components/tickets/TicketDetail.svelte",
    ],
    raison:
      "TicketCreateModal ne vit que sur pages/tickets.astro et pages/owner/tickets.astro ; TicketDetail ne vit que sur pages/ticket-detail.astro. Jamais co-montés.",
  },
  "ticket-overdue-badge": {
    fichiers: ["components/tickets/TicketDetail.svelte", "components/tickets/TicketList.svelte"],
    raison:
      "Même raison que ticket-cancel-btn : pages/ticket-detail.astro vs pages/tickets.astro, routes distinctes.",
  },
  "user-name": {
    fichiers: ["components/UserListAdmin.svelte", "components/admin/AdminGdprPanel.svelte"],
    raison:
      "pages/admin/users.astro et pages/admin/gdpr.astro sont deux routes distinctes.",
  },
  "user-email": {
    fichiers: ["components/UserListAdmin.svelte", "components/admin/AdminGdprPanel.svelte"],
    raison: "Même raison que user-name.",
  },
  "contractor-report-form": {
    fichiers: ["pages/contractor-report/index.astro", "pages/contractor/index.astro"],
    raison: "Deux routes distinctes, jamais rendues ensemble.",
  },
};

describe("les ancrages partagés entre plusieurs fichiers sont classés (#868)", () => {
  const mesure = fichiersParTestid();
  const doublons = new Map(
    [...mesure.entries()].filter(([, fichiers]) => fichiers.size > 1),
  );

  it("ne mesure aucun doublon qui ne soit pas classé", () => {
    const nonClasses = [...doublons.keys()].filter((id) => !(id in CLASSIFICATION));

    expect(
      nonClasses,
      `${nonClasses.length} data-testid partagé(s) entre plusieurs fichiers ` +
        `sans classification écrite : ${nonClasses.join(", ")}.\n\n` +
        `Un ancrage neuf, présent dans ≥2 fichiers, doit être tranché : les ` +
        `deux porteurs sont-ils co-montables sur le même écran ? Si oui, ` +
        `c'est un bug (cf. #868 building-selector-empty) — renommer l'un ` +
        `des deux et ajouter l'entrée ici avec la preuve (routes vérifiées). ` +
        `Si non, ajouter l'entrée avec la raison de non-collision.`,
    ).toEqual([]);
  });

  it("ne fige aucune classification devenue fausse (fichiers disparus ou en trop)", () => {
    const ecarts: string[] = [];
    for (const [id, { fichiers }] of Object.entries(CLASSIFICATION)) {
      const reel = doublons.get(id);
      const reelTrie = reel ? [...reel].sort() : [];
      const attenduTrie = [...fichiers].sort();
      if (JSON.stringify(reelTrie) !== JSON.stringify(attenduTrie)) {
        ecarts.push(
          `${id} : attendu [${attenduTrie.join(", ")}], mesuré [${reelTrie.join(", ")}]`,
        );
      }
    }

    expect(
      ecarts,
      `Classification obsolète — un ancrage documenté ici ne correspond plus ` +
        `à ce que le code contient réellement :\n\n${ecarts.join("\n")}`,
    ).toEqual([]);
  });

  it("les trois collisions corrigées par #868 ne reviennent pas", () => {
    // building-selector-empty et delete-inspection-button ne devraient plus
    // avoir qu'un seul porteur — s'ils réapparaissent ici groupés dans >1
    // fichier, la collision qu'#868 a résolue est revenue.
    expect(doublons.has("building-selector-empty")).toBe(false);
    expect(doublons.has("delete-inspection-button")).toBe(false);

    // cancel-button reste légitimement partagé (ExpenseDetail, InvoiceForm),
    // mais ExpenseDocuments — le vrai doublon avec ExpenseDetail, co-monté
    // sur la même page — ne doit plus y figurer.
    const porteursCancelButton = doublons.get("cancel-button") ?? new Set();
    expect(porteursCancelButton.has("components/ExpenseDocuments.svelte")).toBe(
      false,
    );
  });
});
