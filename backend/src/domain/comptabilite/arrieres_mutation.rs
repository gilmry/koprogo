//! Les arriérés retenus lors de la transmission d'un lot.
//!
//! Art. 3.95 :
//!
//! > « Lors de la passation de l'acte authentique, le notaire instrumentant
//! > doit **retenir, sur les sommes dues, les arriérés des charges ordinaires
//! > et extraordinaires en ce compris les frais de récupération judiciaire et
//! > extrajudiciaire des charges**, dus par le copropriétaire sortant, ainsi
//! > que **les frais de transmission des informations** requises en vertu de
//! > l'article 3.94, §§ 1er à 3. »
//!
//! L'état daté portait les arriérés, mais pas la **règle de composition** :
//! quatre postes, et non un seul. Le notaire retient ce que le syndic lui a
//! chiffré ; si le chiffre est incomplet, l'ACP perd la différence, faute de
//! pouvoir la réclamer après coup à un vendeur qui a quitté l'immeuble.
//!
//! Les postes les plus oubliés sont les frais de récupération — le
//! recommandé, l'huissier, l'avocat — et les frais de transmission eux-mêmes,
//! que le vendeur doit et que le syndic avance.
//!
//! **La procédure de contestation compte en jours ouvrables**, et l'article le
//! dit deux fois :
//!
//! > « le notaire instrumentant en avise le syndic par envoi recommandé envoyé
//! > dans les **trois jours ouvrables** qui suivent la passation de l'acte »
//!
//! > « A défaut de saisie-arrêt [...] notifiée dans les **vingt jours
//! > ouvrables** qui suivent la date de l'envoi recommandé [...], le notaire
//! > peut valablement payer le montant des arriérés au copropriétaire
//! > sortant. »
//!
//! C'est la confirmation directe de l'arbitrage tenu sur l'Art. 3.94 : quand
//! le législateur veut des jours ouvrables, il l'écrit. Son silence ailleurs
//! est délibéré, et les délais de l'état daté se comptent donc en jours
//! calendaires.
//!
//! Sur la définition retenue du jour ouvrable, voir [`jours_ouvrables`] : elle
//! est explicite parce qu'elle n'est pas unanime.
//!
//! Voir issue #755.

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use rust_decimal::Decimal;

/// Ce que le notaire doit retenir, poste par poste.
///
/// Les quatre postes sont séparés parce que trois d'entre eux sont
/// régulièrement oubliés, et qu'un total opaque ne permet ni de le vérifier ni
/// de le contester.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArrieresARetenir {
    /// Arriérés de charges **ordinaires**.
    pub charges_ordinaires: Decimal,
    /// Arriérés de charges **extraordinaires**.
    pub charges_extraordinaires: Decimal,
    /// Frais de récupération judiciaire et extrajudiciaire : recommandés,
    /// huissier, avocat. Poste le plus souvent omis.
    pub frais_de_recuperation: Decimal,
    /// Frais de transmission des informations de l'Art. 3.94, §§ 1er à 3 —
    /// ceux que le syndic avance et que le vendeur doit.
    pub frais_de_transmission: Decimal,
}

impl ArrieresARetenir {
    /// Le total à retenir sur le prix.
    pub fn total(&self) -> Decimal {
        self.charges_ordinaires
            + self.charges_extraordinaires
            + self.frais_de_recuperation
            + self.frais_de_transmission
    }

    /// Les postes non nuls, nommés, pour que le décompte remis au notaire soit
    /// lisible et contestable.
    pub fn detail(&self) -> Vec<(&'static str, Decimal)> {
        [
            ("Charges ordinaires", self.charges_ordinaires),
            ("Charges extraordinaires", self.charges_extraordinaires),
            ("Frais de récupération", self.frais_de_recuperation),
            (
                "Frais de transmission (Art. 3.94)",
                self.frais_de_transmission,
            ),
        ]
        .into_iter()
        .filter(|(_, montant)| !montant.is_zero())
        .collect()
    }
}

/// La définition du jour ouvrable retenue ici : **tout jour sauf le dimanche
/// et les jours fériés légaux**.
///
/// Le samedi compte, conformément à la tradition civiliste belge, qui le
/// distingue du « jour ouvré » du droit du travail. La convention est écrite
/// ici parce qu'elle n'est **pas unanime** et qu'un délai de vingt jours mal
/// compté fait perdre à l'ACP le droit de saisir.
///
/// Les jours fériés sont passés en paramètre plutôt que codés en dur : ils
/// varient d'une année à l'autre, et une liste figée dans le code se périmerait
/// en silence.
pub fn jours_ouvrables(depart: NaiveDate, nombre: i64, feries: &[NaiveDate]) -> NaiveDate {
    let mut date = depart;
    let mut restants = nombre;
    while restants > 0 {
        date += Duration::days(1);
        if date.weekday() != Weekday::Sun && !feries.contains(&date) {
            restants -= 1;
        }
    }
    date
}

/// Délai laissé au notaire pour aviser le syndic d'une contestation.
pub const DELAI_AVIS_CONTESTATION_OUVRABLES: i64 = 3;

/// Délai au-delà duquel, sans saisie-arrêt, le notaire peut payer le vendeur.
pub const DELAI_SAISIE_ARRET_OUVRABLES: i64 = 20;

