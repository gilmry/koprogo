=========================================
Open Source PropTech Projects Research
=========================================

:Date: 2025-11-10
:Status: Research Complete
:Focus: EU-compatible GPL/MIT/Apache licensed projects

.. contents:: Table of Contents
   :depth: 3

Executive Summary
=================

Cette recherche identifie les projets open source (GPL-2.0, MIT, Apache-2.0) réutilisables pour les fonctionnalités PropTech de KoproGo. L'objectif est double:

1. **Containerisation**: Encapsuler des projets existants comme microservices
2. **Réimplémentation**: S'inspirer du code en citant l'auteur pour une implémentation Rust

**Focus prioritaire**: Projets compatibles avec les réglementations européennes (GDPR, eIDAS).

Projets Open Source par Domaine
================================

1. Gestion de Copropriété (Property Management)
------------------------------------------------

Diacamma Syndic ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: Diacamma Syndic
:URL: https://github.com/Diacamma2/syndic
:Licence: **GPL-3.0**
:Langage: Python (Django)
:Focus: Gestion de copropriété pour syndics bénévoles (France/Belgique)

**Fonctionnalités couvertes:**

- ✅ Gestion des lots et copropriétaires
- ✅ Comptabilité conforme (France, adaptable PCMN Belge)
- ✅ Appels de fonds et charges
- ✅ Assemblées générales (AG)
- ✅ Documents et procès-verbaux
- ✅ Budget prévisionnel
- ✅ États financiers

**Recommandation:**

⚠️ **GPL-3.0 = Contamination virale**. Ne PAS containeriser directement.

**Action suggérée**: **S'INSPIRER** du code pour réimplémentation Rust:

- Analyser la structure comptable (adaptation PCMN belge)
- Étudier le workflow des assemblées générales
- Documenter les règles métier françaises vs belges
- **Citer Diacamma2** comme source d'inspiration dans la documentation

**Commande d'analyse:**

.. code-block:: bash

   git clone https://github.com/Diacamma2/syndic.git /tmp/diacamma-syndic
   # Analyser: diacamma/syndic/models.py, views.py, accounting/

Condo (Open Condo Software) ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: Condo
:URL: https://github.com/open-condo-software/condo
:Licence: **MIT**
:Langage: Node.js (PostgreSQL 16, Redis 6)
:Focus: SaaS property management avec marketplace

**Fonctionnalités couvertes:**

- ✅ Ticketing (maintenance requests)
- ✅ Gestion des résidents (contacts)
- ✅ Gestion des propriétés
- ✅ Suivi des paiements
- ✅ Facturation
- ✅ Marketplace de services
- ✅ Système d'extensions (mini-apps)

**Recommandation:**

✅ **MIT = Compatible containerisation ET réimplémentation**

**Option 1 - Containerisation** (court terme):

.. code-block:: yaml

   # docker-compose.yml
   services:
     condo:
       image: opencondo/condo:latest
       ports:
         - "3004:3000"
       environment:
         DATABASE_URL: postgresql://...
         REDIS_URL: redis://...
       networks:
         - koprogo-net

**Option 2 - Réimplémentation Rust** (long terme):

- Analyser l'architecture PostgreSQL (schéma DB)
- Étudier le système de ticketing
- S'inspirer du modèle de marketplace
- **Citer Open Condo Software** dans la documentation

**Issues KoproGo concernées:**

- #85 (Maintenance Ticketing) → Inspiration directe
- #52 (Contractor Backoffice) → Marketplace pattern
- #84 (Online Payment) → Payment tracking patterns

MicroRealEstate
~~~~~~~~~~~~~~~

:Projet: MicroRealEstate
:URL: https://github.com/microrealestate/microrealestate
:Licence: MIT
:Langage: Node.js (MongoDB)
:Focus: Gestion locative pour propriétaires

**Fonctionnalités:**

- Location/locataires (non prioritaire pour copropriété)
- Suivi des loyers (adaptable pour charges)
- Documents (bail, quittances)

**Recommandation**: Moins pertinent pour copropriété, focus sur location.


2. Systèmes de Vote Électronique
---------------------------------

ElectionGuard ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: ElectionGuard
:URL: https://github.com/Election-Tech-Initiative/electionguard
:Licence: **MIT**
:Auteur: Microsoft + Election Tech Initiative
:Langage: Python, TypeScript
:Focus: Vote électronique end-to-end verifiable

**Caractéristiques:**

- ✅ Chiffrement homomorphe (votes restent chiffrés)
- ✅ Vérifiabilité end-to-end
- ✅ Audit tiers possible sans compromettre le secret
- ✅ Conforme standards électoraux USA

**⚠️ Limitations pour KoproGo:**

- ❌ Non compatible eIDAS (authentification européenne)
- ❌ Pas de support itsme® (Belgique)
- ❌ Focus élections publiques, pas AG de copropriété

**Recommandation:**

**S'INSPIRER** des concepts cryptographiques uniquement:

- Chiffrement homomorphe pour compter les votes sans les déchiffrer
- Système de bulletins vérifiables
- **NE PAS utiliser directement** (complexité excessive pour AG)

**Action suggérée:**

Pour **Issue #46** (Meeting Voting System):

