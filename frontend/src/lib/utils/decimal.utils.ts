/**
 * Conversion des `Decimal` du backend, serialises en STRING JSON.
 *
 * `rust_decimal` serialise les `Decimal` en chaine (ADR-0008), et le contrat
 * publie l'assume : `docs/api/openapi.json` declare ces champs `type: string`.
 *
 * Le piege est que `*` et `/` convertissent implicitement la chaine, alors que
 * `+` CONCATENE. Un meme champ se comporte donc correctement dans un calcul et
 * silencieusement faux dans une somme. Deux defauts de production trouves le
 * 2026-08-26/27 venaient exactement de la :
 *
 *   - `total_voting_power_*` : le decompte des pouvoirs de vote d'AG ne
 *     s'affichait jamais (Art. 3.87) ;
 *   - `ownership_percentage` : l'avertissement « les quotites devraient faire
 *     100% » restait affiche en permanence, et le garde-fou de depassement des
 *     formulaires recevait NaN.
 *
 * Passer par cette fonction rend la conversion explicite et uniforme.
 */
export function toNumber(v: string | number | null | undefined): number {
  const n = typeof v === "number" ? v : Number.parseFloat(String(v ?? ""));
  return Number.isFinite(n) ? n : 0;
}
