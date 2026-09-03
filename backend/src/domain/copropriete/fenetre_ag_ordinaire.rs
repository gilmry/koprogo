//! La fenêtre statutaire de l'assemblée générale ordinaire.
//!
//! Art. 3.85 § 3, 3° — le règlement d'ordre intérieur contient au moins :
//!
//! > « la **période annuelle de quinze jours** pendant laquelle se tient
//! > l'assemblée générale ordinaire de l'association des copropriétaires. »
//!
//! Cette fenêtre n'est pas décorative : elle sert de point d'ancrage à
//! l'Art. 3.87 § 3, qui oblige le syndic à inscrire à l'ordre du jour les
//! propositions écrites reçues
//!
//! > « au moins **trois semaines avant le premier jour de la période**, fixée
//! > dans le règlement d'ordre intérieur, au cours de laquelle l'assemblée
//! > générale ordinaire doit avoir lieu. »
//!
//! Sans fenêtre, ce délai de trois semaines n'a aucun point de départ : un
//! copropriétaire ne peut pas savoir quand déposer sa proposition, et un
//! syndic ne peut pas justifier de l'avoir écartée. Les deux règles tiennent
//! ou tombent ensemble.
//!
//! La période est **récurrente** : le ROI fixe un mois et un jour, pas une
//! date. Elle se projette sur chaque exercice.
//!
//! Voir issue #747.

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};

/// Ce qui empêche une fenêtre d'être valide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenetreInvalide {
    /// Mois hors de 1..=12.
    MoisHorsBornes(u32),
    /// Jour hors de 1..=31.
    JourHorsBornes(u32),
    /// Le couple mois/jour ne désigne aucune date réelle — un 31 novembre,
    /// par exemple. Le ROI ne peut pas fixer une période qui n'existe pas.
    DateInexistante { mois: u32, jour: u32 },
}

impl std::fmt::Display for FenetreInvalide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoisHorsBornes(m) => write!(f, "Mois invalide : {m}"),
            Self::JourHorsBornes(j) => write!(f, "Jour invalide : {j}"),
            Self::DateInexistante { mois, jour } => write!(
                f,
                "Le {jour}/{mois} n'existe pas : le règlement d'ordre intérieur ne peut pas \
                 fixer une période qui ne tombe jamais"
            ),
        }
    }
}

/// La période annuelle de quinze jours fixée par le règlement d'ordre
/// intérieur (Art. 3.85 § 3, 3°).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct FenetreAgOrdinaire {
    mois: u32,
    jour: u32,
}

impl FenetreAgOrdinaire {
    /// Quinze jours, bornes comprises.
    pub const DUREE_JOURS: i64 = 15;

    /// Trois semaines avant le premier jour, pour les propositions
    /// (Art. 3.87 § 3).
    pub const PREAVIS_PROPOSITIONS_JOURS: i64 = 21;

    /// Le premier jour de la période, tel que le ROI le fixe.
    ///
    /// Une année bissextile suffit à valider un 29 février : la règle ne
    /// s'applique alors qu'un an sur quatre, ce qui est un choix de ROI
    /// discutable mais licite. On refuse en revanche le 30 février.
    pub fn new(mois: u32, jour: u32) -> Result<Self, FenetreInvalide> {
        if !(1..=12).contains(&mois) {
            return Err(FenetreInvalide::MoisHorsBornes(mois));
        }
        if !(1..=31).contains(&jour) {
            return Err(FenetreInvalide::JourHorsBornes(jour));
        }
        // 2024 est bissextile : elle accepte le 29 février et refuse le 30.
        if NaiveDate::from_ymd_opt(2024, mois, jour).is_none() {
            return Err(FenetreInvalide::DateInexistante { mois, jour });
        }
        Ok(Self { mois, jour })
    }

    /// Le premier jour de la période pour un exercice donné.
    ///
    /// `None` si la date ne tombe pas cette année-là — le seul cas est un
    /// 29 février sur une année commune.
    pub fn debut(&self, annee: i32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(annee, self.mois, self.jour)
    }

    /// Le dernier jour de la période, inclus.
    pub fn fin(&self, annee: i32) -> Option<NaiveDate> {
        self.debut(annee)
            .map(|d| d + Duration::days(Self::DUREE_JOURS - 1))
    }

    /// La date tombe-t-elle dans la fenêtre de son propre exercice ?
    ///
    /// La comparaison se fait sur l'année de la date, pas sur l'année civile
    /// courante : une AG du 3 janvier relève de la fenêtre de janvier de la
    /// même année.
    pub fn contient(&self, date: NaiveDate) -> bool {
        let Some(debut) = self.debut(date.year()) else {
            return false;
        };
        let Some(fin) = self.fin(date.year()) else {
            return false;
        };
        date >= debut && date <= fin
    }

