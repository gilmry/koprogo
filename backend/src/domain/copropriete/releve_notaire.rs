//! Le relevé des dettes réclamé par le notaire.
//!
//! Art. 3.89 § 5, 5° — le syndic est chargé :
//!
//! > « de fournir le **relevé des dettes** visées à l'article 3.94, § 2, dans
//! > les **trente jours** de la demande qui lui en est faite **par le
//! > notaire**. »
//!
//! Le délai est documenté depuis longtemps et l'état daté porte les arriérés.
//! Ce qui manquait, c'est le **suivi** : une demande de notaire n'était pas un
//! objet du domaine, donc le dépassement était invisible.
//!
//! Il n'est pas anodin. Ce relevé conditionne une vente : le notaire ne peut
//! pas passer l'acte sans savoir ce que le vendeur doit à l'association. Un
//! syndic qui laisse filer bloque une transaction entre tiers et engage sa
//! responsabilité, souvent sans s'en apercevoir — c'est précisément ce qu'un
//! suivi rend visible **avant** l'échéance plutôt qu'après la plainte.
//!
//! À distinguer des délais de l'Art. 3.94, qui portent sur l'**état daté**
//! lui-même : quinze jours calendaires pour une demande simple (§ 1er), trente
//! pour une demande notariale par recommandé (§ 2). Ici, c'est le relevé des
//! dettes du § 2 qui est visé, et son délai court depuis la demande du
//! notaire.
//!
//! Voir issue #752.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Le délai de l'Art. 3.89 § 5, 5°, en jours calendaires.
///
/// Calendaires, comme tous les délais de ce chapitre : l'Art. 3.31 § 2 dit
/// « jour ouvrable » quand il le veut, et son silence ailleurs est délibéré.
pub const DELAI_JOURS: i64 = 30;

/// Une demande de relevé adressée par un notaire.
#[derive(Debug, Clone, PartialEq)]
pub struct DemandeDeReleve {
    pub id: Uuid,
    pub notaire: String,
    /// Le lot dont la vente est en cours.
    pub unit_id: Uuid,
    /// Date de réception par le syndic — c'est elle qui fait courir le délai.
    pub recue_le: DateTime<Utc>,
    pub echeance: DateTime<Utc>,
    /// Date de fourniture du relevé, si le syndic l'a fourni.
    pub fournie_le: Option<DateTime<Utc>>,
}

/// L'état d'une demande, dit du point de vue du syndic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatDemande {
    /// Fournie dans les temps.
    HonoreeATemps,
    /// Fournie, mais après l'échéance.
    HonoreeEnRetard,
    /// Pas encore fournie, délai non écoulé.
    EnCours,
    /// Pas fournie, délai écoulé. Une vente est bloquée.
    EnDefaut,
}

impl DemandeDeReleve {
    pub fn nouvelle(notaire: String, unit_id: Uuid, recue_le: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            notaire,
            unit_id,
            recue_le,
            echeance: recue_le + Duration::days(DELAI_JOURS),
            fournie_le: None,
        }
    }

    pub fn fournir(&mut self, le: DateTime<Utc>) {
        self.fournie_le = Some(le);
    }

    pub fn etat(&self, moment: DateTime<Utc>) -> EtatDemande {
        match self.fournie_le {
            Some(fournie) if fournie <= self.echeance => EtatDemande::HonoreeATemps,
            Some(_) => EtatDemande::HonoreeEnRetard,
            None if moment <= self.echeance => EtatDemande::EnCours,
            None => EtatDemande::EnDefaut,
        }
    }

    /// Combien de jours restent avant l'échéance ?
    ///
    /// Négatif une fois dépassée. Sert à alerter **avant** plutôt qu'à
    /// constater après.
    pub fn jours_restants(&self, moment: DateTime<Utc>) -> i64 {
        (self.echeance - moment).num_days()
    }
}