/// Date limite pour que le notaire avise le syndic d'une contestation.
pub fn limite_avis_contestation(passation: NaiveDate, feries: &[NaiveDate]) -> NaiveDate {
    jours_ouvrables(passation, DELAI_AVIS_CONTESTATION_OUVRABLES, feries)
}

/// Date à partir de laquelle, faute de saisie-arrêt, le notaire peut payer les
/// arriérés au copropriétaire sortant.
///
/// Le compte part de **l'envoi du recommandé**, pas de la passation de l'acte.
pub fn liberation_des_fonds(envoi_recommande: NaiveDate, feries: &[NaiveDate]) -> NaiveDate {
    jours_ouvrables(envoi_recommande, DELAI_SAISIE_ARRET_OUVRABLES, feries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn le(annee: i32, mois: u32, jour: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(annee, mois, jour).expect("date valide")
    }

    #[test]
    fn happy_le_total_additionne_les_quatre_postes() {
        let arrieres = ArrieresARetenir {
            charges_ordinaires: dec!(1200),
            charges_extraordinaires: dec!(3500),
            frais_de_recuperation: dec!(280.50),
            frais_de_transmission: dec!(125),
        };
        assert_eq!(arrieres.total(), dec!(5105.50));
    }

    /// Le poste le plus souvent oublié, et ce qu'il coûte de l'oublier.
    ///
    /// Si le chiffre remis au notaire est incomplet, l'ACP perd la différence :
    /// elle ne pourra pas la réclamer après coup à un vendeur qui a quitté
    /// l'immeuble.
    #[test]
    fn security_oublier_les_frais_de_recuperation_ampute_la_retenue() {
        let complet = ArrieresARetenir {
            charges_ordinaires: dec!(1200),
            frais_de_recuperation: dec!(280.50),
            ..Default::default()
        };
        let ampute = ArrieresARetenir {
            charges_ordinaires: dec!(1200),
            ..Default::default()
        };
        assert_eq!(complet.total() - ampute.total(), dec!(280.50));
    }

    #[test]
    fn happy_le_detail_ne_montre_que_les_postes_non_nuls() {
        let arrieres = ArrieresARetenir {
            charges_ordinaires: dec!(1200),
            frais_de_transmission: dec!(125),
            ..Default::default()
        };
        let detail = arrieres.detail();
        assert_eq!(detail.len(), 2);
        assert_eq!(detail[0].0, "Charges ordinaires");
        assert_eq!(detail[1].0, "Frais de transmission (Art. 3.94)");
    }

    #[test]
    fn negative_un_vendeur_a_jour_ne_doit_rien() {
        let rien = ArrieresARetenir::default();
        assert_eq!(rien.total(), Decimal::ZERO);
        assert!(rien.detail().is_empty());
    }

    // ── Les jours ouvrables ────────────────────────────────────────

    /// Le samedi compte, le dimanche non.
    ///
    /// La convention est écrite parce qu'elle n'est pas unanime : le « jour
    /// ouvrable » civiliste n'est pas le « jour ouvré » du droit du travail.
    #[test]
    fn happy_le_samedi_est_ouvrable_le_dimanche_ne_lest_pas() {
        // Le 2026-06-04 est un jeudi.
        let jeudi = le(2026, 6, 4);
        assert_eq!(jeudi.weekday(), Weekday::Thu);

        // +3 ouvrables : vendredi, samedi, lundi (le dimanche saute).
        assert_eq!(jours_ouvrables(jeudi, 3, &[]), le(2026, 6, 8));
    }

    #[test]
    fn happy_un_ferie_repousse_dun_jour() {
        let jeudi = le(2026, 6, 4);
        let vendredi_ferie = le(2026, 6, 5);
        assert_eq!(
            jours_ouvrables(jeudi, 3, &[vendredi_ferie]),
            le(2026, 6, 9),
            "un jour de plus que sans le férié"
        );
    }

    #[test]
    fn happy_le_notaire_a_trois_jours_ouvrables_pour_aviser() {
        let passation = le(2026, 6, 4);
        assert_eq!(limite_avis_contestation(passation, &[]), le(2026, 6, 8));
    }

    /// Le compte des vingt jours part de **l'envoi du recommandé**, pas de la
    /// passation de l'acte.
    #[test]
    fn happy_les_vingt_jours_partent_de_lenvoi_du_recommande() {
        let envoi = le(2026, 6, 8);
        let liberation = liberation_des_fonds(envoi, &[]);
        assert!(liberation > envoi);
        // Vingt jours ouvrables représentent plus de trois semaines
        // calendaires, puisque les dimanches ne comptent pas.
        assert!((liberation - envoi).num_days() >= 23);
    }

    /// @security — compter en jours calendaires libérerait les fonds trop tôt.
    ///
    /// L'ACP perdrait le droit de saisir sur des arriérés qu'elle est encore
    /// dans les temps de contester.
    #[test]
    fn security_le_calcul_calendaire_libererait_les_fonds_trop_tot() {
        let envoi = le(2026, 6, 8);
        let en_ouvrables = liberation_des_fonds(envoi, &[]);
        let en_calendaires = envoi + Duration::days(20);
        assert!(
            en_ouvrables > en_calendaires,
            "les jours ouvrables allongent le délai, ils ne le raccourcissent pas"
        );
    }
}
