/**
 * Configuration centralisée de l'application
 *
 * Lit les variables d'environnement avec des fallbacks appropriés
 * pour le développement local.
 */

// Extend window interface to include __ENV__
declare global {
  interface Window {
    __ENV__?: {
      API_URL?: string;
    };
  }
}

// URL de base de l'API backend
// Priorité: window.__ENV__.API_URL (runtime) > import.meta.env.PUBLIC_API_URL (build-time) > fallback local
const getApiUrl = (): string => {
  // En environnement browser, utiliser window.__ENV__ si disponible
  if (typeof window !== "undefined" && window.__ENV__?.API_URL) {
    return window.__ENV__.API_URL;
  }

  // Sinon, utiliser la variable d'environnement de build
  if (typeof import.meta !== "undefined" && import.meta.env) {
    return import.meta.env.PUBLIC_API_URL || "http://127.0.0.1:8080/api/v1";
  }

  return "http://127.0.0.1:8080/api/v1";
};

export const API_URL = getApiUrl();

// Helper pour construire les URLs des endpoints API
export const apiEndpoint = (path: string): string => {
  // S'assurer que le path commence par /
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  // Toujours récupérer l'URL la plus récente au moment de l'appel
  const apiUrl = getApiUrl();
  return `${apiUrl}${normalizedPath}`;
};

// Exemples d'utilisation:
// apiEndpoint('/auth/login') => 'http://127.0.0.1:8080/api/v1/auth/login'
// apiEndpoint('/buildings') => 'http://127.0.0.1:8080/api/v1/buildings'
