use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type d'assemblée générale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum MeetingType {
    Ordinary,      // Assemblée Générale Ordinaire (AGO)
    Extraordinary, // Assemblée Générale Extraordinaire (AGE)
}

/// Statut de l'assemblée
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
pub enum MeetingStatus {
    Scheduled,
    Completed,
    Cancelled,
}

/// Représente une assemblée générale de copropriétaires
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meeting {
    pub id: Uuid,

    /// L'ACP dont c'est l'assemblée.
    ///
    /// Art. 3.87 § 1er : « Chaque propriétaire d'un lot fait partie de
    /// l'assemblée générale ». L'assemblée est l'organe de l'association, pas
    /// une réunion que le syndic organiserait pour son compte. Il la tient
    /// (Art. 3.87 § 2), il ne la possède pas. Cf. ADR-0045.
    pub acp_id: Uuid,

    /// Le syndic qui a tenu l'assemblée, conservé comme trace d'auteur.
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub meeting_type: MeetingType,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_date: DateTime<Utc>,
    pub location: String,
    pub status: MeetingStatus,
    pub agenda: Vec<String>,
    pub attendees_count: Option<i32>,
    // Quorum — Art. 3.87 §5 CC : AG valide si >50% des quotes-parts présentes/représentées
    pub quorum_validated: bool,
    pub quorum_percentage: Option<f64>, // % des quotes-parts présentes/représentées (0.0-100.0)
    pub total_quotas: Option<Decimal>,  // Total millièmes du bâtiment (Decimal exact — ADR-0008)
    pub present_quotas: Option<Decimal>, // Millièmes présents + représentés (Decimal exact — ADR-0008)
    // Second Convocation — Issue #311 (Art. 3.87 §5 CC: No quorum required for 2nd convocation)
    pub is_second_convocation: bool, // true = 2e convocation (no quorum check needed)
    // PV Distribution — Issue #313: Track when AG minutes are sent to owners
    pub minutes_document_id: Option<Uuid>, // FK to Document
    pub minutes_sent_at: Option<DateTime<Utc>>, // When PV was distributed
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Meeting {
    pub fn new(
        acp_id: Uuid,
        organization_id: Uuid,
        building_id: Uuid,
        meeting_type: MeetingType,
        title: String,
        description: Option<String>,
        scheduled_date: DateTime<Utc>,
        location: String,
    ) -> Result<Self, String> {
        if title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }
        if location.is_empty() {
            return Err("Location cannot be empty".to_string());
        }

        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            acp_id,
            organization_id,
            building_id,
            meeting_type,
            title,
            description,
            scheduled_date,
            location,
            status: MeetingStatus::Scheduled,
            agenda: Vec::new(),
            attendees_count: None,
            quorum_validated: false,
            quorum_percentage: None,
            total_quotas: None,
            present_quotas: None,
            is_second_convocation: false, // Default: first convocation
            minutes_document_id: None,
            minutes_sent_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn add_agenda_item(&mut self, item: String) -> Result<(), String> {
        if item.is_empty() {
            return Err("Agenda item cannot be empty".to_string());
        }
        self.agenda.push(item);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Transition d'état pure : `Scheduled → Completed`.
    ///
    /// **Track H Story H3** : ancien `complete(attendees_count)` renommé en
    /// `complete_internal()`. La validation des invariants Art. 3.87 §3-5 CC
    /// (convocations envoyées, votes clôturés, présences enregistrées,
    /// quorum, minutes draft) est gérée par
    /// `assert_can_complete(&checklist)` (cf. story H3).
    ///
    /// Le use-case `complete_meeting()` enchaîne :
    /// 1. `completion_checker.build_checklist(meeting_id)` (port DB)
    /// 2. `meeting.assert_can_complete(&checklist)?` (gate métier — 422 narratif)
    /// 3. `meeting.complete_internal()?` (state machine — cette méthode)
    /// 4. `repository.update(&meeting)`
    ///
    /// `attendees_count` (legacy param) reste accepté par le handler pour
    /// compat backward ; depuis Track H Story H3 la source de vérité est
    /// `checklist.attended_quotas` (présents + représentés agrégés DB-side).
    pub fn complete_internal(&mut self) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled => {
                self.status = MeetingStatus::Completed;
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Meeting is already completed".to_string()),
            MeetingStatus::Cancelled => Err("Cannot complete a cancelled meeting".to_string()),
        }
    }

    /// **DEPRECATED** — préservé pour la compatibilité backward des tests
    /// internes au domain qui historiquement passaient `attendees_count` à
    /// `complete()`. Délègue à `complete_internal()` puis stocke
    /// `attendees_count` sur l'entité.
    ///
    /// Nouveaux call-sites : utiliser `complete_internal()` après
    /// `assert_can_complete(&checklist)` — la valeur d'attendance est
    /// dérivée de la checklist (Art. 3.87 §5 CC, present_quotas DB-side).
    #[deprecated(
        note = "Use complete_internal() after assert_can_complete(&checklist) — Track H Story H3"
    )]
    pub fn complete(&mut self, attendees_count: i32) -> Result<(), String> {
        self.complete_internal()?;
        self.attendees_count = Some(attendees_count);
        Ok(())
    }

    /// Track H Story H3 — Vérifie que toutes les conditions Art. 3.87 §3-5 CC
    /// sont réunies pour clôturer la réunion. Retourne `Err` avec la liste
    /// exhaustive des invariants manquants pour permettre au FE de guider
    /// le syndic (cf. composant `<MissingInvariantsList>`).
    ///
    /// **Logique métier** :
    /// - `convocations_sent` (Art. 3.87 §3 CC — convocations envoyées en amont)
    /// - `open_resolutions == 0` (Art. 3.87 §4 CC — tous votes clôturés)
    /// - `attendance_recorded` (Art. 3.87 §5 CC — présences enregistrées)
    /// - **Quorum double** (Art. 3.87 §5, Story H9) : (A) têtes > 50 % strict
    ///   ET quotités ≥ 50 % inclusif, OU (B) alternative quotités > 3/4 strict.
    ///   Volet quotités KO → `QuorumNotReached` ; volet têtes KO →
    ///   `HeadCountQuorumNotReached`. `total_* == 0` → volet KO (pas de div/0).
    /// - `minutes_draft_exists` (PV draft sauvegardé avant clôture)
    ///
    /// **Pureté** : aucune I/O, aucune dépendance infra. La checklist est
    /// construite par le port `MeetingCompletionCheckerPort` (DB-side).
    pub fn assert_can_complete(
        &self,
        checklist: &MeetingCompletionChecklist,
    ) -> Result<(), MeetingNotCompletableError> {
        let mut missing: Vec<MissingInvariant> = Vec::new();

        if !checklist.convocations_sent {
            missing.push(MissingInvariant::ConvocationsNotSent);
        }
        if checklist.open_resolutions > 0 {
            missing.push(MissingInvariant::VotesNotClosed {
                open_resolutions: checklist.open_resolutions,
            });
        }
        if !checklist.attendance_recorded {
            missing.push(MissingInvariant::AttendanceNotRecorded);
        }

        // Story H9 — Quorum DOUBLE (Art. 3.87 §5 CC) :
        //   (A) primaire   : têtes > 50% (strict) ET quotités ≥ 50% (inclusif)
        //   (B) alternative: quotités > 3/4 (strict), quelles que soient les têtes
        //   sinon → 2e convocation (gérée ailleurs : `is_second_convocation`).
        // Comparaisons par multiplication croisée : exact (Decimal), pas de
        // division ⇒ pas de div/0 ni d'arrondi. `total_* <= 0` → volet KO.
        let quotas_alternative_ok =
            Self::quotas_three_quarters_reached(checklist.attended_quotas, checklist.total_quotas);
        if !quotas_alternative_ok {
            // Volet quotités ≥ 50% (inclusif — « au moins la moitié »).
            let quotas_half_ok =
                Self::quotas_half_reached(checklist.attended_quotas, checklist.total_quotas);
            if !quotas_half_ok {
                missing.push(MissingInvariant::QuorumNotReached {
                    attended_quotas: checklist.attended_quotas,
                    total_quotas: checklist.total_quotas,
                });
            }
            // Volet têtes > 50% (strict — « plus de la moitié des copropriétaires »).
            let heads_ok = Self::heads_majority_reached(
                checklist.present_owners_count,
                checklist.total_owners_count,
            );
            if !heads_ok {
                missing.push(MissingInvariant::HeadCountQuorumNotReached {
                    present_owners_count: checklist.present_owners_count,
                    total_owners_count: checklist.total_owners_count,
                });
            }
        }

        if !checklist.minutes_draft_exists {
            missing.push(MissingInvariant::MinutesDraftMissing);
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(MeetingNotCompletableError {
                meeting_id: self.id,
                missing,
            })
        }
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled => {
                self.status = MeetingStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Cannot cancel a completed meeting".to_string()),
            MeetingStatus::Cancelled => Err("Meeting is already cancelled".to_string()),
        }
    }

    pub fn reschedule(&mut self, new_date: DateTime<Utc>) -> Result<(), String> {
        match self.status {
            MeetingStatus::Scheduled | MeetingStatus::Cancelled => {
                self.scheduled_date = new_date;
                self.status = MeetingStatus::Scheduled;
                self.updated_at = Utc::now();
                Ok(())
            }
            MeetingStatus::Completed => Err("Cannot reschedule a completed meeting".to_string()),
        }
    }

    pub fn is_upcoming(&self) -> bool {
        self.status == MeetingStatus::Scheduled && self.scheduled_date > Utc::now()
    }

    // ------------------------------------------------------------------
    // Art. 3.87 §5 CC — prédicats du quorum double (#661)
    //
    // Source UNIQUE de la règle : `assert_can_complete` (chemin présentiel,
    // Story H9) et `AgSession::is_combined_quorum_reached` (chemin hybride
    // présentiel + distanciel) appellent tous deux ces fonctions. Avant #661,
    // chaque chemin portait son propre littéral `50`, avec des sémantiques qui
    // avaient déjà divergé (strict ici, inclusif là) sans que rien ne le
    // signale.
    //
    // Toutes les comparaisons se font par **multiplication croisée** en
    // `Decimal` : exact, sans division, donc sans arrondi ni division par zéro.
    // ------------------------------------------------------------------

    /// Alternative de l'Art. 3.87 §5 : quotités présentes **> 3/4** du total,
    /// quel que soit le nombre de têtes.
    pub fn quotas_three_quarters_reached(attended_quotas: Decimal, total_quotas: Decimal) -> bool {
        total_quotas > Decimal::ZERO && attended_quotas * dec!(4) > total_quotas * dec!(3)
    }

    /// Volet **quotités** du quorum double : au moins la moitié des quotes-parts
    /// (**inclusif** — « au moins la moitié »).
    pub fn quotas_half_reached(attended_quotas: Decimal, total_quotas: Decimal) -> bool {
        total_quotas > Decimal::ZERO && attended_quotas * dec!(2) >= total_quotas
    }

    /// Volet **têtes** du quorum double : plus de la moitié des copropriétaires
    /// présents ou représentés (**strict** — « plus de la moitié »).
    pub fn heads_majority_reached(present_owners_count: i32, total_owners_count: i32) -> bool {
        total_owners_count > 0 && present_owners_count * 2 > total_owners_count
    }

    /// Quorum double complet (Art. 3.87 §5 CC) :
    ///   (A) têtes > 50% **ET** quotités ≥ 50%, **ou**
    ///   (B) quotités > 3/4 seules.
    pub fn double_quorum_reached(
        attended_quotas: Decimal,
        total_quotas: Decimal,
        present_owners_count: i32,
        total_owners_count: i32,
    ) -> bool {
        Self::quotas_three_quarters_reached(attended_quotas, total_quotas)
            || (Self::quotas_half_reached(attended_quotas, total_quotas)
                && Self::heads_majority_reached(present_owners_count, total_owners_count))
    }

    /// Valide le quorum de l'AG (Art. 3.87 §5 CC).
    /// Quorum atteint si les quotes-parts présentes/représentées dépassent 50% du total.
    /// Retourne Ok(true) si quorum atteint, Ok(false) si insuffisant (2e convocation requise).
    ///
    /// Decimal exact — ADR-0008. `quorum_percentage` reste en f64 pour compat
    /// affichage (mémoire `no-f64-in-money` : f64 OK pour pourcentage display, pas pour
    /// montants/quotas).
    pub fn validate_quorum(
        &mut self,
        present_quotas: Decimal,
        total_quotas: Decimal,
    ) -> Result<bool, String> {
        use rust_decimal::prelude::ToPrimitive;
        if total_quotas <= Decimal::ZERO {
            return Err("Total quotas must be positive".to_string());
        }
        if present_quotas < Decimal::ZERO {
            return Err("Present quotas cannot be negative".to_string());
        }
        if present_quotas > total_quotas {
            return Err("Present quotas cannot exceed total quotas".to_string());
        }

        let percentage = (present_quotas / total_quotas) * dec!(100);
        // Volet **quotités seul** — `quorum_validated` sert de garde au vote
        // (cf. `check_quorum_for_voting`). Le quorum double complet — têtes ET
        // quotités, Art. 3.87 §5 — est jugé par `double_quorum_reached`, que
        // `assert_can_complete` applique à la clôture (#661).
        //
        // ⚠ DIVERGENCE CONSTATÉE, VOLONTAIREMENT NON CORRIGÉE ICI (#661) :
        // ce seuil est **strict** (> 50%) alors que `quotas_half_reached`,
        // utilisé par la clôture H9, est **inclusif** (≥ 50%). L'Art. 3.87 §5
        // dit « pour autant qu'ils possèdent au moins la moitié des
        // quotes-parts » — l'inclusif est donc le bon. Ce chemin-ci est plus
        // restrictif que la loi : il refuse un quorum à 50% pile que la loi
        // accepte. Défaut réel, mais changer le comportement du garde-fou de
        // vote dépasse le périmètre de #661 (qui porte sur le type, pas sur le
        // seuil) et casserait `test_quorum_not_reached_at_50_percent_exact`,
        // écrit pour la sémantique stricte. À trancher dans une story dédiée.
        let quorum_reached = percentage > dec!(50);

        self.present_quotas = Some(present_quotas);
        self.total_quotas = Some(total_quotas);
        self.quorum_percentage = percentage.to_f64();
        self.quorum_validated = quorum_reached;
        self.updated_at = Utc::now();

        Ok(quorum_reached)
    }

    /// Vérifie si le quorum est atteint avant d'autoriser un vote.
    /// Retourne Err si le quorum n'a pas encore été validé ou n'est pas atteint.
    ///
    /// EXCEPTION (Art. 3.87 §5 CC): No quorum check required for second convocation (is_second_convocation = true).
    /// Belgian law: 2e convocation = voting allowed without quorum requirement.
    pub fn check_quorum_for_voting(&self) -> Result<(), String> {
        // Art. 3.87 §5 CC: No quorum check needed for 2nd convocation
        if self.is_second_convocation {
            return Ok(());
        }

        if self.quorum_percentage.is_none() {
            return Err("Quorum has not been validated yet (Art. 3.87 §5 CC)".to_string());
        }
        if !self.quorum_validated {
            let pct = self.quorum_percentage.unwrap_or(0.0);
            return Err(format!(
                "Quorum not reached: {:.1}% present (>50% required, Art. 3.87 §5 CC). \
                 A second convocation is required.",
                pct
            ));
        }
        Ok(())
    }

    /// Sets minutes as sent (Issue #313: PV distribution tracking).
    /// Can only be called once meeting is Completed.
    pub fn set_minutes_sent(&mut self, document_id: Uuid) -> Result<(), String> {
        if self.status != MeetingStatus::Completed {
            return Err("Minutes can only be sent after meeting is completed".to_string());
        }
        self.minutes_document_id = Some(document_id);
        self.minutes_sent_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Checks if minutes are overdue (Issue #313: 30 days after meeting completion).
    /// Returns true if meeting is Completed, minutes not yet sent, and >30 days have passed.
    pub fn is_minutes_overdue(&self) -> bool {
        if self.status != MeetingStatus::Completed {
            return false;
        }
        if self.minutes_sent_at.is_some() {
            return false;
        }
        // Minutes are overdue if more than 30 days have passed since completion/update
        let deadline = self.updated_at + Duration::days(30);
        Utc::now() > deadline
    }
}

/// Track H Story H3 — Invariant légal manquant pour clôturer une AG.
///
/// Énuméré au lieu d'un simple message string : le frontend rend une liste
/// `<MissingInvariantsList>` typée par variant (label i18n distinct par
/// invariant) et conserve les métadonnées (open_resolutions, quotas).
///
/// **Pourquoi enum + struct fields** : permet au FE de localiser sans
/// parser de message, et de proposer la bonne action de correction (ex:
/// router vers panel résolutions si `VotesNotClosed`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type")]
pub enum MissingInvariant {
    /// Art. 3.87 §3 CC — Convocations pas envoyées (au moins une non `sent`).
    ConvocationsNotSent,
    /// Art. 3.87 §4 CC — Au moins une résolution est encore `Pending`.
    VotesNotClosed { open_resolutions: i32 },
    /// Art. 3.87 §5 CC — `present_quotas` non renseigné côté meeting.
    AttendanceNotRecorded,
    /// Art. 3.87 §5 CC — Volet **quotités** du quorum double non atteint :
    /// quotités présentes < 50 % (inclusif) ET < 3/4 (alternative). `total_quotas
    /// == 0` est mappé ici (pas de div by zero).
    QuorumNotReached {
        attended_quotas: Decimal,
        total_quotas: Decimal,
    },
    /// Story H9 — Art. 3.87 §5 CC — Volet **têtes** du quorum double non
    /// atteint : ≤ 50 % des copropriétaires présents/représentés (« plus de la
    /// moitié » requis, strict), et l'alternative > 3/4 des quotités n'est pas
    /// remplie. `total_owners_count == 0` est mappé ici.
    HeadCountQuorumNotReached {
        present_owners_count: i32,
        total_owners_count: i32,
    },
    /// PV draft (minutes) pas encore enregistré (cf. `minutes_document_id`).
    MinutesDraftMissing,
}

/// Track H Story H3 — Snapshot des conditions Art. 3.87 §3-5 CC pour
/// décider de la clôturabilité d'une AG.
///
/// **Pureté** : struct de données, agrégée par
/// `MeetingCompletionCheckerPort::build_checklist()` (1 query SQL agrégée).
/// `Meeting::assert_can_complete(&checklist)` consomme cette struct sans
/// faire d'I/O — propriété hexagonale.
///
/// **Decimal exact** (mémoire `no-f64-in-money`) : `attended_quotas` et
/// `total_quotas` sont en `Decimal` (millièmes / dix-millièmes selon acte
/// de base). Convention : `total_quotas = SUM(units.quota)` du building.
#[derive(Debug, Clone, PartialEq)]
pub struct MeetingCompletionChecklist {
    pub convocations_sent: bool,
    /// 0 = toutes résolutions clôturées (status != 'Pending').
    pub open_resolutions: i32,
    pub attendance_recorded: bool,
    /// Somme des quotas présents+représentés (cf. `meetings.present_quotas`).
    pub attended_quotas: Decimal,
    /// Somme des quotas du building (SUM units.quota).
    pub total_quotas: Decimal,
    /// Story H9 — Nombre de copropriétaires présents OU représentés (têtes).
    /// Source : `meetings.present_owners_count` (saisi par le syndic, comme
    /// `present_quotas`). Volet « têtes » du quorum double (Art. 3.87 §5).
    pub present_owners_count: i32,
    /// Story H9 — Nombre total de copropriétaires du building (têtes).
    /// Source : `COUNT(DISTINCT owner)` du building (calculé DB-side).
    pub total_owners_count: i32,
    /// `meetings.minutes_document_id IS NOT NULL`.
    pub minutes_draft_exists: bool,
}

/// Track H Story H3 — Erreur typée signalant que l'AG ne peut pas être
/// clôturée. Liste exhaustive des invariants manquants pour permettre au
/// FE de rendre `<MissingInvariantsList>` actionnable.
///
/// Mappée vers HTTP 422 `MEETING_NOT_COMPLETABLE` via `From<>` dans
/// `application/error.rs`. Le bridge `From<>` vers `String` permet aux
/// use-cases legacy `Result<_, String>` (cf. `complete_meeting()` qui n'a
/// pas encore migré vers `AppError`) de propager l'erreur via `?` ; le
/// handler décode le préfixe `MEETING_NOT_COMPLETABLE:` pour répondre 422
/// + payload structuré.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeetingNotCompletableError {
    pub meeting_id: Uuid,
    pub missing: Vec<MissingInvariant>,
}

