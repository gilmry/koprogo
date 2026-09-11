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
 * Les pages visitées, et le nombre de tableaux qu'on EXIGE d'y trouver.
 *
 * Le compte attendu est la vérification d'aveuglement, et elle n'est pas
 * théorique : trois des sept pages d'origine ne rendaient aucun tableau sous
 * un stub vide, si bien que la règle y passait au vert sans rien examiner.
 *
 * **Deux pages manquent volontairement.** `/payment-reminders/` et
 * `/etats-dates/` ont besoin d'une réponse plus précise qu'une ligne
 * générique — la première ne rend aucun tableau, la seconde lève deux erreurs
 * de page. Les couvrir demande de décrire leur DTO, ce qui est un travail à
 * part ; il est noté sur #866 plutôt que masqué par un vert.
 */
const PAGES: {
  role: Role;
  chemin: string;
  donnees?: RegExp;
  tableaux: number;
}[] = [
  {
    role: "accountant",
    chemin: "/budgets/",
    donnees: /\/api\/v1\/budgets/,
    tableaux: 1,
  },
  { role: "syndic", chemin: "/journal-entries/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/acps/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/users/", tableaux: 1 },
  { role: "superadmin", chemin: "/admin/organizations/", tableaux: 1 },
];

for (const { role, chemin, donnees, tableaux } of PAGES) {
  test(`@edge ${chemin} — aucun tableau coupé en silence`, async ({ page }) => {
    await ouvreEnTantQue(page, role, chemin);
    await repond(page, /\/api\/v1\/buildings/, DEUX_IMMEUBLES);
    if (donnees) {
      await repond(page, donnees, {
        data: [LIGNE],
        items: [LIGNE],
        total: 1,
        page: 1,
        per_page: 20,
      });
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

test("@edge le détecteur voit bien un tableau coupé", async ({ page }) => {
  // Sans ce contrôle, la règle ci-dessus passerait au vert sur une page sans
  // tableau, ou si la remontée d'ancêtres ne trouvait jamais de conteneur.
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
    "Un conteneur en `overflow:hidden` a été lu comme défilant.",
  ).toBe(false);
});
