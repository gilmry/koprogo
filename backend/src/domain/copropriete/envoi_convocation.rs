//! Le mode d'envoi de la convocation.
//!
//! Art. 3.87 § 3, alinéa 3 :
//!
//! > « La convocation est effectuée **par envoi recommandé**, à moins que les
//! > destinataires n'aient accepté, **individuellement, explicitement et par
//! > écrit**, de recevoir la convocation par un autre moyen de communication.
//! > Les convocations envoyées à la **dernière adresse connue du syndic à la
//! > date de l'envoi** sont réputées régulières. »
//!
//! Le délai de quinze jours était appliqué. L'accord au courriel ne l'était
//! pas : le logiciel envoyait par courriel sans que rien ne prouve que le
//! destinataire l'avait accepté.
//!
//! L'enjeu n'est pas formel. Une convocation irrégulière rend l'assemblée
//! attaquable, et ce sont ses décisions — travaux, budgets, mandats — qui
//! tombent avec elle. Un copropriétaire mécontent d'un vote n'a qu'à montrer
//! qu'il n'avait jamais accepté le courriel.
//!
//! Les trois adverbes sont cumulatifs et chacun exclut une pratique courante :
//!
//! - **individuellement** — une clause du règlement d'ordre intérieur ne vaut
//!   pas accord. Chaque destinataire donne le sien ;
//! - **explicitement** — cocher une case d'inscription ne suffit pas si elle ne
//!   porte pas sur ce point précis ;
//! - **par écrit** — un accord verbal en assemblée ne laisse rien à produire
//!   le jour où il est contesté.
//!
//! La dernière phrase protège le syndic en sens inverse : il répond de la
//! dernière adresse **qu'il connaissait à la date de l'envoi**, pas de celle
//! qu'un copropriétaire lui a communiquée le lendemain. L'adresse doit donc
//! être figée au moment de l'envoi, pas relue après coup.
//!
//! Voir issue #749.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Par quel canal la convocation part.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeDenvoi {
    /// Le mode de droit commun, toujours valable.
    Recommande,
    /// Tout autre moyen : courriel, plateforme, remise en main propre.
    AutreMoyen,
}

/// L'accord donné par un destinataire à un autre mode d'envoi.
///
/// Les trois qualités de l'article sont des champs, pas un booléen : un accord
/// qui n'en remplit que deux n'en est pas un, et il faut pouvoir dire lequel
/// manque.
#[derive(Debug, Clone, PartialEq)]
pub struct AccordAutreMoyen {
    pub destinataire: Uuid,
    /// Donné par ce destinataire lui-même, et non par une clause générale.
    pub individuel: bool,
    /// Portant explicitement sur le mode d'envoi des convocations.
    pub explicite: bool,
    /// Consigné par écrit, et donc produisible.
    pub ecrit: bool,
    pub donne_le: DateTime<Utc>,
}

impl AccordAutreMoyen {
    /// L'accord remplit-il les trois conditions cumulatives ?
    pub fn est_valable(&self) -> bool {
        self.individuel && self.explicite && self.ecrit
    }

    /// Ce qui lui manque, nommé.
    pub fn qualites_manquantes(&self) -> Vec<&'static str> {
        let mut manques = Vec::new();
        if !self.individuel {
            manques.push("individuel");
        }
        if !self.explicite {
            manques.push("explicite");
        }
        if !self.ecrit {
            manques.push("écrit");
        }
        manques
    }
}

/// Pourquoi un envoi est irrégulier.
#[derive(Debug, Clone, PartialEq)]
pub enum EnvoiIrregulier {
    /// Envoyé autrement que par recommandé, sans aucun accord.
    SansAccord { destinataire: Uuid },
    /// Un accord existe mais ne remplit pas les trois conditions.
    AccordIncomplet {
        destinataire: Uuid,
        manques: Vec<&'static str>,
    },
    /// L'accord est postérieur à l'envoi.
    ///
    /// Il ne régularise rien : au moment de l'envoi, le destinataire n'avait
    /// pas encore accepté.
    AccordPosterieurALenvoi {
        destinataire: Uuid,
        envoye_le: DateTime<Utc>,
        accord_donne_le: DateTime<Utc>,
    },
}

impl std::fmt::Display for EnvoiIrregulier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SansAccord { destinataire } => write!(
                f,
                "Art. 3.87 § 3 : convocation envoyée à {destinataire} par un autre moyen que \
                 le recommandé, sans accord de sa part."
            ),
            Self::AccordIncomplet {
                destinataire,
                manques,
            } => write!(
                f,
                "Art. 3.87 § 3 : l'accord de {destinataire} n'est pas {} — les trois \
                 qualités sont cumulatives.",
                manques.join(", ni ")
            ),
            Self::AccordPosterieurALenvoi {
                destinataire,
                envoye_le,
                accord_donne_le,
            } => write!(
                f,
                "Art. 3.87 § 3 : convocation envoyée à {destinataire} le {} alors que son \
                 accord date du {} — au moment de l'envoi, il n'avait pas accepté.",
                envoye_le.date_naive(),
                accord_donne_le.date_naive()
            ),
        }
    }
}

