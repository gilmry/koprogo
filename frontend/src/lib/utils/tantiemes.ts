/**
 * Formatage des quotes-parts (tantièmes).
 *
 * Les quotes-parts sont des `Decimal` côté backend (ADR-0008 § A : tout ce qui
 * alimente un seuil légal est `Decimal`, jamais `f64`). Sérialisées en JSON,
 * elles arrivent donc en **chaîne** — c'est ce que déclare le type généré
 * depuis l'OpenAPI, `quota: string`.
 *
 * Trois composants les passaient à `Math.round()`. JavaScript convertit
 * silencieusement, donc l'affichage semblait juste ; TypeScript, lui, avait
 * raison de refuser. Le jour où une quote-part arrive avec un séparateur
 * décimal inattendu ou une valeur vide, `Math.round` rend `NaN` et l'écran
 * affiche « NaN/1000èmes » à un copropriétaire.
 *
 * La quote-part détermine la voix en assemblée (Art. 3.87 § 6) et la part de
 * charges. Ce n'est pas un chiffre d'agrément.
 */

/**
 * Rend la quote-part telle quelle, **sans arrondi**, ou `null` si elle est
 * absente ou illisible.
 *
 * Distincte de `tantiemes` parce que la validation ne doit pas arrondir : une
 * quote-part de 0,4 doit être refusée comme non positive après arrondi à 0,
 * pas acceptée parce qu'elle vaut 0,4. Et 0,6 ne doit pas devenir 1.
 */
/**
 * Rend la quote-part en tantièmes entiers, ou `null` si elle est absente ou
 * illisible.
 *
 * `null` plutôt que zéro : une quote-part inconnue n'est pas une quote-part
 * nulle. Zéro voudrait dire « ce lot ne vote pas », ce qui est une tout autre
 * affirmation.
 */
export function valeurQuota(quota: string | number | null | undefined): number | null {
  if (quota === null || quota === undefined || quota === "") return null;
  const valeur = typeof quota === "number" ? quota : Number(quota);
  return Number.isFinite(valeur) ? valeur : null;
}

export function tantiemes(quota: string | number | null | undefined): number | null {
  const valeur = valeurQuota(quota);
  return valeur === null ? null : Math.round(valeur);
}

/**
 * Rend la quote-part prête à afficher, base comprise : « 250/1000èmes ».
 *
 * `base` vient de l'acte de base (`total_tantiemes`) et vaut souvent 1000 ou
 * 10000. Elle n'est pas codée en dur ici : la coder en dur est exactement le
 * défaut corrigé par la Story H1.
 */
export function formatTantiemes(
  quota: string | number | null | undefined,
  base: number | string = 1000,
): string {
  const valeur = tantiemes(quota);
  if (valeur === null) return "—";
  const denominateur = tantiemes(base) ?? 1000;
  return `${valeur}/${denominateur}èmes`;
}
