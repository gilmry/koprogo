============================================================================================
Issue #235: R&D: Backoffice prestataires PWA - Magic link, ordres de service et compte-rendu
============================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/235>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Dans le cadre des contrats fournitures et services de l'ACP (révisés annuellement en AG ordinaire),
   les prestataires contractuels doivent pouvoir accéder a un backoffice simplifié dans la PWA pour
   rendre compte de leurs interventions. Ce backoffice est déclenché par des ordres de service issus
   de tickets approuvés.
   
   **Issues liées**: #85 (Tickets), #91 (Quotes), #95 (Marketplace), #228 (R&D Marketplace), #87 (PWA)
   
   ## Objectifs de la R&D
   
   ### 1. Magic Link Authentication pour prestataires
   
   - **Modèle d'authentification sans mot de passe** :
     - Token JWT signé avec expiration (24h-72h configurable)
     - Lien unique par ordre de service (1 lien = 1 intervention)
     - Révocation possible par le syndic
     - Rate limiting anti-abus (max 5 accès/h par lien)
   - **Sécurité** :
     - Token hashé en base (pas en clair)
     - Scope limité : accès uniquement au compte-rendu de l'intervention assignée
     - IP logging pour audit (GDPR Article 30)
     - Expiration automatique après soumission du compte-rendu
   - **UX prestataire** :
     - 0 inscription, 0 mot de passe
     - Le prestataire reçoit un email/SMS avec lien direct
     - Interface mobile-first (PWA installable)
   
   ### 2. Workflow Ticket -> Ordre de service -> Compte-rendu
   
   ```
   Ticket créé (Open)
       |
       v
   Ticket assigné au prestataire contractuel (Assigned)
       |
       v
   Ticket approuvé par syndic (Approved)
       |
       v
   Ordre de service généré automatiquement
       + Magic link envoyé au prestataire (email + SMS)
       + Notification au syndic
       |
       v
   Prestataire accède au backoffice simplifié via magic link
       |
       v
   Prestataire soumet compte-rendu :
       - Description de l'intervention
       - Photos (pièces changées, résultat final)
       - Durée de l'intervention
       - Matériaux utilisés (optionnel)
       - Signature électronique (optionnel)
       |
       v
   Syndic reçoit notification + valide le compte-rendu
       |
       v
   Quotation/évaluation de l'intervention par le syndic
       + Mise a jour du rating prestataire (#91 contractor_rating)
       |
       v
   Ticket clôturé (Closed) + historique dans carnet d'entretien (#134)
   ```
   
   ### 3. Entité ServiceOrder (nouvelle)
   
   ```rust
   struct ServiceOrder {
       id: Uuid,
       organization_id: Uuid,
       building_id: Uuid,
       ticket_id: Uuid,                    // Lien vers ticket approuvé
       contractor_id: Uuid,                // Prestataire contractuel
       contract_id: Option<Uuid>,          // Contrat de service annuel
       magic_link_token_hash: String,      // Token hashé (bcrypt/argon2)
       magic_link_expires_at: DateTime,    // Expiration du lien
       magic_link_used_at: Option<DateTime>,
       status: ServiceOrderStatus,         // Created, Sent, Accessed, ReportSubmitted, Validated, Rejected
       description: String,                // Instructions pour le prestataire
       report_description: Option<String>, // Compte-rendu texte
       report_photos: Vec<String>,         // Paths des photos uploadées
       report_duration_minutes: Option<i32>,
       report_materials: Option<String>,
       report_submitted_at: Option<DateTime>,
       rating: Option<i32>,                // 1-5 étoiles
       rating_comment: Option<String>,
       created_at: DateTime,
       updated_at: DateTime,
   }
   ```
   
   ### 4. Contrats de service annuels (ServiceContract)
   
   ```rust
   struct ServiceContract {
       id: Uuid,
       organization_id: Uuid,
       building_id: Uuid,
       contractor_id: Uuid,
       contractor_name: String,
       contract_type: ServiceContractType, // Maintenance, Cleaning, Security, Gardening, Elevator, etc.
       description: String,
       start_date: Date,
       end_date: Date,                     // Révisé annuellement en AG ordinaire
       annual_amount_cents: i64,
       payment_frequency: PaymentFrequency, // Monthly, Quarterly, Annually
       auto_renew: bool,
       ag_approval_meeting_id: Option<Uuid>, // AG qui a approuvé le contrat
       status: ContractStatus,             // Active, Expired, Terminated, PendingRenewal
       created_at: DateTime,
       updated_at: DateTime,
   }
   ```
   
   ### 5. Backoffice simplifié PWA (frontend)
   
   - **Vue prestataire** (accessible via magic link, sans navigation complète) :
     - Détails de l'ordre de service (description, bâtiment, urgence)
     - Formulaire de compte-rendu (texte + photos)
     - Upload photo avec compression client-side (max 5 photos, max 5MB chacune)
     - Bouton "Soumettre le compte-rendu"
     - Historique de ses interventions passées (optionnel)
   - **Offline-first** :
     - Formulaire fonctionne hors ligne (IndexedDB + Background Sync)
     - Photos stockées localement puis synchronisées
     - Confirmation visuelle de synchronisation
   
   ### 6. Intégration modules existants
   
   | Module | Intégration |
   |--------|-------------|
   | Tickets (#85) | Ticket approuvé -> ServiceOrder automatique |
   | Quotes (#91) | contractor_rating mis a jour après évaluation |
   | Documents (#storage) | Photos uploadées stockées via StorageProvider |
   | Work Reports (#134) | Compte-rendu intégré au carnet d'entretien numérique |
   | Notifications (#86) | Email/SMS magic link + notifications syndic |
   | PWA (#87) | Backoffice prestataire dans la PWA (offline-first) |
   
   ## Points de décision
   
   - [ ] Durée d'expiration du magic link (24h, 48h, 72h, configurable ?)
   - [ ] Canal d'envoi du magic link (email seul, email+SMS, choix syndic ?)
   - [ ] Signature électronique du compte-rendu (nécessaire juridiquement ?)
   - [ ] Accès historique interventions pour le prestataire (oui/non ?)
   - [ ] Photos : compression côté client ou côté serveur ?
   - [ ] Faut-il un entity ServiceContract séparé ou enrichir le modèle Quote ?
   - [ ] Multi-prestataires par ticket (ex: plombier + électricien) ?
   
   ## Conformité légale belge
   
   - **Loi copropriété** : Contrats de fournitures/services soumis au vote AG ordinaire (art. 577-8 CC)
   - **GDPR** : Données prestataire (nom, email, téléphone) sous consentement légitime (art. 6.1.b)
   - **Preuve des travaux** : Photos horodatées font foi en cas de litige (valeur probante numérique)
   
   ## Livrables attendus
   
   - [ ] Document d'architecture technique (RST)
   - [ ] Migration SQL (tables service_orders, service_contracts)
   - [ ] Domain entities + Use Cases (hexagonal)
   - [ ] API REST (8-12 endpoints)
   - [ ] Frontend PWA backoffice prestataire (Svelte)
   - [ ] Tests BDD (feature file Gherkin)
   
   ## Estimation
   
   - **Complexité** : Haute (3 nouvelles entités, magic link auth, PWA offline, photo upload)
   - **Jalon cible** : Jalon 3 (Features Différenciantes) ou Jalon 4 (Automation)
   - **Dépendances** : #85 (Tickets), #91 (Quotes), #87 (PWA), #134 (Work Reports)

.. raw:: html

   </div>

