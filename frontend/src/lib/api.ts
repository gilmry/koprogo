import { locale } from "svelte-i18n";
import { get } from "svelte/store";
import { toast } from "../stores/toast";
import { authStore } from "../stores/auth";
import { getAccessToken, clearAccessToken } from "./accessToken";
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

  // WP-FE1 : access token en mémoire (jamais localStorage).
  const token = getAccessToken();

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
 * Optional flags for `apiFetch`. `silent: true` suppresses the auto-toast on
 * 4xx (used by best-effort reads like `tryGetOrganizationName` where a 403
 * is an expected, non-actionable degradation — not a user-facing error).
 */
export interface ApiFetchOptions extends RequestInit {
  silent?: boolean;
}

/**
 * Le détail d'une réponse d'erreur, quand il est présentable à l'écran.
 *
 * Le backend sert `details` tantôt en chaîne (erreur de désérialisation
 * serde), tantôt en objet typé (`{code, ...}` des erreurs métier). On
 * n'affiche que la première forme : un objet brut ne dirait rien à un
 * utilisateur, et les erreurs typées ont déjà leurs gestionnaires dédiés
 * dans `lib/utils/conformity.ts` et `lib/utils/meetingCompletion.ts`.
 */
function detailsPresentables(body: any): string | undefined {
  if (typeof body?.details === "string") return body.details;
  if (typeof body?.details?.message === "string") return body.details.message;
  return undefined;
}

/**
 * Erreur d'API qui **conserve le corps de la réponse**.
 *
 * Le serveur répond aux 400 avec une précision remarquable :
 *
 *     {"error": "Invalid request body",
 *      "details": "Json deserialize error: missing field `acp_id` at line 1 column 192"}
 *
 * `apiFetch` ne retenait que `error` et levait une `Error` **nue**. Tout le
 * reste — le nom du champ fautif, sa position, le code HTTP — était perdu
 * avant d'atteindre le premier appelant.
 *
 * Coût mesuré : en cinq recettes navigateur, trois actions d'écriture ont
 * échoué en silence — `acp_id` à la création d'immeuble, `recipient_owner_ids`
 * à l'envoi de convocation, `total_voting_power` à la clôture d'un vote. À
 * chaque fois le serveur nommait le champ, et à chaque fois l'écran affichait
 * « Invalid request body ». Il a fallu lire le code pour diagnostiquer ce que
 * le premier utilisateur venu aurait vu.
 *
 * Coût invisible, plus gênant : `lib/utils/conformity.ts` et
 * `lib/utils/meetingCompletion.ts` savent tous deux extraire `details` et
 * **ne pouvaient jamais correspondre**, faute de trouver la propriété sur une
 * `Error` nue. Deux fonctionnalités mortes sous des tests verts, parce que
 * ces tests fabriquaient l'objet d'erreur à la main sans jamais exercer
 * `apiFetch`.
 *
 * Voir l'issue #782.
 */
export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly details?: unknown,
    readonly body?: unknown,
  ) {
    super(message);
    this.name = "ApiError";
  }

  /** Le détail servi par le serveur, quand il est présentable à l'écran. */
  get detailsText(): string | undefined {
    if (typeof this.details === "string") return this.details;
    if (this.details && typeof this.details === "object") {
      const message = (this.details as Record<string, unknown>).message;
      if (typeof message === "string") return message;
    }
    return undefined;
  }
}

/**
 * Enhanced fetch with automatic language headers and error handling
 */
