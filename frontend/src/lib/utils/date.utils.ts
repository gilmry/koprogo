import { get } from "svelte/store";
import { locale } from "svelte-i18n";

/**
 * Map svelte-i18n locale code to Belgian locale for Intl.DateTimeFormat.
 */
function getBelgianLocale(): string {
  const current = get(locale) ?? "fr";
  const map: Record<string, string> = {
    nl: "nl-BE",
    fr: "fr-BE",
    de: "de-BE",
    en: "en-GB",
  };
  return map[current] ?? "fr-BE";
}

/**
 * Format a date string for display. Returns "-" for null/undefined/empty values.
 */
export function formatDate(
  dateString: string | null | undefined,
  format: "short" | "long" | "datetime" = "long",
): string {
  if (!dateString) return "-";
  const date = new Date(dateString);
  if (isNaN(date.getTime())) return "-";

  const loc = getBelgianLocale();

  switch (format) {
    case "short":
      return date.toLocaleDateString(loc);
    case "long":
      return date.toLocaleDateString(loc, {
        year: "numeric",
        month: "long",
        day: "numeric",
      });
    case "datetime":
      return date.toLocaleDateString(loc, {
        year: "numeric",
        month: "long",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
  }
}

/** Short date: "25/03/2026" */
export function formatDateShort(dateString: string | null | undefined): string {
  return formatDate(dateString, "short");
}

/** Long date with time: "25 maart 2026, 14:30" */
export function formatDateTime(dateString: string | null | undefined): string {
  return formatDate(dateString, "datetime");
}

/**
 * Check if a date is in the past (overdue).
 * Returns false for null/undefined dates or if status is in excludeStatuses.
 */
export function isOverdue(
  dueDate: string | null | undefined,
  currentStatus?: string,
  closedStatuses: string[] = ["Closed", "Cancelled", "Completed"],
): boolean {
  if (!dueDate) return false;
  if (currentStatus && closedStatuses.includes(currentStatus)) return false;
  return new Date(dueDate) < new Date();
}

/** Convert a date input value to ISO with noon time: "2026-03-25T12:00:00Z" */
export function toISODateNoon(date: string): string {
  return `${date}T12:00:00Z`;
}

/** Today's date as ISO string (YYYY-MM-DD) */
export function todayISO(): string {
  return new Date().toISOString().split("T")[0];
}

/** Date N months from today as ISO string (YYYY-MM-DD) */
export function defaultDueDate(monthsFromNow: number = 1): string {
  const date = new Date();
  date.setMonth(date.getMonth() + monthsFromNow);
  return date.toISOString().split("T")[0];
}
