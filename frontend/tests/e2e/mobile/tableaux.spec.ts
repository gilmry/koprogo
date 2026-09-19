import { test, expect } from "@playwright/test";
import { DEUX_IMMEUBLES, ouvreEnTantQue, repond, type Role } from "./socle";

/**
 * Aucun tableau n'est coupé en silence (#866).
 *
 * ── Le défaut d'origine ────────────────────────────────────────────────────
 *
 * Six tableaux de sept à neuf colonnes vivaient dans une carte en
 * `overflow-hidden` — posé pour arrondir les coins, et qui coupait le tableau
 * en prime. Sur un téléphone, les colonnes de droite étaient **absentes** :
 * pas de barre, pas d'ombre, aucun signe qu'il y a plus. L'écran avait l'air
 * complet.
 *
 * Un débordement se voit et se comprend. Un découpage se croit complet, et
 * c'est bien pire : un syndic qui consulte ses états datés ne voit pas la
 * colonne « Échéance » et ne sait pas qu'elle existe.
 *
 * ── Ce que ce test vérifie, et pourquoi c'est une règle générale ───────────
 *
 * Il ne liste pas les six fichiers corrigés — il énonce l'invariant et
 * l'applique à TOUT tableau rendu sur les pages visitées :
 *
 *   si un tableau est plus large que son conteneur,
 *   alors ce conteneur défile (`overflow-x: auto|scroll`)
 *   et il est atteignable au clavier (`tabindex`).
 *
 * Ainsi le septième tableau, écrit demain, est couvert sans qu'on pense à
 * l'ajouter ici. Une liste de fichiers se périme ; une règle, non.
 *
 * Le second point n'est pas une politesse : `scrollable-region-focusable`
 * l'exige, et sans lui les colonnes de droite restent hors de portée de qui
 * n'a pas de souris — donc coupées, autrement.
 */

/**
 * Une ligne générique, qui suffit à faire naître un tableau.
 *
 * Les listes vides affichent un état vide À LA PLACE du tableau : sans donnée,
 * la règle ci-dessous ne regarde rien. Le témoin l'a montré — remettre
 * `overflow-hidden` sur `BudgetList` ne faisait échouer aucun test, parce
 * qu'aucun tableau n'était rendu.
 */
const LIGNE = {
  id: "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
  name: "Exemple",
  title: "Exemple",
  reference: "REF-1",
  status: "Pending",
  year: 2026,
  amount: 1000,
  total_amount: 1000,
  currency: "EUR",
  building_id: "11111111-1111-1111-1111-111111111111",
  building_name: "Résidence Les Érables",
  acp_id: "22222222-2222-2222-2222-222222222222",
  acp_name: "ACP Les Érables",
  created_at: "2026-01-15T10:00:00Z",
  updated_at: "2026-01-15T10:00:00Z",
  due_date: "2026-02-15",
  is_active: true,
};

/**
 * Un état daté avec une dette réelle sur un lot identifié.
 *
 * `EtatDateList` a besoin de sa propre forme de donnée — `LIGNE` ne suffit
 * pas : la page fait DEUX appels (`/etats-dates` et `/etats-dates/stats`), et
 * un stub générique servi aux deux fait planter `stats.average_processing_days
 * .toFixed(1)` sur un objet qui n'a pas ce champ. C'est cette page qui restait
 * exclue de la règle générale, faute d'un DTO propre (#866).
 *
 * Le solde négatif sur le lot 12B est la donnée que le `@security` ci-dessous
 * vérifie : c'est une dette d'un copropriétaire identifiable par son lot, sur
 * un document opposable (Art. 3.87 §3 CC), pas une ligne d'exemple anonyme.
 */
