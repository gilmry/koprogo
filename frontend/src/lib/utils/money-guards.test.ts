import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Garde-fous STATIQUES sur les montants.
 *
 * Non-régression du rapport « workflows financiers » du 2026-09-01. Ces tests
 * ne montent aucun composant : ils relisent le code source. C'est délibéré —
 * les deux défauts ci-dessous sont invisibles à l'exécution d'un test unitaire
 * classique (l'un compile et affiche NaN, l'autre compile et affiche un
 * montant divisé par 100), mais parfaitement détectables à la lecture.
 */

// Même résolution que `locales.test.ts` : `import.meta.url` pointe sur un
// module transformé par Vite, pas sur le fichier source.
const SRC = join(import.meta.dirname ?? __dirname, "..", "..");

function fichiersSources(dir: string, acc: string[] = []): string[] {
  for (const nom of readdirSync(dir)) {
    const chemin = join(dir, nom);
    if (statSync(chemin).isDirectory()) {
      if (nom === "types" || nom === "node_modules") continue;
      fichiersSources(chemin, acc);
    } else if (/\.(svelte|ts)$/.test(nom) && !/\.test\.ts$/.test(nom)) {
      acc.push(chemin);
    }
  }
  return acc;
}

const FICHIERS = fichiersSources(SRC);

