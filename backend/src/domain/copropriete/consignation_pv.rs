//! La consignation et la transmission du procès-verbal.
//!
//! Art. 3.87 § 12 :
//!
//! > « Le syndic **consigne** les décisions visées aux paragraphes 10 et 11
//! > dans le **registre prévu à l'article 3.93, § 4**, dans les **trente
//! > jours** suivant l'assemblée générale, et **transmet** celles-ci, **dans
//! > le même délai**, à tout titulaire d'un droit réel sur un lot disposant
//! > [...] du droit de vote à l'assemblée générale, et **aux autres syndics**.
//! > Si l'un d'eux n'a pas reçu le procès-verbal dans le délai fixé, il en
//! > informe le syndic par écrit. »
//!
//! Deux obligations distinctes, un seul délai. On peut satisfaire l'une sans
//! l'autre — consigner au registre sans rien envoyer est un cas courant, et
//! reste un manquement.
//!
//! La transmission se suit **destinataire par destinataire**. Un envoi global
//! qui a échoué pour un seul copropriétaire est un défaut vis-à-vis de
//! celui-là, et c'est lui qui pourra s'en prévaloir : la loi lui donne
//! d'ailleurs le droit d'en informer le syndic par écrit, ce qui suppose qu'il
//! puisse constater le manque.
//!
//! Le délai court **depuis l'assemblée**, pas depuis la signature du PV ni
//! depuis sa rédaction. Un syndic qui tarde à rédiger consomme son propre
//! délai.
//!
//! Le § 11 est inclus : les décisions prises à l'unanimité par écrit, sans
//! assemblée, suivent le même régime. Leur point de départ est alors la date
//! de la décision.
//!
//! Voir issue #744.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Le délai unique de l'Art. 3.87 § 12.
pub const DELAI_JOURS: i64 = 30;

/// Où en est le syndic de ses deux obligations.
#[derive(Debug, Clone, PartialEq)]
pub struct EtatConsignation {
    /// Date de l'assemblée, ou de la décision écrite unanime (§ 11).
    pub tenue_le: DateTime<Utc>,
    /// Date limite, commune aux deux obligations.
    pub echeance: DateTime<Utc>,
    /// Date de consignation au registre de l'Art. 3.93 § 4.
    pub consigne_le: Option<DateTime<Utc>>,
    /// Destinataires à qui le PV n'a pas encore été transmis.
    pub non_transmis: Vec<Uuid>,
}

impl EtatConsignation {
    pub fn consignation_faite(&self) -> bool {
        self.consigne_le.is_some()
    }

    pub fn transmission_faite(&self) -> bool {
        self.non_transmis.is_empty()
    }

    /// Le syndic est-il en défaut à cette date ?
    ///
    /// Avant l'échéance, un PV non consigné n'est pas un manquement : le
    /// syndic est simplement dans son délai.
    pub fn en_defaut_le(&self, moment: DateTime<Utc>) -> bool {
        moment > self.echeance && !(self.consignation_faite() && self.transmission_faite())
    }

    /// Ce qu'il reste à faire, dit en clair.
    pub fn manquements(&self, moment: DateTime<Utc>) -> Vec<String> {
        if !self.en_defaut_le(moment) {
            return Vec::new();
        }
        let mut manques = Vec::new();
        if !self.consignation_faite() {
            manques.push(format!(
                "Art. 3.87 § 12 : procès-verbal non consigné au registre (échéance {})",
                self.echeance.date_naive()
            ));
        }
        if !self.transmission_faite() {
            manques.push(format!(
                "Art. 3.87 § 12 : procès-verbal non transmis à {} destinataire(s) (échéance {})",
                self.non_transmis.len(),
                self.echeance.date_naive()
            ));
        }
        manques
    }
}

/// L'échéance des deux obligations : trente jours après l'assemblée.
pub fn echeance(tenue_le: DateTime<Utc>) -> DateTime<Utc> {
    tenue_le + Duration::days(DELAI_JOURS)
}

