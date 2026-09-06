//! Peut-on encore convoquer régulièrement pour cette date ?
//!
//! ── Le défaut que ce module ferme ──────────────────────────────────────────
//!
//! Recette du 2026-09-06 (RN-9). Un syndic crée une assemblée pour le
//! 20 septembre. Il clique « Créer une convocation » et reçoit :
//!
//!     Meeting date too soon. Ordinary meeting requires 15 days notice.
//!     Minimum send date would be 2026-09-05 10:00
//!
//! **La règle est juste** — Art. 3.87 § 3 : « Sauf dans les cas d'urgence, la
//! convocation est communiquée quinze jours au moins avant la date de
//! l'assemblée. » Le défaut est sa **temporalité** : le contrôle arrive au
//! moment où il ne reste plus qu'à subir.
//!
//! L'application a laissé créer cette assemblée sans rien dire, puis refuse de
//! la convoquer. Le syndic n'en sort qu'en la supprimant.
//!
//! ── Pourquoi avertir plutôt que refuser ────────────────────────────────────
//!
//! Interdire la création serait faux, et pour trois raisons vérifiables :
//!
//!   1. **L'urgence est prévue par le texte lui-même** — « sauf dans les cas
//!      d'urgence ». Une assemblée convoquée dans l'urgence est régulière.
//!   2. **Une assemblée peut être encodée après coup**, pour tenir le registre
//!      d'une réunion déjà tenue.
//!   3. **Une seconde convocation** (Art. 3.87 § 5) suit une première dont le
//!      quorum a manqué : sa date est contrainte par l'échec précédent.
//!
//! Le rôle de ce module n'est donc pas de dire non, mais de dire **quand**,
//! assez tôt pour qu'on puisse encore changer la date.
//!
//! Voir #780, verrou 1.

use super::convocation::ConvocationType;
use chrono::{DateTime, Duration, Utc};

/// Ce que l'on peut dire d'une date d'assemblée au moment où on la saisit.
#[derive(Debug, Clone, PartialEq)]
pub enum DelaiDeConvocation {
    /// La convocation peut encore partir dans les temps.
    ///
    /// Porte la date limite d'envoi, pour qu'un écran puisse l'afficher plutôt
    /// que de laisser l'utilisateur la calculer.
    Tenable { date_limite_envoi: DateTime<Utc> },

    /// Le délai de quinze jours ne peut plus être tenu.
    ///
    /// Ce n'est pas un refus : l'assemblée reste créable. C'est un avertissement
    /// à donner **à la saisie**, avec ce qu'il faudrait pour le tenir.
    TropCourt {
        date_limite_envoi: DateTime<Utc>,
        jours_manquants: i64,
    },

    /// La date est déjà passée : on encode une assemblée tenue.
    ///
    /// Aucun avertissement de délai n'a de sens ici, et en produire un
    /// apprendrait à ignorer les avertissements.
    DejaTenue,
}

