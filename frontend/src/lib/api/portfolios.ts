// Story 2.2 — Portfolios API client (consumed by BuildingSelector for
// favoris-first ordering + portefeuilles équipe section).
//
// Aligne avec les endpoints backend Story 2.1
// (cf. `backend/src/infrastructure/web/handlers/portfolio_handlers.rs`) :
// - `POST /portfolios`
// - `GET /portfolios` (owned + shared)
// - `POST /portfolios/{id}/buildings` (is_favorite: bool)
// - `DELETE /portfolios/{id}/buildings/{building_id}`

import { api } from "../api";

export interface PortfolioResponseDto {
  id: string;
  owner_user_id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface PortfolioBuildingResponseDto {
  portfolio_id: string;
  building_id: string;
  is_favorite: boolean;
}

export interface CreatePortfolioDto {
  name: string;
  description?: string | null;
}

export interface AddBuildingDto {
  building_id: string;
  is_favorite?: boolean;
}

/**
 * Liste les portfolios du user courant (owned + ceux partagés avec lui).
 *
 * Owner role : devrait retourner [] (l'API exclut les owners — pas de
 * portefeuilles pour les copropriétaires, cf. Story 2.1 / ADR-0011).
 */
export async function listPortfolios(): Promise<PortfolioResponseDto[]> {
  return api.get<PortfolioResponseDto[]>("/portfolios");
}

/**
 * Liste des buildings d'un portfolio (triés favoris d'abord — cf. backend
 * `list_portfolio_buildings` order by `is_favorite DESC, added_at DESC`).
 */
export async function listPortfolioBuildings(
  portfolioId: string,
): Promise<PortfolioBuildingResponseDto[]> {
  return api.get<PortfolioBuildingResponseDto[]>(
    `/portfolios/${encodeURIComponent(portfolioId)}/buildings`,
  );
}

/**
 * Toggle favorite status d'un building dans un portfolio.
 *
 * Le backend Story 2.1 expose l'ajout via POST {building_id, is_favorite}.
 * Pour basculer un is_favorite existant, le pattern actuel est :
 *   1. Si déjà dans le portfolio, on POST à nouveau (idempotent en upsert)
 *   2. Sinon on POST avec is_favorite=true
 * (Story 2.5 introduira `PUT /portfolios/{id}/buildings/{bid}` dédié.)
 */
export async function toggleFavorite(
  portfolioId: string,
  buildingId: string,
  isFavorite: boolean,
): Promise<PortfolioBuildingResponseDto> {
  const body: AddBuildingDto = {
    building_id: buildingId,
    is_favorite: isFavorite,
  };
  return api.post<PortfolioBuildingResponseDto>(
    `/portfolios/${encodeURIComponent(portfolioId)}/buildings`,
    body,
  );
}

/**
 * Crée un portfolio (le user courant devient owner_user_id côté backend).
 */
export async function createPortfolio(
  dto: CreatePortfolioDto,
): Promise<PortfolioResponseDto> {
  return api.post<PortfolioResponseDto>("/portfolios", dto);
}
