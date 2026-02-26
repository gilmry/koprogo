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
    let errorMessage = `API Error: ${response.status}`;
    try {
      const errorText = await response.text();
      try {
        const errorData = JSON.parse(errorText);
        errorMessage = errorData.error || errorMessage;
      } catch {
        if (errorText) errorMessage = errorText;
      }
    } catch {
      // Body unreadable, keep default message
    }
    throw new Error(errorMessage);
  }

  // Handle 204 No Content responses (empty body)
  if (response.status === 204) {
    return undefined as T;
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

    // No need to parse response for DELETE (typically 204 No Content)
  },
};

export { buildHeaders };

/**
 * Get the owner record for the currently authenticated user.
 * Uses the JWT organization_id to find the correct owner in multi-org contexts.
 * Returns null if no owner record is linked to this user.
 */
export async function getMyOwner(): Promise<{
  id: string;
  organization_id: string;
  first_name: string;
  last_name: string;
  email: string;
} | null> {
  try {
    return await api.get("/owners/me");
  } catch {
    return null;
  }
}

/**
 * Return the absolute URL for the metrics endpoint.
 */
export function getMetricsUrl(): string {
  const metricsBase = API_BASE_URL.replace(/\/api\/v1\/?$/, "");
  return `${metricsBase}/metrics`;
}

/**
 * Call for Funds API functions
 */
export const callForFundsApi = {
  /**
   * List all calls for funds (optionally filtered by building)
   */
  async list(buildingId?: string) {
    const url = buildingId
      ? `/call-for-funds?building_id=${buildingId}`
      : "/call-for-funds";
    return api.get(url);
  },

  /**
   * Get a specific call for funds by ID
   */
  async getById(id: string) {
    return api.get(`/call-for-funds/${id}`);
  },

  /**
   * Create a new call for funds
   */
  async create(data: {
    building_id: string;
    title: string;
    description: string;
    total_amount: number;
    contribution_type: string;
    call_date: string;
    due_date: string;
    account_code?: string;
  }) {
    return api.post("/call-for-funds", data);
  },

  /**
   * Send a call for funds (generates individual contributions)
   */
  async send(id: string) {
    return api.post(`/call-for-funds/${id}/send`, {});
  },

  /**
   * Cancel a call for funds
   */
  async cancel(id: string) {
    return api.put(`/call-for-funds/${id}/cancel`, {});
  },

  /**
   * Delete a draft call for funds
   */
  async delete(id: string) {
    return api.delete(`/call-for-funds/${id}`);
  },

  /**
   * Get overdue calls for funds
   */
  async getOverdue() {
    return api.get("/call-for-funds/overdue");
  },
};

/**
 * Owner Contributions API functions
 */
export const ownerContributionsApi = {
  /**
   * List all owner contributions (with optional filters)
   */
  async list(filters?: {
    owner_id?: string;
    building_id?: string;
    status?: string;
  }) {
    let url = "/owner-contributions";
    if (filters) {
      const params = new URLSearchParams();
      if (filters.owner_id) params.append("owner_id", filters.owner_id);
      if (filters.building_id)
        params.append("building_id", filters.building_id);
      if (filters.status) params.append("status", filters.status);
      if (params.toString()) url += `?${params.toString()}`;
    }
    return api.get(url);
  },

  /**
   * Get a specific contribution by ID
   */
  async getById(id: string) {
    return api.get(`/owner-contributions/${id}`);
  },

  /**
   * Create a manual owner contribution
   */
  async create(data: {
    owner_id: string;
    unit_id?: string;
    description: string;
    amount: number;
    contribution_type: string;
    contribution_date: string;
    account_code?: string;
  }) {
    return api.post("/owner-contributions", data);
  },

  /**
   * Mark a contribution as paid
   */
  async markAsPaid(
    id: string,
    data: {
      payment_date: string;
      payment_method?: string;
      payment_reference?: string;
    },
  ) {
    return api.put(`/owner-contributions/${id}/mark-paid`, data);
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
 *
 * // Call for Funds
 * const calls = await callForFundsApi.list('building-id');
 * await callForFundsApi.send('call-id');
 */
