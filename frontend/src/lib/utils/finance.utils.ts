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
export function formatCurrency(amount: number): string {
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
