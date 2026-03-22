===================================================================================================================================
Issue #279: feat: AGE agile — demande 1/5 quotités + concertation officielle + convocation auto distance (Art. 3.87 §2 al.2 CC)
===================================================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software legal-compliance,governance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-14
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/279>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte légal
   **Art. 3.87 §2 al.2 CC** (CP05 dans `docs/legal/coproprietaire/droits_obligations.rst`) :
   > Un ou plusieurs copropriétaires possédant au moins **1/5 des quotes-parts** peuvent demander la convocation d'une AG extraordinaire. **Si le syndic refuse ou est inactif, ils peuvent la convoquer eux-mêmes.**
   
   Combiné avec **Art. 3.87 §1 CC** : l'AG peut se tenir *«à distance si la convocation le prévoit»*.
   
   Ce combo crée un outil de **résilience démocratique** : aucune inertie du syndic, du CdC, ou d'une urgence ne peut bloquer la copropriété.
   
   ## Fonctionnalités demandées
   
   ### 1. Pétition AGE avec seuil 1/5 automatique
   - Formulaire de demande d'AGE : copropriétaire saisit le motif urgent + son identité
   - KoproGo **calcule en temps réel** les quotes-parts co-signataires et affiche la progression vers le seuil 1/5
   - Notification aux autres copropriétaires : "Demande d'AGE en cours — X% des quotités atteints, il manque Y%"
   - Quand le seuil 1/5 est atteint : alerte automatique au syndic + délai de réponse 15 jours
   
   ### 2. Concertation officielle pré-AGE (gravée dans le marbre)
   - Espace de concertation structuré : contributions écrites des copropriétaires
   - Archivage GDPR-compliant (Art. 30 — trace des débats pré-AG)
   - Vote consultatif en amont pour tester la majorité (réutilise le module `Poll`)
   - Résultat de la concertation : automatiquement annexé à la convocation officielle
   
   ### 3. Convocation AGE auto-générée si syndic inactif
   - Si syndic ne répond pas dans les 15 jours : KoproGo propose la convocation **par les copropriétaires eux-mêmes** (conforme à l'Art. 3.87 §2 al.2)
   - Délai légal : **15 jours avant** l'AGE (identique à l'AGO, voir `audit_conformite.rst`)
   - Option **distance + présentiel** (Art. 3.87 §1 — visioconférence si convocation le prévoit)
   - Génération PDF convocation officielle avec mentions légales obligatoires
   
   ### 4. Urgences et cas particuliers
   - **Actes conservatoires** (T01 — Art. 3.89 §5 4°) : syndic peut agir seul MAIS CdC peut déclencher une AGE urgente en cas de défaillance
   - **Blocage CdC** : même mécanisme 1/5 permet de court-circuiter le CdC défaillant
   - Délai accéléré d'urgence (à valider avec juriste — potentiellement 8 jours en cas de péril imminent)
   
   ## Entités / Modifications
   
   ### Nouvelle entité : `AgeRequest` (demande d'AGE)
   `requester_id`, `building_id`, `reason` (texte), `co_signatories[]` (owner_ids), `total_shares_pct`, `threshold_reached` (bool), `syndic_notified_at`, `syndic_response_deadline`, `self_convocation_triggered` (bool), `status` (Collecting/ThresholdReached/SyndicNotified/ConvocationGenerated/Rejected)
   
   ### Modifications `Convocation`
   - Ajouter `requested_by_owners: bool` (true si convoquée par les copros eux-mêmes)
   - Ajouter `age_request_id` (nullable — lien avec la demande)
   - Ajouter `has_video_option`, `video_url` (déjà prévu en issue #274)
   
   ### Module Poll (réutilisation)
   - Créer un `Poll` de type `YesNo/MultipleChoice` lié à l'AGE pour la concertation préalable
   
   ## REST API (~8 nouveaux endpoints)
   - `POST /buildings/:id/age-requests` — initier une demande d'AGE
   - `POST /age-requests/:id/cosign` — co-signer (ajouter ses quotes-parts)
   - `GET /age-requests/:id` — état de la demande + progression seuil
   - `GET /buildings/:id/age-requests` — historique des demandes
   - `PUT /age-requests/:id/notify-syndic` — notification formelle au syndic
   - `POST /age-requests/:id/self-convoke` — déclencher la convocation par les copros
   - `GET /age-requests/:id/concertation` — espace de concertation (lié à Poll)
   - `PUT /age-requests/:id/reject` — syndic ou CdC répond et convoque en bonne et due forme
   
   ## Lien avec autres features
   - #274 AG Visioconférence (distance = par défaut pour les AGE agiles)
   - #272 Workflow 2e convocation (quorum non atteint → AGE possible)
   - #277 Guide légal contextuel (CP05 affiché en contexte sur la page demande AGE)
   
   ## Definition of Done
   - [ ] AgeRequest entity avec state machine
   - [ ] Calcul seuil 1/5 temps réel
   - [ ] Notification syndic auto à J+15 si inactif
   - [ ] Convocation auto-générée par copros si syndic défaillant (PDF légal)
   - [ ] Concertation pré-AGE avec archive GDPR
   - [ ] Option visioconférence incluse dans convocation AGE

.. raw:: html

   </div>

