// Cliquet — le texte blanc sur un fond de couleur doit passer WCAG 2.1 AA.
//
// ## Le défaut que ce cliquet ferme
//
// Deux fois le 2026-09-17, axe-core a relevé un contraste insuffisant sur du
// texte blanc posé sur un fond Tailwind de niveau 600 :
//
//   SyndicDashboard   orange-600 sur blanc → ~3,5:1
//   OnboardingWizard  blanc sur sky-600    → 4,02:1
//
// Le seuil AA pour du texte de taille normale est **4,5:1**.
//
// Ces deux-là n'ont été vus que parce que ces deux écrans ont un test
// d'accessibilité. Les autres n'en ont pas : le défaut est donc invisible
// partout ailleurs, et il se reproduira à chaque nouveau bouton.
//
// ## Pourquoi un calcul et non une liste interdite
//
// « le niveau 600 ne passe pas » serait FAUX. Mesuré ici même :
//
//   blue-600   5,17:1  ✓        green-600  3,30:1  ✗
//   indigo-600 6,29:1  ✓        amber-600  3,18:1  ✗
//   red-600    4,83:1  ✓        sky-600    4,02:1  ✗
//
// La luminance dépend de la teinte, pas seulement du niveau. Une liste
// interdite se tromperait dans les deux sens : elle bannirait `blue-600`
// qui passe, et laisserait filer `green-600` qui échoue.
//
// ## Ce qu'il lit
//
// La palette vient de `tailwindcss/colors`, en OKLCH depuis la v4, convertie
// ici en sRGB. La conversion a été vérifiée contre la mesure d'axe-core :
// `sky-600` rend `#0084d1` des deux côtés.

import { describe, it, expect } from "vitest";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import couleurs from "tailwindcss/colors";

const SEUIL_AA = 4.5;

/** OKLCH → sRGB 0-255. Vérifié : sky-600 → #0084d1, comme axe-core. */
function oklchVersSrgb(
  L: number,
  C: number,
  hDeg: number,
): [number, number, number] {
  const h = (hDeg * Math.PI) / 180;
  const a = C * Math.cos(h);
  const b = C * Math.sin(h);
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (L - 0.0894841775 * a - 0.291485548 * b - 1.0 * b) ** 3;
  const sBon = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  void s;
  const lineaire = [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * sBon,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * sBon,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * sBon,
  ];
  return lineaire.map((v) => {
    const x = v <= 0.0031308 ? 12.92 * v : 1.055 * Math.pow(v, 1 / 2.4) - 0.055;
    return Math.max(0, Math.min(255, Math.round(x * 255)));
  }) as [number, number, number];
}

function luminance([r, g, b]: [number, number, number]): number {
  const canal = (v: number) => {
    const c = v / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  };
  return 0.2126 * canal(r) + 0.7152 * canal(g) + 0.0722 * canal(b);
}

/** Contraste avec le BLANC (luminance 1). */
function contrasteAvecBlanc(rgb: [number, number, number]): number {
  return 1.05 / (luminance(rgb) + 0.05);
}

