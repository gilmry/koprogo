// Story 2.2 — Buildings API client (search/list pour BuildingSelector).
//
// Aligne avec les endpoints backend exposés dans
// `backend/src/infrastructure/web/handlers/building_handlers.rs` :
// - `GET /buildings?page=&per_page=` (list paginé scope-filtré par rôle).
//
// Pas de full-text search backend dédié à ce jour (cf. Story 2.5 deep-links
// pour un endpoint search `?q=` premier-niveau). Le composant
// `BuildingSelector` filtre côté client la première page paginée — acceptable
// tant qu'une organisation reste ≤ 500 buildings (cf. AC @edge debounce
// 150ms + pagination 20). Au-delà, Story 2.5 introduira `?search=<q>`.

import { api } from "../api";
import type { Building } from "../types";

/**
 * Réponse paginée typée pour `/buildings`. Aligné avec
 * `backend::application::dto::pagination::PageResponse<T>`.
 */
export interface PaginatedBuildings {
  data: Building[];
  pagination: {
    page: number;
    per_page: number;
    total_items: number;
    total_pages: number;
  };
}

/**
 * Liste paginée des buildings dans le scope du user courant.
 *
 * Le filtrage rôle-based est appliqué côté backend (ADR-0010 §3.3) :
 * - syndic/accountant/admin : tous les buildings de leur org.
 * - owner : buildings où l'user est lié via `unit_owners` (read-only).
 * - superadmin : tous les buildings (multi-org).
 */
export async function listBuildings(
  page = 1,
  perPage = 20,
): Promise<PaginatedBuildings> {
  const params = new URLSearchParams({
    page: String(page),
    per_page: String(perPage),
  });
  return api.get<PaginatedBuildings>(`/buildings?${params.toString()}`);
}

/**
 * Recherche côté client (filtre case-insensitive sur `name` + `city`).
 *
 * Fetch la première page (per_page max raisonnable = 100) puis filtre en
 * mémoire. Pour un cabinet > 100 buildings, il faudra introduire le param
 * backend `?search=<q>` (Story 2.5 deep-links). Le composant garde un
 * debounce 150ms pour amortir le coût réseau de l'unique fetch.
 *
 * @param query Texte saisi (peut être vide → retourne premiers résultats).
 * @param limit Nombre maximum de résultats à retourner (cap 20 par défaut).
 */
export async function searchBuildings(
  query: string,
  limit = 20,
): Promise<Building[]> {
  const page = await listBuildings(1, Math.max(100, limit));
  const q = query.trim().toLowerCase();
  if (q === "") {
    return page.data.slice(0, limit);
  }
  return page.data
    .filter((b) => {
      const name = (b.name ?? "").toLowerCase();
      const city = (b.city ?? "").toLowerCase();
      const address = (b.address ?? "").toLowerCase();
      return name.includes(q) || city.includes(q) || address.includes(q);
    })
    .slice(0, limit);
}

/**
 * Récupère un building par ID — utilisé après un deep-link pour hydrater
 * le scope. Lance une erreur si le backend retourne 403 / 404 (capturée
 * en amont pour set `scopeError`).
 */
export async function getBuilding(id: string): Promise<Building> {
  return api.get<Building>(`/buildings/${encodeURIComponent(id)}`);
}
