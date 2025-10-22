/**
 * Configuration pour les tests E2E
 */

// URL de l'API backend pour les tests
// Peut être modifié via process.env.API_URL
export const API_URL = process.env.API_URL || 'http://127.0.0.1:8080';

// Helper pour construire des endpoints API
export const apiEndpoint = (path: string): string => {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${API_URL}${normalizedPath}`;
};