const ETAT_DATE_LOT_12B = {
  id: "33333333-3333-3333-3333-333333333333",
  organization_id: "22222222-2222-2222-2222-222222222222",
  building_id: "11111111-1111-1111-1111-111111111111",
  unit_id: "44444444-4444-4444-4444-444444444444",
  reference_date: "2026-01-15",
  requested_date: "2026-01-10",
  generated_date: null,
  delivered_date: null,
  status: "requested",
  language: "fr",
  reference_number: "ED-2026-0001",
  notary_name: "Maître Dupont",
  notary_email: "notaire@example.com",
  notary_phone: null,
  building_name: "Résidence Les Érables",
  building_address: "Rue des Érables 1, 1000 Bruxelles",
  unit_number: "12B",
  unit_floor: "2",
  unit_area: 85,
  ordinary_charges_quota: 100,
  extraordinary_charges_quota: 50,
  owner_balance: -4200.5,
  arrears_amount: 4200.5,
  monthly_provision_amount: 150,
  total_balance: -4200.5,
  approved_works_unpaid: 0,
  additional_data: {},
  pdf_file_path: null,
  created_at: "2026-01-10T09:00:00Z",
  updated_at: "2026-01-15T09:00:00Z",
  is_overdue: true,
  is_expired: false,
  days_since_request: 12,
};

const LISTE_ETATS_DATES = {
  data: [ETAT_DATE_LOT_12B],
  page: 1,
  per_page: 20,
  total: 1,
};

const STATS_ETATS_DATES = {
  total_requests: 1,
  requested_count: 1,
  in_progress_count: 0,
  generated_count: 0,
  delivered_count: 0,
  expired_count: 0,
  overdue_count: 1,
  average_processing_days: 12,
};

/**
 * Une relance de paiement.
 *
 * Même défaut de forme que les états datés, mais inversé : `/payment-reminders`
 * renvoie un TABLEAU NU (`reminders: PaymentReminder[]`), pas `{data: [...]}`.
 * Servir la forme `{data: [LIGNE], ...}` fait planter `reminders.filter(...)`
 * dans le `$derived` de `PaymentReminderList` (`reminders` reçoit un objet, pas
 * un tableau) — la page ne rendait alors aucun tableau, faussement au vert.
 */
const RAPPEL_JEAN_DUPONT = {
  id: "55555555-5555-5555-5555-555555555555",
  organization_id: "22222222-2222-2222-2222-222222222222",
  expense_id: "66666666-6666-6666-6666-666666666666",
  owner_id: "77777777-7777-7777-7777-777777777777",
  owner_name: "Jean Dupont",
  owner_email: "jean.dupont@example.com",
  level: "FirstReminder",
  status: "Pending",
  amount_owed: 500,
  penalty_amount: 25,
  total_amount: 525,
  due_date: "2026-01-01",
  days_overdue: 20,
  delivery_method: "Email",
  sent_date: "2026-01-05",
  opened_date: null,
  pdf_path: null,
  tracking_number: null,
  notes: null,
  created_at: "2026-01-05T09:00:00Z",
  updated_at: "2026-01-05T09:00:00Z",
};

const LISTE_PAYMENT_REMINDERS = [RAPPEL_JEAN_DUPONT];

const STATS_PAYMENT_REMINDERS = {
  total_owed: 500,
  total_penalties: 25,
  reminder_counts: [{ level: "FirstReminder", count: 1 }],
  status_counts: [{ status: "Pending", count: 1 }],
};

/**
 * Les pages visitées, et le nombre de tableaux qu'on EXIGE d'y trouver.
 *
 * Le compte attendu est la vérification d'aveuglement, et elle n'est pas
 * théorique : trois des sept pages d'origine ne rendaient aucun tableau sous
 * un stub vide, si bien que la règle y passait au vert sans rien examiner.
 *
 * `fixtures` porte une ou plusieurs réponses ciblées, posées APRÈS le
 * fourre-tout du socle (cf. `repond`). `/etats-dates/` et `/payment-reminders/`
 * en ont chacune besoin de DEUX — liste et stats — sans quoi la page plante
 * avant d'atteindre son tableau (cf. commentaires des fixtures ci-dessus).
 * C'est précisément le travail que le commentaire précédent de ce fichier
 * notait comme manquant plutôt que de le masquer par un vert.
 */
