//! Registre des modules activables par ACP — Story 5.1 (#585), ADR-0015.
//!
//! Une ACP n'allume que les capacités dont elle a besoin. Ce fichier porte
//! le vocabulaire (`Module`) et l'état (`AcpEnabledModule`) ; les règles
//! d'activation vivent dans `module_registry_use_cases`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Les capacités que KoproGo sait allumer ou éteindre par copropriété.
///
/// Les variantes doivent rester alignées, dans les deux sens, sur la
/// contrainte `acp_enabled_modules_module_check`
/// (`migrations/20260917000000_create_acp_enabled_modules.sql`). La garde
/// `garde_enum_contre_contrainte` casse si l'une des deux listes devance
/// l'autre — c'est elle qui rend cet alignement vérifié plutôt que promis.
///
/// Le frontend tient la même liste dans `frontend/src/lib/api/modules.ts`
/// (`MODULE_NAMES`), dette documentée en tête de ce fichier-là, à résorber
/// en important les types générés dès que ce handler est au schéma OpenAPI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    /// Identité, comptes, rôles. **Toujours actif** : sans lui, plus personne
    /// ne peut se connecter pour rallumer quoi que ce soit.
    Identity,
    Community,
    Ticketing,
    Accounting,
    Governance,
    Maintenance,
    Portfolio,
}

impl Module {
    /// Toutes les variantes, dans l'ordre de la contrainte SQL.
    pub const ALL: [Module; 7] = [
        Module::Identity,
        Module::Community,
        Module::Ticketing,
        Module::Accounting,
        Module::Governance,
        Module::Maintenance,
        Module::Portfolio,
    ];

    /// Le nom porté en base et sur le fil HTTP.
    pub fn as_str(&self) -> &'static str {
        match self {
            Module::Identity => "identity",
            Module::Community => "community",
            Module::Ticketing => "ticketing",
            Module::Accounting => "accounting",
            Module::Governance => "governance",
            Module::Maintenance => "maintenance",
            Module::Portfolio => "portfolio",
        }
    }

    /// Un module « toujours actif » ne peut pas être éteint (403).
    ///
    /// Ce n'est pas une politique configurable : c'est une propriété de la
    /// capacité. Éteindre `identity` rendrait l'ACP inaccessible et donc
    /// irrécupérable par ses propres administrateurs.
    pub fn est_toujours_actif(&self) -> bool {
        matches!(self, Module::Identity)
    }

    /// Reconnaît un nom de module. `None` = nom inconnu, que l'appelant
    /// traduit en 422 (Story 5.1 @negative : `foobar`).
    ///
    /// Volontairement **strict sur la casse** : le nom vient d'un chemin
    /// d'URL, et accepter `Accounting` comme `accounting` ferait diverger la
    /// clé d'unicité en base du nom reçu.
    pub fn depuis_nom(nom: &str) -> Option<Module> {
        Module::ALL.into_iter().find(|m| m.as_str() == nom)
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// L'état d'un module pour une ACP donnée.
///
/// `archived_at` est la seule lecture de l'état : `None` = actif. Le couple
/// (acp_id, module) est unique — réactiver réutilise la ligne et remet
/// `archived_at` à `None`, ce qui laisse les données du module intactes
/// (INV-27). C'est cette réversibilité qui rend la désactivation acceptable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcpEnabledModule {
    pub id: Uuid,
    pub acp_id: Uuid,
    pub module: Module,
    pub enabled_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

impl AcpEnabledModule {
    pub fn est_actif(&self) -> bool {
        self.archived_at.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_nom_inconnu_nest_pas_un_module() {
        assert_eq!(Module::depuis_nom("foobar"), None);
        assert_eq!(Module::depuis_nom(""), None);
    }

    #[test]
    fn la_casse_nest_pas_rattrapee() {
        // Le nom vient d'un chemin d'URL. L'accepter en majuscules ferait
        // diverger la clé d'unicité en base du nom reçu.
        assert_eq!(Module::depuis_nom("Accounting"), None);
        assert_eq!(Module::depuis_nom("accounting"), Some(Module::Accounting));
    }

    #[test]
    fn chaque_variante_fait_laller_retour_par_son_nom() {
        for module in Module::ALL {
            assert_eq!(Module::depuis_nom(module.as_str()), Some(module));
        }
    }

    #[test]
    fn identity_est_le_seul_module_toujours_actif() {
        let toujours_actifs: Vec<&str> = Module::ALL
            .into_iter()
            .filter(|m| m.est_toujours_actif())
            .map(|m| m.as_str())
            .collect();
        assert_eq!(toujours_actifs, vec!["identity"]);
    }

    #[test]
    fn archived_at_est_la_seule_lecture_de_letat() {
        let mut m = AcpEnabledModule {
            id: Uuid::new_v4(),
            acp_id: Uuid::new_v4(),
            module: Module::Community,
            enabled_at: Utc::now(),
            archived_at: None,
        };
        assert!(m.est_actif());
        m.archived_at = Some(Utc::now());
        assert!(!m.est_actif());
    }
}