impl std::fmt::Display for MeetingNotCompletableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Meeting {} not completable: {} missing invariant(s)",
            self.meeting_id,
            self.missing.len()
        )
    }
}

impl std::error::Error for MeetingNotCompletableError {}

#[cfg(test)]
#[allow(deprecated)] // tests legacy `complete()` — Track H Story H3
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_create_meeting_success() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            Some("Assemblée générale ordinaire annuelle".to_string()),
            future_date,
            "Salle des fêtes".to_string(),
        );

        assert!(meeting.is_ok());
        let meeting = meeting.unwrap();
        assert_eq!(meeting.organization_id, org_id);
        assert_eq!(meeting.status, MeetingStatus::Scheduled);
        assert!(meeting.is_upcoming());
    }

    #[test]
    fn test_add_agenda_item() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.add_agenda_item("Approbation des comptes".to_string());
        assert!(result.is_ok());
        assert_eq!(meeting.agenda.len(), 1);
    }

    #[test]
    fn test_complete_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.complete(45);
        assert!(result.is_ok());
        assert_eq!(meeting.status, MeetingStatus::Completed);
        assert_eq!(meeting.attendees_count, Some(45));
        assert!(!meeting.is_upcoming());
    }

    #[test]
    fn test_complete_already_completed_fails() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        meeting.complete(45).unwrap();
        let result = meeting.complete(50);
        assert!(result.is_err());
        assert_eq!(meeting.attendees_count, Some(45)); // Should not change
    }

    #[test]
    fn test_cancel_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.cancel();
        assert!(result.is_ok());
        assert_eq!(meeting.status, MeetingStatus::Cancelled);
    }

    #[test]
    fn test_quorum_reached_above_50_percent() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 600 millièmes présents sur 1000 = 60% → quorum atteint
        let result = meeting.validate_quorum(dec!(600), dec!(1000));
        assert!(result.is_ok());
        assert!(result.unwrap());
        assert!(meeting.quorum_validated);
        assert!((meeting.quorum_percentage.unwrap() - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_quorum_not_reached_at_50_percent_exact() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 500 millièmes sur 1000 = exactement 50% → quorum NON atteint (Art. 3.87 §5 : >50% requis)
        let result = meeting.validate_quorum(dec!(500), dec!(1000));
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!meeting.quorum_validated);
    }

    #[test]
    fn test_quorum_not_reached_below_50_percent() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // 400 millièmes sur 1000 = 40% → quorum non atteint
        let result = meeting.validate_quorum(dec!(400), dec!(1000));
        assert!(result.is_ok());
        assert!(!result.unwrap());
        assert!(!meeting.quorum_validated);
    }

    #[test]
    fn test_check_quorum_blocks_vote_when_not_validated() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.check_quorum_for_voting();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not been validated yet"));
    }

    #[test]
    fn test_check_quorum_skipped_for_second_convocation() {
        // Art. 3.87 §5 CC: No quorum check for 2nd convocation
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Extraordinary,
            "2e Convocation AGE".to_string(),
            Some("Deuxième convocation - sans quorum".to_string()),
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Mark as second convocation
        meeting.is_second_convocation = true;

        // Should allow voting even without quorum validation
        let result = meeting.check_quorum_for_voting();
        assert!(result.is_ok(), "2nd convocation should skip quorum check");
    }

    #[test]
    fn test_check_quorum_blocks_vote_when_quorum_not_reached() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        meeting.validate_quorum(dec!(400), dec!(1000)).unwrap();
        let result = meeting.check_quorum_for_voting();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("second convocation"));
    }

    #[test]
    fn test_quorum_invalid_total_quotas() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let result = meeting.validate_quorum(dec!(100), dec!(0));
        assert!(result.is_err());
    }

    #[test]
    fn test_reschedule_meeting() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        let new_date = Utc::now() + Duration::days(60);
        let result = meeting.reschedule(new_date);
        assert!(result.is_ok());
        assert_eq!(meeting.scheduled_date, new_date);
    }

    #[test]
    fn test_set_minutes_sent_success() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting first
        meeting.complete(45).unwrap();
        let result = meeting.set_minutes_sent(doc_id);

        // Assert
        assert!(result.is_ok());
        assert_eq!(meeting.minutes_document_id, Some(doc_id));
        assert!(meeting.minutes_sent_at.is_some());
    }

    #[test]
    fn test_set_minutes_sent_before_completion_fails() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Try to send minutes while meeting is still Scheduled
        let result = meeting.set_minutes_sent(doc_id);

        // Assert
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Minutes can only be sent after meeting is completed"
        );
    }

    #[test]
    fn test_is_minutes_overdue_not_completed() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act & Assert
        assert!(!meeting.is_minutes_overdue()); // Not completed yet
    }

    #[test]
    fn test_is_minutes_overdue_sent() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);
        let doc_id = Uuid::new_v4();

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act
        meeting.complete(45).unwrap();
        meeting.set_minutes_sent(doc_id).unwrap();

        // Assert
        assert!(!meeting.is_minutes_overdue()); // Minutes sent
    }

    #[test]
    fn test_is_minutes_overdue_past_30_days() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting and manually set updated_at to >30 days ago
        meeting.complete(45).unwrap();
        meeting.updated_at = Utc::now() - Duration::days(31);

        // Assert
        assert!(meeting.is_minutes_overdue()); // >30 days without sending minutes
    }

    #[test]
    fn test_is_minutes_overdue_within_30_days() {
        // Arrange
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let future_date = Utc::now() + Duration::days(30);

        let mut meeting = Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO 2024".to_string(),
            None,
            future_date,
            "Salle des fêtes".to_string(),
        )
        .unwrap();

        // Act: Complete the meeting
        meeting.complete(45).unwrap();

        // Assert
        assert!(!meeting.is_minutes_overdue()); // Within 30 days
    }
}