const PAGES: {
  role: Role;
  chemin: string;
  fixtures?: { motif: RegExp; corps: unknown }[];
  tableaux: number;
}[] = [
  {
    role: "accountant",
    chemin: "/budgets/",
    fixtures: [
      {
        motif: /\/api\/v1\/budgets/,
        corps: {
          data: [LIGNE],
          items: [LIGNE],
          total: 1,
          page: 1,
          per_page: 20,
        },
      },
    ],
    tableaux: 1,
  },
  { role: "syndic", chemin: "/journal-entries/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/acps/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/users/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/organizations/", tableaux: 1 },
  {
    // Colonnes : reference, buildingUnit, notary, refDate, balance, status,
    // delay, actions — le pire cas cité par la story #866 (9 colonnes mesurées
    // à l'origine, 8 après le nettoyage des faux positifs).
    role: "syndic",
    chemin: "/etats-dates/",
    fixtures: [
      // La regex de la liste ne doit PAS matcher `/etats-dates/stats` : elle
      // exige un `?` ou une fin de chaîne juste après `etats-dates`.
      { motif: /\/api\/v1\/etats-dates(\?|$)/, corps: LISTE_ETATS_DATES },
      { motif: /\/api\/v1\/etats-dates\/stats/, corps: STATS_ETATS_DATES },
    ],
    tableaux: 1,
  },
  {
    // Colonnes : level, owner, amount, penalties, daysOverdue, status,
    // sentDate — huit colonnes dans la mesure d'origine.
    role: "syndic",
    chemin: "/payment-reminders/",
    fixtures: [
      {
        motif: /\/api\/v1\/payment-reminders$/,
        corps: LISTE_PAYMENT_REMINDERS,
      },
      {
        motif: /\/api\/v1\/payment-reminders\/stats/,
        corps: STATS_PAYMENT_REMINDERS,
      },
    ],
    tableaux: 1,
  },
];

for (const { role, chemin, fixtures, tableaux } of PAGES) {
  test(`@edge ${chemin} — aucun tableau coupé en silence`, async ({ page }) => {
    await ouvreEnTantQue(page, role, chemin);
    await repond(page, /\/api\/v1\/buildings/, DEUX_IMMEUBLES);
    if (fixtures && fixtures.length > 0) {
      for (const { motif, corps } of fixtures) {
        await repond(page, motif, corps);
      }
      await page.reload();
      await page.waitForSelector("[data-testid='tabbar']");
    }

    const vus = await page.locator("table").count();
    expect(
      vus,
      `${chemin} ne rend aucun tableau : la règle qui suit passerait au vert ` +
        `sans rien examiner. C'est ce qui arrivait sur trois pages sur sept.`,
    ).toBeGreaterThanOrEqual(tableaux);

    const coupes = await page.evaluate(() => {
      const fautes: string[] = [];
      document.querySelectorAll("table").forEach((table, i) => {
        const largeurTable = table.scrollWidth;

        // Le conteneur défilant est le premier ancêtre dont la largeur
        // visible diffère : c'est lui qui décide si la droite du tableau
        // existe encore.
        let conteneur: HTMLElement | null = table.parentElement;
        while (conteneur && conteneur.clientWidth >= largeurTable) {
          conteneur = conteneur.parentElement;
        }
        if (!conteneur) return; // Le tableau tient : rien à garder.

        const style = getComputedStyle(conteneur);
        const defile = /auto|scroll/.test(style.overflowX);
        const atteignable = conteneur.hasAttribute("tabindex");
        const nom = `table#${i} (${table.querySelectorAll("thead th").length} colonnes)`;

        if (!defile) {
          fautes.push(
            `${nom} mesure ${largeurTable} px dans un conteneur de ` +
              `${conteneur.clientWidth} px en overflow-x:${style.overflowX} — ` +
              `les colonnes de droite sont COUPÉES, sans aucun signe.`,
          );
        } else if (!atteignable) {
          fautes.push(
            `${nom} défile mais son conteneur n'a pas de \`tabindex\` : ` +
              `au clavier, la droite du tableau reste hors de portée.`,
          );
        }
      });
      return fautes;
    });

    expect(coupes, coupes.join("\n")).toEqual([]);
  });
}

/**
 * @security — un état daté est un document opposable (Art. 3.87 §3 CC), pas
 * une liste anonyme : ce que la règle générale ci-dessus vérifie en abstrait
 * (« un conteneur défile »), ce test le vérifie sur la donnée concrète que le
 * découpage silencieux ferait disparaître — le solde d'un lot nommé.
 *
 * Avant #866, un syndic consultant l'état daté du lot 12B sur son téléphone ne
 * voyait pas sa colonne « Solde » : pas de barre, pas d'ombre, aucun signe
 * qu'une dette de 4 200,50 € existait derrière le bord de l'écran.
 */
