// Story 2.2 — Buildings API client (search/list pour BuildingSelector).
//
// Aligne avec les endpoints backend exposés dans
// `backend/src/infrastructure/web/handlers/building_handlers.rs` :
// - `GET /buildings?page=&per_page=&search=` (list paginé scope-filtré par
//   rôle, `search` = ILIKE name/city/address côté serveur).

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
 * Recherche côté backend (`?search=`, ILIKE name/city/address) — un filtrage
 * client sur les 100 premiers buildings (par created_at DESC) ratait les
 * buildings récents dès que >100 buildings existaient dans le scope visible
 * (constaté en E2E CI en scope superadmin, toutes orgs confondues).
 *
 * @param query Texte saisi (peut être vide → retourne premiers résultats).
 * @param limit Nombre maximum de résultats à retourner (cap 20 par défaut).
 */
export async function searchBuildings(
  query: string,
  limit = 20,
): Promise<Building[]> {
  const q = query.trim();
  const params = new URLSearchParams({
    page: "1",
    per_page: String(limit),
  });
  if (q !== "") {
    params.set("search", q);
  }
  const page = await api.get<PaginatedBuildings>(
    `/buildings?${params.toString()}`,
  );
  return page.data.slice(0, limit);
}

/**
 * Récupère un building par ID — utilisé après un deep-link pour hydrater
 * le scope. Lance une erreur si le backend retourne 403 / 404 (capturée
 * en amont pour set `scopeError`).
 */
export async function getBuilding(id: string): Promise<Building> {
  return api.get<Building>(`/buildings/${encodeURIComponent(id)}`);
}
