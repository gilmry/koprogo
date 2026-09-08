// Domain Entity: SyndicMandate — mandat de gestion d'une ACP
//
// Une ACP est une entité juridique à part entière : elle a son numéro BCE,
// son acte de base, ses lots, ses copropriétaires et sa comptabilité. Un
// syndic n'en est que le MANDATAIRE, désigné par l'assemblée générale et
// révocable par elle (Art. 3.89 CC).
//
// Le modèle exprimait ce lien par un simple champ `Acp.organization_id`.
// Changer de syndic ÉCRASE donc le précédent : le système ne sait plus qui
// gérait l'ACP avant, ni depuis quand le nouveau la gère. Or cette date
// n'est pas cosmétique — un état daté porte sur une date de référence et
// engage le syndic en fonction À CETTE DATE (Art. 3.94). Sans historique de
// mandat, la question « qui était mandataire le 12 mars ? » n'a pas de
// réponse.
//
// Cette entité rend le mandat explicite et daté. Elle n'enlève rien à
// `Acp::organization_id`, qui reste la lecture rapide du mandataire courant.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// La durée maximale d'un mandat de syndic, en jours.
///
/// ── Art. 3.89 § 1er, alinéa 2 ─────────────────────────────────────────────
///
/// > « Le syndic est désigné par le règlement de copropriété ou par une
/// > décision de l'assemblée générale. **La durée de son mandat ne peut
/// > excéder trois ans**, mais est renouvelable par décision expresse de
/// > l'assemblée générale. »
///
/// Trois ans, et **renouvelable par décision EXPRESSE**. C'est cette dernière
/// précision qui donne son sens au plafond : sans elle, un mandat se
/// reconduirait tacitement et l'assemblée perdrait le seul rendez-vous où elle
/// juge son mandataire. Le plafond n'est pas une formalité de durée, c'est un
/// rendez-vous imposé.
///
/// ── Pourquoi 3 × 365 et non une arithmétique de calendrier ───────────────
///
/// Trois années civiles comptent une ou deux journées bissextiles selon la
/// date de départ. Prendre 1095 jours donne un plafond LÉGÈREMENT plus court
/// que trois années calendaires dans le cas bissextile — donc plus strict que
/// la loi, jamais plus permissif. C'est le bon sens de l'arrondi pour une
/// borne maximale : on ne dépasse pas par erreur de calcul.
///
/// Ne pas confondre avec `MAX_MANDATE_DURATION_DAYS = 365 * 5` de
/// `mandate.rs` : celui-là borne les mandats de professionnels externes —
/// avocat, notaire, architecte — et c'est une hygiène anti-abus, pas cet
/// article. La confusion a déjà été faite (#837).
pub const DUREE_MAXIMALE_JOURS: i64 = 3 * 365;

/// Erreurs de validation d'un mandat de gestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyndicMandateError {
    /// La date de fin précède la date de début.
    EndBeforeStart,
    /// Le mandat est déjà clos : on ne le révoque pas deux fois.
    AlreadyEnded,
    /// La révocation précède la prise d'effet.
    RevocationBeforeStart,
    /// La fin proposée dépasse les trois ans de l'Art. 3.89 § 1er.
    DureeExcedeTroisAns,
}

impl std::fmt::Display for SyndicMandateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndBeforeStart => {
                write!(f, "Mandate end date cannot precede its start date")
            }
            Self::AlreadyEnded => write!(f, "Mandate is already ended"),
            Self::RevocationBeforeStart => {
                write!(f, "Mandate cannot be revoked before it takes effect")
            }
            Self::DureeExcedeTroisAns => write!(
                f,
                "La durée du mandat de syndic ne peut excéder trois ans \
                 (Art. 3.89 § 1er). Il est renouvelable par décision expresse \
                 de l'assemblée générale."
            ),
        }
    }
}

impl std::error::Error for SyndicMandateError {}

