use super::meeting::MeetingMode;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Voting-power upper bound (Art. 3.87 §7 CC envelope, 10000 dix-millièmes).
const MAX_VOTING_POWER: Decimal = dec!(10000);

/// Choix de vote d'un copropriétaire
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteChoice {
    Pour,       // Vote en faveur (For)
    Contre,     // Vote contre (Against)
    Abstention, // Abstention
}

/// Méthode d'authentification du votant (Story 4.2, Art. 3.87 §1er, §4 CC,
/// #48). `Presence` couvre la signature de la feuille de présence en AG
/// physique ; `Proxy` une procuration papier en bonne et due forme ;
/// `Itsme`/`Eid` l'authentification forte requise pour un vote à distance
/// (Art. 3.87 §1er : « à distance au moyen d'une communication
/// électronique » suppose de savoir QUI a voté).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteAuthMethod {
    Presence,
    Proxy,
    Itsme,
    Eid,
}

impl VoteAuthMethod {
    pub fn from_db_string(s: &str) -> Result<Self, String> {
        match s {
            "presence" => Ok(Self::Presence),
            "proxy" => Ok(Self::Proxy),
            "itsme" => Ok(Self::Itsme),
            "eid" => Ok(Self::Eid),
            other => Err(format!("Unknown vote auth method: {other}")),
        }
    }

    pub fn to_db_str(&self) -> &'static str {
        match self {
            Self::Presence => "presence",
            Self::Proxy => "proxy",
            Self::Itsme => "itsme",
            Self::Eid => "eid",
        }
    }

    /// Authentification qui engage réellement le votant, indépendamment de
    /// toute procuration (itsme/eID). `Presence` ne l'est pas : c'est une
    /// simple déclaration, vérifiable seulement par la présence physique
    /// qu'un vote à distance ne permet justement pas de constater.
    pub fn is_strong(&self) -> bool {
        matches!(self, Self::Itsme | Self::Eid)
    }
}

/// Story 4.2 — refus opposé par `assert_vote_auth_sufficient`. Mappé vers
/// AppError (`application/error.rs`) : `Missing` en 422, `Insufficient` en
/// 403.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VoteAuthError {
    /// Un vote sans méthode déclarée n'est pas exploitable en cas de
    /// contestation : on ne sait même pas comment le votant a été identifié.
    #[error("La méthode d'authentification du vote est obligatoire")]
    Missing,

    /// Le mode de l'AG (remote/hybrid, Art. 3.87 §1er CC) exige une méthode
    /// qui engage le votant : itsme/eID, ou une procuration en bonne et due
    /// forme (Art. 3.87 §4). `presence` ne fait qu'affirmer une présence que
    /// la modalité distancielle ne permet justement pas de vérifier.
    #[error(
        "Authentification insuffisante pour un vote en mode {mode:?} : {auth_method:?} \
         n'engage pas le votant (Art. 3.87 §1er, §4 CC)"
    )]
    Insufficient {
        mode: MeetingMode,
        auth_method: VoteAuthMethod,
    },
}

/// Story 4.2 — Art. 3.87 §1er, §4 CC : valide `auth_method` contre la
/// modalité de l'AG avant d'autoriser un vote.
///
/// `is_proxy_vote` distingue un `auth_method: Proxy` réel (le bulletin porte
/// effectivement un `proxy_owner_id`, dont les conditions — plafond de trois
/// procurations — se vérifient par ailleurs) d'une simple étiquette : se
/// déclarer mandataire sans l'être ne peut pas suffire à voter à distance.
///
/// En AG physique (`InPerson`), aucune méthode n'est jugée insuffisante :
/// c'est la présence elle-même qui authentifie.
pub fn assert_vote_auth_sufficient(
    mode: MeetingMode,
    auth_method: Option<VoteAuthMethod>,
    is_proxy_vote: bool,
) -> Result<VoteAuthMethod, VoteAuthError> {
    let auth_method = auth_method.ok_or(VoteAuthError::Missing)?;

    if !mode.requires_strong_vote_auth() {
        return Ok(auth_method);
    }

    let suffisant = match auth_method {
        VoteAuthMethod::Proxy => is_proxy_vote,
        other => other.is_strong(),
    };

    if suffisant {
        Ok(auth_method)
    } else {
        Err(VoteAuthError::Insufficient { mode, auth_method })
    }
}

