//! DTOs du registre de modules — Story 5.1 (#585).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Réponse de `GET /acps/{id}/modules`.
///
/// La forme est **imposée par la moitié frontend déjà livrée**
/// (`frontend/src/lib/api/modules.ts`, `EnabledModulesResponseDto`), qui a
/// été écrite en attendant cette story. `acp_id` est en `snake_case` et les
/// modules sont des chaînes : c'est ce que le client lit aujourd'hui.
/// Une fois ce handler au schéma OpenAPI, le frontend peut remplacer son DTO
/// écrit à la main par le type généré — la dette est notée en tête de son
/// fichier.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnabledModulesResponseDto {
    pub acp_id: String,
    pub modules: Vec<String>,
}
