import { get } from "svelte/store";
import { locale } from "svelte-i18n";
import type { Document, DocumentUploadPayload } from "./types";

/**
 * API base URL - from runtime config, build-time env, or default to localhost
 * Priority: window.__ENV__.API_URL > import.meta.env.PUBLIC_API_URL > default
 */
export const API_BASE_URL =
  (typeof window !== "undefined" && (window as any).__ENV__?.API_URL) ||
  import.meta.env.PUBLIC_API_URL ||
  "http://localhost:8080/api/v1";

/**
 * Get current language code for API headers
 */
function getCurrentLanguage(): string {
  const currentLocale = get(locale);
  return currentLocale || "nl"; // Default to Dutch
}

function buildHeaders(
  additional?: HeadersInit,
  includeJsonContentType: boolean = true,
): Headers {
  const headers = new Headers();

  if (includeJsonContentType) {
    headers.set("Content-Type", "application/json");
  }

  headers.set("Accept-Language", getCurrentLanguage());

  const token =
    typeof window !== "undefined"
      ? localStorage.getItem("koprogo_token")
      : null;

  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  if (additional) {
    const extra = new Headers(additional);
    extra.forEach((value, key) => {
      if (value === undefined || value === null) {
        headers.delete(key);
      } else {
        headers.set(key, value);
      }
    });
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
    headers: buildHeaders(options.headers as HeadersInit | undefined, true),
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
      headers: buildHeaders(undefined, false),
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

  uploadDocument: async (payload: DocumentUploadPayload): Promise<Document> => {
    const url = `${API_BASE_URL}/documents`;
    const formData = new FormData();
    formData.append("file", payload.file);
    formData.append("building_id", payload.buildingId);
    formData.append("document_type", payload.documentType);
    formData.append("title", payload.title);
    if (payload.description) {
      formData.append("description", payload.description);
    }
    formData.append("uploaded_by", payload.uploadedBy);

    const response = await fetch(url, {
      method: "POST",
      headers: buildHeaders(undefined, false),
      body: formData,
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || `Upload failed: ${response.status}`);
    }

    return response.json();
  },

  deleteDocument: async (id: string): Promise<void> => {
    const url = `${API_BASE_URL}/documents/${id}`;
    const response = await fetch(url, {
      method: "DELETE",
      headers: buildHeaders(),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error || `Delete failed: ${response.status}`);
    }
  },
};

export { buildHeaders };

/**
 * Return the absolute URL for the metrics endpoint.
 */
export function getMetricsUrl(): string {
  const metricsBase = API_BASE_URL.replace(/\/api\/v1\/?$/, "");
  return `${metricsBase}/metrics`;
}

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
