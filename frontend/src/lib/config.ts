/**
 * Configuration centralisée de l'application
 *
 * Lit les variables d'environnement avec des fallbacks appropriés
 * pour le développement local.
 */

// URL de base de l'API backend
// Fallback: http://127.0.0.1:8080 pour le développement local
const getApiUrl = (): string => {
  // En environnement serveur (SSR), import.meta.env peut ne pas être disponible
  if (typeof import.meta !== "undefined" && import.meta.env) {
    return import.meta.env.PUBLIC_API_URL || "http://127.0.0.1:8080";
  }
  return "http://127.0.0.1:8080";
};

export const API_URL = getApiUrl();

// Helper pour construire les URLs des endpoints API
export const apiEndpoint = (path: string): string => {
  // S'assurer que le path commence par /
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `${API_URL}${normalizedPath}`;
};

// Exemples d'utilisation:
// apiEndpoint('/api/v1/auth/login') => 'http://127.0.0.1:8080/api/v1/auth/login'
// apiEndpoint('api/v1/buildings') => 'http://127.0.0.1:8080/api/v1/buildings'