/// Le délai est-il tenable pour cette assemblée, à cet instant ?
///
/// `maintenant` est passé en paramètre plutôt que lu de l'horloge : une règle
/// qui lit l'heure ne se teste qu'en attendant, et une règle qu'on ne peut pas
/// tester au bord ne se teste pas du tout.
pub fn evaluer(
    date_assemblee: DateTime<Utc>,
    type_de_convocation: &ConvocationType,
    maintenant: DateTime<Utc>,
) -> DelaiDeConvocation {
    if date_assemblee <= maintenant {
        return DelaiDeConvocation::DejaTenue;
    }

    let jours = type_de_convocation.minimum_notice_days();
    let date_limite_envoi = date_assemblee - Duration::days(jours);

    if maintenant <= date_limite_envoi {
        DelaiDeConvocation::Tenable { date_limite_envoi }
    } else {
        // Le nombre de jours dont il faudrait reculer la date d'assemblée.
        // On arrondit vers le haut : à douze heures près, il manque un jour.
        let manque = maintenant - date_limite_envoi;
        let jours_manquants = (manque.num_seconds() as f64 / 86_400.0).ceil() as i64;
        DelaiDeConvocation::TropCourt {
            date_limite_envoi,
            jours_manquants: jours_manquants.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(jours: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_757_000_000, 0).unwrap() + Duration::days(jours)
    }

    #[test]
    fn happy_une_assemblee_dans_un_mois_est_tenable() {
        let r = evaluer(t(30), &ConvocationType::Ordinary, t(0));
        match r {
            DelaiDeConvocation::Tenable { date_limite_envoi } => {
                assert_eq!(date_limite_envoi, t(15));
            }
            autre => panic!("attendu Tenable, obtenu {autre:?}"),
        }
    }

    /// La borne exacte : envoyer le quinzième jour AVANT est régulier.
    ///
    /// « Quinze jours au moins » se compte en jours pleins ; le jour de l'envoi
    /// compte. Se tromper d'un jour ici rendrait irrégulière une convocation
    /// qui ne l'est pas, ou l'inverse.
    #[test]
    fn edge_la_borne_des_quinze_jours_est_tenable() {
        let r = evaluer(t(15), &ConvocationType::Ordinary, t(0));
        assert!(matches!(r, DelaiDeConvocation::Tenable { .. }));
    }

    #[test]
    fn negative_un_jour_de_moins_ne_tient_plus() {
        let r = evaluer(t(15), &ConvocationType::Ordinary, t(1));
        match r {
            DelaiDeConvocation::TropCourt {
                jours_manquants, ..
            } => assert_eq!(jours_manquants, 1),
            autre => panic!("attendu TropCourt, obtenu {autre:?}"),
        }
    }

    /// Le cas exact de la recette : une assemblée à cinq jours.
    #[test]
    fn negative_le_cas_de_la_recette_manque_de_dix_jours() {
        let r = evaluer(t(5), &ConvocationType::Ordinary, t(0));
        match r {
            DelaiDeConvocation::TropCourt {
                jours_manquants,
                date_limite_envoi,
            } => {
                assert_eq!(jours_manquants, 10);
                assert_eq!(date_limite_envoi, t(-10));
            }
            autre => panic!("attendu TropCourt, obtenu {autre:?}"),
        }
    }

    /// Une assemblée déjà tenue ne reçoit aucun avertissement.
    ///
    /// Avertir ici apprendrait à ignorer les avertissements — c'est ainsi
    /// qu'un garde-fou finit désactivé.
    #[test]
    fn edge_une_assemblee_passee_ne_declenche_aucun_avertissement() {
        assert_eq!(
            evaluer(t(-1), &ConvocationType::Ordinary, t(0)),
            DelaiDeConvocation::DejaTenue
        );
    }

    /// L'instant exact de l'assemblée compte comme tenue, pas comme à venir.
    #[test]
    fn edge_linstant_meme_de_lassemblee_compte_comme_tenue() {
        assert_eq!(
            evaluer(t(0), &ConvocationType::Ordinary, t(0)),
            DelaiDeConvocation::DejaTenue
        );
    }

    /// Les trois types partagent le même délai (Art. 3.87 § 3 et § 5).
    ///
    /// Ce test existe parce que la loi de 2019 a UNIFIÉ ces délais : la
    /// seconde convocation obéissait autrefois à une règle distincte, et un
    /// lecteur pressé pourrait « rétablir » l'ancienne.
    #[test]
    fn happy_les_trois_types_partagent_le_delai_de_quinze_jours() {
        for type_de in [
            ConvocationType::Ordinary,
            ConvocationType::Extraordinary,
            ConvocationType::SecondConvocation,
        ] {
            assert!(
                matches!(
                    evaluer(t(15), &type_de, t(0)),
                    DelaiDeConvocation::Tenable { .. }
                ),
                "{type_de:?} devrait tenir à quinze jours"
            );
            assert!(
                matches!(
                    evaluer(t(14), &type_de, t(0)),
                    DelaiDeConvocation::TropCourt { .. }
                ),
                "{type_de:?} ne devrait pas tenir à quatorze jours"
            );
        }
    }
}