/** `oklch(58.8% 0.158 241.966)` → [0.588, 0.158, 241.966]. */
function lireOklch(valeur: string): [number, number, number] | null {
  const m = valeur.match(/oklch\(\s*([\d.]+)%\s+([\d.]+)\s+([\d.]+)/);
  if (!m) return null;
  return [Number(m[1]) / 100, Number(m[2]), Number(m[3])];
}

function contrasteDe(teinte: string, niveau: string): number | null {
  // `couleurs` mélange des palettes et des scalaires (`inherit`, `white`,
  // `transparent`) : on écarte les seconds avant d'indexer.
  const palette = (
    couleurs as unknown as Record<string, Record<string, string> | string>
  )[teinte];
  if (typeof palette !== "object" || palette === null) return null;
  const valeur = palette[niveau];
  if (!valeur) return null;
  const oklch = lireOklch(valeur);
  if (!oklch) return null;
  return contrasteAvecBlanc(oklchVersSrgb(...oklch));
}

function fichiersSvelte(racine: string): string[] {
  const trouves: string[] = [];
  for (const nom of readdirSync(racine)) {
    const chemin = join(racine, nom);
    if (statSync(chemin).isDirectory()) trouves.push(...fichiersSvelte(chemin));
    else if (nom.endsWith(".svelte")) trouves.push(chemin);
  }
  return trouves;
}

/** Les couples (teinte, niveau) posés sous du texte blanc, par fichier. */
function couplesSousTexteBlanc(): Map<string, Set<string>> {
  const par = new Map<string, Set<string>>();
  for (const chemin of fichiersSvelte("src")) {
    const source = readFileSync(chemin, "utf8");
    for (const attribut of source.match(/class="[^"]*"/g) ?? []) {
      if (!attribut.includes("text-white")) continue;
      // Seulement l'état AU REPOS.
      //
      // Une chaîne de classes Tailwind mélange les états :
      // `text-white … disabled:bg-gray-300 hover:bg-sky-800`. Compter tous
      // les `bg-` faisait remonter `gray-50` à 1,05:1 — un faux positif,
      // puisque ce fond n'est jamais sous le texte blanc au repos.
      //
      // La règle : un `bg-` précédé d'un `:` appartient à une variante
      // (`disabled:`, `hover:`, `focus:`, `group-hover:`…) et n'est pas
      // l'état que mesure axe-core. On ne garde que les autres.
      for (const m of attribut.matchAll(/(^|[\s"])bg-([a-z]+)-(\d{2,3})\b/g)) {
        const cle = `${m[2]}-${m[3]}`;
        if (!par.has(cle)) par.set(cle, new Set());
        par.get(cle)!.add(chemin.replace("src/", ""));
      }
    }
  }
  return par;
}

/**
 * Couples encore fautifs au 2026-09-17. Mesuré, pas souhaité.
 *
 * Les corriger revient à foncer d'un cran (`-600` → `-700`) dans 35
 * endroits, ce qui change l'aspect de l'application sur de nombreux écrans.
 * C'est une décision de conception, posée en #942 — pas quelque chose à
 * glisser dans une passe de tests.
 *
 * Ce nombre ne doit que DESCENDRE.
 */
const COUPLES_FAUTIFS_AU_2026_09_17 = [
  // teinte-niveau     ratio mesuré     où
  "green-600", //      3,30:1           22 emplois
  "amber-600", //      3,18:1           13 emplois
  "yellow-600", //     2,94:1           ProfilePanel, BudgetDetail, EtatDateDetail, InspectionDetail…
  "red-500", //        3,81:1           SyndicDashboard, EtatDateDetail, EtatDateList…
  "orange-600", //     3,60:1           PaymentReminderDetail, SkillOfferDetail
];

describe("contraste du texte blanc sur fond de couleur (WCAG 2.1 AA)", () => {
  it("le_cliquet_lit_bien_quelque_chose", () => {
    const couples = couplesSousTexteBlanc();
    expect(couples.size).toBeGreaterThan(5);
  });

  it("@happy la conversion OKLCH reproduit la mesure d'axe-core", () => {
    // Témoin : axe-core a rendu `#0084d1` pour `sky-600`. Si cette
    // assertion tombe, la palette a changé ou la conversion est fausse —
    // et tout le reste du fichier ne mesure plus rien.
    const sky = (couleurs as unknown as Record<string, Record<string, string>>)
      .sky;
    const oklch = lireOklch(sky["600"]);
    expect(oklch).not.toBeNull();
    const [r, g, b] = oklchVersSrgb(...oklch!);
    const hex =
      "#" + [r, g, b].map((v) => v.toString(16).padStart(2, "0")).join("");
    expect(hex).toBe("#0084d1");
  });

  it("@security aucun NOUVEAU couple ne passe sous le seuil AA", () => {
    const fautifs: string[] = [];
    for (const [couple, fichiers] of couplesSousTexteBlanc()) {
      const [teinte, niveau] = couple.split(/-(?=\d+$)/);
      const ratio = contrasteDe(teinte, niveau);
      if (ratio === null) continue; // teinte hors palette (primary, etc.)
      if (ratio < SEUIL_AA && !COUPLES_FAUTIFS_AU_2026_09_17.includes(couple)) {
        fautifs.push(
          `  ${couple} → ${ratio.toFixed(2)}:1 (seuil ${SEUIL_AA}) — ` +
            [...fichiers].sort().slice(0, 4).join(", "),
        );
      }
    }
    fautifs.sort();
    expect(
      fautifs.join("\n"),
      "Du texte blanc est posé sur un fond dont le contraste est insuffisant.\n\n" +
        "Foncez d'un cran : le niveau 700 de la même teinte passe dans tous " +
        "les cas mesurés. Ne retirez pas `text-white` pour contourner — le " +
        "problème est le FOND.\n",
    ).toBe("");
  });

  it("@edge les couples tolérés le sont encore, et pas plus", () => {
    // Un cliquet qui tolère un couple déjà corrigé se tait pour rien, et
    // laisserait sa réintroduction passer inaperçue.
    const utilises = new Set(couplesSousTexteBlanc().keys());
    const toleres_disparus = COUPLES_FAUTIFS_AU_2026_09_17.filter(
      (c) => !utilises.has(c),
    );
    expect(
      toleres_disparus,
      "Ces couples ne sont plus employés : retirez-les de " +
        "COUPLES_FAUTIFS_AU_2026_09_17 pour que le cliquet se resserre.",
    ).toEqual([]);
  });
});