export async function apiFetch<T = any>(
  endpoint: string,
  options: ApiFetchOptions = {},
): Promise<T> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}${endpoint}`;

  // #550 strate 2 : course init() vs onMount des composants.
  //
  // Symptôme observé en live console : composants (NotificationBell,
  // listes diverses, AdminDashboard) mountent et appellent `api.get()`
  // AVANT que `authStore.init()` n'ait fini son silent-refresh → pas de
  // token en mémoire → "Missing authorization header" 401.
  //
  // Si pas de token + pas un endpoint /auth/*, on attend le refresh
  // in-flight (mémoïsé via `inflightRefresh` dans authStore — un seul
  // POST /auth/refresh partagé entre tous les callers concurrents).
  // Coût : 1 extra round trip pour visiteurs vraiment unauth (refresh
  // fail rapide). Au pire : 1 POST 401, puis clearSession ; rien de
  // cassant.
  if (!getAccessToken() && !endpoint.startsWith("/auth/")) {
    await authStore.refreshAccessToken();
  }

  const response = await fetch(url, {
    ...options,
    headers: buildHeaders(options.headers as HeadersInit | undefined, true),
  });

  if (!response.ok) {
    let errorMessage = `API Error: ${response.status}`;
    // Le corps parsé est CONSERVÉ jusqu'au `throw` : c'est lui qui porte
    // `details`, et c'est sa perte ici qui rendait les 400 indéchiffrables.
    let errorBody: any;
    try {
      const errorText = await response.text();
      try {
        errorBody = JSON.parse(errorText);
        errorMessage = errorBody.error || errorBody.message || errorMessage;
      } catch {
        if (errorText) errorMessage = errorText;
      }
    } catch {
      // Body unreadable, keep default message
    }

    // Mask raw DB / internal errors — never expose Postgres errors to end users
    const raw = errorMessage.toLowerCase();
    if (
      raw.includes("database error") ||
      raw.includes("fkey") ||
      raw.includes("constraint") ||
      raw.includes("sqlx") ||
      raw.includes("postgres")
    ) {
      errorMessage = `Une erreur est survenue (${response.status}). Merci de réessayer ou de contacter le support.`;
    } else {
      // STORY-P7-401: map well-known English backend error strings to
      // localized user-facing messages. Add entries here as they surface.
      const knownErrors: Record<string, string> = {
        "no owner record linked to this user":
          "Aucun compte copropriétaire associé à ce profil.",
        "invalid email or password": "Email ou mot de passe invalide.",
        "token expired": "Session expirée. Veuillez vous reconnecter.",
        "invalid token": "Session expirée. Veuillez vous reconnecter.",
        "not found": "Ressource introuvable.",
        unauthorized: "Authentification requise.",
        forbidden: "Accès refusé.",
      };
      const mapped = knownErrors[errorMessage.toLowerCase().trim()];
      if (mapped) errorMessage = mapped;
    }

    let toastEmis = false;
    // Toast automatique selon le code HTTP — sauf si `silent: true` (best-effort
    // reads où un 4xx est une dégradation attendue, pas une erreur utilisateur).
    if (!options.silent) {
      if (response.status === 429) {
        toast.error("Trop de tentatives. Réessayez dans 15 minutes.");
      } else if (response.status >= 500) {
        toast.error("Erreur serveur. Veuillez réessayer.");
      } else if (response.status === 401) {
        // Clear stale token and dedupe toast across parallel 401s
        if (typeof window !== "undefined") {
          const hadToken = getAccessToken() !== null;
          clearAccessToken();
          if (hadToken && !(window as any).__koprogo_session_expired_shown__) {
            (window as any).__koprogo_session_expired_shown__ = true;
            toast.warning("Session expirée. Veuillez vous reconnecter.");
            setTimeout(() => {
              (window as any).__koprogo_session_expired_shown__ = false;
            }, 5000);
          }
        }
      } else if (response.status === 403) {
        toast.warning(
          "Accès refusé. Vous n'avez pas les permissions nécessaires.",
        );
      } else if (response.status >= 400) {
        // Le détail du serveur en seconde ligne : c'est lui qui nomme le
        // champ fautif. `toastEmis` évite le doublon avec
        // `withErrorHandling`, qui émettait un second toast au libellé
        // différent — la déduplication du store ne les fusionnait pas.
        toast.error(errorMessage, 7000, detailsPresentables(errorBody));
        toastEmis = true;
      }
    } else if (response.status === 401 && typeof window !== "undefined") {
      // Silent 401 : on clear quand même le token périmé (cohérence session)
      // mais pas de toast — l'appelant gère le `null` retourné.
      clearAccessToken();
    }

    throw new ApiError(errorMessage, response.status, errorBody?.details, {
      ...errorBody,
      // L'appelant sait ainsi qu'un toast est déjà parti et n'en ajoute pas.
      __toastEmis: toastEmis,
    });
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
  get: <T = any>(endpoint: string, options?: ApiFetchOptions): Promise<T> => {
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
