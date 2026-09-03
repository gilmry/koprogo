//! L'acquisition de la personnalité juridique par l'ACP.
//!
//! Art. 3.86 § 1er :
//!
//! > « L'association des copropriétaires acquiert la personnalité juridique au
//! > moment où sont réunies les **deux conditions suivantes** :
//! > 1° la **naissance de l'indivision** par la cession ou l'attribution d'un
//! > lot au moins ;
//! > 2° la **transcription** de l'acte de base et du règlement de copropriété
//! > dans les registres du bureau compétent de l'Administration générale de la
//! > Documentation patrimoniale. »
//!
//! Et § 2, dont l'asymétrie est le cœur de cette modélisation :
//!
//! > « En cas d'omission ou de retard dans la transcription des statuts,
//! > l'association des copropriétaires **ne pourra se prévaloir de la
//! > personnalité juridique à l'égard des tiers**, lesquels auront **néanmoins
//! > la faculté d'en faire état contre elle**. »
//!
//! Autrement dit : une ACP dont les statuts ne sont pas transcrits ne peut pas
//! opposer sa personnalité à un fournisseur, mais ce fournisseur peut la lui
//! opposer. La protection joue **dans un seul sens**, contre l'association
//! négligente et en faveur du tiers. Modéliser cela par un simple booléen
//! ferait perdre exactement ce qui compte.
//!
//! Conséquence pratique pour le logiciel : tant que la transcription manque,
//! l'ACP ne devrait pas pouvoir *engager* — signer un contrat, ouvrir un
//! compte, ester en justice — alors qu'elle reste tenue de ce qu'on lui
//! réclame.
//!
//! Hors périmètre ici : les associations partielles, qui « ne peuvent disposer
//! de la personnalité juridique qu'à partir du moment où l'association
//! principale dont elles dépendent dispose elle-même » (§ 2, dernière phrase).
//! Elles sont reportées en v0.2.0 par l'ADR-0010.
//!
//! Voir issue #740.

use chrono::NaiveDate;

/// L'état de la personnalité juridique d'une ACP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonnaliteJuridique {
    /// Aucun lot n'a été cédé ni attribué : pas d'indivision, donc pas
    /// d'association. Un immeuble entier encore aux mains du promoteur.
    Inexistante,
    /// L'indivision est née mais les statuts ne sont pas transcrits.
    ///
    /// L'ACP ne peut pas s'en prévaloir envers un tiers ; un tiers, lui, peut
    /// la lui opposer (Art. 3.86 § 2).
    NonOpposableAuxTiers { depuis: NaiveDate },
    /// Les deux conditions sont réunies.
    Acquise { depuis: NaiveDate },
}

impl PersonnaliteJuridique {
    /// L'ACP peut-elle **invoquer** sa personnalité contre un tiers ?
    ///
    /// C'est ce qui conditionne sa capacité à engager : signer un contrat,
    /// ouvrir un compte à son nom, agir en justice.
    pub fn opposable_par_lacp(&self) -> bool {
        matches!(self, Self::Acquise { .. })
    }

    /// Un tiers peut-il l'**opposer** à l'ACP ?
    ///
    /// Oui dès que l'indivision est née, transcription ou non. C'est la
    /// dissymétrie voulue par l'article : le retard de transcription ne
    /// profite pas à l'association qui l'a laissé traîner.
    pub fn opposable_par_un_tiers(&self) -> bool {
        matches!(
            self,
            Self::Acquise { .. } | Self::NonOpposableAuxTiers { .. }
        )
    }

    /// Depuis quand, le cas échéant.
    pub fn depuis(&self) -> Option<NaiveDate> {
        match self {
            Self::Inexistante => None,
            Self::NonOpposableAuxTiers { depuis } | Self::Acquise { depuis } => Some(*depuis),
        }
    }
}

impl std::fmt::Display for PersonnaliteJuridique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inexistante => write!(
                f,
                "Aucun lot cédé ni attribué : l'indivision n'est pas née (Art. 3.86 § 1er, 1°)"
            ),
            Self::NonOpposableAuxTiers { depuis } => write!(
                f,
                "Indivision née le {depuis} mais statuts non transcrits : l'ACP ne peut pas se \
                 prévaloir de sa personnalité envers les tiers, qui peuvent la lui opposer \
                 (Art. 3.86 § 2)"
            ),
            Self::Acquise { depuis } => {
                write!(f, "Personnalité juridique acquise le {depuis} (Art. 3.86 § 1er)")
            }
        }
    }
}

