//! Le régime comptable applicable à une ACP.
//!
//! Art. 3.89 § 5, 15° — le syndic est chargé :
//!
//! > « de tenir les comptes de l'association des copropriétaires de manière
//! > claire, précise et détaillée suivant le **plan comptable minimum
//! > normalisé** à établir par le Roi. Toute copropriété **de moins de vingt
//! > lots à l'exclusion des caves, des garages et parkings** est autorisée à
//! > tenir une **comptabilité simplifiée** reflétant au minimum les recettes
//! > et les dépenses, la situation de trésorerie ainsi que les mouvements des
//! > disponibilités. »
//!
//! Le plan comptable minimum normalisé est celui de l'arrêté royal du
//! 12 juillet 2012, implémenté depuis longtemps ici. Ce qui manquait, c'est la
//! **question préalable** : cette ACP y est-elle seulement tenue ?
//!
//! Deux pièges dans le décompte, et ils vont dans le même sens — celui de
//! surestimer le nombre de lots et d'imposer une comptabilité complète à une
//! petite copropriété qui a le droit de faire plus simple :
//!
//! 1. **caves, garages et parkings sont exclus**. Ce n'est pas le nombre de
//!    lots de l'acte de base : un immeuble de 15 appartements avec 15 caves et
//!    20 parkings compte 15 lots au sens de cet article, pas 50 ;
//! 2. **« moins de vingt »** est strict. Vingt lots imposent le plan complet ;
//!    dix-neuf ne l'imposent pas.
//!
//! Le régime n'est jamais saisi à la main : il se **dérive** du décompte. Une
//! copropriété qui franchit le seuil change de régime parce qu'elle a vendu un
//! lot, pas parce qu'un syndic l'a décidé.
//!
//! Voir issue #746.

use crate::domain::copropriete::unit::UnitType;

/// Le régime comptable auquel une ACP est tenue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegimeComptable {
    /// Plan comptable minimum normalisé (AR du 12/07/2012), obligatoire dès
    /// vingt lots.
    Complet,
    /// Comptabilité simplifiée : recettes, dépenses, situation de trésorerie,
    /// mouvements des disponibilités.
    Simplifie,
}

impl RegimeComptable {
    /// Le seuil légal, en lots comptés au sens de l'Art. 3.89 § 5, 15°.
    pub const SEUIL_LEGAL: usize = 20;

    pub fn est_simplifie(&self) -> bool {
        matches!(self, Self::Simplifie)
    }
}

impl std::fmt::Display for RegimeComptable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Complet => write!(f, "plan comptable minimum normalisé (AR du 12/07/2012)"),
            Self::Simplifie => write!(f, "comptabilité simplifiée (Art. 3.89 § 5, 15°)"),
        }
    }
}

/// Un lot compte-t-il dans le décompte légal ?
///
/// L'article exclut « les caves, les garages et parkings ». `Other` est
/// compté : le doute profite à l'obligation la plus stricte, puisque se
/// tromper dans ce sens fait tenir une comptabilité plus détaillée que
/// nécessaire, tandis que l'erreur inverse met l'ACP en défaut.
pub fn compte_dans_le_seuil(nature: UnitType) -> bool {
    !matches!(nature, UnitType::Cellar | UnitType::Parking)
}

/// Le nombre de lots au sens de l'Art. 3.89 § 5, 15°.
pub fn lots_comptes(natures: &[UnitType]) -> usize {
    natures
        .iter()
        .copied()
        .filter(|n| compte_dans_le_seuil(*n))
        .count()
}

/// Le régime auquel une ACP est tenue, dérivé de ses lots.
pub fn regime_applicable(natures: &[UnitType]) -> RegimeComptable {
    if lots_comptes(natures) < RegimeComptable::SEUIL_LEGAL {
        RegimeComptable::Simplifie
    } else {
        RegimeComptable::Complet
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lots(appartements: usize, caves: usize, parkings: usize, commerces: usize) -> Vec<UnitType> {
        let mut v = vec![UnitType::Apartment; appartements];
        v.extend(vec![UnitType::Cellar; caves]);
        v.extend(vec![UnitType::Parking; parkings]);
        v.extend(vec![UnitType::Commercial; commerces]);
        v
    }

    #[test]
    fn happy_une_petite_copropriete_peut_tenir_une_comptabilite_simplifiee() {
        let regime = regime_applicable(&lots(12, 0, 0, 0));
        assert_eq!(regime, RegimeComptable::Simplifie);
    }

    #[test]
    fn happy_une_grande_copropriete_est_tenue_au_plan_normalise() {
        let regime = regime_applicable(&lots(25, 0, 0, 0));
        assert_eq!(regime, RegimeComptable::Complet);
    }

    /// Le piège du décompte : caves et parkings ne comptent pas.
    ///
    /// Cet immeuble a cinquante lots à l'acte de base et quinze au sens de
    /// l'article. Le compter à cinquante lui imposerait une comptabilité
    /// complète à laquelle la loi ne l'oblige pas.
    #[test]
    fn happy_caves_et_parkings_sortent_du_decompte() {
        let natures = lots(15, 15, 20, 0);
        assert_eq!(natures.len(), 50, "cinquante lots à l'acte de base");
        assert_eq!(lots_comptes(&natures), 15, "quinze au sens de l'article");
        assert_eq!(regime_applicable(&natures), RegimeComptable::Simplifie);
    }

    /// @edge — « moins de vingt » est strict : la borne est exclusive.
    #[test]
    fn edge_dix_neuf_lots_restent_simplifies() {
        assert_eq!(
            regime_applicable(&lots(19, 30, 40, 0)),
            RegimeComptable::Simplifie
        );
    }

    /// @edge — et vingt lots basculent.
    #[test]
    fn edge_vingt_lots_imposent_le_plan_normalise() {
        assert_eq!(regime_applicable(&lots(20, 0, 0, 0)), RegimeComptable::Complet);
    }

    /// Les commerces comptent : l'article n'exclut que caves, garages et
    /// parkings.
    #[test]
    fn happy_les_commerces_comptent_dans_le_seuil() {
        assert_eq!(
            regime_applicable(&lots(18, 0, 0, 2)),
            RegimeComptable::Complet
        );
    }

    /// @security — un lot de nature indéterminée compte.
    ///
    /// Le doute profite à l'obligation la plus stricte : se tromper dans ce
    /// sens fait tenir une comptabilité plus détaillée que nécessaire, alors
    /// que l'erreur inverse met l'ACP en défaut.
    #[test]
    fn security_un_lot_de_nature_indeterminee_compte() {
        let mut natures = lots(19, 0, 0, 0);
        natures.push(UnitType::Other);
        assert_eq!(regime_applicable(&natures), RegimeComptable::Complet);
    }

    /// @negative — une ACP sans lot encodé n'est tenue à rien de complet.
    ///
    /// Le cas se produit avant que le SuperAdmin ait fini d'encoder l'acte de
    /// base. Il ne doit pas déclencher une obligation comptable sur du vide.
    #[test]
    fn negative_une_acp_sans_lot_reste_simplifiee() {
        assert_eq!(regime_applicable(&[]), RegimeComptable::Simplifie);
    }
}
