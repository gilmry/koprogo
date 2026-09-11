/** Belgian VAT rates (AR 12/07/2012) */
export const BELGIAN_VAT_RATES = [0, 6, 12, 21] as const;

/**
 * Calculate VAT amount and total including VAT.
 * Uses banker's rounding to 2 decimal places.
 */
export function calculateVAT(
  amountExclVat: number,
  vatRatePercent: number,
): { vatAmount: number; amountInclVat: number } {
  const vatAmount =
    Math.round(((amountExclVat * vatRatePercent) / 100) * 100) / 100;
  const amountInclVat = Math.round((amountExclVat + vatAmount) * 100) / 100;
  return { vatAmount, amountInclVat };
}

/**
 * Calculate a single line item total with VAT.
 */
export function calculateLineItem(
  quantity: number,
  unitPrice: number,
  vatRatePercent: number,
): { amountExclVat: number; vatAmount: number; amountInclVat: number } {
  const amountExclVat = Math.round(quantity * unitPrice * 100) / 100;
  const { vatAmount, amountInclVat } = calculateVAT(
    amountExclVat,
    vatRatePercent,
  );
  return { amountExclVat, vatAmount, amountInclVat };
}

/**
 * Aggregate multiple line items into totals.
 */
export function aggregateLineItems(
  items: { amount_excl_vat: number; vat_amount: number }[],
): { totalHT: number; totalVAT: number; totalTTC: number } {
  const totalHT = items.reduce((sum, item) => sum + item.amount_excl_vat, 0);
  const totalVAT = items.reduce((sum, item) => sum + item.vat_amount, 0);
  return { totalHT, totalVAT, totalTTC: totalHT + totalVAT };
}

/**
 * Format an amount in cents to a display string: "12,50 €"
 */
export function formatAmount(cents: number): string {
  return formatCurrency(cents / 100);
}

/**
 * Format a currency amount using Belgian locale: "1 234,56 €"
 */
/**
 * Ce qu'on affiche quand le montant n'existe pas.
 *
 * Pas « 0,00 € » : un budget non renseigné n'est pas un budget nul, et
 * l'écrire ainsi affirme au comptable quelque chose de faux. Le tiret cadratin
 * dit « pas de valeur », ce qui est l'information exacte.
 */
const MONTANT_ABSENT = "—";

/**
 * Un montant, ou un tiret s'il n'y en a pas.
 *
 * ── Pourquoi la garde, alors que le type dit `number` ────────────────────
 *
 * Parce que ce `number` vient d'une réponse d'API désérialisée, où TypeScript
 * ne vérifie rien. Un champ absent arrive en `undefined`, et
 * `Intl.NumberFormat.format(undefined)` rend **« NaN € »**.
 *
 * Mesuré au banc mobile le 2026-09-11 : `NaN €` s'affichait sur l'écran des
 * budgets, au comptable. Cette fonction a 105 appelants, tous sur des
 * montants ; aucun test ne la couvrait.
 */
export function formatCurrency(amount: number): string {
  if (!Number.isFinite(amount)) return MONTANT_ABSENT;
  return new Intl.NumberFormat("fr-BE", {
    style: "currency",
    currency: "EUR",
  }).format(amount);
}

/** Convert UI percentage (e.g. 50) to API decimal (0.5) */
export function percentageToDecimal(uiValue: number): number {
  return uiValue / 100;
}

/** Convert API decimal (0.5) to UI percentage (50) */
export function decimalToPercentage(apiValue: number): number {
  return apiValue * 100;
}