/// Vote d'un propriétaire sur une résolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub struct Vote {
    pub id: Uuid,
    pub resolution_id: Uuid,
    pub owner_id: Uuid,
    pub unit_id: Uuid,
    pub vote_choice: VoteChoice,
    pub voting_power: Decimal, // Tantièmes/millièmes du lot (Decimal exact — ADR-0008)
    pub proxy_owner_id: Option<Uuid>, // ID du mandataire si vote par procuration
    pub voted_at: DateTime<Utc>,
    /// Story 4.2 — comment le votant a été authentifié (#48). Par défaut
    /// `Presence` (`Vote::new`) : seul `cast_vote`, une fois `auth_method`
    /// validé contre la modalité de l'AG, appelle `new_with_auth_method`.
    pub auth_method: VoteAuthMethod,
}

impl Vote {
    /// Crée un nouveau vote
    pub fn new(
        resolution_id: Uuid,
        owner_id: Uuid,
        unit_id: Uuid,
        vote_choice: VoteChoice,
        voting_power: Decimal,
        proxy_owner_id: Option<Uuid>,
    ) -> Result<Self, String> {
        // Validation du pouvoir de vote
        if voting_power <= Decimal::ZERO {
            return Err("Voting power must be positive".to_string());
        }
        if voting_power > MAX_VOTING_POWER {
            return Err("Voting power exceeds maximum (10000 dix-millièmes)".to_string());
        }

        // Validation de la procuration
        if let Some(proxy_id) = proxy_owner_id {
            if proxy_id == owner_id {
                return Err("Owner cannot be their own proxy".to_string());
            }
        }

        Ok(Self {
            id: Uuid::new_v4(),
            resolution_id,
            owner_id,
            unit_id,
            vote_choice,
            voting_power,
            proxy_owner_id,
            voted_at: Utc::now(),
            auth_method: VoteAuthMethod::Presence,
        })
    }

    /// Story 4.2 — variante de `new()` qui pose explicitement `auth_method`.
    ///
    /// Employée par le cas d'usage `cast_vote`, une fois la méthode validée
    /// contre la modalité de l'AG (`assert_vote_auth_sufficient`). Les autres
    /// appelants (tests de plafonnement, procurations, conflits d'intérêts)
    /// ne portent pas cette dimension et gardent `new()`, qui vaut
    /// `Presence` par défaut.
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_auth_method(
        resolution_id: Uuid,
        owner_id: Uuid,
        unit_id: Uuid,
        vote_choice: VoteChoice,
        voting_power: Decimal,
        proxy_owner_id: Option<Uuid>,
        auth_method: VoteAuthMethod,
    ) -> Result<Self, String> {
        let mut vote = Self::new(
            resolution_id,
            owner_id,
            unit_id,
            vote_choice,
            voting_power,
            proxy_owner_id,
        )?;
        vote.auth_method = auth_method;
        Ok(vote)
    }

    /// Vérifie si le vote est exprimé par procuration
    pub fn is_proxy_vote(&self) -> bool {
        self.proxy_owner_id.is_some()
    }

    /// Retourne l'ID du votant effectif (propriétaire ou mandataire)
    pub fn effective_voter_id(&self) -> Uuid {
        self.proxy_owner_id.unwrap_or(self.owner_id)
    }

    /// Modifie le choix de vote (seulement si pas encore enregistré)
    pub fn change_vote(&mut self, new_choice: VoteChoice) -> Result<(), String> {
        // En pratique, cette méthode ne serait appelée que pendant une fenêtre de temps limitée
        // Ici on autorise le changement, mais dans l'application on pourrait ajouter une validation
        // basée sur le timing (ex: vote modifiable uniquement dans les 5 minutes)
        self.vote_choice = new_choice;
        self.voted_at = Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vote_success() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(150), // 150 millièmes
            None,
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert_eq!(vote.resolution_id, resolution_id);
        assert_eq!(vote.owner_id, owner_id);
        assert_eq!(vote.unit_id, unit_id);
        assert_eq!(vote.vote_choice, VoteChoice::Pour);
        assert_eq!(vote.voting_power, dec!(150));
        assert!(!vote.is_proxy_vote());
        assert_eq!(vote.effective_voter_id(), owner_id);
    }

