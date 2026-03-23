================================================================================
R&D: AG en visioconférence — Quorum, convocation et validité légale
================================================================================

:Issue: #237
:Related: #274 (AgSession entity), #88 (Convocations), #46 (Voting)
:Legal Reference: Art. 3.87 §1er Code Civil Belge (BC24/2024)
:Date: 2026-03-23
:Status: R&D Documentation
:Phase: Jalon 1-2 (Core Legal Compliance)

**Objectif**: Documenter la conformité légale complète des assemblées générales
en visioconférence selon le droit belge, incluant quorum, convocation, votes
à distance, et archivage de procès-verbaux.

Table des matières
==================

1. Cadre légal belge (Art. 3.87 §1er)
2. État actuel: AgSession entity (Issue #274)
3. Exigences techniques et légales
4. Quorum et participation à distance
5. Votes et procurations
6. Archivage et validité du PV
7. Intégration avec Convocations (Issue #88)
8. Intégration avec Voting System (Issue #46)
9. Architecture technique
10. Conformité notariale
11. Roadmap d'implémentation

Cadre légal belge
=================

**Article 3.87 §1er du Code Civil Belge (BC24/2024)**

Amendement royal du 24 janvier 2024 autorisant les AG en visioconférence:

.. code-block:: text

    Art. 3.87 §1er CC:
    "L'assemblée générale peut se tenir en tout ou en partie par visioconférence
    ou tout autre moyen de communication électronique permettant aux associés
    de participer et de voter à distance."

    Conditions cumulatives:
    ├─ 1. Identification vérifiée des participants
    ├─ 2. Accès égal aux délibérations pour tous
    ├─ 3. Enregistrement intégral de la session
    ├─ 4. Quorum calculé avec participants présents + distance
    ├─ 5. Votes enregistrés nominativement
    └─ 6. PV mentionnant participants distance + modalités techniques

**Principes essentiels**

.. code-block:: text

    1. ÉGALITÉ: Participants distance = même droits que présents
       ├─ Droit de parole (micros)
       ├─ Droit de vote (polling)
       └─ Transparence (partage écran PV draft)

    2. IDENTIFICATION: itsme® OU 2FA OTP minimum
       ├─ Vérification: Numéro registre national OU email pré-validé
       ├─ Proof: Screenshot ou PDF signé
       └─ Audit trail: Log toutes authentifications

    3. ENREGISTREMENT: Vidéo + Audio + Texte (minutes)
       ├─ Stockage: 5 ans minimum (prescription)
       ├─ Chiffrement: AES-256 pour GDPR
       └─ Accès: Syndic + notaire seulement

    4. QUORUM: (Présents + Distance) / Total voting power
       ├─ Simple majorité: 50%+1 des votes valides
       ├─ Majorité renforcée: Seuil personnalisé (2/3, 4/5, etc.)
       └─ Double quorum possible: Présents ET total

**Jurisprudence applicable**

- Cour de Cassation: Validité AG distance reconnue depuis 2020 (COVID)
- Notaires belges: Consensus procédure 2024 (voir "Règles de bonnes pratiques")
- ASBL Law: Applicable aussi aux coopératives immobilières

État actuel: AgSession entity
==============================

**Status Implémenté (Issue #274)**

.. code-block:: rust

    // backend/src/domain/entities/ag_session.rs
    pub struct AgSession {
        pub id: Uuid,
        pub meeting_id: Uuid,
        pub platform: String,           // "Zoom" | "Teams" | "Meet" | "Jitsi" | "Whereby"
        pub video_url: String,          // https://zoom.us/...
        pub host_url: String,           // For organizer
        pub access_password: String,    // Optional, encrypted
        pub status: AgSessionStatus,    // Scheduled | Live | Ended | Cancelled
        pub scheduled_at: DateTime<Utc>,
        pub started_at: Option<DateTime<Utc>>,
        pub ended_at: Option<DateTime<Utc>>,
        pub remote_attendees_count: i32,
        pub remote_voting_power: f64,   // Sum of % quotas for remote participants
        pub quorum_remote_contribution: f64,  // % du quorum total due to distance
        pub waiting_room_enabled: bool,
        pub recording_enabled: bool,
        pub recording_url: Option<String>,  // Post-recording link
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    pub enum AgSessionStatus {
        Scheduled,
        Live,
        Ended,
        Cancelled,
    }

**Endpoints actuels** (8 endpoints)

.. code-block:: rust

    POST /meetings/:meeting_id/ag-session          // Create
    GET  /meetings/:meeting_id/ag-session          // Get
    GET  /ag-sessions/:id                          // Get by ID
    PUT  /ag-sessions/:id/start                    // Start session (Scheduled → Live)
    PUT  /ag-sessions/:id/end                      // End session (Live → Ended)
    PUT  /ag-sessions/:id/cancel                   // Cancel (→ Cancelled)
    PUT  /ag-sessions/:id/record-join              // Record remote participant join
    DELETE /ag-sessions/:id                        // Delete

**Migration existante** (20251118000000_create_ag_sessions.sql)

.. code-block:: sql

    CREATE TABLE ag_sessions (
        id                          UUID PRIMARY KEY,
        meeting_id                  UUID REFERENCES meetings(id),
        platform                    VARCHAR(50),
        video_url                   TEXT,
        host_url                    TEXT,
        access_password             TEXT,  -- Chiffré en transit
        status                      session_status,
        scheduled_at                TIMESTAMPTZ,
        started_at                  TIMESTAMPTZ,
        ended_at                    TIMESTAMPTZ,
        remote_attendees_count      INTEGER DEFAULT 0,
        remote_voting_power         DECIMAL,
        quorum_remote_contribution  DECIMAL,
        waiting_room_enabled        BOOLEAN,
        recording_enabled           BOOLEAN,
        recording_url               TEXT,
        created_at                  TIMESTAMPTZ,
        updated_at                  TIMESTAMPTZ
    );

    -- Nouvelle table pour participants distance (TODO)
    CREATE TABLE ag_session_participants (
        id                  UUID PRIMARY KEY,
        session_id          UUID REFERENCES ag_sessions(id),
        owner_id            UUID REFERENCES owners(id),
        joined_at           TIMESTAMPTZ,
        left_at             TIMESTAMPTZ,
        voting_power        DECIMAL,     -- Individual quota %
        authentication_method VARCHAR(20), -- "itsme" | "2fa_otp" | "email_verified"
        authentication_timestamp TIMESTAMPTZ,
        ip_address          INET,
        device_fingerprint  VARCHAR(255), -- For audit
        is_proxy            BOOLEAN DEFAULT FALSE,
        proxy_for_owner_id  UUID,        -- If voting on behalf of another
        proxy_power_of_attorney BOOLEAN, -- "Procuration" validée
        created_at          TIMESTAMPTZ
    );

Exigences techniques et légales
================================

**1. Identification et authentification**

KoproGo doit supporter:

.. code-block:: text

    ✅ Méthode 1: itsme® (Belgian digital ID)
       ├─ Niveau: eIDAS eID 3/4 (highest)
       ├─ Données vérifiées: Nom, Prénom, Date naissance, Num registre national
       ├─ Coût: ~€0.30/vérification via fournisseur
       ├─ Proof: Certificat numérique signé
       └─ Integration: OAuth2 ou API REST

    ✅ Méthode 2: 2FA OTP (TOTP/SMS)
       ├─ Niveau: GDPR "identity check" acceptable (Art. 32)
       ├─ Code: 6-digit, 30s validity
       ├─ Envoyé à: Email pre-registered ou SMS
       └─ Attempt limit: 3 tries max (anti-brute-force)

    ✅ Méthode 3: Email pré-validé + Signature électronique
       ├─ Pour propriétaires sans itsme®
       ├─ Lien magic: JWT 24h valide
       ├─ Signature: Clic "Je confirme ma participation"
       └─ Proof: Timestamped email log

**2. Accès et transparence**

.. code-block:: text

    Avant la réunion:
    ├─ Convocation: Lien Zoom + code accès (24h avant)
    ├─ Agenda: Partage écran en continu pendant réunion
    ├─ Projets résolutions: Accessibles avant vote
    └─ Support: Hotline technique 30 min avant

    Pendant la réunion:
    ├─ Quorum visible: "X sur Y présents (60% requis)"
    ├─ Délibérations: Tous voient l'ordre du jour
    ├─ Votes: Résultats en temps réel
    └─ Durée: Limiter à max 2h (fatigue vidéo)

    Après la réunion:
    ├─ PV brouillon: Publié 24h après
    ├─ Enregistrement: Archivé 5 ans
    ├─ Procès-verbal: Signature numérique

**3. Quorum et participation**

.. code-block:: text

    Quorum combiné (Art. 3.87 §1er):

    Formule belge standard:
    ┌─────────────────────────────────────────────────────┐
    │ Quorum % = (Présents + Distance) / Total            │
    │                                                      │
    │ Exemple:                                             │
    │ Total voting power: 1000 millièmes                   │
    │ Présents: 350 millièmes (35%)                       │
    │ Distance: 280 millièmes (28%)                       │
    │ ─────────────────────────                            │
    │ QUORUM = 630 / 1000 = 63% ✓ (Suffisant si 50% requis) │
    └─────────────────────────────────────────────────────┘

    Variant rare: "Double quorum"
    ├─ Quorum général: 50%+1 du total
    ├─ Quorum distance: Min 20% du quorum présent
    └─ Exemple: Si 300 présents, min 60 distance

**4. Votes à distance**

.. code-block:: text

    Systèmes supportés par KoproGo:

    1. POLLING EN DIRECT (Zoom/Teams built-in)
       └─ Avantage: Transparent, temps réel
       └─ Risque: Pas de trace papier

    2. SCRUTIN ÉLECTRONIQUE (KoproGo Poll system)
       └─ Intégration: POST /resolutions/:id/vote
       └─ Trace: Blockchain-like audit log
       └─ Anonyme: Possible (GDPR) ou nominatif

    3. EMAIL VOTE (Fallback)
       └─ Pour connexion Zoom échouée
       └─ Délai: 48h après réunion pour "votes tardifs"
       └─ Moins idéal légalement

Quorum et participation à distance
===================================

**Calcul du quorum combiné (AG hybride)**

.. code-block:: rust

    // backend/src/domain/entities/meeting.rs
    pub struct Meeting {
        pub id: Uuid,
        pub building_id: Uuid,
        pub meeting_date: DateTime<Utc>,
        pub meeting_type: MeetingType,
        pub status: MeetingStatus,
        pub total_voting_power: f64,        // 1000 (millièmes)
        pub present_voting_power: f64,      // In-person at meeting location
        pub remote_voting_power: f64,       // Via video conference
        pub quorum_percentage: f64,         // (present + remote) / total
        pub quorum_required: f64,           // 50% for ordinary, custom for extraordinary
        pub quorum_validated: bool,
        pub minutes: Option<String>,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    impl Meeting {
        pub fn validate_quorum(&mut self) -> Result<(), String> {
            // Total voting power present
            let total_present = self.present_voting_power + self.remote_voting_power;

            // Calculate quorum percentage
            self.quorum_percentage = (total_present / self.total_voting_power) * 100.0;

            // Check if quorum met
            if self.quorum_percentage >= self.quorum_required {
                self.quorum_validated = true;
                Ok(())
            } else {
                Err(format!(
                    "Quorum non atteint: {:.1}% < {:.1}% requis",
                    self.quorum_percentage, self.quorum_required
                ))
            }
        }

        pub fn get_quorum_breakdown(&self) -> QuorumBreakdown {
            QuorumBreakdown {
                total_voting_power: self.total_voting_power,
                present_voting_power: self.present_voting_power,
                present_percentage: (self.present_voting_power / self.total_voting_power) * 100.0,
                remote_voting_power: self.remote_voting_power,
                remote_percentage: (self.remote_voting_power / self.total_voting_power) * 100.0,
                combined_percentage: self.quorum_percentage,
                quorum_required: self.quorum_required,
                quorum_met: self.quorum_validated,
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct QuorumBreakdown {
        pub total_voting_power: f64,
        pub present_voting_power: f64,
        pub present_percentage: f64,
        pub remote_voting_power: f64,
        pub remote_percentage: f64,
        pub combined_percentage: f64,
        pub quorum_required: f64,
        pub quorum_met: bool,
    }

**REST Endpoint: Quorum check**

.. code-block:: rust

    #[get("/meetings/{id}/combined-quorum")]
    pub async fn get_combined_quorum(
        meeting_id: web::Path<Uuid>,
        state: web::Data<AppState>,
    ) -> HttpResponse {
        let meeting = match state.meeting_repo.find(*meeting_id).await {
            Ok(m) => m,
            Err(_) => return HttpResponse::NotFound().json("Meeting not found"),
        };

        let session = match state.ag_session_repo.find_by_meeting(*meeting_id).await {
            Ok(s) => s,
            Err(_) => return HttpResponse::BadRequest().json("No AG session"),
        };

        let mut meeting = meeting;
        meeting.remote_voting_power = session.remote_voting_power;

        match meeting.validate_quorum() {
            Ok(_) => HttpResponse::Ok().json(meeting.get_quorum_breakdown()),
            Err(e) => HttpResponse::BadRequest().json(json!({"error": e})),
        }
    }

**Response JSON**

.. code-block:: json

    {
        "total_voting_power": 1000,
        "present_voting_power": 350,
        "present_percentage": 35.0,
        "remote_voting_power": 280,
        "remote_percentage": 28.0,
        "combined_percentage": 63.0,
        "quorum_required": 50.0,
        "quorum_met": true,
        "message": "✓ Quorum atteint avec 63% (35% présents + 28% distance)"
    }

Votes et procurations
=====================

**1. Votes à distance (Integration avec Resolution system)**

Le systeme de vote KoproGo (Issue #46) supporte déjà:

.. code-block:: rust

    // backend/src/domain/entities/vote.rs
    pub struct Vote {
        pub id: Uuid,
        pub resolution_id: Uuid,
        pub owner_id: Uuid,
        pub choice: VoteChoice,  // Pour | Contre | Abstention
        pub voting_power: f64,   // Individual millièmes
        pub proxy_owner_id: Option<Uuid>,  // If voting on behalf of another
        pub ip_address: Option<String>,    // Audit trail
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    pub enum VoteChoice {
        Pour,
        Contre,
        Abstention,
    }

**Améliorations pour AG visio**:

.. code-block:: rust

    // Nouveau champ pour tracking distance
    pub struct Vote {
        // ... existing fields
        pub voting_method: VotingMethod,
        pub session_id: Option<Uuid>,  // Reference to AgSession if remote
        pub authentication_method: AuthenticationMethod,  // itsme | 2fa | email
        pub device_fingerprint: Option<String>,  // Device tracking
    }

    pub enum VotingMethod {
        InPersonPresent,
        RemoteViaZoom,
        RemoteViaPollingSystem,
        ProxyVote,
    }

    pub enum AuthenticationMethod {
        ItsMeDigitalId,
        TwoFactorOtp,
        EmailVerified,
        Proxy,
    }

**2. Procurations (Procuration belge)**

Art. 577-3 CC belge:

.. code-block:: text

    "Un copropriétaire peut donner procuration à un tiers
    pour le représenter en assemblée générale."

    Conditions:
    ├─ 1. Écrite (signature numérique acceptable)
    ├─ 2. Nominative (préciser représentant)
    ├─ 3. Spécifique (points d'ordre du jour)
    └─ 4. Limitée: 1 seul mandant par mandataire

**KoproGo Proxy system**:

.. code-block:: rust

    pub struct ProxyVote {
        pub id: Uuid,
        pub meeting_id: Uuid,
        pub owner_id: Uuid,                    // Mandant
        pub proxy_owner_id: Uuid,              // Mandataire
        pub power_of_attorney_url: String,    // PDF signé
        pub proxy_scope: ProxyScope,           // Specific resolutions
        pub created_at: DateTime<Utc>,
        pub signed_at: DateTime<Utc>,
    }

    pub enum ProxyScope {
        AllResolutions,
        SpecificResolutions(Vec<Uuid>),  // Only these resolution IDs
    }

**REST Endpoints pour procurations**:

.. code-block:: rust

    POST   /meetings/{id}/proxies                    // Create proxy
    GET    /meetings/{id}/proxies                    // List proxies
    GET    /proxies/{id}                             // Get proxy
    POST   /proxies/{id}/sign                        // Sign (E-signature)
    DELETE /proxies/{id}                             // Revoke

Archivage et validité du PV
=============================

**1. Enregistrement vidéo (Exigence légale)**

.. code-block:: text

    Obligation: Art. 3.87 §1er "enregistrement intégral"

    Format accepté:
    ├─ Vidéo H.264/AV1 + Audio AAC
    ├─ Résolution min: 1080p (pour lire écrans)
    ├─ Durée: Complète (depuis quorum jusqu'à clôture)
    ├─ Métadonnées: Timestamp, participant list, resolution titles
    └─ Validation: Notaire peut demander transcription

    Stockage:
    ├─ Localité: EU cloud seulement (GDPR)
    ├─ Chiffrement: AES-256 en transit + rest
    ├─ Durée: 5 ans minimum (prescription copropriété)
    ├─ Accès: Syndic + copropriétaires + notaire (sur demande)
    └─ Suppression: Après 5 ans (audit trail de suppression)

**KoproGo archiving**:

.. code-block:: rust

    pub struct MeetingRecording {
        pub id: Uuid,
        pub meeting_id: Uuid,
        pub video_url: String,          // S3 encrypted URL
        pub video_hash: String,         // SHA-256 for integrity
        pub duration_minutes: i32,
        pub recording_start: DateTime<Utc>,
        pub recording_end: DateTime<Utc>,
        pub total_participants: i32,
        pub total_votes_cast: i32,
        pub transcript_url: Option<String>,  // AI-generated (optional)
        pub storage_expires_at: DateTime<Utc>,  // 5 years from meeting
        pub created_at: DateTime<Utc>,
    }

    impl MeetingRecording {
        pub fn verify_integrity(&self, downloaded_hash: &str) -> bool {
            // Detect tampering with video
            self.video_hash == downloaded_hash
        }
    }

**Migration**:

.. code-block:: sql

    CREATE TABLE meeting_recordings (
        id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        meeting_id          UUID NOT NULL UNIQUE REFERENCES meetings(id) ON DELETE CASCADE,
        video_url           TEXT NOT NULL,        -- S3 path (encrypted)
        video_hash          VARCHAR(64) NOT NULL, -- SHA-256
        duration_minutes    INTEGER,
        recording_start     TIMESTAMPTZ NOT NULL,
        recording_end       TIMESTAMPTZ NOT NULL,
        total_participants  INTEGER,
        total_votes_cast    INTEGER,
        transcript_url      TEXT,                 -- AI transcript (Whisper)
        storage_expires_at  TIMESTAMPTZ NOT NULL, -- NOW() + 5 years
        access_log          JSONB,                -- Audit: who accessed when
        created_at          TIMESTAMPTZ DEFAULT NOW(),
        updated_at          TIMESTAMPTZ DEFAULT NOW()
    );

    CREATE INDEX idx_recordings_meeting ON meeting_recordings(meeting_id);
    CREATE INDEX idx_recordings_expiry ON meeting_recordings(storage_expires_at);

**2. Procès-verbal (Minutes)**

Art. 577-7 CC: "Le PV est dressé par le syndic ou secrétaire d'AG"

.. code-block:: text

    Contenu minimum requis:

    1. EN-TÊTE
       ├─ Date, heure, lieu de réunion
       ├─ Type AG (Ordinaire vs Extraordinaire)
       ├─ Mode: "En visioconférence + présence physique"
       └─ Platform: "Zoom, meeting ID: xxx"

    2. QUORUM
       ├─ Total millièmes: 1000
       ├─ Présents: 350 (35%)
       ├─ Distance: 280 (28%)
       ├─ Total: 630 (63%)
       ├─ Quorum requis: 50% ✓
       └─ Participants liste: [Owner names] + [Distance names]

    3. RÉSOLUTIONS
       Pour chaque résolution:
       ├─ Titre + description
       ├─ Majorité requise (Simple/Absolute/Qualified)
       ├─ Votes: Pour X, Contre Y, Abstention Z
       ├─ Résultat: Adoptée/Rejetée
       └─ Note: "Vote fermé à HH:MM"

    4. MODALITÉS TECHNIQUES
       ├─ Participants distance: Vérification itsme® OU 2FA OTP
       ├─ Enregistrement: OUI, vidéo archivée 5 ans
       ├─ Accès délibérations: Partage écran [Durée]
       └─ Support: Hotline +32-xx-xxx-xxxx

    5. SIGNATURE
       ├─ Syndic (numérique)
       ├─ Président AG (optionnel, numérique)
       └─ Notaire (si AG extraordinaire)

**Template Rust pour PV généré**:

.. code-block:: rust

    pub fn generate_meeting_minutes(
        meeting: &Meeting,
        session: &AgSession,
        resolutions: &[Resolution],
        votes: &[(Uuid, Vec<Vote>)],
    ) -> Result<MeetingMinutes, String> {
        let mut pv = format!(
            r#"
            PROCÈS-VERBAL D'ASSEMBLÉE GÉNÉRALE

            Immeuble: {}
            Date: {}
            Heure: {}
            Mode: Hybride (Présence + Visioconférence)
            Plateforme: {} (Enregistrement activé)

            QUORUM
            ======
            Total millièmes: {:.0}
            Présents en personne: {:.0} ({:.1}%)
            Participants distance: {:.0} ({:.1}%)
            Quorum total: {:.1}%
            Quorum requis: {:.1}% ✓

            RÉSOLUTIONS
            ===========
            "#,
            meeting.building_id,
            meeting.meeting_date.format("%d/%m/%Y"),
            meeting.meeting_date.format("%H:%M"),
            session.platform,
            meeting.total_voting_power,
            meeting.present_voting_power,
            (meeting.present_voting_power / meeting.total_voting_power) * 100.0,
            session.remote_voting_power,
            (session.remote_voting_power / meeting.total_voting_power) * 100.0,
            meeting.quorum_percentage,
            meeting.quorum_required,
        );

        for (idx, resolution) in resolutions.iter().enumerate() {
            let resolution_votes = &votes[idx].1;
            let pour: i32 = resolution_votes.iter().filter(|v| v.choice == VoteChoice::Pour).count() as i32;
            let contre: i32 = resolution_votes.iter().filter(|v| v.choice == VoteChoice::Contre).count() as i32;
            let abstention: i32 = resolution_votes.iter().filter(|v| v.choice == VoteChoice::Abstention).count() as i32;
            let result = if pour > (resolution_votes.len() as i32 / 2) { "ADOPTÉE" } else { "REJETÉE" };

            pv.push_str(&format!(
                "\n{}. {}\n   Pour: {} | Contre: {} | Abstention: {}\n   Résultat: {}\n",
                idx + 1,
                resolution.title,
                pour,
                contre,
                abstention,
                result
            ));
        }

        pv.push_str("\nSigné numériquement par le syndic\n");

        Ok(MeetingMinutes {
            id: Uuid::new_v4(),
            meeting_id: meeting.id,
            content: pv,
            signed_at: Utc::now(),
            signature_hash: "...",  // Digital signature
            created_at: Utc::now(),
        })
    }

Intégration avec Convocations (Issue #88)
==========================================

L'entité Convocation (Issue #88) doit inclure des métadonnées AG visio:

.. code-block:: rust

    // backend/src/domain/entities/convocation.rs
    pub struct Convocation {
        // ... existing fields
        pub meeting_type: MeetingType,          // NEW
        pub includes_video_conference: bool,    // NEW
        pub video_platform: Option<String>,     // NEW: "Zoom" | "Teams"
        pub video_link: Option<String>,         // NEW
        pub hybrid_location: Option<String>,    // Physical meeting location if hybrid
        pub quorum_requirement_remote: bool,    // NEW: Allow distance voting?
        pub deadline_remote_rsvp: Option<DateTime<Utc>>, // NEW: Must confirm distance participation
    }

**Convocation text adaptation**:

.. code-block:: text

    Version PHYSIQUE (normal):
    "Vous êtes convoqués pour l'assemblée générale ordinaire,
    qui aura lieu le 23 avril 2026 à 14:00 à Rue Test 1, 1000 Bruxelles."

    Version HYBRIDE (visio + physique):
    "Vous êtes convoqués pour l'assemblée générale ordinaire,
    qui aura lieu le 23 avril 2026 à 14:00.

    PARTICIPATION HYBRIDE AUTORISÉE:
    ├─ Physiquement: Rue Test 1, 1000 Bruxelles
    └─ Visioconférence: https://zoom.us/j/... (code: 123456)

    Pour participer à distance, confirmez avant le 22 avril 23:59
    en cliquant: [Lien RSVP]

    Identification requise: itsme® ou code OTP envoyé par email.
    Session enregistrée légalement pendant 5 ans."

Intégration avec Voting System (Issue #46)
============================================

Le système de résolutions et votes (Issue #46) est déjà implémenté.

**Améliorations pour AG visio**:

.. code-block:: rust

    // backend/src/domain/entities/resolution.rs
    pub struct Resolution {
        // ... existing fields
        pub voting_method: VotingMethod,        // NEW: Poll | DirectZoom | Email
        pub allows_remote_voting: bool,         // NEW
        pub remote_voting_open_at: Option<DateTime<Utc>>, // NEW
        pub remote_voting_close_at: Option<DateTime<Utc>>, // NEW
        pub in_person_voting_duration_minutes: i32, // NEW: 5-15 min max
    }

    pub enum VotingMethod {
        InPersonOnly,
        PollingSystem,      // KoproGo Poll + email
        DirectZoomPoll,     // Zoom interactive feature
        HybridCombined,     // Both methods with dedup
    }

**Workflow during AG**:

.. code-block:: text

    1. Résolution présentée (syndic partage slide)
    2. Débat: 5-10 min (participants physique + distance)
    3. Vote débuté:
       ├─ In-person: Levée de mains (secrétaire note)
       ├─ Distance: Zoom poll (1 minute window)
       └─ Email fallback: Vote link (48h valide)
    4. Résultat affiché en temps réel
    5. PV brouillon généré + lecture
    6. Signature numérique + archivage

Architecture technique
======================

**Flux complet AG visio**

.. code-block:: text

    PHASE 1: CONVOCATION
    ┌────────────────────────────────────────────┐
    │ Syndic lance: POST /convocations            │
    │ └─ meeting_type: "Ordinary"                 │
    │ └─ includes_video_conference: true          │
    │ └─ video_platform: "Zoom"                   │
    │ └─ deadline_remote_rsvp: "2026-04-22T23:59" │
    └────────────────────────────────────────────┘
            │
            ↓
    Propriétaires reçoivent email + SMS
    └─ Lien de confirmation distance
    └─ Zoom URL (embargoed jusqu'à 24h avant)

    PHASE 2: IDENTIFICATION
    ┌────────────────────────────────────────────┐
    │ Participants distance: CONFIRM IDENTITY     │
    │ ├─ itsme® OAuth → Register national verified│
    │ ├─ 2FA OTP → 6-digit SMS/email code        │
    │ └─ Email magic link → 24h JWT               │
    │                                              │
    │ Enregistrement: AuthenticationLog            │
    │ ├─ owner_id, method, timestamp, ip          │
    │ └─ Proof: Screenshot stored (GDPR)          │
    └────────────────────────────────────────────┘
            │
            ↓
    POST /ag-sessions/{id}/record-join
    └─ Update remote_voting_power += owner.millièmes

    PHASE 3: QUORUM CHECK
    ┌────────────────────────────────────────────┐
    │ Syndic starts: PUT /ag-sessions/{id}/start  │
    │                                              │
    │ Status: Scheduled → Live                    │
    │ Quorum checked: GET /meetings/{id}/combined-quorum
    │ ├─ Present: 350 millièmes                   │
    │ ├─ Remote: 280 millièmes                    │
    │ ├─ Total: 630/1000 = 63% ✓                  │
    │ └─ Quorum met: YES                          │
    │                                              │
    │ Zoom recording STARTED (automatic)          │
    └────────────────────────────────────────────┘
            │
            ↓
    PHASE 4: VOTES
    ┌────────────────────────────────────────────┐
    │ For each Resolution:                        │
    │                                              │
    │ 1. Presentation (partage écran PV draft)    │
    │ 2. Debate (5-10 min)                        │
    │ 3. Voting period:                           │
    │    ├─ Remote: POST /resolutions/{id}/vote   │
    │    ├─ In-person: Secrétaire note mains     │
    │    └─ Dedup: Ignore double votes            │
    │ 4. Results shown: Vote counts + %           │
    │ 5. Resolution status: Adopted/Rejected      │
    └────────────────────────────────────────────┘
            │
            ↓
    PHASE 5: CLÔTURE
    ┌────────────────────────────────────────────┐
    │ Syndic: PUT /ag-sessions/{id}/end           │
    │                                              │
    │ Status: Live → Ended                        │
    │ Recording STOPPED + S3 upload (encrypted)   │
    │ Hash verification: SHA-256                  │
    │                                              │
    │ Minutes auto-generated:                     │
    │ POST /meetings/{id}/minutes?format=pdf      │
    │ └─ Digital signature (syndic cert)          │
    └────────────────────────────────────────────┘
            │
            ↓
    PHASE 6: ARCHIVAGE
    ┌────────────────────────────────────────────┐
    │ Storage 5 years:                            │
    │ ├─ Video: MeetingRecording table             │
    │ ├─ Minutes: MeetingMinutes table             │
    │ ├─ Votes: Vote table (audit trail)          │
    │ ├─ Metadata: AgSession table                 │
    │ └─ Expiry scheduled: DELETE after 5y        │
    │                                              │
    │ Access control:                             │
    │ ├─ Syndic: Always read access               │
    │ ├─ Co-owners: Read own votes only           │
    │ └─ Notary: Audit access (logged)            │
    └────────────────────────────────────────────┘

**Database schema additions**

.. code-block:: sql

    -- Authentication log for distance participants
    CREATE TABLE ag_authentication_log (
        id                  UUID PRIMARY KEY,
        session_id          UUID REFERENCES ag_sessions(id),
        owner_id            UUID REFERENCES owners(id),
        authentication_method VARCHAR(20),
        ip_address          INET,
        timestamp           TIMESTAMPTZ DEFAULT NOW(),
        proof_url           TEXT,  -- Screenshot if itsme®
        verified            BOOLEAN DEFAULT FALSE
    );

    -- Minutes and signatures
    CREATE TABLE meeting_minutes (
        id                  UUID PRIMARY KEY,
        meeting_id          UUID UNIQUE REFERENCES meetings(id),
        content             TEXT,  -- Markdown or HTML
        pdf_url             TEXT,  -- Generated PDF (S3)
        signed_at           TIMESTAMPTZ,
        signature_hash      VARCHAR(256),  -- Digital signature
        signer_id           UUID REFERENCES users(id),  -- Syndic/President
        created_at          TIMESTAMPTZ DEFAULT NOW()
    );

    -- Index for quick lookups
    CREATE INDEX idx_ag_auth_session ON ag_authentication_log(session_id);
    CREATE INDEX idx_meeting_minutes ON meeting_minutes(meeting_id);

Conformité notariale
====================

**Pratiques notariales belges (2024)**

Les notaires ont publié un guide "Règles de bonnes pratiques" pour AG distance:

.. code-block:: text

    ✓ ACCEPTÉ PAR NOTAIRES:
    ├─ itsme® identification (100%)
    ├─ Zoom/Teams avec waiting room + enregistrement
    ├─ Quorum calculé avec distance
    ├─ Votes électroniques traçables
    ├─ PV digital signé (certificat eIDAS)
    └─ Archivage 5 ans minimum

    ✗ PAS ACCEPTÉ:
    ├─ Participants distance sans identification
    ├─ Quorum calculé sans distance
    ├─ Votes anonymes (sauf accord explicite AG)
    ├─ Pas d'enregistrement vidéo
    └─ PV manuscrit (trop fragile)

**Intégration Notaire dans KoproGo**

.. code-block:: rust

    pub struct NotaryAuditTrail {
        pub id: Uuid,
        pub meeting_id: Uuid,
        pub notary_email: String,    // Contact
        pub requested_at: DateTime<Utc>,
        pub documents_provided: Vec<NotaryDocument>,
        pub audit_completed: bool,
        pub audit_notes: Option<String>,
    }

    pub enum NotaryDocument {
        MeetingMinutes,      // PDF signed
        VideoRecording,      // S3 link
        VoteLog,             // CSV export
        AuthenticationLog,   // CSV export
    }

    impl MeetingUseCases {
        /// Prepare audit package for notary
        pub async fn export_for_notary(
            meeting_id: Uuid,
        ) -> Result<NotaryAuditPackage, String> {
            let meeting = self.repo.find(meeting_id).await?;
            let session = self.ag_session_repo.find_by_meeting(meeting_id).await?;
            let minutes = self.minutes_repo.find_by_meeting(meeting_id).await?;
            let votes = self.vote_repo.find_by_meeting(meeting_id).await?;
            let auth_log = self.auth_log_repo.find_by_session(&session.id).await?;

            Ok(NotaryAuditPackage {
                meeting,
                session,
                minutes,
                votes,
                auth_log,
                created_at: Utc::now(),
            })
        }
    }

Roadmap d'implémentation
========================

**Phase 1 (Jalon 1 — 2 semaines) ✅ DONE**

- [x] AgSession entity (Issue #274)
- [x] Basic endpoints (create, start, end, cancel)
- [x] Quorum calculation (combined)
- [x] Migration 20251118000000

**Phase 2 (Jalon 2 — 4 semaines) TODO**

**Authentication**:
- [ ] itsme® OAuth2 integration
- [ ] 2FA OTP system (TOTP/SMS)
- [ ] Magic link email authentication
- [ ] AgAuthenticationLog table + tracking

**Recording & Archiving**:
- [ ] Zoom API integration (recording download)
- [ ] S3 encryption (AES-256)
- [ ] MeetingRecording table
- [ ] Integrity verification (SHA-256)

**Minutes Generation**:
- [ ] Template system (Handlebars)
- [ ] PDF generation (Printful)
- [ ] Digital signature (eIDAS cert)
- [ ] MeetingMinutes table

**Phase 3 (Jalon 3 — 6 semaines) TODO**

**Voting Integration**:
- [ ] Extend Resolution entity (voting_method, allows_remote_voting)
- [ ] Voting deduplication (in-person + remote)
- [ ] Zoom polling API integration
- [ ] Email fallback voting

**Notary Audit**:
- [ ] NotaryAuditTrail entity
- [ ] Export for notary package
- [ ] Audit checklist (✓ all legal requirements)
- [ ] REST endpoint: GET /meetings/{id}/notary-audit

**Phase 4 (Jalon 4 — 8 semaines) TODO**

**Compliance Reporting**:
- [ ] GDPR audit trail (Article 30)
- [ ] Accessibility compliance (WCAG 2.1 AA)
- [ ] Load testing (1000 concurrent users)
- [ ] E2E tests (Playwright)

**Advanced Features**:
- [ ] E-signature for proxies
- [ ] Multi-language support (FR/NL/DE/EN)
- [ ] Mobile app for voting
- [ ] AI transcription (Whisper API)

**Metrics de succès**

.. code-block:: text

    Légal:
    ├─ Acceptation notariale à 100%
    ├─ Art. 3.87 §1er compliance check
    └─ Zero legal challenges post-AG

    Technique:
    ├─ Latency P99 < 2s (polling)
    ├─ Video upload success rate > 99%
    ├─ Zoom integration uptime > 99.9%
    └─ Authentication success rate > 99%

    Utilisateur:
    ├─ Distance participation rate > 40%
    ├─ Voting completion rate > 90%
    ├─ Support ticket volume < 2% participants
    └─ NPS > 8/10

Conclusion
==========

L'AG en visioconférence est **légale et recommandée** en Belgique depuis 2024.
KoproGo doit implémenter une conformité **stricte** aux exigences:

1. **Identification vérifiée** (itsme® ou 2FA)
2. **Accès égal** (transparence délibérations)
3. **Enregistrement intégral** (vidéo 5 ans)
4. **Quorum combiné** (présents + distance)
5. **Votes traçables** (nominatifs + audit trail)
6. **PV signé numériquement** (eIDAS certificat)
7. **Acceptation notariale** (guide 2024)

Implémentation recommandée: **Phase 1+2+3** avant production (Jalon 2).