describe("montants — garde-fous statiques", () => {
  /**
   * Champs `Decimal` côté Rust, donc CHAÎNES en JSON. Les additionner avec `+`
   * concatène au lieu d'additionner (défaut F14 : « NaN/1000èmes »).
   *
   * La liste est volontairement restreinte aux champs réellement sommés dans
   * l'interface : une liste exhaustive des ~60 champs `Decimal` produirait des
   * faux positifs sur de simples affichages, et un garde-fou qui crie pour
   * rien finit désactivé.
   */
  const CHAMPS_DECIMAUX = [
    "quota",
    "amount_due",
    "amount_owed",
    "total_amount",
    "penalty_amount",
    "ownership_percentage",
    "total_voting_power_pour",
    "total_voting_power_contre",
    "total_voting_power_abstention",
    "present_quotas",
    "total_quotas",
  ];

  it("n'additionne jamais un champ Decimal sans le convertir", () => {
    const motif = new RegExp(
      // `+ <qqch>.<champ>` sans passage par toNumber/parseFloat/Number.
      String.raw`\+\s*\(?\s*[A-Za-z_$][\w$]*\.(` + CHAMPS_DECIMAUX.join("|") + String.raw`)\b`,
      "g",
    );

    const fautes: string[] = [];
    for (const f of FICHIERS) {
      const lignes = readFileSync(f, "utf-8").split("\n");
      lignes.forEach((ligne, i) => {
        // Les commentaires citent volontiers le motif fautif pour l'expliquer :
        // sans ce filtre, la documentation du défaut le fait rouvrir.
        const nue = ligne.trim();
        if (nue.startsWith("//") || nue.startsWith("*") || nue.startsWith("/*")) return;
        if (ligne.includes("toNumber(") || ligne.includes("parseFloat(")) return;
        if (motif.test(ligne)) {
          fautes.push(`${relative(SRC, f)}:${i + 1}  ${ligne.trim().slice(0, 110)}`);
        }
        motif.lastIndex = 0;
      });
    }

    expect(
      fautes,
      "Ces lignes additionnent un `Decimal` sérialisé en chaîne : `+` va " +
        "concaténer, pas additionner. Passer par `toNumber()` " +
        "(src/lib/utils/decimal.utils.ts).\n" +
        fautes.join("\n"),
    ).toEqual([]);
  });

  /**
   * `finance.utils.formatAmount(cents)` DIVISE PAR 100. Un helper local du même
   * nom qui recevrait des euros produit un affichage 100 fois trop petit — et
   * le jour où quelqu'un ajoute l'import partagé au fichier, ou déplace une
   * ligne d'un composant à l'autre, l'erreur devient silencieuse.
   *
   * `InvoiceList.svelte` portait exactement ce doublon (renommé `formatEuros`).
   */
  it("ne redéfinit jamais `formatAmount` localement", () => {
    const motif = /function\s+formatAmount\s*\(|const\s+formatAmount\s*=/;
    const fautes = FICHIERS.filter((f) => {
      if (f.endsWith("finance.utils.ts")) return false; // la définition légitime
      return motif.test(readFileSync(f, "utf-8"));
    }).map((f) => relative(SRC, f));

    expect(
      fautes,
      "`formatAmount` est défini dans `lib/utils/finance.utils.ts` et attend " +
        "des CENTIMES. Un homonyme local qui reçoit des euros est une erreur " +
        "de facteur 100 en attente. Nommer autrement (`formatEuros`).\n" +
        fautes.join("\n"),
    ).toEqual([]);
  });

  /**
   * Défaut F9 : `BudgetStatusBadge` indexait un dictionnaire à clés minuscules
   * avec un statut que l'API renvoie en PascalCase (`Draft`, `Submitted`).
   * Aucune clé ne correspondait ; le repli affichait l'énumération brute, en
   * anglais, sur une interface française.
   *
   * La casse n'est PAS devinable à la lecture du frontend, et elle n'est pas
   * uniforme côté backend : `EtatDateStatus` porte `#[serde(rename_all)]` et
   * sort en `snake_case`, `BudgetStatus` ne l'avait que sur son attribut
   * `sqlx` — donc sortait en PascalCase. Seul le contrat tranche.
   *
   * Ce test lit donc `docs/api/openapi.json` et exige que chaque badge soit
   * capable de rendre TOUTES les valeurs déclarées de son énumération.
   */
  it("rend toutes les valeurs d'énumération déclarées au contrat", () => {
    const contrat = JSON.parse(
      readFileSync(join(SRC, "..", "..", "docs", "api", "openapi.json"), "utf-8"),
    );
    const enums: Record<string, string[]> = {};
    for (const [nom, def] of Object.entries<any>(contrat.components.schemas)) {
      if (Array.isArray(def?.enum)) enums[nom] = def.enum;
    }
    expect(Object.keys(enums).length).toBeGreaterThan(10);

    const badges = FICHIERS.filter((f) => /StatusBadge\.svelte$/.test(f));
    expect(badges.length).toBeGreaterThan(0);

    const fautes: string[] = [];
    for (const f of badges) {
      const src = readFileSync(f, "utf-8");
      const cles = [...src.matchAll(/^\s*['"]([A-Za-z_]+)['"]\s*:\s*\{/gm)].map((m) => m[1]);
      if (cles.length === 0) continue;

      const normalise = /\.toLowerCase\(\)\]/.test(src);
      const rendable = (v: string) =>
        normalise ? cles.includes(v.toLowerCase()) : cles.includes(v);

      // Rattache le badge à l'énumération du contrat qu'il couvre le mieux.
      let meilleur: [string, string[]] | null = null;
      for (const [nom, valeurs] of Object.entries(enums)) {
        const communs = valeurs.filter((v) =>
          cles.some((c) => c.toLowerCase() === v.toLowerCase()),
        ).length;
        const score = communs / valeurs.length;
        if (score >= 0.6 && (!meilleur || communs > meilleur[1].length * 0.6)) {
          meilleur = [nom, valeurs];
        }
      }
      // Énumération hors contrat : rien à comparer, c'est la dette de
      // couverture OpenAPI (scripts/check-openapi-coverage.sh), pas ce test.
      if (!meilleur) continue;

      const manquantes = meilleur[1].filter((v) => !rendable(v));
      if (manquantes.length > 0) {
        fautes.push(
          `${relative(SRC, f)} — ${meilleur[0]} : ne rend pas ${manquantes.join(", ")}`,
        );
      }
    }

    expect(
      fautes,
      "Ces badges ne couvrent pas toutes les valeurs que l'API peut renvoyer : " +
        "le statut s'affichera brut, en anglais. Aligner les clés sur " +
        "`docs/api/openapi.json`, ou normaliser la casse à l'indexation.\n" +
        fautes.join("\n"),
    ).toEqual([]);
  });
});
