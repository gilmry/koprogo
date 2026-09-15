//! Le lien notaire — accès signé, temporaire et révocable à un état daté.
//!
//! [ADR 0048](../../../../../docs/adr/0048-identite-notaire-et-routes-publiques.md)
//! a tranché la destination : `GET /etats-dates/reference/{reference_number}`
//! cesse d'être publique. Un état daté porte les dettes d'un copropriétaire
//! nommé ; le rendre lisible à quiconque connaît une référence revenait à
//! publier une situation financière individuelle derrière un identifiant
//! devinable, transmissible et jamais révocable.
//!
//! [ADR 0051](../../../../../docs/adr/0051-lien-notaire-sept-jours-renouvelable.md)
//! a tranché la modalité : un jeton signé, émis par le syndic, qui vaut
//! **sept jours calendaires**, autorise **plusieurs lectures** pendant cette
//! fenêtre, et que le syndic peut **renouveler** ou **révoquer** avant terme.
//!
//! Distinct de [`super::releve_notaire::DemandeDeReleve`], qui suit le délai
//! **légal** de fourniture (Art. 3.89 § 5, 5°, trente jours). Ici, la durée
//! est un paramètre de **sécurité** : le temps pendant lequel un secret reste
//! opposable. ADR 0051 explique pourquoi les deux durées ne doivent pas
//! coïncider.
//!
//! Contrairement au [`crate::domain::plateforme::magic_link::MagicLink`]
//! générique — à usage unique (`consumed_at`) — ce jeton est **multi-lecture
//! par construction** : aucun champ ne marque une consultation comme
//! consommée. Chaque lecture est journalisée au niveau use-case/handler, pas
//! ici.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Durée de validité d'un lien, fixée par ADR 0051.
pub const DUREE_JOURS: i64 = 7;

/// Erreurs domaine pures — zéro dépendance application/infra (pureté
/// hexagonale, cf. CLAUDE.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LienNotaireError {
    /// `etat_date_id` est nil : rien à protéger.
    EtatDateIdNul,
    /// `emis_par` est nil : aucun syndic ne porte l'émission.
    EmisParNul,
    /// Un lien révoqué ne se renouvelle pas — il faut en émettre un nouveau.
    DejaRevoque,
}

impl std::fmt::Display for LienNotaireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EtatDateIdNul => write!(f, "etat_date_id must not be nil"),
            Self::EmisParNul => write!(f, "emis_par must not be nil"),
            Self::DejaRevoque => write!(f, "a revoked notary link cannot be renewed"),
        }
    }
}

impl std::error::Error for LienNotaireError {}

/// Un lien notaire vers un état daté précis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LienNotaire {
    pub id: Uuid,
    /// L'état daté que ce lien donne le droit de lire — jamais réinterprété :
    /// une lecture qui vise un autre état daté avec ce jeton doit échouer.
    pub etat_date_id: Uuid,
    /// SHA-256 hex du jeton clair. Le jeton clair n'est JAMAIS stocké.
    pub token_hash: String,
    /// Le syndic qui a émis le lien (traçabilité).
    pub emis_par: Uuid,
    pub cree_le: DateTime<Utc>,
    pub expire_le: DateTime<Utc>,
    /// `Some` si le syndic a révoqué le lien avant terme.
    pub revoque_le: Option<DateTime<Utc>>,
    pub revoque_par: Option<Uuid>,
    /// Date du dernier renouvellement, s'il y en a eu un.
    pub renouvele_le: Option<DateTime<Utc>>,
    pub mis_a_jour_le: DateTime<Utc>,
}

impl LienNotaire {
    /// Émet un nouveau lien. Retourne l'entité persistée ET le jeton clair,
    /// qui doit être renvoyé UNE FOIS au syndic et jamais stocké ailleurs.
    pub fn emettre(etat_date_id: Uuid, emis_par: Uuid) -> Result<(Self, String), LienNotaireError> {
        if etat_date_id.is_nil() {
            return Err(LienNotaireError::EtatDateIdNul);
        }
        if emis_par.is_nil() {
            return Err(LienNotaireError::EmisParNul);
        }

        let clair = Self::generer_jeton();
        let token_hash = Self::hacher(&clair);
        let maintenant = Utc::now();

        let entite = Self {
            id: Uuid::new_v4(),
            etat_date_id,
            token_hash,
            emis_par,
            cree_le: maintenant,
            expire_le: maintenant + Duration::days(DUREE_JOURS),
            revoque_le: None,
            revoque_par: None,
            renouvele_le: None,
            mis_a_jour_le: maintenant,
        };

        Ok((entite, clair))
    }