test("@security /etats-dates/ — le solde du lot 12B n'est pas coupé en silence", async ({
  page,
}) => {
  await ouvreEnTantQue(page, "syndic", "/etats-dates/");
  await repond(page, /\/api\/v1\/buildings/, DEUX_IMMEUBLES);
  await repond(page, /\/api\/v1\/etats-dates(\?|$)/, LISTE_ETATS_DATES);
  await repond(page, /\/api\/v1\/etats-dates\/stats/, STATS_ETATS_DATES);
  await page.reload();
  await page.waitForSelector("[data-testid='tabbar']");

  const ligne = page.getByTestId("etat-date-row").first();
  await expect(ligne).toBeVisible();

  // 5ᵉ colonne du `<thead>` : reference(0), buildingUnit(1), notary(2),
  // refDate(3), balance(4). C'est elle qui portait la dette coupée.
  const celluleSolde = ligne.locator("td").nth(4);
  await expect(celluleSolde).toContainText("200");

  const verdict = await page.evaluate(() => {
    const rangee = document.querySelector("[data-testid='etat-date-row']");
    const table = rangee?.closest("table");
    const conteneur = table?.parentElement;
    if (!table || !conteneur) return null;
    return {
      tableTientEntier: conteneur.clientWidth >= table.scrollWidth,
      defile: /auto|scroll/.test(getComputedStyle(conteneur).overflowX),
      atteignable: conteneur.hasAttribute("tabindex"),
    };
  });

  expect(
    verdict,
    "La ligne du lot 12B ou son conteneur sont introuvables : ce témoin ne " +
      "peut rien garantir sur sa dette.",
  ).not.toBeNull();

  if (!verdict!.tableTientEntier) {
    expect(
      verdict!.defile,
      "Le tableau déborde et son conteneur ne défile pas : le solde du " +
        "lot 12B (colonne « Solde ») peut être coupé sans aucun signe.",
    ).toBe(true);
    expect(
      verdict!.atteignable,
      "Le conteneur défile mais n'a pas de `tabindex` : au clavier, le " +
        "solde du lot 12B reste hors de portée.",
    ).toBe(true);
  }
});

/**
 * @negative — étant donné un `overflow-hidden` réintroduit sur un tableau,
 * la suite doit échouer.
 *
 * On ne le simule pas en éditant un composant en direct (ce serait fragile et
 * dupliquerait six fois le même montage) : on le simule au niveau où vit la
 * règle elle-même — la détection d'ancêtre défilant. Sans ce contrôle, la
 * règle des tests `@edge` ci-dessus passerait au vert sur une page sans
 * tableau, ou si la remontée d'ancêtres ne trouvait jamais de conteneur ; elle
 * passerait AUSSI au vert si elle confondait un `overflow:hidden` avec un
 * conteneur défilant. C'est ce second cas — la régression exacte de #866 —
 * que ce test garde rouge.
 */
test("@negative un `overflow-hidden` réintroduit sur un tableau est détecté comme une coupe", async ({
  page,
}) => {
  await ouvreEnTantQue(page, "accountant", "/budgets/");

  const detecte = await page.evaluate(() => {
    const carte = document.createElement("div");
    carte.style.cssText = "overflow:hidden;width:200px";
    const table = document.createElement("table");
    table.innerHTML =
      "<thead><tr><th>a</th><th>b</th></tr></thead><tbody><tr>" +
      "<td style='min-width:400px'>large</td><td style='min-width:400px'>large</td></tr></tbody>";
    carte.appendChild(table);
    document.getElementById("main-content")?.appendChild(carte);

    let conteneur: HTMLElement | null = table.parentElement;
    while (conteneur && conteneur.clientWidth >= table.scrollWidth) {
      conteneur = conteneur.parentElement;
    }
    return {
      conteneurTrouve: conteneur !== null,
      defile: conteneur
        ? /auto|scroll/.test(getComputedStyle(conteneur).overflowX)
        : null,
    };
  });

  expect(
    detecte.conteneurTrouve,
    "La remontée d'ancêtres n'a trouvé aucun conteneur pour un tableau " +
      "volontairement trop large : la règle ne peut rien voir.",
  ).toBe(true);
  expect(
    detecte.defile,
    "Un conteneur en `overflow:hidden` a été lu comme défilant : la suite " +
      "ne détecterait pas la régression de #866 si elle était réintroduite.",
  ).toBe(false);
});