/// L'état de la personnalité, dérivé des deux conditions.
///
/// Elle s'acquiert au moment où la **seconde** condition est remplie : c'est
/// la plus tardive des deux dates qui compte, pas la première.
pub fn personnalite(
    premiere_cession_de_lot: Option<NaiveDate>,
    transcription_statuts: Option<NaiveDate>,
) -> PersonnaliteJuridique {
    let Some(cession) = premiere_cession_de_lot else {
        // Sans indivision, la transcription seule ne fait rien naître.
        return PersonnaliteJuridique::Inexistante;
    };
    match transcription_statuts {
        Some(transcription) => PersonnaliteJuridique::Acquise {
            depuis: cession.max(transcription),
        },
        None => PersonnaliteJuridique::NonOpposableAuxTiers { depuis: cession },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le(annee: i32, mois: u32, jour: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(annee, mois, jour).expect("date valide")
    }

    #[test]
    fn happy_les_deux_conditions_reunies_donnent_la_personnalite() {
        let p = personnalite(Some(le(2026, 3, 1)), Some(le(2026, 2, 1)));
        assert_eq!(p, PersonnaliteJuridique::Acquise { depuis: le(2026, 3, 1) });
        assert!(p.opposable_par_lacp());
        assert!(p.opposable_par_un_tiers());
    }

    /// C'est la plus tardive des deux dates qui fait naître la personnalité :
    /// « au moment où sont réunies les deux conditions ».
    #[test]
    fn edge_la_personnalite_nait_a_la_seconde_condition_remplie() {
        let transcription_apres = personnalite(Some(le(2026, 1, 10)), Some(le(2026, 5, 20)));
        assert_eq!(transcription_apres.depuis(), Some(le(2026, 5, 20)));

        let cession_apres = personnalite(Some(le(2026, 5, 20)), Some(le(2026, 1, 10)));
        assert_eq!(cession_apres.depuis(), Some(le(2026, 5, 20)));
    }

    #[test]
    fn negative_sans_lot_cede_il_ny_a_pas_dassociation() {
        // Un immeuble entier encore aux mains du promoteur : les statuts ont
        // beau être transcrits, l'indivision n'est pas née.
        let p = personnalite(None, Some(le(2026, 2, 1)));
        assert_eq!(p, PersonnaliteJuridique::Inexistante);
        assert!(!p.opposable_par_lacp());
        assert!(!p.opposable_par_un_tiers());
    }

    /// Le cœur de l'article : l'asymétrie du § 2.
    #[test]
    fn security_sans_transcription_la_protection_ne_joue_que_dans_un_sens() {
        let p = personnalite(Some(le(2026, 3, 1)), None);

        assert!(
            !p.opposable_par_lacp(),
            "l'ACP ne peut pas se prévaloir de sa personnalité envers un tiers"
        );
        assert!(
            p.opposable_par_un_tiers(),
            "mais un tiers peut la lui opposer : le retard de transcription ne \
             profite pas à l'association qui l'a laissé traîner"
        );
    }

    #[test]
    fn happy_lasymetrie_disparait_une_fois_les_statuts_transcrits() {
        let avant = personnalite(Some(le(2026, 3, 1)), None);
        let apres = personnalite(Some(le(2026, 3, 1)), Some(le(2026, 4, 1)));

        assert_ne!(avant.opposable_par_lacp(), apres.opposable_par_lacp());
        assert_eq!(
            avant.opposable_par_un_tiers(),
            apres.opposable_par_un_tiers(),
            "de ce côté-là, rien ne change : le tiers pouvait déjà"
        );
    }

    #[test]
    fn edge_les_deux_conditions_le_meme_jour() {
        let p = personnalite(Some(le(2026, 3, 1)), Some(le(2026, 3, 1)));
        assert_eq!(p.depuis(), Some(le(2026, 3, 1)));
        assert!(p.opposable_par_lacp());
    }

    #[test]
    fn negative_une_acp_vide_nest_opposable_par_personne() {
        let p = personnalite(None, None);
        assert_eq!(p, PersonnaliteJuridique::Inexistante);
        assert_eq!(p.depuis(), None);
    }
}