    /// SHA-256 hex du jeton clair. Public pour que le repository/handler
    /// puisse hacher un jeton reçu et le chercher.
    pub fn hacher(clair: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(clair.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 32 octets aléatoires, encodés base64url (~256 bits d'entropie).
    fn generer_jeton() -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        let mut octets = [0u8; 32];
        for o in octets.iter_mut() {
            *o = rand::random::<u8>();
        }
        URL_SAFE_NO_PAD.encode(octets)
    }

    pub fn est_expire(&self, moment: DateTime<Utc>) -> bool {
        moment > self.expire_le
    }

    pub fn est_revoque(&self) -> bool {
        self.revoque_le.is_some()
    }

    /// Valide = ni expiré, ni révoqué. Aucune notion de consommation : un
    /// même lien reste valide sur plusieurs lectures (ADR 0051).
    pub fn est_valide(&self, moment: DateTime<Utc>) -> bool {
        !self.est_expire(moment) && !self.est_revoque()
    }

    /// Renouvelle le lien pour sept nouveaux jours à partir de `moment`.
    ///
    /// C'est un ACTE explicite du syndic, pas un automatisme (ADR 0051) —
    /// autorisé même après expiration : une vente qui dépasse sept jours est
    /// normale, et c'est au syndic de le constater. Seule la révocation est
    /// terminale.
    pub fn renouveler(&mut self, moment: DateTime<Utc>) -> Result<(), LienNotaireError> {
        if self.est_revoque() {
            return Err(LienNotaireError::DejaRevoque);
        }
        self.expire_le = moment + Duration::days(DUREE_JOURS);
        self.renouvele_le = Some(moment);
        self.mis_a_jour_le = moment;
        Ok(())
    }

    /// Révoque le lien avant terme. Idempotent : une seconde révocation ne
    /// réécrit pas le moment ni l'auteur de la première (même idiome que
    /// `MagicLink::consume`).
    pub fn revoquer(&mut self, moment: DateTime<Utc>, revoque_par: Uuid) {
        if self.revoque_le.is_none() {
            self.revoque_le = Some(moment);
            self.revoque_par = Some(revoque_par);
        }
        self.mis_a_jour_le = moment;
    }
}

// ============================================================================
// Tests — taxonomie 4 catégories obligatoire (CRITICAL.md #3)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn paire() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    // ------------------------------------------------------------------------
    // @happy
    // ------------------------------------------------------------------------

    #[test]
    fn happy_emettre_retourne_un_lien_valide_sept_jours() {
        let (etat_date_id, emis_par) = paire();
        let (lien, clair) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();

        assert_eq!(lien.etat_date_id, etat_date_id);
        assert_eq!(lien.emis_par, emis_par);
        assert!(lien.est_valide(Utc::now()));
        assert_eq!(
            (lien.expire_le - lien.cree_le).num_days(),
            DUREE_JOURS,
            "ADR 0051 fixe la durée à sept jours"
        );
        assert!(
            clair.len() >= 40,
            "jeton clair trop court : {}",
            clair.len()
        );
    }

    #[test]
    fn happy_hacher_est_deterministe_et_produit_64_caracteres_hex() {
        let h1 = LienNotaire::hacher("un-jeton");
        let h2 = LienNotaire::hacher("un-jeton");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn happy_lecture_dans_la_fenetre_est_valide() {
        // Lecture au jour 3 : dans la fenêtre des sept jours.
        let (etat_date_id, emis_par) = paire();
        let (lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let jour_3 = lien.cree_le + Duration::days(3);
        assert!(lien.est_valide(jour_3));
    }

    #[test]
    fn happy_renouveler_prolonge_de_sept_jours_depuis_le_moment_donne() {
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let renouvellement = lien.cree_le + Duration::days(6);

        lien.renouveler(renouvellement).unwrap();

        assert_eq!(lien.renouvele_le, Some(renouvellement));
        assert_eq!(lien.expire_le, renouvellement + Duration::days(DUREE_JOURS));
    }

    // ------------------------------------------------------------------------
    // @edge — bornes de l'expiration, J+7 / J+8, renouvellement en chaîne
    // ------------------------------------------------------------------------

    #[test]
    fn edge_lecture_a_j7_pile_est_encore_valide() {
        let (etat_date_id, emis_par) = paire();
        let (lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        assert!(
            lien.est_valide(lien.expire_le),
            "l'instant exact de l'échéance n'est pas encore un dépassement"
        );
    }

    #[test]
    fn edge_lecture_a_j8_apres_lecheance_est_invalide() {
        let (etat_date_id, emis_par) = paire();
        let (lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let j8 = lien.expire_le + Duration::seconds(1);
        assert!(!lien.est_valide(j8));
        assert!(lien.est_expire(j8));
    }

    #[test]
    fn edge_renouvellement_en_chaine_repousse_lecheance_a_chaque_fois() {
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();

        let premiere_echeance = lien.expire_le;
        let renouvellement_1 = premiere_echeance - Duration::days(1);
        lien.renouveler(renouvellement_1).unwrap();
        let deuxieme_echeance = lien.expire_le;
        assert!(deuxieme_echeance > premiere_echeance);

        let renouvellement_2 = deuxieme_echeance - Duration::days(1);
        lien.renouveler(renouvellement_2).unwrap();
        assert!(lien.expire_le > deuxieme_echeance);
        assert_eq!(lien.renouvele_le, Some(renouvellement_2));
    }

    #[test]
    fn edge_renouveler_un_lien_deja_expire_le_repousse_depuis_maintenant() {
        // « Une vente qui dépasse sept jours est normale » (ADR 0051) : le
        // renouvellement n'est pas bloqué par une expiration déjà passée.
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let bien_apres_lecheance = lien.expire_le + Duration::days(10);

        lien.renouveler(bien_apres_lecheance).unwrap();

        assert!(lien.est_valide(bien_apres_lecheance));
    }

    // ------------------------------------------------------------------------
    // @security — jeton forgé (hash), lien révoqué, distinction des lecteurs
    // ------------------------------------------------------------------------

    #[test]
    fn security_deux_emissions_produisent_des_jetons_et_hachages_distincts() {
        let (etat_date_id, emis_par) = paire();
        let (lien_a, clair_a) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let (lien_b, clair_b) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();

        assert_ne!(clair_a, clair_b);
        assert_ne!(lien_a.token_hash, lien_b.token_hash);
        assert_ne!(lien_a.id, lien_b.id);
    }

    #[test]
    fn security_le_jeton_clair_ne_vaut_jamais_le_hachage_stocke() {
        let (etat_date_id, emis_par) = paire();
        let (lien, clair) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        assert_ne!(clair, lien.token_hash);
        assert_eq!(LienNotaire::hacher(&clair), lien.token_hash);
    }

    #[test]
    fn security_lien_revoque_est_invalide_meme_avant_lecheance() {
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let revocateur = Uuid::new_v4();

        lien.revoquer(Utc::now(), revocateur);

        assert!(lien.est_revoque());
        assert!(!lien.est_valide(Utc::now()));
        assert_eq!(lien.revoque_par, Some(revocateur));
    }

    #[test]
    fn security_double_revocation_est_idempotente_et_garde_le_premier_auteur() {
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        let premier = Uuid::new_v4();
        let second = Uuid::new_v4();

        lien.revoquer(Utc::now(), premier);
        let premiere_date = lien.revoque_le;
        lien.revoquer(Utc::now() + Duration::seconds(5), second);

        assert_eq!(lien.revoque_le, premiere_date);
        assert_eq!(lien.revoque_par, Some(premier));
    }

    #[test]
    fn security_renouveler_un_lien_revoque_est_refuse() {
        let (etat_date_id, emis_par) = paire();
        let (mut lien, _) = LienNotaire::emettre(etat_date_id, emis_par).unwrap();
        lien.revoquer(Utc::now(), Uuid::new_v4());

        let err = lien.renouveler(Utc::now()).unwrap_err();
        assert_eq!(err, LienNotaireError::DejaRevoque);
    }

    // ------------------------------------------------------------------------
    // @negative — défaillance correcte, jamais de panic
    // ------------------------------------------------------------------------

    #[test]
    fn negative_etat_date_id_nul_est_rejete() {
        let err = LienNotaire::emettre(Uuid::nil(), Uuid::new_v4()).unwrap_err();
        assert_eq!(err, LienNotaireError::EtatDateIdNul);
    }

    #[test]
    fn negative_emis_par_nul_est_rejete() {
        let err = LienNotaire::emettre(Uuid::new_v4(), Uuid::nil()).unwrap_err();
        assert_eq!(err, LienNotaireError::EmisParNul);
    }

    #[test]
    fn negative_display_des_erreurs_ne_panique_pas_et_reste_lisible() {
        for err in [
            LienNotaireError::EtatDateIdNul,
            LienNotaireError::EmisParNul,
            LienNotaireError::DejaRevoque,
        ] {
            assert!(!err.to_string().is_empty());
        }
    }
}