/// Les demandes qui appellent une relance, triées par urgence.
///
/// `seuil_alerte_jours` est la marge à partir de laquelle on prévient : à sept
/// jours, un syndic a encore le temps d'agir ; le jour même, il ne l'a plus.
pub fn a_relancer(
    demandes: &[DemandeDeReleve],
    moment: DateTime<Utc>,
    seuil_alerte_jours: i64,
) -> Vec<&DemandeDeReleve> {
    let mut urgentes: Vec<&DemandeDeReleve> = demandes
        .iter()
        .filter(|d| {
            matches!(d.etat(moment), EtatDemande::EnCours | EtatDemande::EnDefaut)
                && d.jours_restants(moment) <= seuil_alerte_jours
        })
        .collect();
    urgentes.sort_by_key(|d| d.echeance);
    urgentes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn il_y_a(jours: i64) -> DateTime<Utc> {
        Utc::now() - Duration::days(jours)
    }

    fn demande(recue_il_y_a: i64) -> DemandeDeReleve {
        DemandeDeReleve::nouvelle(
            "Me Dupont".to_string(),
            Uuid::new_v4(),
            il_y_a(recue_il_y_a),
        )
    }

    #[test]
    fn happy_lecheance_tombe_trente_jours_apres_la_demande() {
        let d = demande(0);
        assert_eq!(d.echeance, d.recue_le + Duration::days(30));
    }

    #[test]
    fn happy_un_releve_fourni_dans_les_temps_est_honore() {
        let mut d = demande(20);
        d.fournir(il_y_a(5));
        assert_eq!(d.etat(Utc::now()), EtatDemande::HonoreeATemps);
    }

    #[test]
    fn happy_avant_lecheance_la_demande_est_simplement_en_cours() {
        assert_eq!(demande(10).etat(Utc::now()), EtatDemande::EnCours);
    }

    /// Le cas qui bloque une vente.
    #[test]
    fn negative_passe_trente_jours_sans_releve_le_syndic_est_en_defaut() {
        assert_eq!(demande(40).etat(Utc::now()), EtatDemande::EnDefaut);
    }

    /// @edge — le trentième jour, le syndic est encore dans son délai.
    #[test]
    fn edge_le_jour_de_lecheance_nest_pas_encore_un_defaut() {
        let d = demande(30);
        assert_eq!(d.etat(d.echeance), EtatDemande::EnCours);
        assert_eq!(
            d.etat(d.echeance + Duration::seconds(1)),
            EtatDemande::EnDefaut
        );
    }

    /// Fournir en retard ne réécrit pas l'histoire.
    ///
    /// Le notaire a attendu, la vente a été retardée : l'état le dit, même une
    /// fois le relevé remis.
    #[test]
    fn negative_un_releve_fourni_en_retard_reste_marque_comme_tel() {
        let mut d = demande(40);
        d.fournir(il_y_a(2));
        assert_eq!(d.etat(Utc::now()), EtatDemande::HonoreeEnRetard);
    }

    #[test]
    fn happy_le_compte_a_rebours_previent_avant_lecheance() {
        // Le moment est fixé à partir de la demande elle-même : le calculer
        // avec `Utc::now()` ferait dériver le résultat de quelques
        // microsecondes, et `num_days()` tronque — cinq jours moins un
        // battement de cil valent quatre.
        let d = demande(25);
        let vingt_cinq_jours_apres = d.recue_le + Duration::days(25);
        assert_eq!(d.jours_restants(vingt_cinq_jours_apres), 5);
    }

    #[test]
    fn happy_il_devient_negatif_une_fois_lecheance_passee() {
        assert!(demande(40).jours_restants(Utc::now()) < 0);
    }

    /// Le point de la relance : alerter avant, pas constater après.
    #[test]
    fn happy_les_demandes_urgentes_remontent_triees_par_echeance() {
        let demandes = vec![demande(28), demande(40), demande(5), demande(26)];
        let urgentes = a_relancer(&demandes, Utc::now(), 7);

        assert_eq!(
            urgentes.len(),
            3,
            "celle reçue il y a 5 jours n'est pas urgente"
        );
        assert!(
            urgentes[0].echeance <= urgentes[1].echeance,
            "la plus pressée en premier"
        );
        assert!(urgentes[0].echeance <= urgentes[2].echeance);
    }

    #[test]
    fn happy_une_demande_deja_honoree_ne_se_relance_pas() {
        let mut honoree = demande(28);
        honoree.fournir(il_y_a(1));
        assert!(a_relancer(&[honoree], Utc::now(), 7).is_empty());
    }
}
