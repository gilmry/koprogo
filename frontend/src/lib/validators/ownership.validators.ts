import type { ValidationResult } from "./common.validators";

/**
 * Validate that adding a new ownership percentage won't exceed 100%.
 * Belgian law (Article 577-2 §4 Code Civil): total shares must equal 100%.
 *
 * @param newPercentage - The percentage to add/change (in UI scale, e.g. 50 = 50%)
 * @param currentTotal - Current total of all active owners (in API scale, e.g. 0.5 = 50%)
 * @param tolerance - Tolerance for floating point rounding (default 0.01)
 * @returns null if valid, error message string if invalid
 */
export function validateOwnershipPercentage(
  newPercentage: number,
  currentTotal: number,
  tolerance: number = 0.01,
  errorMessage?: string,
): ValidationResult {
  if (newPercentage <= 0) {
    return errorMessage ?? "Ownership percentage must be greater than 0";
  }
  if (newPercentage > 100) {
    return errorMessage ?? "Ownership percentage cannot exceed 100%";
  }

  const availablePercentage = (1 - currentTotal) * 100;
  if (newPercentage > availablePercentage + tolerance) {
    return errorMessage ?? `Ownership percentage exceeds available (${availablePercentage.toFixed(2)}%)`;
  }

  return null;
}
