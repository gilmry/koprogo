//! Story 4.1 — quorum agrégé (présentiel + distanciel + procurations) en
//! `Decimal` strict, `AppError` typé (cluster coord #433 Decimal, #555
//! Result typé — cf. `.claude/rules/CRITICAL.md` §4).
//!
//! Le seuil appliqué est celui de `Meeting::quotas_half_reached` : « au
//! moins la moitié » (Art. 3.87 §5 CC) — borne INCLUSIVE, 50,0 % pile
//! suffit. Aucun seuil légal n'est ré-écrit ici : cf. le commentaire de
//! `AgSession::is_combined_quorum_reached` (`domain/copropriete/ag_session.rs`)
//! sur le prix d'un seuil dupliqué (#661).
//!
//! Ce use-case ne juge que le volet **quotités** agrégé. Le volet **têtes**
//! du quorum double (Art. 3.87 §5) reste porté par
//! `Meeting::assert_can_complete()` / `MeetingCompletionChecklist`, qui
//! comptent les copropriétaires — pas les millièmes — et ne sont pas dans
//! le périmètre de cette story.

use crate::application::error::AppError;
use crate::domain::entities::{Meeting, MeetingMode};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// Entrées agrégées du quorum d'une AG hybride (Story 4.1).
#[derive(Debug, Clone)]
pub struct ComputeQuorumInput {
    pub mode: MeetingMode,
    /// Millièmes des copropriétaires présents physiquement.
    pub in_person_quotas: Decimal,
    /// Millièmes des copropriétaires connectés à distance.
    pub remote_quotas: Decimal,
    /// Millièmes représentés par procuration (Art. 3.87 §7 CC).
    pub proxy_quotas: Decimal,
    /// Total des millièmes du bâtiment/ACP.
    pub total_quotas: Decimal,
    /// Art. 3.87 §1er CC : la participation à distance suppose une identité
    /// authentifiée fortement — cf. Story 4.2 (#577). Tant que 4.2 n'est pas
    /// câblée, cette confirmation vient de l'appelant (session distancielle
    /// vérifiée ou non). `compute_quorum` REFUSE de compter des quotités
    /// distancielles non authentifiées plutôt que de les ignorer en
    /// silence : un quorum silencieusement amputé serait aussi faux qu'un
    /// quorum gonflé.
    pub remote_strong_auth_confirmed: bool,
}

/// Résultat du calcul de quorum agrégé.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuorumResult {
    pub attended_quotas: Decimal,
    pub total_quotas: Decimal,
    pub quorum_percentage: Decimal,
    pub quorum_reached: bool,
}