1. **Phase 1 (VPS)**: Vote simple en DB PostgreSQL avec authentification JWT
2. **Phase 2 (K3s)**: Ajouter signature numérique (itsme® - Issue #48)
3. **Phase 3 (K8s)**: Si besoin légal, étudier chiffrement homomorphe inspiré d'ElectionGuard

Helios Voting
~~~~~~~~~~~~~

:Projet: Helios
:URL: https://github.com/benadida/helios-server
:Licence: **Apache 2.0** (backend) + **GPL-3.0** (frontend)
:Langage: Python (Django)
:Focus: Vote en ligne open-audit

**Recommandation:**

⚠️ Frontend GPL-3.0 = problématique. Backend Apache OK.

**Action**: Même recommandation qu'ElectionGuard (inspiration cryptographique uniquement).


VotoSocial (Blockchain)
~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: VotoSocial
:URL: https://votosocial.github.io/
:Licence: Open Source (détails non précisés)
:Tech: Blockchain + colored coins
:Focus: Vote public immuable

**⚠️ Limitations:**

- ❌ Votes publics et traçables (incompatible secret du vote AG)
- ❌ Blockchain = complexité excessive pour copropriété
- ❌ Pas de support GDPR (droit à l'oubli impossible sur blockchain publique)

**Recommandation**: **NE PAS UTILISER**. Blockchain publique incompatible GDPR Article 17.


3. Systèmes de Notifications Multi-Canal
-----------------------------------------

Novu ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~

:Projet: Novu
:URL: https://github.com/novuhq/novu
:Licence: **MIT**
:Langage: Node.js + TypeScript
:Focus: Infrastructure de notifications omnicanal

**Fonctionnalités:**

- ✅ Email (SMTP, SendGrid, Mailgun, etc.)
- ✅ SMS (Twilio, SNS, etc.)
- ✅ Push notifications (Web Push, FCM, APNs)
- ✅ In-app notifications (Inbox)
- ✅ Slack, Discord, Teams, WhatsApp
- ✅ Templates multi-langues
- ✅ Préférences utilisateur (opt-in/opt-out)
- ✅ File d'attente asynchrone
- ✅ Retry logic
- ✅ Delivery tracking

**Recommandation:**

✅ **MIT = EXCELLENT candidat pour containerisation**

**Option 1 - Microservice Novu** (RECOMMANDÉ):

.. code-block:: yaml

   # docker-compose.yml
   services:
     novu-api:
       image: ghcr.io/novuhq/novu/api:latest
       environment:
         NODE_ENV: production
         REDIS_URL: redis://redis:6379
         MONGO_URL: mongodb://mongo:27017/novu
       ports:
         - "3005:3000"

     novu-worker:
       image: ghcr.io/novuhq/novu/worker:latest
       environment:
         REDIS_URL: redis://redis:6379

**Intégration KoproGo** (Rust):

.. code-block:: rust

   // backend/src/infrastructure/notifications/novu_client.rs
   use reqwest::Client;

   pub struct NovuClient {
       api_key: String,
       base_url: String,
       client: Client,
   }

   impl NovuClient {
       pub async fn trigger_notification(
           &self,
           event: &str,
           recipient_id: &str,
           payload: serde_json::Value,
       ) -> Result<(), String> {
           // POST https://novu-api:3000/v1/events/trigger
           // Crédit: Novu (MIT) - github.com/novuhq/novu
       }
   }

**Issues KoproGo concernées:**

- #86 (Multi-Channel Notifications) → Remplacement direct
- #88 (Automatic AG Convocations) → Email delivery

**Option 2 - Bibliothèques Rust natives:**

Si pas de containerisation:

- **lettre** (MIT) pour email SMTP
- Custom implementation pour push notifications

ntfy (Apache 2.0)
~~~~~~~~~~~~~~~~~

:Projet: ntfy
:URL: https://ntfy.sh
:Licence: **Apache 2.0** + **GPL-2.0** (dual)
:Langage: Go
:Focus: Push notifications simples (PUT/POST)

**Recommandation**: Plus simple que Novu, mais moins de canaux. Bon pour MVP.


4. Gestion Documentaire (DMS)
------------------------------

**⚠️ AUCUN projet Rust mature trouvé**

Paperless-ngx
~~~~~~~~~~~~~

:Projet: Paperless-ngx
:URL: https://github.com/paperless-ngx/paperless-ngx
:Licence: GPL-3.0
:Langage: Python
:Focus: Document scanning + OCR + archiving

**Recommandation**: GPL-3.0 = contamination. **NE PAS containeriser**.

**Action suggérée**: **Implémentation Rust native** avec bibliothèques:

.. code-block:: rust

   // Bibliothèques Rust pour DMS (MIT/Apache)

   // Upload/Download fichiers
   use actix_multipart::Multipart; // Apache-2.0 / MIT
   use actix_files::NamedFile;     // Apache-2.0 / MIT

   // Stockage S3-compatible
   use rusoto_s3::S3Client;        // MIT
   // OU
   use aws_sdk_s3;                 // Apache-2.0

   // Métadonnées
   use serde::{Serialize, Deserialize};
   use sqlx::PgPool;               // Apache-2.0 / MIT

**Pattern d'implémentation** (Issue #76):

.. code-block:: rust

   // backend/src/domain/entities/document.rs
   pub struct Document {
       pub id: Uuid,
       pub title: String,
       pub file_path: String,           // OU S3 key
       pub document_type: DocumentType, // MeetingMinutes, Invoice, etc.
       pub mime_type: String,
       pub size_bytes: i64,
       pub uploaded_by: Uuid,
       pub building_id: Option<Uuid>,
       pub created_at: DateTime<Utc>,
   }

   // backend/src/infrastructure/storage/s3_storage.rs
   pub struct S3Storage {
       client: S3Client,
       bucket: String,
   }

   impl StoragePort for S3Storage {
       async fn upload(&self, file: &[u8], key: &str) -> Result<String, String> {
           // MinIO ou AWS S3
       }
   }

**Pas de containerisation nécessaire** - implémentation native suffisante.


5. Timebank / SEL (Système d'Échange Local)
--------------------------------------------

TimeOverflow ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: TimeOverflow
:URL: https://github.com/coopdevs/timeoverflow
:Licence: **AGPL-3.0**
:Langage: Ruby on Rails
:Focus: Time banking (1 heure = 1 crédit)

**Fonctionnalités:**

- ✅ Échange de services entre membres
- ✅ Comptabilité temps (crédits)
- ✅ Annuaire de compétences
- ✅ Historique des transactions

**⚠️ Problème: AGPL-3.0**

L'AGPL est **plus restrictive que GPL**:

- GPL: contamination si linkage statique
- AGPL: contamination si accès réseau (SaaS = trigger AGPL)

**Recommandation:**

❌ **NE PAS containeriser** (AGPL contaminerait KoproGo via API)

✅ **RÉIMPLÉMENTER en Rust** avec inspiration:

.. code-block:: rust

   // backend/src/domain/entities/sel_transaction.rs
   // Inspiré de TimeOverflow (AGPL-3.0) - github.com/coopdevs/timeoverflow
   // Réimplémenté pour compatibilité licence

   pub struct SelTransaction {
       pub id: Uuid,
       pub giver_id: Uuid,        // Qui donne le service
       pub receiver_id: Uuid,     // Qui reçoit le service
       pub service_type: String,  // "Bricolage", "Jardinage", etc.
       pub hours: f32,            // 1 heure = 1 crédit
       pub description: String,
       pub transaction_date: DateTime<Utc>,
       pub status: TransactionStatus, // Pending, Confirmed, Disputed
   }

   pub struct SelBalance {
       pub owner_id: Uuid,
       pub credits_earned: f32,   // Services donnés
       pub credits_spent: f32,    // Services reçus
       pub balance: f32,          // earned - spent
   }

**Issues KoproGo concernées:**

- #49 (Community Features - SEL)
- #99 (Community Modules)


6. Ticketing / Helpdesk
-----------------------

**⚠️ AUCUN projet Rust mature trouvé**

osTicket, FreeScout, UVdesk
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:Licences: GPL / MIT (selon projet)
:Langages: PHP
:Focus: Support ticketing

**Recommandation:**

Tous en PHP. **Implémentation Rust native** préférable pour cohérence architecture.

**Pattern suggéré** (Issue #85):

.. code-block:: rust

   // backend/src/domain/entities/maintenance_ticket.rs
   pub struct MaintenanceTicket {
       pub id: Uuid,
       pub title: String,
       pub description: String,
       pub category: TicketCategory, // Plumbing, Electricity, etc.
       pub priority: Priority,       // Critical, High, Medium, Low
       pub status: TicketStatus,     // Open, InProgress, Resolved, Closed
       pub building_id: Uuid,
       pub unit_id: Option<Uuid>,
       pub reported_by: Uuid,
       pub assigned_to: Option<Uuid>,
       pub photos: Vec<String>,      // S3 keys
       pub created_at: DateTime<Utc>,
       pub resolved_at: Option<DateTime<Utc>>,
   }

Pas besoin de projet externe - fonctionnalité simple à implémenter.


7. IoT et Building Automation
------------------------------

ThingsBoard ⭐ RECOMMANDÉ
~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: ThingsBoard
:URL: https://github.com/thingsboard/thingsboard
:Licence: **Apache 2.0**
:Langage: Java
:Focus: Plateforme IoT complète

**Fonctionnalités:**

- ✅ Multi-protocole (MQTT, CoAP, HTTP, LwM2M, LoRaWAN, NB-IoT)
- ✅ Gestion d'assets et devices
- ✅ Dashboards temps réel
- ✅ Rules engine (alertes, actions)
- ✅ Intégration cloud (AWS, Azure, GCP)
- ✅ Data analytics
- ✅ OTA (Over-the-Air updates)

**Recommandation:**

✅ **Apache 2.0 = EXCELLENT pour containerisation**

**Use case KoproGo**:

- Monitoring ascenseurs (Issue #89 - Digital Maintenance Logbook)
- Capteurs énergétiques (consommation eau/électricité)
- Capteurs environnementaux (température, humidité, CO2)
- Alertes maintenance prédictive
- Conformité inspections techniques obligatoires

**Architecture microservice:**

.. code-block:: yaml

   # docker-compose.yml
   services:
     thingsboard:
       image: thingsboard/tb-postgres:latest
       ports:
         - "9090:9090"  # UI
         - "1883:1883"  # MQTT
         - "5683:5683/udp"  # CoAP
       environment:
         TB_QUEUE_TYPE: rabbitmq
       volumes:
         - tb-data:/data
       networks:
         - koprogo-iot-net

**Intégration Rust** (backend KoproGo):

.. code-block:: rust

   // backend/src/infrastructure/iot/thingsboard_client.rs
   use reqwest::Client;

   pub struct ThingsBoardClient {
       base_url: String,
       access_token: String,
   }

   impl ThingsBoardClient {
       pub async fn get_telemetry(
           &self,
           device_id: &str,
           keys: &[&str],
       ) -> Result<serde_json::Value, String> {
           // GET /api/plugins/telemetry/DEVICE/{deviceId}/values/timeseries
           // Crédit: ThingsBoard (Apache 2.0)
       }
   }

**Bibliothèques Rust pour capteurs IoT:**

.. code-block:: toml

   # Cargo.toml
   [dependencies]
   rumqttc = "0.24"        # MQTT client (Apache-2.0)
   coap = "0.16"           # CoAP client (MIT)
   serde_json = "1.0"      # JSON (MIT/Apache)

OpenRemote
~~~~~~~~~~

:Projet: OpenRemote
:URL: https://openremote.io
:Licence: **AGPL-3.0**
:Focus: IoT + Energy Management

**⚠️ AGPL-3.0** = contamination via réseau.

**Recommandation**: Préférer ThingsBoard (Apache 2.0).


EMQX (MQTT Broker)
~~~~~~~~~~~~~~~~~~

:Projet: EMQX
:URL: https://github.com/emqx/emqx
:Licence: **Apache 2.0**
:Langage: Erlang
:Focus: MQTT broker haute performance

**Recommandation:**

Si besoin d'un broker MQTT indépendant (ThingsBoard inclut déjà un broker).

.. code-block:: yaml

   services:
     emqx:
       image: emqx/emqx:latest
       ports:
         - "1883:1883"   # MQTT
         - "8883:8883"   # MQTT/SSL
         - "18083:18083" # Dashboard
       environment:
         EMQX_NAME: koprogo-mqtt
         EMQX_LOADED_PLUGINS: "emqx_dashboard,emqx_auth_username"


8. Blockchain et Smart Contracts
---------------------------------

Hyperledger Fabric ⭐ RECOMMANDÉ (avec réserves)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

:Projet: Hyperledger Fabric
:URL: https://github.com/hyperledger/fabric
:Licence: **Apache 2.0**
:Langage: Go (chaincode: Go, Java, Node.js, Rust)
:Focus: Blockchain privée pour entreprises

**Cas d'usage PropTech:**

- ✅ **Traçabilité des votes AG** (immuable, auditable)
- ✅ **État Daté** (Issue #80) - snapshot immuable pour ventes
- ✅ **Historique de propriété** (changements de copropriétaires)
- ✅ **Contrats intelligents** (automated workflows)

**⚠️ MAIS - Considérations critiques:**

1. **GDPR Article 17 (Droit à l'oubli):**

   - Blockchain = immuable
   - GDPR = droit d'effacement des données personnelles
   - **CONFLIT FONDAMENTAL**

2. **Solution: Blockchain privée + Hash only:**

   .. code-block:: rust

      // NE PAS stocker sur blockchain:
      struct Vote {
          owner_name: String,  // ❌ Donnée personnelle
          email: String,       // ❌ Donnée personnelle
      }

      // STOCKER sur blockchain:
      struct VoteProof {
          vote_hash: String,      // ✅ Hash SHA-256
          resolution_id: Uuid,    // ✅ ID technique
          timestamp: DateTime,    // ✅ Horodatage
          merkle_root: String,    // ✅ Preuve cryptographique
      }

      // Données personnelles en PostgreSQL (effaçable GDPR)
      // Blockchain = uniquement preuves cryptographiques

**Hyperledger Fabric Rust SDK:**

:Projet: fabric-contract-api-rust
:URL: https://github.com/hyperledgendary/fabric-contract-api-rust
:Statut: Technology Preview (WebAssembly chaincode)

.. code-block:: rust

   // Exemple chaincode Rust pour Hyperledger Fabric
   use fabric_contract::contract::*;

   #[derive(Serialize, Deserialize)]
   struct VoteProof {
       resolution_id: String,
       vote_hash: String,
       timestamp: i64,
   }

   #[contract_impl]
   impl VotingContract {
       pub fn record_vote_proof(&self, ctx: TransactionContext, proof: VoteProof) {
           // Enregistrement immuable du hash de vote
           // Crédit: Hyperledger Fabric Rust API (Apache 2.0)
       }
   }

**Recommandation FINALE blockchain:**

⚠️ **Phase 3+ uniquement** (K8s Production - Issue #94+)

**Raisons:**

1. Complexité infrastructure (cluster Fabric = min 4 peers + orderers)
2. Overhead performance (consensus distribué)
3. GDPR compliance complexe
4. Use case limité (votes AG = 1-2x/an)

**Alternative simple (Phases 1-2):**

.. code-block:: rust

   // backend/src/domain/services/vote_audit.rs
   // Audit trail simple sans blockchain

   pub struct VoteAuditLog {
       pub id: Uuid,
       pub vote_hash: String,        // SHA-256 du vote
       pub resolution_id: Uuid,
       pub timestamp: DateTime<Utc>,
       pub merkle_root: String,      // Merkle tree local
       pub signature: String,        // Ed25519 signature
   }

   // Stockage PostgreSQL avec trigger immutable
   // CREATE TABLE vote_audit_logs (...) WITH (security_invoker = true);
   // REVOKE DELETE ON vote_audit_logs FROM ALL;

**Ethereum / Web3 pour PropTech:**

❌ **NON RECOMMANDÉ** pour KoproGo:

- Blockchain publique = coûts gas
- GDPR incompatible (données publiques)
- Overhead inutile pour copropriété


Bibliothèques Rust pour Blockchain
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Si besoin futur:

.. code-block:: toml

   # Cargo.toml - Blockchain libs

   [dependencies]
   # Ethereum
   ethers = "2.0"              # MIT/Apache - Web3 + smart contracts

   # Substrate (Polkadot)
   substrate-api-client = "0.18"  # Apache-2.0

   # Cryptographie
   sha2 = "0.10"               # MIT/Apache - SHA-256
   ed25519-dalek = "2.1"       # BSD-3-Clause - Signatures
   merkle = "0.14"             # MIT - Merkle trees


Bibliothèques Rust Proptech-Ready
==================================

Stack complet Rust (MIT/Apache) pour KoproGo:

Backend Web
-----------

.. code-block:: toml

   [dependencies]
   # Framework (DÉJÀ UTILISÉ)
   actix-web = "4.9"           # Apache-2.0 / MIT
   actix-multipart = "0.7"     # Apache-2.0 / MIT (upload fichiers)
   actix-files = "0.6"         # Apache-2.0 / MIT (download fichiers)

   # Database (DÉJÀ UTILISÉ)
   sqlx = { version = "0.8", features = ["postgres", "uuid", "chrono"] }  # Apache-2.0 / MIT

   # Async runtime (DÉJÀ UTILISÉ)
   tokio = { version = "1.41", features = ["full"] }  # MIT

PDF Generation
--------------

.. code-block:: toml

   [dependencies]
   # Option 1: High-level (RECOMMANDÉ)
   genpdf = "0.2"              # Apache-2.0 / MIT

   # Option 2: Low-level (plus de contrôle)
   printpdf = "0.7"            # MIT
   lopdf = "0.34"              # MIT

Email
-----

.. code-block:: toml

   [dependencies]
   lettre = { version = "0.11", features = ["tokio1-native-tls", "smtp-transport"] }  # MIT

IoT / MQTT
----------

.. code-block:: toml

   [dependencies]
   rumqttc = "0.24"            # Apache-2.0 (MQTT client)
   coap = "0.16"               # MIT (CoAP pour capteurs)

Stockage S3
-----------

.. code-block:: toml

   [dependencies]
   # Option 1: AWS SDK
   aws-sdk-s3 = "1.0"          # Apache-2.0

   # Option 2: Rusoto (legacy)
   rusoto_s3 = "0.48"          # MIT

Cryptographie
-------------

.. code-block:: toml

   [dependencies]
   sha2 = "0.10"               # MIT/Apache (hash)
   ed25519-dalek = "2.1"       # BSD-3-Clause (signatures)
   argon2 = "0.5"              # MIT/Apache (password hashing)

Paiements (Stripe)
------------------

.. code-block:: toml

   [dependencies]
   async-stripe = { version = "0.39", features = ["runtime-tokio-hyper"] }  # MIT/Apache


Recommandations par Issue KoproGo
==================================

Phase 1 (VPS MVP)
-----------------

Issue #45 - File Upload UI
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Bibliothèques Rust:**

- ``actix-multipart`` (Apache/MIT) - déjà dans stack
- Pattern: Inspiration Condo (MIT) pour validation + preview

**Action:** Implémentation native Rust ✅

Issue #46 - Meeting Voting System
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Inspiration:**

- ElectionGuard (MIT) - concepts cryptographiques **uniquement**
- TimeOverflow (AGPL) - **réimplémentation** sans contamination

**Action:**

1. Phase 1: Vote simple PostgreSQL
2. Phase 2: Signature itsme® (Issue #48)
3. Phase 3: Optionnellement chiffrement homomorphe

Issue #47 - PDF Generation
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Bibliothèques Rust:**

- ``genpdf`` (Apache/MIT) pour templates
- ``printpdf`` (MIT) pour contrôle bas niveau

**Inspiration:** Diacamma Syndic (GPL) - **étudier structure PDF**, réimplémenter

**Action:** Implémentation native Rust ✅

Issue #73 - Invoice Workflow
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**✅ DÉJÀ IMPLÉMENTÉ**

**Inspiration supplémentaire:** Condo (MIT) - invoice line items pattern

Issue #76 - Document Upload/Download
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Bibliothèques Rust:**

- ``actix-files`` (Apache/MIT)
- ``aws-sdk-s3`` (Apache) ou MinIO

**Action:** Implémentation native Rust ✅ (pas de DMS externe nécessaire)

Issue #79 - Belgian PCMN
~~~~~~~~~~~~~~~~~~~~~~~~~

**✅ DÉJÀ IMPLÉMENTÉ**

**Inspiration:** Diacamma Syndic (GPL) - comptabilité française, adapter Belgique

Issue #80 - État Daté
~~~~~~~~~~~~~~~~~~~~~

**PDF + Blockchain (optionnel):**

- Phase 1: PDF avec ``genpdf``
- Phase 3: Hash immuable (blockchain ou audit log PostgreSQL)

**Action:** Implémentation native Rust ✅

Issue #83 - Payment Recovery
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**✅ DÉJÀ IMPLÉMENTÉ**

Issue #84 - Online Payment
~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Bibliothèques Rust:**

- ``async-stripe`` (MIT/Apache) pour Stripe
- Inspiration: Condo (MIT) - payment tracking patterns

**Action:** Implémentation native Rust ✅

Issue #85 - Maintenance Ticketing
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Inspiration:** Condo (MIT) - ticketing system

**Action:** Réimplémentation Rust inspirée de Condo ✅

Issue #86 - Multi-Channel Notifications
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**RECOMMANDATION FORTE:**

✅ **Containeriser Novu** (MIT) comme microservice

.. code-block:: yaml

   services:
     novu-api:
       image: ghcr.io/novuhq/novu/api:latest
     novu-worker:
       image: ghcr.io/novuhq/novu/worker:latest

**Alternative:** ``lettre`` (MIT) pour email uniquement

Issue #88 - AG Convocations
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Dépendances:**

- Issue #47 (PDF) + Issue #86 (Email)
- Inspiration: Diacamma Syndic (GPL) - délais légaux, ordre du jour

**Action:** Implémentation native Rust ✅

Issue #89 - Digital Maintenance Logbook
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**IoT Integration:**

✅ **Containeriser ThingsBoard** (Apache 2.0) pour capteurs/alertes

**Action:** Microservice ThingsBoard + API Rust ✅


Phase 2 (K3s Automation)
-------------------------

Issue #49 - Community Features (SEL)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Inspiration:** TimeOverflow (AGPL) - **réimplémentation obligatoire**

**Action:** Rust implementation avec citation TimeOverflow ✅

Issue #52 - Contractor Backoffice
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Inspiration:** Condo (MIT) - marketplace pattern

**Action:** Réimplémentation Rust inspirée de Condo ✅


Phase 3 (K8s Production)
-------------------------

Issue #94-99 - Advanced Features
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Blockchain (optionnel):**

- Hyperledger Fabric (Apache 2.0) pour audit immuable
- **Rust chaincode** via fabric-contract-api-rust
- ⚠️ GDPR: Hash uniquement, pas de données personnelles

**Action:** Évaluation Phase 3+ uniquement


Architecture Microservices Recommandée
=======================================

Stack KoproGo avec Intégrations Open Source
--------------------------------------------

.. code-block:: yaml

   # docker-compose.production.yml
   version: '3.8'

   services:
     # ============ KOPROGO CORE (Rust) ============
     koprogo-api:
       build: ./backend
       image: koprogo/api:latest
       ports:
         - "8080:8080"
       environment:
         DATABASE_URL: postgresql://...
         NOVU_API_URL: http://novu-api:3000
         THINGSBOARD_URL: http://thingsboard:9090
       networks:
         - koprogo-core
         - koprogo-integrations

     koprogo-frontend:
       build: ./frontend
       image: koprogo/frontend:latest
       ports:
         - "3000:3000"
       networks:
         - koprogo-core

     postgres:
       image: postgres:15-alpine
       environment:
         POSTGRES_DB: koprogo_db
       volumes:
         - postgres-data:/var/lib/postgresql/data
       networks:
         - koprogo-core

     # ============ NOTIFICATIONS (Novu - MIT) ============
     novu-api:
       image: ghcr.io/novuhq/novu/api:0.24
       environment:
         REDIS_URL: redis://redis:6379
         MONGO_URL: mongodb://mongo-novu:27017/novu
         JWT_SECRET: ${NOVU_JWT_SECRET}
       networks:
         - koprogo-integrations
         - novu-internal

     novu-worker:
       image: ghcr.io/novuhq/novu/worker:0.24
       environment:
         REDIS_URL: redis://redis:6379
         MONGO_URL: mongodb://mongo-novu:27017/novu
       networks:
         - novu-internal

     mongo-novu:
       image: mongo:6
       volumes:
         - mongo-novu-data:/data/db
       networks:
         - novu-internal

     # ============ IOT PLATFORM (ThingsBoard - Apache 2.0) ============
     thingsboard:
       image: thingsboard/tb-postgres:3.7
       ports:
         - "9090:9090"   # HTTP UI
         - "1883:1883"   # MQTT
         - "5683:5683/udp"  # CoAP
       environment:
         TB_QUEUE_TYPE: in-memory  # OU rabbitmq pour prod
       volumes:
         - tb-data:/data
         - tb-logs:/var/log/thingsboard
       networks:
         - koprogo-integrations
         - iot-internal

     # ============ SHARED SERVICES ============
     redis:
       image: redis:7-alpine
       networks:
         - koprogo-integrations
         - novu-internal

     # ============ MONITORING (Prometheus + Grafana) ============
     prometheus:
       image: prom/prometheus:latest
       volumes:
         - ./infrastructure/prometheus.yml:/etc/prometheus/prometheus.yml
         - prometheus-data:/prometheus
       networks:
         - koprogo-monitoring

     grafana:
       image: grafana/grafana:latest
       ports:
         - "3001:3000"
       environment:
         GF_SECURITY_ADMIN_PASSWORD: ${GRAFANA_PASSWORD}
       volumes:
         - grafana-data:/var/lib/grafana
       networks:
         - koprogo-monitoring

   networks:
     koprogo-core:
     koprogo-integrations:
     koprogo-monitoring:
     novu-internal:
     iot-internal:

   volumes:
     postgres-data:
     mongo-novu-data:
     tb-data:
     tb-logs:
     prometheus-data:
     grafana-data:

**Crédit Open Source:**

- Novu (MIT): https://github.com/novuhq/novu
- ThingsBoard (Apache 2.0): https://github.com/thingsboard/thingsboard
- Inspiration: Diacamma Syndic (GPL-3.0), Condo (MIT), TimeOverflow (AGPL-3.0)


Matrice de Décision: Containeriser vs Réimplémenter
====================================================

+---------------------------+-------------+------------------+---------------------+----------------------+
| Projet                    | Licence     | Containerisation | Réimplémentation    | Recommandation       |
+===========================+=============+==================+=====================+======================+
| **Diacamma Syndic**       | GPL-3.0     | ❌ Contamination | ✅ S'inspirer       | **Réimplémenter**    |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **Condo**                 | MIT         | ✅ Compatible    | ✅ Compatible       | **Les deux options** |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **Novu**                  | MIT         | ✅ **RECOMMANDÉ**| ⚠️ Complexe (Node) | **Containeriser**    |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **ThingsBoard**           | Apache 2.0  | ✅ **RECOMMANDÉ**| ⚠️ Complexe (Java) | **Containeriser**    |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **TimeOverflow**          | AGPL-3.0    | ❌ Contamination | ✅ Obligatoire      | **Réimplémenter**    |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **ElectionGuard**         | MIT         | ⚠️ Trop complexe | ✅ Concepts crypto  | **Inspiration**      |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **Helios Voting**         | Apache/GPL  | ❌ GPL frontend  | ✅ Concepts crypto  | **Inspiration**      |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **Paperless-ngx**         | GPL-3.0     | ❌ Contamination | ✅ Pattern DMS      | **Réimplémenter**    |
+---------------------------+-------------+------------------+---------------------+----------------------+
| **Hyperledger Fabric**    | Apache 2.0  | ⚠️ Phase 3+      | ✅ Rust chaincode   | **Phase 3 optionnel**|
+---------------------------+-------------+------------------+---------------------+----------------------+
| **OpenRemote**            | AGPL-3.0    | ❌ Contamination | ⚠️ Non prioritaire  | **Éviter**           |
+---------------------------+-------------+------------------+---------------------+----------------------+

**Légende:**

- ✅ Recommandé / Compatible
- ⚠️ Avec conditions / Complexe
- ❌ Non recommandé / Incompatible


Checklist Conformité Européenne
================================

Tous les projets recommandés doivent respecter:

GDPR (Règlement Général sur la Protection des Données)
-------------------------------------------------------

✅ **Projets compatibles:**

- Novu: Préférences utilisateur (opt-in/opt-out) ✅
- ThingsBoard: Data retention policies configurables ✅
- Condo: Open source, données contrôlées ✅

⚠️ **Projets à adapter:**

- Blockchain: **Hash uniquement**, pas de données personnelles
- TimeOverflow: Réimplémentation avec anonymisation

❌ **Projets incompatibles:**

- Blockchain publique (Ethereum, VotoSocial)

eIDAS (Identification Électronique et Services de Confiance)
-------------------------------------------------------------

🇧🇪 **Belgique - itsme®** (Issue #48):

- ElectionGuard: ❌ Non compatible
- Helios: ❌ Non compatible
- **Solution**: Intégration OpenID Connect (OIDC) custom avec itsme®

🇪🇺 **eID readers:**

- Support futur pour cartes d'identité électroniques européennes

Accessibilité (Directive EU 2016/2102)
---------------------------------------

- Frontend Astro + Svelte: Audit WCAG 2.1 AA requis
- ThingsBoard UI: Vérifier accessibilité dashboards

Efficacité Énergétique (EU Ecodesign)
--------------------------------------

- ThingsBoard: Monitoring consommation énergétique ✅
- Objectif KoproGo: < 0.5g CO2/request (CLAUDE.md)


Plan d'Implémentation Recommandé
=================================

Phase 1 (VPS MVP - Nov 2025 - Fév 2026)
----------------------------------------

**Implémentations natives Rust:**

1. ✅ **PDF Generation** (``genpdf`` MIT/Apache)

   - État Daté (Issue #80)
   - Relevés de charges (Issue #77)
   - Procès-verbaux AG (Issue #47)

2. ✅ **Document Management** (``actix-files`` + S3)

   - Upload/Download (Issue #76)
   - Stockage MinIO ou AWS S3
   - Pas de DMS externe (trop simple)

3. ✅ **Ticketing** (custom Rust)

   - Maintenance requests (Issue #85)
   - Inspiration: Condo (MIT)

4. ✅ **Email** (``lettre`` MIT)

   - Convocations AG (Issue #88)
   - Relances paiement (Issue #83 - déjà fait)

**Études préliminaires:**

- Analyser Diacamma Syndic (GPL) - comptabilité, AG
- Analyser Condo (MIT) - ticketing, marketplace
- Documenter différences France/Belgique (PCMN vs PCG)

Phase 2 (K3s Automation - Mar - Mai 2026)
------------------------------------------

**Containerisation microservices:**

1. ✅ **Novu** (MIT) - Notifications multi-canal

   .. code-block:: bash

      docker-compose up novu-api novu-worker

   - Issue #86: Email, SMS, Push, In-app
   - Intégration API Rust: ``reqwest`` client

2. ⚠️ **ThingsBoard** (Apache 2.0) - IoT (optionnel)

   .. code-block:: bash

      docker-compose up thingsboard

   - Issue #89: Digital Maintenance Logbook
   - Capteurs: Ascenseur, énergie, environnement
   - MQTT/CoAP avec ``rumqttc`` (Rust)

**Réimplémentations Rust:**

1. ✅ **SEL/Timebank** (inspiré TimeOverflow AGPL)

   - Issue #49: Community Features
   - Issue #99: SEL module
   - Crédits temps, échange services

2. ✅ **Contractor Backoffice** (inspiré Condo MIT)

   - Issue #52: Work reports, payment tracking
   - Photo upload, material tracking

Phase 3 (K8s Production - Jun - Août 2026)
-------------------------------------------

**Optionnel - Blockchain:**

1. ⚠️ **Hyperledger Fabric** (Apache 2.0)

   - Issue #94+: Blockchain audit trail
   - **Rust chaincode** (fabric-contract-api-rust)
   - **GDPR**: Hash uniquement, pas de PII
   - Use case: Votes AG immuables

2. **Alternative simple:**

   - Audit log PostgreSQL avec triggers immutables
   - Merkle tree local (``merkle`` crate MIT)
   - Signatures Ed25519 (``ed25519-dalek`` BSD)

**Advanced features:**

- Mobile app (React Native ou Flutter)
- Real-time notifications (WebSocket)
- Advanced analytics


Documentation et Attribution
=============================

Fichier CREDITS.md
------------------

Créer ``CREDITS.md`` à la racine:

.. code-block:: markdown

   # Open Source Credits

   KoproGo utilise et s'inspire des projets open source suivants:

   ## Containerised Services

   - **Novu** (MIT) - Multi-channel notifications
     - GitHub: https://github.com/novuhq/novu
     - Usage: Microservice pour email, SMS, push notifications

   - **ThingsBoard** (Apache 2.0) - IoT platform
     - GitHub: https://github.com/thingsboard/thingsboard
     - Usage: Building automation, sensor monitoring

   ## Inspirations (Reimplemented in Rust)

   - **Diacamma Syndic** (GPL-3.0) - Copropriété management
     - GitHub: https://github.com/Diacamma2/syndic
     - Inspiration: Accounting workflows, AG procedures
     - Authors: Diacamma2 team

   - **TimeOverflow** (AGPL-3.0) - Time banking
     - GitHub: https://github.com/coopdevs/timeoverflow
     - Inspiration: SEL/LETS credit system
     - Authors: Coopdevs cooperative

   - **Condo** (MIT) - Property management SaaS
     - GitHub: https://github.com/open-condo-software/condo
     - Inspiration: Ticketing system, marketplace patterns
     - Authors: Open Condo Software team

   ## Rust Libraries (Direct Usage)

   - actix-web (Apache-2.0 / MIT)
   - sqlx (Apache-2.0 / MIT)
   - genpdf (Apache-2.0 / MIT)
   - lettre (MIT)
   - rumqttc (Apache-2.0)

   Voir Cargo.toml pour la liste complète.

Commentaires dans le Code
--------------------------

.. code-block:: rust

   // backend/src/domain/entities/sel_transaction.rs

   //! SEL (Système d'Échange Local) - Time banking implementation
   //!
   //! Inspiré de TimeOverflow (AGPL-3.0) - https://github.com/coopdevs/timeoverflow
   //! Réimplémenté en Rust pour compatibilité licence et architecture hexagonale
   //!
   //! Auteurs originaux: Coopdevs cooperative
   //! KoproGo implementation: 2025

   use uuid::Uuid;
   use chrono::{DateTime, Utc};

   /// Transaction SEL: 1 heure de service = 1 crédit
   /// Pattern inspiré de TimeOverflow::Transaction model
   pub struct SelTransaction {
       pub id: Uuid,
       pub giver_id: Uuid,
       pub receiver_id: Uuid,
       // ...
   }


Références et Resources
========================

Documentation des Projets
--------------------------

**Gestion de Copropriété:**

- Diacamma Syndic: https://www.diacamma.org/
- Condo: https://github.com/open-condo-software/condo

**Notifications:**

- Novu: https://novu.co/
- Novu Docs: https://docs.novu.co/

**IoT:**

- ThingsBoard: https://thingsboard.io/
- ThingsBoard Docs: https://thingsboard.io/docs/

**Blockchain:**

- Hyperledger Fabric: https://www.hyperledger.org/projects/fabric
- Fabric Rust SDK: https://hyperledgendary.github.io/fabric-contract-api-rust/

**Timebank:**

- TimeOverflow: https://www.timeoverflow.org/

**Voting:**

- ElectionGuard: https://www.electionguard.vote/
- Helios Voting: https://documentation.heliosvoting.org/

Bibliothèques Rust
------------------

- Actix Web: https://actix.rs/
- rumqttc: https://github.com/bytebeamio/rumqtt
- genpdf: https://sr.ht/~ireas/genpdf-rs/
- lettre: https://lettre.rs/
- ethers-rs: https://github.com/gakonst/ethers-rs

GDPR et Conformité EU
---------------------

- GDPR Official Text: https://gdpr-info.eu/
- eIDAS Regulation: https://digital-strategy.ec.europa.eu/en/policies/eidas-regulation
- itsme® (Belgium): https://www.itsme.be/

PropTech Resources
------------------

- EU Smart Buildings: https://energy.ec.europa.eu/topics/energy-efficiency/energy-efficient-buildings/smart-buildings_en
- Belgian Copropriété Law: Code Civil Livre III, Titre VIII bis


Conclusion et Prochaines Étapes
================================

Résumé des Recommandations
---------------------------

✅ **À CONTAINERISER (Microservices):**

1. **Novu** (MIT) - Notifications → Issue #86
2. **ThingsBoard** (Apache 2.0) - IoT → Issue #89 (optionnel)

✅ **À RÉIMPLÉMENTER en Rust:**

1. **Gestion documentaire** - Inspiration Paperless-ngx (GPL) → Issue #76
2. **SEL/Timebank** - Inspiration TimeOverflow (AGPL) → Issue #49
3. **Ticketing** - Inspiration Condo (MIT) → Issue #85
4. **PDF Generation** - Bibliothèques Rust natives → Issues #47, #77, #80

⚠️ **ÉTUDIER sans utiliser directement:**

1. **Diacamma Syndic** (GPL) - Patterns comptabilité, AG
2. **ElectionGuard** (MIT) - Concepts cryptographiques vote
3. **Condo** (MIT) - Architecture marketplace

❌ **À ÉVITER:**

1. Blockchain publique (Ethereum, VotoSocial) - GDPR incompatible
2. Projets AGPL (contamination réseau)
3. Projets GPL en containerisation (contamination)

Actions Immédiates (Sprint actuel)
-----------------------------------

1. **Créer CREDITS.md** avec attributions open source
2. **Documenter** choix architecturaux (ce fichier RST)
3. **Tester** Novu en local (docker-compose)
4. **Analyser** Diacamma Syndic:

   .. code-block:: bash

      cd /tmp
      git clone https://github.com/Diacamma2/syndic.git
      # Étudier: diacamma/syndic/models.py (DB schema)
      #          diacamma/syndic/views.py (workflows AG)

5. **Planifier** réimplémentation SEL (TimeOverflow inspiration)

Actions Court Terme (Phase 1 - Q1 2026)
----------------------------------------

- [ ] Implémenter PDF generation (``genpdf``) - Issues #47, #77, #80
- [ ] Implémenter Document upload/download - Issue #76
- [ ] Implémenter Ticketing system - Issue #85
- [ ] Intégration email (``lettre``) - Issue #88
- [ ] Documentation différences PCMN (BE) vs PCG (FR)

Actions Moyen Terme (Phase 2 - Q2 2026)
----------------------------------------

- [ ] Déployer Novu (microservice) - Issue #86
- [ ] Implémenter SEL/Timebank - Issue #49
- [ ] Contractor backoffice - Issue #52
- [ ] (Optionnel) Déployer ThingsBoard - Issue #89

Actions Long Terme (Phase 3 - Q3 2026+)
----------------------------------------

- [ ] Évaluer besoin blockchain (Hyperledger Fabric)
- [ ] Advanced analytics
- [ ] Mobile app
- [ ] Real-time features

---

**Document maintenu par**: KoproGo Core Team
**Dernière mise à jour**: 2025-11-10
**Version**: 1.0
