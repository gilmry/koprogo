//! Le conseil de copropriété.
//!
//! Art. 3.90 § 1er :
//!
//! > « Dans tout immeuble ou groupe d'immeubles **d'au moins vingt lots à
//! > l'exclusion des caves, garages et parkings**, un conseil de copropriété
//! > est constitué par la **première assemblée générale**. Ce conseil, dont
//! > peuvent être membre **les titulaires d'un droit réel disposant d'un droit
//! > de vote** à l'assemblée générale, est chargé de **veiller à la bonne
//! > exécution par le syndic de ses missions** [...] »
//!
//! Et § 2 : en dessous de vingt lots, l'assemblée **peut** en constituer un,
//! « composé de la même manière et chargé des mêmes missions ». La faculté
//! remplace l'obligation, le reste ne change pas.
//!
//! § 3 : nomination « à la **majorité absolue**, pour **chaque membre
//! séparément** », et le mandat « dure **jusqu'à la prochaine assemblée
//! générale ordinaire** et est renouvelable ».
//!
//! **Le décompte est le même que celui de la comptabilité simplifiée**
//! (Art. 3.89 § 5, 15°) : vingt lots, caves, garages et parkings exclus. Le
//! législateur emploie deux fois la même définition, et le code la partage au
//! lieu de la réécrire — s'il la réécrivait, les deux finiraient par diverger.
//!
//! La borne, en revanche, n'est pas la même : la comptabilité simplifiée vaut
//! « de **moins de** vingt lots », le conseil est obligatoire « d'**au moins**
//! vingt ». Elles se complètent exactement, mais les inverser ferait basculer
//! une copropriété de vingt lots du mauvais côté des deux.
//!
//! Voir issue #753.

use super::decompte_des_lots::lots_comptes;
use super::unit::UnitType;
use super::unit_owner::OwnershipType;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Le seuil de l'Art. 3.90 § 1er, en lots comptés.
pub const SEUIL_OBLIGATION: usize = 20;

/// Le conseil est-il obligatoire, facultatif, ou déjà là ?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegimeDuConseil {
    /// Au moins vingt lots : la première assemblée doit le constituer.
    Obligatoire,
    /// Moins de vingt lots : l'assemblée peut le constituer (§ 2).
    Facultatif,
}

/// Ce qui empêche une nomination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NominationRefusee {
    /// Seuls les titulaires d'un droit réel **disposant du droit de vote**
    /// peuvent siéger.
    SansDroitDeVote,
    /// Art. 3.89 § 9 : dans une même ACP, le syndic ne peut être ni membre du
    /// conseil de copropriété ni commissaire aux comptes.
    LeSyndicNePeutPasSieger,
}

impl std::fmt::Display for NominationRefusee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SansDroitDeVote => write!(
                f,
                "Art. 3.90 § 1er : seuls les titulaires d'un droit réel disposant du droit \
                 de vote peuvent siéger au conseil de copropriété."
            ),
            Self::LeSyndicNePeutPasSieger => write!(
                f,
                "Art. 3.89 § 9 : dans une même ACP, le syndic ne peut être ni membre du \
                 conseil de copropriété ni commissaire aux comptes."
            ),
        }
    }
}

/// Le régime applicable, dérivé du décompte légal des lots.
///
/// La borne est **inclusive** : vingt lots rendent le conseil obligatoire.
pub fn regime(natures_des_lots: &[UnitType]) -> RegimeDuConseil {
    if lots_comptes(natures_des_lots) >= SEUIL_OBLIGATION {
        RegimeDuConseil::Obligatoire
    } else {
        RegimeDuConseil::Facultatif
    }
}

/// Un membre du conseil, avec son mandat borné.
#[derive(Debug, Clone, PartialEq)]
pub struct MembreDuConseil {
    pub owner_id: Uuid,
    pub nomme_le: DateTime<Utc>,
    /// Le mandat court jusqu'à la prochaine AG ordinaire (§ 3). Tant qu'elle
    /// ne s'est pas tenue, la date est inconnue — d'où l'`Option`.
    pub echu_le: Option<DateTime<Utc>>,
}

impl MembreDuConseil {
    /// Nomme un membre, si la loi le permet.
    ///
    /// `nature_du_droit` et `dispose_du_droit_de_vote` viennent de la
    /// détention : l'article vise « les titulaires d'un droit réel disposant
    /// d'un droit de vote », donc les deux conditions.
    pub fn nommer(
        owner_id: Uuid,
        nature_du_droit: OwnershipType,
        dispose_du_droit_de_vote: bool,
        est_le_syndic: bool,
        nomme_le: DateTime<Utc>,
    ) -> Result<Self, NominationRefusee> {
        if est_le_syndic {
            return Err(NominationRefusee::LeSyndicNePeutPasSieger);
        }
        if !dispose_du_droit_de_vote {
            return Err(NominationRefusee::SansDroitDeVote);
        }
        // La nature du droit ne restreint pas au-delà : usufruitier,
        // nu-propriétaire ou indivisaire peuvent siéger dès lors qu'ils ont le
        // droit de vote. L'article dit « titulaires d'un droit réel », sans
        // autre distinction.
        let _ = nature_du_droit;
        Ok(Self {
            owner_id,
            nomme_le,
            echu_le: None,
        })
    }

