//! Cas d'usage du registre de modules — Story 5.1 (#585), ADR-0015.
//!
//! Les règles d'activation vivent ici, pas dans l'adaptateur SQL ni dans le
//! handler :
//!
//! - **lire** l'état demande d'être dans la portée de l'ACP ;
//! - **changer** l'état demande en plus le droit de muter (SuperAdmin ou
//!   Admin cabinet), la même frontière que pour modifier l'ACP elle-même ;
//! - `identity` ne s'éteint pour personne — ce n'est pas un défaut de droits
//!   mais une propriété de la capacité (`est_toujours_actif`).
//!
//! La garde de portée est **déléguée** à `AcpUseCases::assert_can_see_acp`
//! plutôt que recopiée : un seul chemin, un seul test à maintenir. Si la
//! règle de portée change, elle change une fois.

use std::sync::Arc;

use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::ModuleRegistry;
use crate::application::use_cases::acp_use_cases::{AcpCaller, AcpUseCases};
use crate::domain::entities::Module;

pub struct ModuleRegistryUseCases {
    registry: Arc<dyn ModuleRegistry>,
    acp_use_cases: Arc<AcpUseCases>,
}

impl ModuleRegistryUseCases {
    pub fn new(registry: Arc<dyn ModuleRegistry>, acp_use_cases: Arc<AcpUseCases>) -> Self {
        Self {
            registry,
            acp_use_cases,
        }
    }

    /// Traduit un nom reçu sur le fil en module connu.
    ///
    /// Rendu 422 `UnknownModule` (Story 5.1 @negative) — distinct d'un 403
    /// `ModuleDisabled` : le client doit pouvoir distinguer « ce nom
    /// n'existe pas » de « ce module est éteint », sinon il réessaie un nom
    /// qui ne marchera jamais.
    pub fn reconnaitre(nom: &str) -> Result<Module, AppError> {
        Module::depuis_nom(nom).ok_or_else(|| AppError::UnknownModule {
            module: nom.to_string(),
        })
    }

    /// Les modules actifs d'une ACP, pour un appelant dans sa portée.
    pub async fn list_enabled(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
    ) -> Result<Vec<Module>, AppError> {
        self.acp_use_cases
            .assert_can_see_acp(caller, acp_id)
            .await?;
        self.registry.list_enabled(acp_id).await
    }

    /// Allume un module. Idempotent.
    pub async fn enable(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
        module: Module,
    ) -> Result<(), AppError> {
        self.assert_peut_changer_letat(caller, acp_id).await?;
        self.registry.enable(acp_id, module).await
    }

    /// Éteint un module. Idempotent, et sans rien détruire (INV-27).
    ///
    /// Le refus sur `identity` est vérifié **après** la portée et les droits :
    /// un appelant hors portée ne doit pas apprendre, via un 403 différent,
    /// quels modules l'ACP possède.
    pub async fn disable(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
        module: Module,
    ) -> Result<(), AppError> {
        self.assert_peut_changer_letat(caller, acp_id).await?;
        if module.est_toujours_actif() {
            return Err(AppError::ModuleAlwaysOn {
                module: module.to_string(),
            });
        }
        self.registry.disable(acp_id, module).await
    }

    /// Chemin chaud de `ModuleGuard`. Pas de garde de portée ici : le
    /// middleware a déjà résolu et vérifié la portée avant d'appeler.
    pub async fn is_enabled(&self, acp_id: Uuid, module: Module) -> Result<bool, AppError> {
        if module.est_toujours_actif() {
            return Ok(true);
        }
        self.registry.is_enabled(acp_id, module).await
    }

    async fn assert_peut_changer_letat(
        &self,
        caller: &AcpCaller,
        acp_id: Uuid,
    ) -> Result<(), AppError> {
        self.acp_use_cases
            .assert_can_see_acp(caller, acp_id)
            .await?;
        if !caller.can_mutate() {
            return Err(AppError::Forbidden(
                "Seul un administrateur peut activer ou désactiver un module".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_inconnu_donne_422_et_pas_403() {
        // Le client doit pouvoir distinguer « n'existe pas » de « éteint ».
        let err = ModuleRegistryUseCases::reconnaitre("foobar").unwrap_err();
        assert!(matches!(err, AppError::UnknownModule { ref module } if module == "foobar"));
        assert_eq!(
            actix_web::ResponseError::status_code(&err),
            actix_web::http::StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn un_nom_connu_est_reconnu() {
        assert_eq!(
            ModuleRegistryUseCases::reconnaitre("accounting").unwrap(),
            Module::Accounting
        );
    }

    #[test]
    fn eteindre_identity_est_un_403_type_pas_une_chaine() {
        let err = AppError::ModuleAlwaysOn {
            module: Module::Identity.to_string(),
        };
        assert_eq!(
            actix_web::ResponseError::status_code(&err),
            actix_web::http::StatusCode::FORBIDDEN
        );
    }

    #[test]
    fn module_disabled_porte_le_nom_du_module() {
        // `ModuleGate` est fail-closed : il doit pouvoir nommer le module
        // éteint, pas seulement constater un refus.
        let err = AppError::ModuleDisabled {
            module: "accounting".to_string(),
        };
        assert_eq!(
            actix_web::ResponseError::status_code(&err),
            actix_web::http::StatusCode::FORBIDDEN
        );
        assert!(err.to_string().contains("accounting"));
    }
}
