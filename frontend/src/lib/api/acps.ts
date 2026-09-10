// Client API des ACP (Association des Copropriétaires).
//
// Endpoints backend — `backend/src/infrastructure/web/handlers/acp_handlers.rs` :
//   POST   /acps          création      (superadmin ou admin de cabinet)
//   GET    /acps          liste         (filtrée par rôle via `list_scope`)
//   GET    /acps/{id}     détail        (scope guard)
//   PUT    /acps/{id}     mise à jour   (superadmin ou admin de cabinet)
//   DELETE /acps/{id}     suppression   (superadmin ou admin de cabinet)
//
// Les types viennent désormais de `types/api.d.ts`, généré depuis la spec
// OpenAPI. Ils étaient auparavant recopiés à la main dans ce fichier et
// avaient déjà divergé : `AcpResponseDto` et `CreateAcpDto` omettaient tous
// deux `total_tantiemes`, le dénominateur de l'acte de base (Art. 3.84 CC).
// La cause était que `acp_handlers` n'était pas déclaré dans
// `infrastructure/openapi.rs` — les annotations `#[utoipa::path]` existaient
// mais utoipa ne collecte que ce qui est listé, donc rien n'était généré et le
// gate anti-drift ne voyait rien (cf. #732).

import { api } from "../api";
import type { components } from "../../types/api";

export type AcpResponseDto = components["schemas"]["AcpResponseDto"];
export type CreateAcpDto = components["schemas"]["CreateAcpDto"];
export type UpdateAcpDto = components["schemas"]["UpdateAcpDto"];

/**
 * Récupère une ACP par ID. Lance une erreur si 403 / 404.
 */
export async function getAcp(id: string): Promise<AcpResponseDto> {
  return api.get<AcpResponseDto>(`/acps/${encodeURIComponent(id)}`);
}

/**
 * Liste les ACP visibles pour l'utilisateur connecté (filtré par rôle).
 */
export async function listAcps(): Promise<AcpResponseDto[]> {
  return api.get<AcpResponseDto[]>("/acps");
}

/**
 * Une ACP avec ses métriques agrégées.
 *
 * Le type n'est pas tiré de `components["schemas"]` parce que le DTO backend
 * emploie `#[serde(flatten)]` sur l'ACP : `utoipa` ne le décrit pas comme une
 * intersection dans le schéma généré. Le décrire ici, à la main, avec ce
 * commentaire, vaut mieux qu'un `any` — mais c'est une dette à effacer quand
 * le contrat OpenAPI couvrira ce cas.
 */
export interface AcpAvecMetriques extends AcpResponseDto {
  /** Nombre de blocs rattachés à l'ACP. */
  buildings_count: number;
  /** Lots effectivement encodés, tous blocs confondus. */
  units_count: number;
  /** Lots déclarés à l'acte de base, sommés sur les blocs. */
  declared_units_total: number;
  /**
   * Somme exacte des quotités, servie en CHAÎNE.
   *
   * Une quotité est juridiquement opposable : la convertir en `number` pour
   * l'afficher introduirait une erreur de représentation sur une valeur
   * exacte. On la compare et on l'affiche telle quelle.
   */
  quota_sum: string;
}

/**
 * Les ACP du périmètre, avec leurs métriques.
 *
 * Route distincte de `listAcps` : les métriques coûtent quatre sous-requêtes
 * par ligne côté serveur, et un sélecteur qui n'a besoin que des noms ne doit
 * pas les payer.
 */
export async function listAcpsWithMetrics(): Promise<AcpAvecMetriques[]> {
  return api.get<AcpAvecMetriques[]>("/acps/with-metrics");
}

/**
 * Crée une ACP (superadmin, ou admin dans son propre cabinet).
 */
export async function createAcp(dto: CreateAcpDto): Promise<AcpResponseDto> {
  return api.post<AcpResponseDto>("/acps", dto);
}

/**
 * Met à jour une ACP.
 *
 * `PUT` **full-state** : `name` et les trois champs d'adresse sont
 * obligatoires côté backend, pas des champs partiels. Le formulaire appelant
 * doit donc être pré-rempli avec les valeurs courantes, sous peine d'écraser
 * ce qu'il n'affiche pas.
 *
 * `organization_id` suit une sémantique à trois états volontaire
 * (`Option<Option<String>>` côté Rust) :
 *   - clé absente  → le rattachement au cabinet est **conservé** ;
 *   - `null`       → l'ACP est **détachée** (devient auto-gérée) ;
 *   - un UUID      → rattachement à ce cabinet.
 * Ne jamais envoyer `null` « par défaut » : ce serait un détachement silencieux.
 */
export async function updateAcp(
  id: string,
  dto: UpdateAcpDto,
): Promise<AcpResponseDto> {
  return api.put<AcpResponseDto>(`/acps/${encodeURIComponent(id)}`, dto);
}

/**
 * Supprime une ACP.
 *
 * Le backend l'appelle « archive » mais il s'agit d'une **suppression
 * définitive** (`DELETE FROM acps`), pas d'un archivage réversible.
 *
 * Renvoie **409 Conflict** si l'ACP porte encore des immeubles — il faut les
 * détacher ou les supprimer d'abord. Ce contrôle est explicite côté use case ;
 * sans lui la contrainte de clé étrangère remontait un 500.
 */
export async function archiveAcp(id: string): Promise<void> {
  return api.delete<void>(`/acps/${encodeURIComponent(id)}`);
}