// ============================================================================
// Track H Story H3 — Tests `assert_can_complete` taxonomie 4-cat (CRITICAL #3).
// ============================================================================

#[cfg(test)]
#[allow(deprecated)] // ancien `complete()` reste testé pour la compat backward
mod assert_can_complete_tests {
    use super::*;
    use chrono::Duration;

    /// Construit un meeting standard (Scheduled, AGO).
    fn make_meeting() -> Meeting {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        Meeting::new(
            Uuid::new_v4(), // acp_id
            org_id,
            building_id,
            MeetingType::Ordinary,
            "AGO Track H Story H3".to_string(),
            None,
            Utc::now() + Duration::days(30),
            "Salle des fêtes".to_string(),
        )
        .unwrap()
    }

    /// Checklist 100% conforme : tous invariants présents, quorum atteint.
    fn checklist_all_ok() -> MeetingCompletionChecklist {
        MeetingCompletionChecklist {
            convocations_sent: true,
            open_resolutions: 0,
            attendance_recorded: true,
            attended_quotas: dec!(600),
            total_quotas: dec!(1000),
            // Story H9 — têtes 6/10 = 60% > 50% (volet têtes du quorum double OK).
            present_owners_count: 6,
            total_owners_count: 10,
            minutes_draft_exists: true,
        }
    }

