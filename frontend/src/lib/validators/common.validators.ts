/**
 * Validation result: null = valid, string = error message (or i18n key).
 */
export type ValidationResult = string | null;

/** Check that a value is non-empty. */
export function required(
  value: string | null | undefined,
  errorMessage: string = "This field is required",
): ValidationResult {
  if (!value || !value.trim()) return errorMessage;
  return null;
}

/** Check minimum string length (after trim). */
export function minLength(
  value: string | null | undefined,
  min: number,
  errorMessage?: string,
): ValidationResult {
  if (!value || value.trim().length < min) {
    return errorMessage ?? `Minimum ${min} characters required`;
  }
  return null;
}

/** Check maximum string length. */
export function maxLength(
  value: string | null | undefined,
  max: number,
  errorMessage?: string,
): ValidationResult {
  if (value && value.length > max) {
    return errorMessage ?? `Maximum ${max} characters allowed`;
  }
  return null;
}

/** Validate email format. */
export function isEmail(
  value: string | null | undefined,
  errorMessage: string = "Invalid email address",
): ValidationResult {
  if (!value) return errorMessage;
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  if (!emailRegex.test(value)) return errorMessage;
  return null;
}

/** Check that a number is positive (> 0). */
export function isPositiveNumber(
  value: number | null | undefined,
  errorMessage: string = "Value must be positive",
): ValidationResult {
  if (value === null || value === undefined || value <= 0) return errorMessage;
  return null;
}

/** Check that a number is within a range [min, max]. */
export function isInRange(
  value: number | null | undefined,
  min: number,
  max: number,
  errorMessage?: string,
): ValidationResult {
  if (value === null || value === undefined || value < min || value > max) {
    return errorMessage ?? `Value must be between ${min} and ${max}`;
  }
  return null;
}

/** Check that password and confirmation match. */
export function passwordMatch(
  password: string,
  confirmation: string,
  errorMessage: string = "Passwords do not match",
): ValidationResult {
  if (password !== confirmation) return errorMessage;
  return null;
}

/** Check that a string starts with a specific prefix (e.g., "pm_" for Stripe). */
export function startsWithPrefix(
  value: string | null | undefined,
  prefix: string,
  errorMessage?: string,
): ValidationResult {
  if (!value || !value.startsWith(prefix)) {
    return errorMessage ?? `Value must start with "${prefix}"`;
  }
  return null;
}

/**
 * Check if an errors object has any actual errors (non-null values).
 * Replaces the `Object.keys(errors).length === 0` pattern.
 */
export function hasErrors(errors: Record<string, ValidationResult>): boolean {
  return Object.values(errors).some((v) => v !== null && v !== undefined);
}

/**
 * Collect only the non-null errors into a clean Record<string, string>.
 * Useful for passing to components that expect Record<string, string>.
 */
export function collectErrors(
  errors: Record<string, ValidationResult>,
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(errors)) {
    if (value !== null && value !== undefined) {
      result[key] = value;
    }
  }
  return result;
}
