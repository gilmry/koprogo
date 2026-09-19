//! DTOs du registre de modules — Story 5.1 (#585).

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::entities::Module;

/// Réponse de `GET /acps/{id}/modules`.
///
/// La forme est **imposée par la moitié frontend déjà livrée**
/// (`frontend/src/lib/api/modules.ts`, `EnabledModulesResponseDto`), qui a
/// été écrite en attendant cette story : `acp_id` en `snake_case`.
///
/// `modules` porte `Module` et non `String`. Le fil est identique — `Module`
/// se sérialise en `snake_case` — mais le **schéma** change : il énumère les
/// sept valeurs au lieu d'annoncer « une chaîne quelconque ». C'est ce qui
/// permet à `api.d.ts` d'être plus fort que la liste écrite à la main côté
/// frontend, au lieu d'être plus faible qu'elle.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EnabledModulesResponseDto {
    pub acp_id: String,
    pub modules: Vec<Module>,
}