    #[test]
    fn test_create_vote_with_proxy() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();
        let proxy_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Contre,
            dec!(200),
            Some(proxy_id),
        );

        assert!(vote.is_ok());
        let vote = vote.unwrap();
        assert!(vote.is_proxy_vote());
        assert_eq!(vote.effective_voter_id(), proxy_id);
        assert_eq!(vote.proxy_owner_id, Some(proxy_id));
    }

    #[test]
    fn test_create_vote_zero_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(0),
            None,
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Voting power must be positive");
    }

    #[test]
    fn test_create_vote_negative_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(-50),
            None,
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Voting power must be positive");
    }

    #[test]
    fn test_create_vote_excessive_voting_power_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(15000), // Exceeds max
            None,
        );

        assert!(vote.is_err());
        assert!(vote.unwrap_err().contains("exceeds maximum"));
    }

    #[test]
    fn test_create_vote_self_proxy_fails() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(150),
            Some(owner_id), // Self as proxy
        );

        assert!(vote.is_err());
        assert_eq!(vote.unwrap_err(), "Owner cannot be their own proxy");
    }

    #[test]
    fn test_change_vote() {
        let resolution_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let unit_id = Uuid::new_v4();

        let mut vote = Vote::new(
            resolution_id,
            owner_id,
            unit_id,
            VoteChoice::Pour,
            dec!(150),
            None,
        )
        .unwrap();

        assert_eq!(vote.vote_choice, VoteChoice::Pour);

        let result = vote.change_vote(VoteChoice::Contre);
        assert!(result.is_ok());
        assert_eq!(vote.vote_choice, VoteChoice::Contre);
    }

    #[test]
    fn test_vote_choice_serialization() {
        // Test serialization of VoteChoice enum
        let pour = VoteChoice::Pour;
        let contre = VoteChoice::Contre;
        let abstention = VoteChoice::Abstention;

        let json_pour = serde_json::to_string(&pour).unwrap();
        let json_contre = serde_json::to_string(&contre).unwrap();
        let json_abstention = serde_json::to_string(&abstention).unwrap();

        assert_eq!(json_pour, "\"pour\"");
        assert_eq!(json_contre, "\"contre\"");
        assert_eq!(json_abstention, "\"abstention\"");
    }

    #[test]
    fn test_vote_choice_deserialization() {
        // Test deserialization of VoteChoice enum
        let pour: VoteChoice = serde_json::from_str("\"pour\"").unwrap();
        let contre: VoteChoice = serde_json::from_str("\"contre\"").unwrap();
        let abstention: VoteChoice = serde_json::from_str("\"abstention\"").unwrap();

        assert_eq!(pour, VoteChoice::Pour);
        assert_eq!(contre, VoteChoice::Contre);
        assert_eq!(abstention, VoteChoice::Abstention);
    }

    // ------------------------------------------------------------------------
    // Story 4.2 — `assert_vote_auth_sufficient` (Art. 3.87 §1er, §4 CC, #48)
    // ------------------------------------------------------------------------

    /// @happy — un vote distant authentifié par itsme est accepté.
    #[test]
    fn happy_itsme_suffit_pour_un_vote_distant() {
        let resultat =
            assert_vote_auth_sufficient(MeetingMode::Remote, Some(VoteAuthMethod::Itsme), false);
        assert_eq!(resultat, Ok(VoteAuthMethod::Itsme));
    }

    /// @happy — eID est équivalent à itsme pour l'authentification forte.
    #[test]
    fn happy_eid_suffit_pour_un_vote_hybride() {
        let resultat =
            assert_vote_auth_sufficient(MeetingMode::Hybrid, Some(VoteAuthMethod::Eid), false);
        assert_eq!(resultat, Ok(VoteAuthMethod::Eid));
    }

    /// @edge — une procuration en bonne et due forme (le bulletin porte
    /// effectivement un mandataire) est autorisée à distance : Art. 3.87 §4
    /// régit la procuration elle-même, la limite des trois mandats se
    /// vérifiant par ailleurs (`validate_proxy_limit`).
    #[test]
    fn edge_procuration_reelle_autorisee_a_distance() {
        let resultat =
            assert_vote_auth_sufficient(MeetingMode::Remote, Some(VoteAuthMethod::Proxy), true);
        assert_eq!(resultat, Ok(VoteAuthMethod::Proxy));
    }

    /// @edge — se déclarer `auth_method: proxy` sans que le bulletin porte
    /// réellement un mandataire n'est qu'une étiquette : ça n'engage
    /// personne de plus qu'une simple déclaration de présence.
    #[test]
    fn edge_proxy_declare_sans_mandat_reel_est_insuffisant() {
        let resultat =
            assert_vote_auth_sufficient(MeetingMode::Remote, Some(VoteAuthMethod::Proxy), false);
        assert_eq!(
            resultat,
            Err(VoteAuthError::Insufficient {
                mode: MeetingMode::Remote,
                auth_method: VoteAuthMethod::Proxy,
            })
        );
    }

    /// @edge — une AG en présentiel n'exige aucune authentification forte :
    /// `presence` y suffit toujours, quel que soit le mode déclaré ailleurs.
    #[test]
    fn edge_presence_suffit_en_ag_physique() {
        let resultat = assert_vote_auth_sufficient(
            MeetingMode::InPerson,
            Some(VoteAuthMethod::Presence),
            false,
        );
        assert_eq!(resultat, Ok(VoteAuthMethod::Presence));
    }

    /// @security — déclarer sa présence pour un vote à distance est
    /// exactement la fraude que l'authentification forte doit rendre
    /// impossible (#48) : refusé, pas silencieusement accepté.
    #[test]
    fn security_presence_insuffisante_pour_un_vote_distant() {
        let resultat =
            assert_vote_auth_sufficient(MeetingMode::Remote, Some(VoteAuthMethod::Presence), false);
        assert_eq!(
            resultat,
            Err(VoteAuthError::Insufficient {
                mode: MeetingMode::Remote,
                auth_method: VoteAuthMethod::Presence,
            })
        );
    }

    /// @negative — un vote sans `auth_method` du tout n'est pas exploitable
    /// en cas de contestation, quel que soit le mode de l'AG.
    #[test]
    fn negative_auth_method_absent_est_refuse() {
        let resultat = assert_vote_auth_sufficient(MeetingMode::InPerson, None, false);
        assert_eq!(resultat, Err(VoteAuthError::Missing));
    }

    #[test]
    fn edge_vote_auth_method_db_round_trip() {
        for m in [
            VoteAuthMethod::Presence,
            VoteAuthMethod::Proxy,
            VoteAuthMethod::Itsme,
            VoteAuthMethod::Eid,
        ] {
            assert_eq!(VoteAuthMethod::from_db_string(m.to_db_str()), Ok(m));
        }
    }

    #[test]
    fn negative_vote_auth_method_unknown_db_string_is_rejected() {
        assert!(VoteAuthMethod::from_db_string("carrier_pigeon").is_err());
    }

    /// @happy — `Vote::new` (chemin historique) vaut `Presence` par défaut :
    /// les appelants antérieurs à Story 4.2 ne changent pas de comportement.
    #[test]
    fn happy_vote_new_defaults_to_presence() {
        let vote = Vote::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            VoteChoice::Pour,
            dec!(100),
            None,
        )
        .expect("vote valide");
        assert_eq!(vote.auth_method, VoteAuthMethod::Presence);
    }

    #[test]
    fn happy_vote_new_with_auth_method_sets_field() {
        let vote = Vote::new_with_auth_method(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            VoteChoice::Pour,
            dec!(100),
            None,
            VoteAuthMethod::Itsme,
        )
        .expect("vote valide");
        assert_eq!(vote.auth_method, VoteAuthMethod::Itsme);
    }
}
