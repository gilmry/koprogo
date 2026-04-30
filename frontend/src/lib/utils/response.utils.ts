/**
 * Extract an array from various API response shapes.
 * Handles: raw array, { data: [...] }, { [key]: [...] }.
 *
 * @example
 * const buildings = extractArray(response); // handles response.data, response.buildings, etc.
 * const buildings = extractArray(response, 'buildings');
 */
export function extractArray<T>(response: any, key?: string): T[] {
  if (Array.isArray(response)) return response;
  if (key && response && Array.isArray(response[key])) return response[key];
  if (response && Array.isArray(response.data)) return response.data;
  return [];
}

/**
 * Extract paginated response with metadata.
 */
export function extractPaginated<T>(response: any): {
  data: T[];
  pagination: {
    current_page: number;
    per_page: number;
    total_items: number;
    total_pages: number;
    has_next: boolean;
    has_previous: boolean;
  };
} {
  return {
    data: extractArray<T>(response),
    pagination: response?.pagination ?? {
      current_page: 1,
      per_page: 20,
      total_items: 0,
      total_pages: 0,
      has_next: false,
      has_previous: false,
    },
  };
}
