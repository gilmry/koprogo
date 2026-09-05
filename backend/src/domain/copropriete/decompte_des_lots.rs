//! Le décompte légal des lots.
//!
//! Deux articles emploient **la même définition**, et il n'y en a qu'une :
//!
//! > Art. 3.89 § 5, 15° — « Toute copropriété **de moins de vingt lots à
//! > l'exclusion des caves, des garages et parkings** est autorisée à tenir
//! > une comptabilité simplifiée. »
//!
//! > Art. 3.90 § 1er — « Dans tout immeuble ou groupe d'immeubles **d'au moins
//! > vingt lots à l'exclusion des caves, garages et parkings**, un conseil de
//! > copropriété est constitué. »
//!
//! Le décompte vit ici, dans le contexte `copropriete`, parce qu'un lot est
//! une notion de l'acte de base (Art. 3.85 § 1er) et non une notion comptable.
//! La comptabilité **dérive** son régime de ce décompte ; l'inverse n'aurait
//! pas de sens, et le test d'architecture l'interdit — c'est d'ailleurs lui qui
//! a signalé que ce code s'était d'abord retrouvé du mauvais côté de la
//! frontière.
//!
//! **Les bornes s'opposent, et c'est voulu** : « de moins de vingt » pour la
//! comptabilité simplifiée, « d'au moins vingt » pour le conseil. Elles se
//! complètent exactement, et les inverser ferait basculer une copropriété de
//! vingt lots du mauvais côté des deux.
//!
//! Le piège commun aux deux : **ce n'est pas le nombre de lots de l'acte de
//! base**. Un immeuble de quinze appartements avec quinze caves et vingt
//! parkings compte cinquante lots à l'acte et **quinze** au sens de ces
//! articles.

use super::unit::UnitType;

/// Le seuil légal, commun aux deux articles.
pub const SEUIL_LEGAL: usize = 20;

/// Un lot compte-t-il dans le décompte légal ?
///
/// Les articles excluent « les caves, les garages et parkings ». `Other` est
/// compté : le doute profite à l'obligation la plus stricte, puisque se tromper
/// dans ce sens fait tenir une comptabilité plus détaillée que nécessaire ou
/// constituer un conseil qu'on aurait pu ne pas constituer, tandis que
/// l'erreur inverse met l'ACP en défaut.
pub fn compte_dans_le_seuil(nature: UnitType) -> bool {
    !matches!(nature, UnitType::Cellar | UnitType::Parking)
}

/// Le nombre de lots au sens des Art. 3.89 § 5, 15° et 3.90 § 1er.
pub fn lots_comptes(natures: &[UnitType]) -> usize {
    natures
        .iter()
        .copied()
        .filter(|n| compte_dans_le_seuil(*n))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lots(appartements: usize, caves: usize, parkings: usize) -> Vec<UnitType> {
        let mut v = vec![UnitType::Apartment; appartements];
        v.extend(vec![UnitType::Cellar; caves]);
        v.extend(vec![UnitType::Parking; parkings]);
        v
    }

    /// Le piège, avec ses chiffres.
    #[test]
    fn happy_caves_et_parkings_sortent_du_decompte() {
        let natures = lots(15, 15, 20);
        assert_eq!(natures.len(), 50, "cinquante lots à l'acte de base");
        assert_eq!(lots_comptes(&natures), 15, "quinze au sens de la loi");
    }

    #[test]
    fn happy_les_commerces_comptent() {
        let mut natures = lots(18, 0, 0);
        natures.extend(vec![UnitType::Commercial; 2]);
        assert_eq!(lots_comptes(&natures), 20);
    }

    /// @security — un lot de nature indéterminée compte.
    #[test]
    fn security_un_lot_de_nature_indeterminee_compte() {
        assert!(compte_dans_le_seuil(UnitType::Other));
    }

    #[test]
    fn negative_une_acp_sans_lot_compte_zero() {
        assert_eq!(lots_comptes(&[]), 0);
    }
}