/// Agrège présentiel + distanciel + procurations et juge le quorum
/// (Art. 3.87 §5 CC — « au moins la moitié », borne inclusive).
pub fn compute_quorum(input: ComputeQuorumInput) -> Result<QuorumResult, AppError> {
    if input.total_quotas <= Decimal::ZERO {
        return Err(AppError::Validation(
            "Le total des quotités doit être positif".to_string(),
        ));
    }
    for (label, value) in [
        ("présentielles", input.in_person_quotas),
        ("distancielles", input.remote_quotas),
        ("de procuration", input.proxy_quotas),
    ] {
        if value < Decimal::ZERO {
            return Err(AppError::Validation(format!(
                "Les quotités {label} ne peuvent pas être négatives"
            )));
        }
    }

    // @security — Art. 3.87 §1er CC : une AG remote/hybride ne peut compter
    // des quotités distancielles que si la connexion est authentifiée
    // fortement (cf. Story 4.2 #577).
    if matches!(input.mode, MeetingMode::Remote | MeetingMode::Hybrid)
        && input.remote_quotas > Decimal::ZERO
        && !input.remote_strong_auth_confirmed
    {
        return Err(AppError::Forbidden(
            "Participation à distance sans authentification forte \
             (Art. 3.87 §1er CC — cf. Story 4.2)"
                .to_string(),
        ));
    }

    let attended_quotas = input.in_person_quotas + input.remote_quotas + input.proxy_quotas;
    if attended_quotas > input.total_quotas {
        return Err(AppError::Validation(format!(
            "Quorum agrégé invalide : {attended_quotas} quotités pour un total de {}",
            input.total_quotas
        )));
    }

    let quorum_percentage = (attended_quotas / input.total_quotas) * dec!(100);
    let quorum_reached = Meeting::quotas_half_reached(attended_quotas, input.total_quotas);

    Ok(QuorumResult {
        attended_quotas,
        total_quotas: input.total_quotas,
        quorum_percentage,
        quorum_reached,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> ComputeQuorumInput {
        ComputeQuorumInput {
            mode: MeetingMode::Hybrid,
            in_person_quotas: Decimal::ZERO,
            remote_quotas: Decimal::ZERO,
            proxy_quotas: Decimal::ZERO,
            total_quotas: dec!(1000),
            remote_strong_auth_confirmed: true,
        }
    }

    // ------------------------------------------------------------------
    // @happy
    // ------------------------------------------------------------------

    /// AC Story 4.1 : "10 présentiels + 5 distants + 3 procurations →
    /// quorum agrégé OK selon Decimal somme".
    #[test]
    fn happy_hybrid_10_in_person_5_remote_3_proxy_matches_story_ac() {
        let input = ComputeQuorumInput {
            in_person_quotas: dec!(10),
            remote_quotas: dec!(5),
            proxy_quotas: dec!(3),
            total_quotas: dec!(1000),
            ..base_input()
        };
        let result = compute_quorum(input).unwrap();
        assert_eq!(result.attended_quotas, dec!(18));
        assert_eq!(result.quorum_percentage, dec!(1.8));
    }

    #[test]
    fn happy_hybrid_quorum_reached_above_half() {
        let input = ComputeQuorumInput {
            in_person_quotas: dec!(400),
            remote_quotas: dec!(150),
            proxy_quotas: dec!(60),
            ..base_input()
        };
        let result = compute_quorum(input).unwrap();
        assert_eq!(result.attended_quotas, dec!(610));
        assert!(result.quorum_reached);
    }

    // ------------------------------------------------------------------
    // @edge — les deux côtés du seuil de 50%, testés séparément (sinon la
    // borne n'est pas vraiment testée).
    // ------------------------------------------------------------------

    #[test]
    fn edge_exactly_50_percent_is_reached() {
        let input = ComputeQuorumInput {
            in_person_quotas: dec!(300),
            remote_quotas: dec!(150),
            proxy_quotas: dec!(50),
            ..base_input()
        };
        let result = compute_quorum(input).unwrap();
        assert_eq!(result.quorum_percentage, dec!(50));
        assert!(
            result.quorum_reached,
            "50,0% pile doit être respecté (Art. 3.87 §5 — au moins la moitié)"
        );
    }

    #[test]
    fn edge_49_99_percent_is_refused() {
        let input = ComputeQuorumInput {
            in_person_quotas: dec!(300),
            remote_quotas: dec!(150),
            proxy_quotas: dec!(49.9),
            ..base_input()
        };
        let result = compute_quorum(input).unwrap();
        assert_eq!(result.quorum_percentage, dec!(49.99));
        assert!(!result.quorum_reached, "49,99% doit être refusé");
    }

    #[test]
    fn edge_zero_total_quotas_is_rejected_not_panic() {
        let input = ComputeQuorumInput {
            total_quotas: Decimal::ZERO,
            ..base_input()
        };
        let err = compute_quorum(input).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ------------------------------------------------------------------
    // @security
    // ------------------------------------------------------------------

    #[test]
    fn security_remote_without_strong_auth_is_forbidden() {
        let input = ComputeQuorumInput {
            mode: MeetingMode::Remote,
            remote_quotas: dec!(500),
            remote_strong_auth_confirmed: false,
            ..base_input()
        };
        let err = compute_quorum(input).unwrap_err();
        assert!(matches!(err, AppError::Forbidden(_)));
    }

    #[test]
    fn security_hybrid_without_strong_auth_but_zero_remote_quotas_is_allowed() {
        // Un hybride où personne ne s'est connecté à distance n'a rien à
        // authentifier : refuser ici punirait une AG qui n'a jamais ouvert
        // de session distancielle.
        let input = ComputeQuorumInput {
            mode: MeetingMode::Hybrid,
            in_person_quotas: dec!(600),
            remote_quotas: Decimal::ZERO,
            remote_strong_auth_confirmed: false,
            ..base_input()
        };
        assert!(compute_quorum(input).is_ok());
    }

    #[test]
    fn security_forged_negative_proxy_quotas_are_rejected() {
        let input = ComputeQuorumInput {
            proxy_quotas: dec!(-1),
            ..base_input()
        };
        let err = compute_quorum(input).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    // ------------------------------------------------------------------
    // @negative
    // ------------------------------------------------------------------

    #[test]
    fn negative_attended_exceeds_total_is_rejected() {
        let input = ComputeQuorumInput {
            in_person_quotas: dec!(900),
            remote_quotas: dec!(200),
            ..base_input()
        };
        let err = compute_quorum(input).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn negative_negative_total_quotas_is_rejected() {
        let input = ComputeQuorumInput {
            total_quotas: dec!(-1000),
            ..base_input()
        };
        let err = compute_quorum(input).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
