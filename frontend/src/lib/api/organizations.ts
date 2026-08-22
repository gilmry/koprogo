// Story 2.3 — Organizations API client (lecture optionnelle pour ContextBanner).
//
// Le backend n'expose PAS `GET /organizations/{id}` à ce jour (seul
// `GET /organizations` superadmin-only existe). Le ContextBanner doit
// donc dégrader gracieusement vers 2 niveaux (`ACP · Immeuble`) quand
// l'utilisateur n'a pas accès à la liste des organizations.
//
// Cette fonction encapsule l'appel `/organizations` + résolution par id.
// Elle est conçue pour être catchée (try/catch) côté composant : un 403
// est attendu pour syndic/accountant et ne doit PAS être une erreur visible.

import { api } from "../api";

/**
 * Subset d'`OrganizationResponse` strict minimum requis par ContextBanner.
 *
 * Aligné avec `backend::infrastructure::web::handlers::organization_handlers::to_response`.
 */
export interface OrganizationSummary {
  id: string;
  name: string;
  slug: string;
}

interface OrganizationListResponse {
  data: OrganizationSummary[];
}

/**
 * Tente de résoudre le nom d'une organization (cabinet syndic) par ID.
 *
 * Retourne `null` si :
 * - l'API renvoie 403 (utilisateur non superadmin),
 * - l'organization n'est pas trouvée dans la liste,
 * - toute autre erreur réseau.
 *
 * Le composant ContextBanner utilise ce comportement comme dégradation
 * gracieuse vers une bannière 2 niveaux (`ACP · Immeuble`) quand le
 * cabinet n'est pas résolvable.
 */
export async function tryGetOrganizationName(
  organizationId: string,
): Promise<string | null> {
  try {
    // `silent: true` → pas de toast "Accès refusé" pour les non-superadmin :
    // c'est un endpoint best-effort utilisé par ContextBanner pour enrichir
    // l'affichage en niveau Cabinet. Un 403 est attendu et déclenche juste
    // la dégradation à 2 niveaux côté composant.
    const response = await api.get<OrganizationListResponse>(
      "/organizations?per_page=1000",
      { silent: true },
    );
    const org = response.data.find((o) => o.id === organizationId);
    return org ? org.name : null;
  } catch {
    return null;
  }
}

// Story S2 (docs/maury/syndic-org-users-endpoint) — client de
// `GET /organizations/{id}/users`, org-scopé (syndic/accountant sur leur
// propre org, superadmin sur n'importe laquelle). Remplace les appels
// `GET /users` (superadmin-only, 403 pour un syndic réel) dans les 3
// wrappers `MagicLinksPage`/`MandatesPage`/`ContractorEvaluationsPage`.

/** Subset de `UserResponse` requis par les sélecteurs syndic (mandataire,
 *  contractor, destinataire magic link). Dupliqué localement dans les 3
 *  wrappers avant cette story — factorisé ici. */
export interface OrganizationUserSummary {
  id: string;
  email: string;
  first_name?: string;
  last_name?: string;
  role: string;
}

interface OrganizationUsersResponse {
  data: OrganizationUserSummary[];
}

export async function listOrganizationUsers(
  organizationId: string,
): Promise<OrganizationUsersResponse> {
  return api.get<OrganizationUsersResponse>(
    `/organizations/${encodeURIComponent(organizationId)}/users`,
  );
}