    /// Le mandat prend fin à la prochaine assemblée générale ordinaire.
    pub fn echoir_a_lag_ordinaire(&mut self, tenue_le: DateTime<Utc>) {
        self.echu_le = Some(tenue_le);
    }

    pub fn en_fonction_le(&self, moment: DateTime<Utc>) -> bool {
        moment >= self.nomme_le && self.echu_le.is_none_or(|fin| moment < fin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn lots(appartements: usize, caves: usize, parkings: usize) -> Vec<UnitType> {
        let mut v = vec![UnitType::Apartment; appartements];
        v.extend(vec![UnitType::Cellar; caves]);
        v.extend(vec![UnitType::Parking; parkings]);
        v
    }

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    #[test]
    fn happy_a_partir_de_vingt_lots_le_conseil_est_obligatoire() {
        assert_eq!(regime(&lots(20, 0, 0)), RegimeDuConseil::Obligatoire);
    }

    /// @edge — la borne est inclusive, à l'inverse de celle du régime
    /// comptable.
    ///
    /// La comptabilité simplifiée vaut « de moins de vingt lots », le conseil
    /// est obligatoire « d'au moins vingt ». Elles se complètent exactement,
    /// et les inverser ferait basculer une copropriété de vingt lots du
    /// mauvais côté des deux.
    #[test]
    fn edge_dix_neuf_lots_rendent_le_conseil_facultatif() {
        assert_eq!(regime(&lots(19, 0, 0)), RegimeDuConseil::Facultatif);
        assert_eq!(regime(&lots(20, 0, 0)), RegimeDuConseil::Obligatoire);
    }

    /// Le décompte est celui de l'Art. 3.89 § 5, 15°, partagé et non réécrit.
    #[test]
    fn happy_caves_et_parkings_sortent_du_decompte_ici_aussi() {
        let natures = lots(15, 15, 20);
        assert_eq!(natures.len(), 50);
        assert_eq!(
            regime(&natures),
            RegimeDuConseil::Facultatif,
            "quinze lots au sens de la loi, pas cinquante"
        );
    }

    // ── La nomination ──────────────────────────────────────────────

    #[test]
    fn happy_un_coproprietaire_votant_peut_sieger() {
        assert!(MembreDuConseil::nommer(
            Uuid::new_v4(),
            OwnershipType::FullOwner,
            true,
            false,
            il_y_a(10)
        )
        .is_ok());
    }

    /// L'usufruitier siège s'il a le droit de vote : l'article dit
    /// « titulaires d'un droit réel », sans autre distinction.
    #[test]
    fn happy_un_usufruitier_votant_peut_sieger() {
        assert!(MembreDuConseil::nommer(
            Uuid::new_v4(),
            OwnershipType::Usufruct,
            true,
            false,
            il_y_a(10)
        )
        .is_ok());
    }

    #[test]
    fn negative_sans_droit_de_vote_on_ne_siege_pas() {
        assert_eq!(
            MembreDuConseil::nommer(
                Uuid::new_v4(),
                OwnershipType::BareOwner,
                false,
                false,
                il_y_a(10)
            ),
            Err(NominationRefusee::SansDroitDeVote)
        );
    }

    /// @security — Art. 3.89 § 9 : le syndic ne surveille pas le syndic.
    #[test]
    fn security_le_syndic_ne_peut_pas_sieger_au_conseil_quil_doit_subir() {
        assert_eq!(
            MembreDuConseil::nommer(
                Uuid::new_v4(),
                OwnershipType::FullOwner,
                true,
                true,
                il_y_a(10)
            ),
            Err(NominationRefusee::LeSyndicNePeutPasSieger)
        );
    }

    // ── Le mandat ──────────────────────────────────────────────────

    #[test]
    fn happy_un_mandat_neuf_est_en_cours() {
        let membre = MembreDuConseil::nommer(
            Uuid::new_v4(),
            OwnershipType::FullOwner,
            true,
            false,
            il_y_a(10),
        )
        .unwrap();
        assert!(membre.en_fonction_le(Utc::now()));
    }

    /// Le mandat « dure jusqu'à la prochaine assemblée générale ordinaire »
    /// (§ 3) : il s'éteint le jour où elle se tient.
    #[test]
    fn happy_le_mandat_sacheve_a_la_prochaine_ag_ordinaire() {
        let mut membre = MembreDuConseil::nommer(
            Uuid::new_v4(),
            OwnershipType::FullOwner,
            true,
            false,
            il_y_a(400),
        )
        .unwrap();
        let ag = il_y_a(30);
        membre.echoir_a_lag_ordinaire(ag);

        assert!(membre.en_fonction_le(ag - Duration::days(1)));
        assert!(
            !membre.en_fonction_le(ag),
            "le jour de l'AG, le mandat est échu — comme pour le mandat de \
             syndic, début inclus et fin exclue"
        );
    }
}
