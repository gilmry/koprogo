/**
 * Case-insensitive multi-field text search.
 * Returns items where any of the specified fields contain the query string.
 *
 * @example
 * $: filtered = multiFieldSearch(tickets, searchQuery, ['title', 'description', 'requester_name']);
 */
export function multiFieldSearch<T extends Record<string, any>>(
  items: T[],
  query: string,
  fields: (keyof T)[],
): T[] {
  if (!query || !query.trim()) return items;
  const q = query.toLowerCase();
  return items.filter((item) =>
    fields.some((field) => {
      const value = item[field];
      return typeof value === "string" && value.toLowerCase().includes(q);
    }),
  );
}

/**
 * Apply enum/status filters to items. Filters with value "all" are skipped.
 *
 * @example
 * $: filtered = applyFilters(tickets, {
 *   status: statusFilter,
 *   priority: priorityFilter,
 *   category: categoryFilter,
 * });
 */
export function applyFilters<T extends Record<string, any>>(
  items: T[],
  filters: Record<string, string>,
): T[] {
  return items.filter((item) =>
    Object.entries(filters).every(([field, value]) => {
      if (value === "all" || value === "") return true;
      return item[field] === value;
    }),
  );
}

/**
 * Combine text search and enum filters in one call.
 *
 * @example
 * $: filtered = filterAndSearch(tickets, searchQuery, ['title', 'description'], {
 *   status: statusFilter,
 *   priority: priorityFilter,
 * });
 */
export function filterAndSearch<T extends Record<string, any>>(
  items: T[],
  query: string,
  searchFields: (keyof T)[],
  filters: Record<string, string>,
): T[] {
  return multiFieldSearch(applyFilters(items, filters), query, searchFields);
}