/// L'envoi est-il régulier au regard de l'Art. 3.87 § 3 ?
///
/// Le recommandé l'est toujours, sans condition : c'est le mode de droit
/// commun.
pub fn envoi_regulier(
    destinataire: Uuid,
    mode: ModeDenvoi,
    envoye_le: DateTime<Utc>,
    accord: Option<&AccordAutreMoyen>,
) -> Result<(), EnvoiIrregulier> {
    if mode == ModeDenvoi::Recommande {
        return Ok(());
    }
    let Some(accord) = accord else {
        return Err(EnvoiIrregulier::SansAccord { destinataire });
    };
    if !accord.est_valable() {
        return Err(EnvoiIrregulier::AccordIncomplet {
            destinataire,
            manques: accord.qualites_manquantes(),
        });
    }
    if accord.donne_le > envoye_le {
        return Err(EnvoiIrregulier::AccordPosterieurALenvoi {
            destinataire,
            envoye_le,
            accord_donne_le: accord.donne_le,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    fn accord_complet(destinataire: Uuid, donne_il_y_a: i64) -> AccordAutreMoyen {
        AccordAutreMoyen {
            destinataire,
            individuel: true,
            explicite: true,
            ecrit: true,
            donne_le: il_y_a(donne_il_y_a),
        }
    }

    #[test]
    fn happy_le_recommande_est_toujours_regulier() {
        assert!(envoi_regulier(Uuid::new_v4(), ModeDenvoi::Recommande, il_y_a(20), None).is_ok());
    }

    #[test]
    fn happy_un_courriel_couvert_par_un_accord_complet_est_regulier() {
        let d = Uuid::new_v4();
        assert!(envoi_regulier(
            d,
            ModeDenvoi::AutreMoyen,
            il_y_a(20),
            Some(&accord_complet(d, 400))
        )
        .is_ok());
    }

    /// Le cas constaté : on envoyait par courriel sans rien prouver.
    #[test]
    fn security_un_courriel_sans_accord_rend_la_convocation_irreguliere() {
        let d = Uuid::new_v4();
        assert_eq!(
            envoi_regulier(d, ModeDenvoi::AutreMoyen, il_y_a(20), None),
            Err(EnvoiIrregulier::SansAccord { destinataire: d })
        );
    }

    /// Une clause du règlement d'ordre intérieur ne vaut pas accord :
    /// l'article exige qu'il soit **individuel**.
    #[test]
    fn security_une_clause_generale_du_roi_ne_vaut_pas_accord() {
        let d = Uuid::new_v4();
        let clause = AccordAutreMoyen {
            individuel: false,
            ..accord_complet(d, 400)
        };
        assert_eq!(
            envoi_regulier(d, ModeDenvoi::AutreMoyen, il_y_a(20), Some(&clause)),
            Err(EnvoiIrregulier::AccordIncomplet {
                destinataire: d,
                manques: vec!["individuel"]
            })
        );
    }

    /// Cocher une case d'inscription ne suffit pas si elle ne porte pas sur ce
    /// point précis : l'accord doit être **explicite**.
    #[test]
    fn security_une_case_cochee_a_linscription_ne_suffit_pas() {
        let d = Uuid::new_v4();
        let vague = AccordAutreMoyen {
            explicite: false,
            ..accord_complet(d, 400)
        };
        assert!(matches!(
            envoi_regulier(d, ModeDenvoi::AutreMoyen, il_y_a(20), Some(&vague)),
            Err(EnvoiIrregulier::AccordIncomplet { .. })
        ));
    }

    /// Un accord verbal en assemblée ne laisse rien à produire le jour où il
    /// est contesté : il doit être **écrit**.
    #[test]
    fn security_un_accord_verbal_ne_suffit_pas() {
        let d = Uuid::new_v4();
        let verbal = AccordAutreMoyen {
            ecrit: false,
            ..accord_complet(d, 400)
        };
        assert!(matches!(
            envoi_regulier(d, ModeDenvoi::AutreMoyen, il_y_a(20), Some(&verbal)),
            Err(EnvoiIrregulier::AccordIncomplet { .. })
        ));
    }

    #[test]
    fn negative_les_trois_qualites_manquantes_sont_toutes_nommees() {
        let d = Uuid::new_v4();
        let rien = AccordAutreMoyen {
            individuel: false,
            explicite: false,
            ecrit: false,
            ..accord_complet(d, 400)
        };
        assert_eq!(
            rien.qualites_manquantes(),
            vec!["individuel", "explicite", "écrit"]
        );
    }

    /// @security — un accord donné après coup ne régularise pas l'envoi.
    ///
    /// Au moment où la convocation est partie, le destinataire n'avait pas
    /// accepté. Le régulariser après reviendrait à valider rétroactivement une
    /// assemblée déjà tenue.
    #[test]
    fn security_un_accord_posterieur_ne_regularise_pas_lenvoi() {
        let d = Uuid::new_v4();
        let tardif = accord_complet(d, 5);
        assert!(matches!(
            envoi_regulier(d, ModeDenvoi::AutreMoyen, il_y_a(20), Some(&tardif)),
            Err(EnvoiIrregulier::AccordPosterieurALenvoi { .. })
        ));
    }

    /// @edge — accord donné le jour même de l'envoi : il vaut.
    #[test]
    fn edge_un_accord_du_jour_meme_couvre_lenvoi() {
        let d = Uuid::new_v4();
        let moment = il_y_a(20);
        let accord = AccordAutreMoyen {
            donne_le: moment,
            ..accord_complet(d, 0)
        };
        assert!(envoi_regulier(d, ModeDenvoi::AutreMoyen, moment, Some(&accord)).is_ok());
    }
}