/// Fait le point sur une assemblée.
///
/// `destinataires` sont tous ceux à qui la loi impose la transmission :
/// titulaires d'un droit réel disposant du droit de vote, et les autres
/// syndics. `transmis_a` sont ceux qui l'ont effectivement reçu.
pub fn etat(
    tenue_le: DateTime<Utc>,
    consigne_le: Option<DateTime<Utc>>,
    destinataires: &[Uuid],
    transmis_a: &[Uuid],
) -> EtatConsignation {
    EtatConsignation {
        tenue_le,
        echeance: echeance(tenue_le),
        consigne_le,
        non_transmis: destinataires
            .iter()
            .filter(|d| !transmis_a.contains(d))
            .copied()
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    #[test]
    fn happy_lecheance_tombe_trente_jours_apres_lassemblee() {
        let ag = il_y_a(0);
        assert_eq!(echeance(ag), ag + Duration::days(30));
    }

    #[test]
    fn happy_tout_fait_dans_les_temps_ne_signale_rien() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let etat = etat(il_y_a(10), Some(il_y_a(8)), &[a, b], &[a, b]);

        assert!(etat.consignation_faite());
        assert!(etat.transmission_faite());
        assert!(!etat.en_defaut_le(Utc::now()));
        assert!(etat.manquements(Utc::now()).is_empty());
    }

    /// Avant l'échéance, un PV non consigné n'est pas un manquement.
    #[test]
    fn happy_dans_le_delai_rien_nest_encore_reproche() {
        let etat = etat(il_y_a(10), None, &[Uuid::new_v4()], &[]);
        assert!(!etat.en_defaut_le(Utc::now()));
    }

    /// Les deux obligations sont distinctes : consigner sans transmettre reste
    /// un manquement.
    #[test]
    fn negative_consigner_sans_transmettre_reste_un_defaut() {
        let destinataire = Uuid::new_v4();
        let etat = etat(il_y_a(40), Some(il_y_a(35)), &[destinataire], &[]);

        assert!(etat.consignation_faite());
        assert!(!etat.transmission_faite());
        assert!(etat.en_defaut_le(Utc::now()));

        let manques = etat.manquements(Utc::now());
        assert_eq!(manques.len(), 1);
        assert!(manques[0].contains("non transmis"));
    }

    #[test]
    fn negative_transmettre_sans_consigner_aussi() {
        let destinataire = Uuid::new_v4();
        let etat = etat(il_y_a(40), None, &[destinataire], &[destinataire]);

        let manques = etat.manquements(Utc::now());
        assert_eq!(manques.len(), 1);
        assert!(manques[0].contains("non consigné"));
    }

    #[test]
    fn negative_ne_rien_faire_cumule_les_deux_manquements() {
        let etat = etat(il_y_a(40), None, &[Uuid::new_v4()], &[]);
        assert_eq!(etat.manquements(Utc::now()).len(), 2);
    }

    /// La transmission se suit destinataire par destinataire.
    ///
    /// Un envoi global qui a échoué pour un seul copropriétaire est un défaut
    /// vis-à-vis de celui-là, et c'est lui qui pourra s'en prévaloir.
    #[test]
    fn security_un_seul_destinataire_oublie_suffit_a_faire_defaut() {
        let servis = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let oublie = Uuid::new_v4();
        let mut tous = servis.clone();
        tous.push(oublie);

        let etat = etat(il_y_a(40), Some(il_y_a(35)), &tous, &servis);

        assert_eq!(etat.non_transmis, vec![oublie]);
        assert!(etat.en_defaut_le(Utc::now()));
    }

    /// @edge — le trentième jour, le syndic est encore dans son délai.
    #[test]
    fn edge_le_jour_de_lecheance_nest_pas_encore_un_defaut() {
        let ag = il_y_a(30);
        let etat = etat(ag, None, &[Uuid::new_v4()], &[]);
        assert!(
            !etat.en_defaut_le(etat.echeance),
            "« dans les trente jours » inclut le trentième"
        );
        assert!(etat.en_defaut_le(etat.echeance + Duration::seconds(1)));
    }

    /// Le délai court depuis l'assemblée, pas depuis la rédaction du PV.
    ///
    /// Un syndic qui tarde à rédiger consomme son propre délai.
    #[test]
    fn edge_le_delai_court_depuis_lassemblee() {
        let ag = il_y_a(40);
        let etat = etat(ag, Some(il_y_a(2)), &[], &[]);
        assert!(
            etat.consigne_le.unwrap() > etat.echeance,
            "consigné trente-huit jours après l'AG : hors délai"
        );
    }

    /// @negative — une assemblée sans destinataire n'a rien à transmettre.
    ///
    /// Le cas se produit sur les décisions écrites unanimes du § 11, prises
    /// par tous : il ne reste personne à informer.
    #[test]
    fn negative_sans_destinataire_seule_la_consignation_compte() {
        let etat = etat(il_y_a(40), Some(il_y_a(35)), &[], &[]);
        assert!(etat.transmission_faite());
        assert!(!etat.en_defaut_le(Utc::now()));
    }
}