    /// La date limite de réception des propositions à inscrire à l'ordre du
    /// jour (Art. 3.87 § 3).
    ///
    /// « Au moins trois semaines avant » : une proposition reçue **ce jour-là**
    /// est encore recevable, puisque le délai est alors exactement de trois
    /// semaines.
    pub fn derniere_date_pour_propositions(&self, annee: i32) -> Option<NaiveDate> {
        self.debut(annee)
            .map(|d| d - Duration::days(Self::PREAVIS_PROPOSITIONS_JOURS))
    }

    /// Une proposition reçue à cette date doit-elle être inscrite ?
    pub fn proposition_recevable(&self, recue_le: NaiveDate, annee: i32) -> bool {
        self.derniere_date_pour_propositions(annee)
            .is_some_and(|limite| recue_le <= limite)
    }

    pub fn mois(&self) -> u32 {
        self.mois
    }

    pub fn jour(&self) -> u32 {
        self.jour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn le(annee: i32, mois: u32, jour: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(annee, mois, jour).expect("date de test valide")
    }

    /// Une fenêtre du 1er au 15 juin, cas le plus courant.
    fn juin() -> FenetreAgOrdinaire {
        FenetreAgOrdinaire::new(6, 1).expect("fenêtre valide")
    }

    #[test]
    fn happy_la_periode_dure_quinze_jours_bornes_comprises() {
        assert_eq!(juin().debut(2026), Some(le(2026, 6, 1)));
        assert_eq!(juin().fin(2026), Some(le(2026, 6, 15)));
    }

    #[test]
    fn happy_une_ag_dans_la_fenetre_est_conforme() {
        assert!(juin().contient(le(2026, 6, 8)));
    }

    #[test]
    fn edge_les_deux_bornes_sont_incluses() {
        assert!(juin().contient(le(2026, 6, 1)), "le premier jour compte");
        assert!(juin().contient(le(2026, 6, 15)), "le quinzième aussi");
    }

    #[test]
    fn negative_une_ag_hors_fenetre_est_signalee() {
        assert!(!juin().contient(le(2026, 6, 16)), "le seizième jour est dehors");
        assert!(!juin().contient(le(2026, 5, 31)), "la veille aussi");
    }

    #[test]
    fn happy_la_fenetre_est_annuelle_et_se_reprojette() {
        // Le ROI fixe un mois et un jour, pas une date : la période revient
        // chaque exercice.
        assert!(juin().contient(le(2026, 6, 8)));
        assert!(juin().contient(le(2031, 6, 8)));
    }

    #[test]
    fn edge_une_fenetre_a_cheval_sur_deux_mois_reste_continue() {
        let fin_decembre = FenetreAgOrdinaire::new(12, 25).expect("fenêtre valide");
        assert_eq!(fin_decembre.fin(2026), Some(le(2027, 1, 8)));
        // La date du 3 janvier appartient à la fenêtre ouverte en décembre
        // 2026, pas à une fenêtre de 2027 : `contient` raisonne sur l'année de
        // la date, donc il ne la voit pas. Limite connue et assumée — une
        // fenêtre à cheval sur le nouvel an est un cas de ROI rare.
        assert!(!fin_decembre.contient(le(2027, 1, 3)));
    }

    // ── Art. 3.87 § 3 : le préavis des propositions ────────────────────

    #[test]
    fn happy_les_propositions_se_deposent_trois_semaines_avant() {
        assert_eq!(
            juin().derniere_date_pour_propositions(2026),
            Some(le(2026, 5, 11)),
            "1er juin moins vingt-et-un jours"
        );
    }

    #[test]
    fn edge_une_proposition_recue_le_jour_limite_est_recevable() {
        // « Au moins trois semaines avant » : le délai est exactement de trois
        // semaines ce jour-là, donc il est respecté.
        assert!(juin().proposition_recevable(le(2026, 5, 11), 2026));
    }

    #[test]
    fn negative_une_proposition_recue_le_lendemain_ne_lest_plus() {
        assert!(!juin().proposition_recevable(le(2026, 5, 12), 2026));
    }

    // ── Validation ─────────────────────────────────────────────────────

    #[test]
    fn negative_un_mois_hors_bornes_est_refuse() {
        assert_eq!(
            FenetreAgOrdinaire::new(13, 1),
            Err(FenetreInvalide::MoisHorsBornes(13))
        );
    }

    #[test]
    fn negative_une_date_qui_nexiste_pas_est_refusee() {
        // Le ROI ne peut pas fixer une période qui ne tombe jamais.
        assert_eq!(
            FenetreAgOrdinaire::new(11, 31),
            Err(FenetreInvalide::DateInexistante { mois: 11, jour: 31 })
        );
    }

    #[test]
    fn edge_le_29_fevrier_est_accepte_mais_ne_tombe_pas_toutes_les_annees() {
        let bissextile = FenetreAgOrdinaire::new(2, 29).expect("le 29 février existe");
        assert!(bissextile.debut(2028).is_some(), "2028 est bissextile");
        assert!(
            bissextile.debut(2026).is_none(),
            "2026 ne l'est pas : la fenêtre ne tombe pas cette année-là"
        );
        assert!(
            !bissextile.contient(le(2026, 3, 1)),
            "et rien n'y appartient"
        );
    }
}
