// Story 2.3 — ACPs API client (lecture pour ContextBanner).
//
// Endpoint backend exposé dans
// `backend/src/infrastructure/web/handlers/acp_handlers.rs` :
// - `GET /acps/{id}` (scope guard + filtrage rôle)
//
// Le `AcpResponseDto` expose `name` + `organization_id` (cabinet syndic
// parent — `null` quand ACP auto-gérée, ADR-0010). Pas de `organization_name`
// par défaut : si le composant a besoin du nom du cabinet, il fait un fetch
// secondaire `/organizations` (catché silencieusement si 403 non-superadmin).

import { api } from "../api";

/**
 * Réponse `/acps/{id}` — aligné avec `backend::application::dto::acp::AcpResponseDto`.
 */
export interface AcpResponseDto {
  id: string;
  organization_id: string | null;
  name: string;
  slug: string;
  legal_status: string;
  bce_number: string | null;
  address_street: string;
  address_postal_code: string;
  address_city: string;
  created_at: string;
  updated_at: string;
}

/**
 * Récupère un ACP par ID. Lance une erreur si 403 / 404.
 */
export async function getAcp(id: string): Promise<AcpResponseDto> {
  return api.get<AcpResponseDto>(`/acps/${encodeURIComponent(id)}`);
}

/**
 * Liste les ACPs visibles pour l'utilisateur connecté (filtré par rôle).
 */
export async function listAcps(): Promise<AcpResponseDto[]> {
  return api.get<AcpResponseDto[]>("/acps");
}

export interface CreateAcpDto {
  organization_id: string | null;
  name: string;
  address_street: string;
  address_postal_code: string;
  address_city: string;
  bce_number?: string | null;
}

/**
 * Crée une ACP (admin seulement).
 */
export async function createAcp(dto: CreateAcpDto): Promise<AcpResponseDto> {
  return api.post<AcpResponseDto>("/acps", dto);
}
