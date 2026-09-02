//! Le dossier de gestion d'une ACP, au sens de l'Art. 3.89 § 5, 7° du Code civil.
//!
//! Le syndic sortant doit remettre à son successeur, dans les trente jours,
//! *« l'ensemble du dossier de la gestion de l'immeuble, y compris la
//! comptabilité et les archives »*. La loi désigne donc un ensemble hétérogène
//! de pièces (budgets, écritures, appels de fonds, convocations…) qui ont un
//! point commun : elles appartiennent à l'**ACP**, personne morale (Art. 3.86),
//! et non au mandataire qui les a produites.
//!
//! Ce module rend ce fait explicite dans le code plutôt que de le laisser
//! deviner : une pièce sait de quelle ACP elle relève, et le périmètre d'un
//! syndic se **dérive** de son mandat au lieu d'être gravé dans la pièce.
//!
//! Voir ADR-0045.

use crate::domain::entities::SyndicMandate;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Une pièce du dossier de gestion.
///
/// Implémenter ce trait, c'est déclarer qu'une entité est un acte de gestion
/// posé pour le compte d'une ACP, et donc qu'elle se transmet avec elle.
pub trait PieceDeGestion {
    /// L'ACP dont relève la pièce. C'est elle qui en est propriétaire.
    fn acp_id(&self) -> Uuid;
}

/// Les pièces qu'un syndic peut légitimement consulter à un moment donné.
///
/// Le filtre n'interroge jamais l'auteur de la pièce : il interroge le mandat.
/// Un syndic voit ce que son mandat lui confie, ni plus, ni après.
pub fn perimetre_du_mandataire<'a>(
    pieces: &'a [&'a dyn PieceDeGestion],
    mandats: &[SyndicMandate],
    syndic: Uuid,
    moment: DateTime<Utc>,
) -> Vec<&'a dyn PieceDeGestion> {
    pieces
        .iter()
        .filter(|piece| {
            let acp = piece.acp_id();
            mandats
                .iter()
                .filter(|m| m.acp_id == acp)
                .any(|m| m.covers(moment) && m.organization_id == syndic)
        })
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Budget, Expense, ExpenseCategory};
    use chrono::Duration;
    use rust_decimal_macros::dec;

    /// Le dossier de gestion d'une ACP conforme, tel qu'il se présente le jour
    /// d'une passation. Chaque champ est une famille de pièces que
    /// l'Art. 3.89 § 5, 7° oblige à transmettre.
    struct DossierComplet {
        charge: Expense,
        budget: Budget,
    }

    impl DossierComplet {
        fn pour(acp: Uuid, syndic: Uuid, immeuble: Uuid) -> Self {
            Self {
                charge: Expense::new(
                    acp,
                    syndic,
                    immeuble,
                    ExpenseCategory::Maintenance,
                    "Entretien de la chaudière".to_string(),
                    dec!(1200.00),
                    Utc::now(),
                    None,
                    None,
                    None,
                )
                .expect("charge valide"),
                budget: Budget::new(
                    acp,
                    syndic,
                    immeuble,
                    2026,
                    dec!(48000.00),
                    dec!(12000.00),
                )
                .expect("budget valide"),
            }
        }

        fn pieces(&self) -> Vec<&dyn PieceDeGestion> {
            vec![&self.charge, &self.budget]
        }
    }

    /// Art. 3.89 § 5, 7° : à la passation, le dossier passe en entier au
    /// successeur, et cesse d'être accessible au sortant.
    #[test]
    fn le_dossier_de_gestion_suit_lacp_lors_dune_passation() {
        let acp = Uuid::new_v4();
        let immeuble = Uuid::new_v4();
        let cabinet_sortant = Uuid::new_v4();
        let cabinet_entrant = Uuid::new_v4();

        let passation = Utc::now();
        let avant = passation - Duration::days(30);
        let apres = passation + Duration::days(1);

        // Le dossier a été constitué par le cabinet sortant, pour l'ACP.
        let dossier = DossierComplet::pour(acp, cabinet_sortant, immeuble);
        let pieces = dossier.pieces();

        let mut mandat_sortant = SyndicMandate::new(acp, cabinet_sortant, avant, None);
        mandat_sortant
            .revoke(passation, None, Some("Fin de mandat votée en AG".to_string()))
            .expect("révocation valide");
        let mandat_entrant = SyndicMandate::new(acp, cabinet_entrant, passation, None);
        let mandats = vec![mandat_sortant, mandat_entrant];

        // Avant la passation : le sortant tient le dossier, l'entrant n'existe pas encore.
        assert_eq!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_sortant, avant).len(),
            pieces.len(),
            "le mandataire en fonction doit voir tout le dossier"
        );
        assert!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_entrant, avant).is_empty(),
            "un cabinet sans mandat ne voit rien, même une pièce future"
        );

        // Après la passation : le dossier a suivi l'ACP, sans qu'une seule pièce bouge.
        assert_eq!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_entrant, apres).len(),
            pieces.len(),
            "Art. 3.89 § 5, 7° : le successeur reçoit l'ensemble du dossier"
        );
        assert!(
            perimetre_du_mandataire(&pieces, &mandats, cabinet_sortant, apres).is_empty(),
            "le mandat éteint, le sortant n'a plus de base pour consulter le dossier"
        );
    }

    /// Le dossier reste rattaché même quand personne ne le gère : une ACP
    /// entre deux mandats n'est pas une ACP sans comptabilité.
    #[test]
    fn une_acp_sans_mandataire_conserve_son_dossier() {
        let acp = Uuid::new_v4();
        let ancien_syndic = Uuid::new_v4();
        let passation = Utc::now();

        let dossier = DossierComplet::pour(acp, ancien_syndic, Uuid::new_v4());
        let pieces = dossier.pieces();

        let mut mandat = SyndicMandate::new(acp, ancien_syndic, passation - Duration::days(90), None);
        mandat.revoke(passation, None, None).expect("révocation valide");

        assert!(
            perimetre_du_mandataire(&pieces, &[mandat], ancien_syndic, passation).is_empty(),
            "plus personne ne voit le dossier"
        );
        for piece in &pieces {
            assert_eq!(piece.acp_id(), acp, "mais chaque pièce sait encore à qui elle est");
        }
    }

    /// Deux ACP confiées au même cabinet ne se mélangent pas.
    #[test]
    fn un_syndic_ne_voit_pas_le_dossier_dune_acp_quil_ne_gere_pas() {
        let acp_geree = Uuid::new_v4();
        let acp_voisine = Uuid::new_v4();
        let cabinet = Uuid::new_v4();
        let maintenant = Utc::now();

        let dossier_voisin = DossierComplet::pour(acp_voisine, cabinet, Uuid::new_v4());
        let pieces = dossier_voisin.pieces();

        let mandat = SyndicMandate::new(acp_geree, cabinet, maintenant - Duration::days(10), None);

        assert!(
            perimetre_du_mandataire(&pieces, &[mandat], cabinet, maintenant).is_empty(),
            "un mandat sur une ACP n'ouvre rien sur une autre, même chez le même cabinet"
        );
    }
}