    // ------------------------------------------------------------------------
    // @happy — chemin nominal
    // ------------------------------------------------------------------------

    #[test]
    fn happy_all_invariants_ok_returns_ok() {
        // AC-H3.h1
        let m = make_meeting();
        let c = checklist_all_ok();
        assert!(m.assert_can_complete(&c).is_ok());
    }

    #[test]
    fn happy_then_complete_internal_transitions_to_completed() {
        // AC-H3.h2 — chaînage assert + complete_internal.
        let mut m = make_meeting();
        let c = checklist_all_ok();
        m.assert_can_complete(&c).unwrap();
        m.complete_internal().unwrap();
        assert_eq!(m.status, MeetingStatus::Completed);
    }

    // ------------------------------------------------------------------------
    // @edge — bornes quorum, résolutions, quotas
    // ------------------------------------------------------------------------

    #[test]
    fn edge_quorum_quotas_exact_50_percent_accepted_with_heads_ok() {
        // Story H9 (corrige H3) — Art. 3.87 §5 : quotités « au moins la moitié »
        // = ≥ 50% INCLUSIF. 500/1000 = exactement 50% → volet quotités OK ;
        // têtes 6/10 (checklist_all_ok) OK → quorum double atteint.
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(500),
            total_quotas: dec!(1000),
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&c).is_ok());
    }

    #[test]
    fn edge_quorum_just_above_50_percent_is_accepted() {
        // AC-H3.e2 — 500.0001/1000 OK.
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(500.0001),
            total_quotas: dec!(1000),
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&c).is_ok());
    }

    #[test]
    fn edge_quorum_basis_10000_just_above_50_percent_is_accepted() {
        // Cas acte de base 10000 (cf. Story H1 fix building).
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(5000.5),
            total_quotas: dec!(10000),
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&c).is_ok());
    }

    #[test]
    fn edge_open_resolution_count_one_is_blocking() {
        // AC-H3.e4 — 1 résolution Pending bloque (cancelled/Adopted/Rejected
        // ne comptent pas dans `open_resolutions` — c'est la query qui filtre
        // `status='Pending'`).
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            open_resolutions: 1,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err.missing.iter().any(|m| matches!(
            m,
            MissingInvariant::VotesNotClosed {
                open_resolutions: 1
            }
        )));
    }

    #[test]
    fn edge_total_quotas_zero_returns_quorum_not_reached_not_panic() {
        // AC-H3.n3 — building soft-deleted ou cas exotic : pas de div by zero.
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(0),
            total_quotas: dec!(0),
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err.missing.iter().any(|m| matches!(
            m,
            MissingInvariant::QuorumNotReached { attended_quotas, total_quotas }
                if *attended_quotas == dec!(0) && *total_quotas == dec!(0)
        )));
    }

    // ------------------------------------------------------------------------
    // @security — pas de tampering, payload sain
    // ------------------------------------------------------------------------

    #[test]
    fn security_assert_is_pure_no_state_mutation() {
        // assert_can_complete ne mute pas l'entité. Vérifie qu'après un appel
        // (Ok ou Err) le meeting reste Scheduled (state machine non touchée).
        let m = make_meeting();
        let scheduled_status_before = m.status.clone();
        let _ = m.assert_can_complete(&checklist_all_ok());
        assert_eq!(m.status, scheduled_status_before);

        // Même chose sur un Err.
        let c = MeetingCompletionChecklist {
            convocations_sent: false,
            ..checklist_all_ok()
        };
        let _ = m.assert_can_complete(&c);
        assert_eq!(m.status, scheduled_status_before);
    }

    #[test]
    fn security_attendees_count_param_is_ignored_source_of_truth_is_checklist() {
        // AC-H3.s1 — attendees_count (legacy) n'influence pas assert.
        // Démonstration : `complete_internal()` ne touche pas attendees_count,
        // et `assert_can_complete()` calcule depuis checklist.attended_quotas.
        // Un attaquant qui forge attendees_count via le handler ne peut pas
        // bypasser le quorum.
        let mut m = make_meeting();
        m.attendees_count = Some(99_999); // forged
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(100), // réel : insuffisant
            total_quotas: dec!(1000),
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err
            .missing
            .iter()
            .any(|m| matches!(m, MissingInvariant::QuorumNotReached { .. })));
    }

    // ------------------------------------------------------------------------
    // @negative — défaillance correcte (toutes conditions absentes)
    // ------------------------------------------------------------------------

    #[test]
    fn negative_all_invariants_missing_returns_all_six() {
        // Story H9 — toutes conditions falsy → 6 MissingInvariant exhaustifs
        // (le quorum double ajoute HeadCountQuorumNotReached au volet têtes).
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            convocations_sent: false,
            open_resolutions: 3,
            attendance_recorded: false,
            attended_quotas: dec!(0),
            total_quotas: dec!(1000),
            present_owners_count: 0,
            total_owners_count: 10,
            minutes_draft_exists: false,
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        // 6 : ConvocationsNotSent + VotesNotClosed + AttendanceNotRecorded
        // + QuorumNotReached (quotités) + HeadCountQuorumNotReached (têtes)
        // + MinutesDraftMissing
        assert_eq!(err.missing.len(), 6);
        assert!(matches!(
            err.missing[0],
            MissingInvariant::ConvocationsNotSent
        ));
        assert!(matches!(
            err.missing[1],
            MissingInvariant::VotesNotClosed {
                open_resolutions: 3
            }
        ));
        assert!(matches!(
            err.missing[2],
            MissingInvariant::AttendanceNotRecorded
        ));
        assert!(matches!(
            err.missing[3],
            MissingInvariant::QuorumNotReached { .. }
        ));
        assert!(matches!(
            err.missing[4],
            MissingInvariant::HeadCountQuorumNotReached { .. }
        ));
        assert!(matches!(
            err.missing[5],
            MissingInvariant::MinutesDraftMissing
        ));
    }

    #[test]
    fn negative_only_convocations_missing_returns_one_invariant() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            convocations_sent: false,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert_eq!(err.missing.len(), 1);
        assert_eq!(err.missing[0], MissingInvariant::ConvocationsNotSent);
        assert_eq!(err.meeting_id, m.id);
    }

    #[test]
    fn negative_only_minutes_missing_returns_one_invariant() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            minutes_draft_exists: false,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert_eq!(err.missing.len(), 1);
        assert_eq!(err.missing[0], MissingInvariant::MinutesDraftMissing);
    }

    #[test]
    fn negative_display_does_not_leak_business_internals() {
        // Display = "Meeting X not completable: N missing invariant(s)". Pas
        // de fuite de quotas / IDs sensibles dans le Display public (le
        // détail vit dans le payload structuré JSON 422, pas en clair).
        let err = MeetingNotCompletableError {
            meeting_id: Uuid::new_v4(),
            missing: vec![MissingInvariant::QuorumNotReached {
                attended_quotas: dec!(400),
                total_quotas: dec!(1000),
            }],
        };
        let s = format!("{}", err);
        assert!(s.contains("not completable"));
        // Display ne contient PAS la valeur des quotas (réservé au JSON).
        assert!(!s.contains("400"));
        assert!(!s.contains("1000"));
    }

    #[test]
    fn negative_complete_internal_on_completed_meeting_fails() {
        // AC-H3.n2 — déjà Completed → complete_internal Err, sans toucher au
        // status.
        let mut m = make_meeting();
        m.assert_can_complete(&checklist_all_ok()).unwrap();
        m.complete_internal().unwrap();
        // Re-tente : déjà Completed.
        let res = m.complete_internal();
        assert!(res.is_err());
        assert_eq!(m.status, MeetingStatus::Completed);
    }

    // ------------------------------------------------------------------------
    // Story H9 (CL3) — Quorum DOUBLE têtes + quotités (Art. 3.87 §5) — 4-cat.
    // ------------------------------------------------------------------------

    /// @happy — primaire : têtes > 50% ET quotités ≥ 50% → clôture OK.
    #[test]
    fn happy_double_quorum_primary_heads_and_quotas() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(500), // 50% exact → inclusif OK
            total_quotas: dec!(1000),
            present_owners_count: 6, // 60% têtes OK
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&c).is_ok());
    }

    /// @happy — alternative : quotités > 3/4 suffit même si têtes ≤ 50%.
    #[test]
    fn happy_double_quorum_alternative_three_quarters() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(760), // 76% > 3/4
            total_quotas: dec!(1000),
            present_owners_count: 2, // 20% têtes (insuffisant en primaire)
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&c).is_ok());
    }

    /// @edge — têtes exactement 50% (strict requis) → volet têtes KO,
    /// volet quotités OK (pas de QuorumNotReached).
    #[test]
    fn edge_heads_exactly_50_percent_rejected() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(600),
            total_quotas: dec!(1000),
            present_owners_count: 5, // 5/10 = 50% exact → KO (strict)
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err
            .missing
            .iter()
            .any(|x| matches!(x, MissingInvariant::HeadCountQuorumNotReached { .. })));
        assert!(!err
            .missing
            .iter()
            .any(|x| matches!(x, MissingInvariant::QuorumNotReached { .. })));
    }

    /// @edge — quotités exactement 75% (pas > 3/4) + têtes faibles → KO ;
    /// juste au-dessus de 75% → OK (alternative).
    #[test]
    fn edge_three_quarters_boundary() {
        let m = make_meeting();
        let exactly = MeetingCompletionChecklist {
            attended_quotas: dec!(750), // 75% pile, pas strict > 3/4
            total_quotas: dec!(1000),
            present_owners_count: 3, // 30% têtes
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&exactly).is_err());

        let above = MeetingCompletionChecklist {
            attended_quotas: dec!(750.001), // > 3/4 → alternative OK
            total_quotas: dec!(1000),
            present_owners_count: 3,
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        assert!(m.assert_can_complete(&above).is_ok());
    }

    /// @security — têtes forgées via `attendees_count` (legacy) sans effet :
    /// la source est la checklist (DB COUNT DISTINCT owners).
    #[test]
    fn security_head_count_source_is_checklist_not_forged_field() {
        let mut m = make_meeting();
        m.attendees_count = Some(99_999); // forgé
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(600), // quotités OK
            total_quotas: dec!(1000),
            present_owners_count: 1, // réel : 1/10 têtes → KO
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err
            .missing
            .iter()
            .any(|x| matches!(x, MissingInvariant::HeadCountQuorumNotReached { .. })));
    }

    /// @negative — têtes KO seules (quotités OK) → uniquement HeadCount.
    #[test]
    fn negative_only_head_count_quorum_missing() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(600),
            total_quotas: dec!(1000),
            present_owners_count: 4, // 40% têtes KO
            total_owners_count: 10,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert_eq!(err.missing.len(), 1);
        assert!(matches!(
            err.missing[0],
            MissingInvariant::HeadCountQuorumNotReached {
                present_owners_count: 4,
                total_owners_count: 10
            }
        ));
    }

    /// @negative — total_owners_count == 0 → HeadCountQuorumNotReached (pas de div).
    #[test]
    fn negative_zero_total_owners_is_head_quorum_not_reached() {
        let m = make_meeting();
        let c = MeetingCompletionChecklist {
            attended_quotas: dec!(600),
            total_quotas: dec!(1000),
            present_owners_count: 0,
            total_owners_count: 0,
            ..checklist_all_ok()
        };
        let err = m.assert_can_complete(&c).unwrap_err();
        assert!(err
            .missing
            .iter()
            .any(|x| matches!(x, MissingInvariant::HeadCountQuorumNotReached { .. })));
    }
}

impl crate::domain::services::PieceDeGestion for Meeting {
    fn acp_id(&self) -> Uuid {
        self.acp_id
    }
}