/// Mandat de gestion d'une ACP par un cabinet syndic, borné dans le temps.
///
/// `ended_at == None` signifie « mandat en cours ». Un mandat clos n'est
/// jamais supprimé : c'est lui qui permet de répondre à « qui gérait cette
/// ACP à telle date ».
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyndicMandate {
    pub id: Uuid,
    /// L'ACP gérée. C'est elle qui possède la comptabilité, pas le syndic.
    pub acp_id: Uuid,
    /// Le cabinet syndic mandataire.
    pub organization_id: Uuid,
    /// Prise d'effet du mandat.
    pub started_at: DateTime<Utc>,
    /// Fin du mandat. `None` = en cours.
    pub ended_at: Option<DateTime<Utc>>,
    /// Décision d'assemblée générale ayant désigné le syndic (Art. 3.89 CC).
    ///
    /// Optionnel : la première mise en gestion d'une ACP encodée par le
    /// SuperAdmin SaaS précède l'AG qui la confirmera.
    pub appointed_by_meeting_id: Option<Uuid>,
    /// Décision d'assemblée générale ayant mis fin au mandat.
    pub revoked_by_meeting_id: Option<Uuid>,
    /// Motif de fin, pour la trace : non-renouvellement, révocation,
    /// démission, cession de portefeuille.
    pub end_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SyndicMandate {
    /// Confie une ACP à un cabinet syndic.
    pub fn new(
        acp_id: Uuid,
        organization_id: Uuid,
        started_at: DateTime<Utc>,
        appointed_by_meeting_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            acp_id,
            organization_id,
            started_at,
            ended_at: None,
            appointed_by_meeting_id,
            revoked_by_meeting_id: None,
            end_reason: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Le mandat est-il en cours ?
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Le mandat couvrait-il cette date ?
    ///
    /// Bornes : début INCLUS, fin EXCLUE. Le jour où un mandat s'achève est
    /// le premier jour du suivant — deux syndics ne peuvent pas être en
    /// fonction au même instant sur la même ACP, et aucun instant n'est
    /// laissé sans mandataire au moment d'une passation.
    ///
    /// C'est ce que consulte un état daté : il porte sur une date de
    /// référence et engage le syndic en fonction à cette date.
    pub fn covers(&self, moment: DateTime<Utc>) -> bool {
        if moment < self.started_at {
            return false;
        }
        match self.ended_at {
            Some(fin) => moment < fin,
            None => true,
        }
    }

    /// L'échéance légale du mandat : trois ans après sa prise d'effet.
    ///
    /// Au-delà, le mandat est expiré de plein droit — l'assemblée doit l'avoir
    /// renouvelé par décision expresse (Art. 3.89 § 1er).
    pub fn echeance_legale(&self) -> DateTime<Utc> {
        self.started_at + Duration::days(DUREE_MAXIMALE_JOURS)
    }

    /// Le mandat a-t-il dépassé ses trois ans à cette date ?
    ///
    /// Un mandat sans date de fin qui franchit son échéance N'EST PLUS
    /// valable : le silence de l'assemblée ne le reconduit pas. C'est
    /// précisément ce que le plafond empêche.
    ///
    /// Vrai aussi pour un mandat clos APRÈS son échéance : la clôture tardive
    /// ne régularise pas la période excédentaire.
    pub fn est_expire_de_plein_droit(&self, moment: DateTime<Utc>) -> bool {
        moment >= self.echeance_legale()
    }

    /// Met fin au mandat.
    ///
    /// Refuse une fin postérieure à l'échéance légale : dater la clôture d'un
    /// mandat au-delà de ses trois ans reviendrait à écrire dans le registre
    /// qu'il a couvert une période qu'il ne pouvait pas couvrir.
    pub fn revoke(
        &mut self,
        ended_at: DateTime<Utc>,
        revoked_by_meeting_id: Option<Uuid>,
        end_reason: Option<String>,
    ) -> Result<(), SyndicMandateError> {
        if self.ended_at.is_some() {
            return Err(SyndicMandateError::AlreadyEnded);
        }
        if ended_at < self.started_at {
            return Err(SyndicMandateError::RevocationBeforeStart);
        }
        if ended_at > self.echeance_legale() {
            return Err(SyndicMandateError::DureeExcedeTroisAns);
        }
        self.ended_at = Some(ended_at);
        self.revoked_by_meeting_id = revoked_by_meeting_id;
        self.end_reason = end_reason;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Le mandataire à une date donnée, parmi un historique.
    ///
    /// Renvoie `None` si l'ACP n'était confiée à personne — cas réel : une
    /// ACP encodée par le SuperAdmin mais pas encore confiée, ou une ACP
    /// auto-gérée (ADR-0010).
    pub fn holder_at(mandats: &[SyndicMandate], moment: DateTime<Utc>) -> Option<Uuid> {
        mandats
            .iter()
            .find(|m| m.covers(moment))
            .map(|m| m.organization_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn mandat(debut_il_y_a_jours: i64) -> SyndicMandate {
        SyndicMandate::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Utc::now() - Duration::days(debut_il_y_a_jours),
            None,
        )
    }

    #[test]
    fn happy_un_mandat_neuf_est_en_cours() {
        let m = mandat(30);
        assert!(m.is_active());
        assert!(m.covers(Utc::now()));
        assert!(m.covers(Utc::now() - Duration::days(10)));
    }

    #[test]
    fn happy_un_mandat_ne_couvre_pas_lavant() {
        let m = mandat(30);
        assert!(
            !m.covers(Utc::now() - Duration::days(60)),
            "le syndic n'engage rien avant sa prise de fonction"
        );
    }

    #[test]
    fn happy_revocation_ferme_le_mandat() {
        let mut m = mandat(365);
        let fin = Utc::now() - Duration::days(30);
        m.revoke(fin, Some(Uuid::new_v4()), Some("Non-renouvellement".into()))
            .unwrap();

        assert!(!m.is_active());
        assert!(
            m.covers(Utc::now() - Duration::days(60)),
            "il engageait encore la copropriété avant sa révocation"
        );
        assert!(!m.covers(Utc::now()), "il n'engage plus rien après");
    }

    /// @edge — bornes : début inclus, fin exclue.
    ///
    /// C'est ce qui garantit qu'à l'instant d'une passation il y a
    /// exactement UN mandataire : ni deux, ni zéro.
    #[test]
    fn edge_bornes_debut_inclus_fin_exclue() {
        let debut = Utc::now() - Duration::days(100);
        let fin = Utc::now() - Duration::days(50);
        let mut m = SyndicMandate::new(Uuid::new_v4(), Uuid::new_v4(), debut, None);
        m.revoke(fin, None, None).unwrap();

        assert!(
            m.covers(debut),
            "le jour de la prise de fonction est couvert"
        );
        assert!(
            !m.covers(fin),
            "le jour de la fin appartient au mandat suivant"
        );
        assert!(!m.covers(debut - Duration::milliseconds(1)));
    }

    #[test]
    fn negative_on_ne_revoque_pas_deux_fois() {
        let mut m = mandat(100);
        m.revoke(Utc::now(), None, None).unwrap();
        assert_eq!(
            m.revoke(Utc::now(), None, None),
            Err(SyndicMandateError::AlreadyEnded)
        );
    }

    #[test]
    fn negative_revocation_anterieure_a_la_prise_deffet() {
        let mut m = mandat(10);
        assert_eq!(
            m.revoke(Utc::now() - Duration::days(30), None, None),
            Err(SyndicMandateError::RevocationBeforeStart)
        );
    }

    /// Le cœur de l'affaire : « qui gérait cette ACP le 12 mars ? »
    ///
    /// Un état daté porte sur une date de référence et engage le syndic en
    /// fonction à cette date. Avec un simple champ mutable sur l'ACP, cette
    /// question n'avait pas de réponse : la passation effaçait le prédécesseur.
    #[test]
    fn happy_qui_gerait_lacp_a_telle_date() {
        let acp = Uuid::new_v4();
        let ancien = Uuid::new_v4();
        let nouveau = Uuid::new_v4();
        let passation = Utc::now() - Duration::days(90);

        let mut m1 = SyndicMandate::new(acp, ancien, Utc::now() - Duration::days(730), None);
        m1.revoke(
            passation,
            Some(Uuid::new_v4()),
            Some("Révocation AG".into()),
        )
        .unwrap();
        let m2 = SyndicMandate::new(acp, nouveau, passation, Some(Uuid::new_v4()));
        let historique = vec![m1, m2];

        assert_eq!(
            SyndicMandate::holder_at(&historique, Utc::now() - Duration::days(365)),
            Some(ancien),
            "avant la passation, c'est l'ancien cabinet qui engageait l'ACP"
        );
        assert_eq!(
            SyndicMandate::holder_at(&historique, Utc::now()),
            Some(nouveau)
        );
        // À l'instant exact de la passation : exactement un mandataire.
        assert_eq!(
            SyndicMandate::holder_at(&historique, passation),
            Some(nouveau),
            "la borne de fin étant exclue, la passation ne laisse aucun trou"
        );
    }

    /// @edge — une ACP encodée mais pas encore confiée n'a pas de mandataire.
    #[test]
    fn edge_acp_sans_mandataire() {
        assert_eq!(SyndicMandate::holder_at(&[], Utc::now()), None);
    }

    // ── Art. 3.89 § 1er : la durée du mandat n'excède pas trois ans ────────
    //
    // Le registre légal déclarait cet article couvert et attesté par
    // `happy_un_mandat_neuf_est_en_cours` — un test qui vérifie qu'un mandat
    // de trente jours est en cours, et qui ne dit rien d'un plafond. Le
    // plafond n'existait pas (#847, #837).
    //
    // Ces tests-ci attestent l'obligation elle-même, et leur nom le dit.

    #[test]
    fn happy_lecheance_legale_tombe_trois_ans_apres_la_prise_deffet() {
        let m = mandat(0);
        assert_eq!(
            m.echeance_legale(),
            m.started_at + Duration::days(DUREE_MAXIMALE_JOURS)
        );
    }

    #[test]
    fn security_un_mandat_de_plus_de_trois_ans_est_expire_de_plein_droit() {
        // Le silence de l'assemblée ne reconduit pas : l'Art. 3.89 § 1er veut
        // une décision EXPRESSE de renouvellement.
        let m = mandat(DUREE_MAXIMALE_JOURS + 1);
        assert!(
            m.est_expire_de_plein_droit(Utc::now()),
            "un mandat entamé il y a plus de trois ans doit être expiré, \
             même si personne ne l'a clos"
        );
    }

    #[test]
    fn edge_un_mandat_de_trois_ans_moins_un_jour_court_encore() {
        let m = mandat(DUREE_MAXIMALE_JOURS - 1);
        assert!(!m.est_expire_de_plein_droit(Utc::now()));
    }

    #[test]
    fn edge_le_jour_de_lecheance_le_mandat_est_expire() {
        // Borne INCLUSE : « ne peut excéder trois ans » — le trois-millième
        // quatre-vingt-quinzième jour est déjà de trop.
        let m = mandat(DUREE_MAXIMALE_JOURS);
        assert!(m.est_expire_de_plein_droit(Utc::now()));
    }

    #[test]
    fn negative_une_cloture_datee_au_dela_des_trois_ans_est_refusee() {
        // Dater la clôture au-delà de l'échéance reviendrait à écrire dans le
        // registre que le mandat a couvert une période qu'il ne pouvait pas
        // couvrir.
        let mut m = mandat(10);
        let trop_tard = m.echeance_legale() + Duration::days(1);
        assert_eq!(
            m.revoke(trop_tard, None, None),
            Err(SyndicMandateError::DureeExcedeTroisAns)
        );
        assert!(m.is_active(), "le mandat refusé ne doit pas être clos");
    }

    #[test]
    fn happy_une_cloture_dans_les_trois_ans_est_acceptee() {
        let mut m = mandat(10);
        let dans_les_temps = m.echeance_legale() - Duration::days(1);
        assert!(m.revoke(dans_les_temps, None, None).is_ok());
        assert!(!m.is_active());
    }
}
