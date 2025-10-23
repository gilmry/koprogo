import { get } from "svelte/store";
import { locale } from "svelte-i18n";

/**
 * API base URL - from environment or default to localhost
 */
const API_BASE_URL =
  import.meta.env.PUBLIC_API_URL || "http://localhost:8080/api/v1";

/**
 * Get current language code for API headers
 */
function getCurrentLanguage(): string {
  const currentLocale = get(locale);
  return currentLocale || "nl"; // Default to Dutch
}

/**
 * Get common headers with authentication and language
 */
function getHeaders(
  additionalHeaders: Record<string, string> = {},
): HeadersInit {
  const token = localStorage.getItem("auth_token");

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    "Accept-Language": getCurrentLanguage(),
    ...additionalHeaders,
  };

  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  return headers;
}

/**
 * Enhanced fetch with automatic language headers and error handling
 */
export async function apiFetch<T = any>(
  endpoint: string,
  options: RequestInit = {},
): Promise<T> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}${endpoint}`;

  const response = await fetch(url, {
    ...options,
    headers: getHeaders(options.headers as Record<string, string>),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(error || `API Error: ${response.status}`);
  }

  return response.json();
}

/**
 * API helper methods
 */
export const api = {
  /**
   * GET request
   */
  get: <T = any>(endpoint: string, options?: RequestInit): Promise<T> => {
    return apiFetch<T>(endpoint, { ...options, method: "GET" });
  },

  /**
   * POST request
   */
  post: <T = any>(
    endpoint: string,
    data?: any,
    options?: RequestInit,
  ): Promise<T> => {
    return apiFetch<T>(endpoint, {
      ...options,
      method: "POST",
      body: data ? JSON.stringify(data) : undefined,
    });
  },

  /**
   * PUT request
   */
  put: <T = any>(
    endpoint: string,
    data?: any,
    options?: RequestInit,
  ): Promise<T> => {
    return apiFetch<T>(endpoint, {
      ...options,
      method: "PUT",
      body: data ? JSON.stringify(data) : undefined,
    });
  },

  /**
   * DELETE request
   */
  delete: <T = any>(endpoint: string, options?: RequestInit): Promise<T> => {
    return apiFetch<T>(endpoint, { ...options, method: "DELETE" });
  },

  /**
   * Download file (e.g., PDF, Excel reports)
   */
  download: async (endpoint: string, filename: string): Promise<void> => {
    const url = endpoint.startsWith("http")
      ? endpoint
      : `${API_BASE_URL}${endpoint}`;

    const response = await fetch(url, {
      headers: getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Download failed: ${response.status}`);
    }

    const blob = await response.blob();
    const downloadUrl = window.URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = downloadUrl;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(downloadUrl);
  },
};

/**
 * Example usage:
 *
 * // GET request
 * const buildings = await api.get('/buildings');
 *
 * // POST request
 * const newBuilding = await api.post('/buildings', {
 *   name: 'My Building',
 *   address: '123 Main St'
 * });
 *
 * // Download PCN report
 * await api.download('/pcn/export/pdf/building-id', 'rapport-pcn.pdf');
 */
