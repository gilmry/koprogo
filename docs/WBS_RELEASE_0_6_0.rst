=========================================================
KoproGo - WBS Release 0.6.0
=========================================================

:Version: 1.0
:Date: 11 mars 2026
:Baseline: Release 0.5.0 (Mars 2026)
:Cible: Release 0.6.0
:Methode: DDD-BDD-TDD-SOLID (conforme WBS_PROJET_COMPLET.rst)
:Auteur: Claude Code
:Statut: Planification

.. contents:: Table des matieres
   :depth: 3
   :local:

=========================================================
1. CONTEXTE ET BASELINE
=========================================================

1.1 Etat post-release 0.5.0
------------------------------

La release 0.5.0 a livre :

- **BDD complet** : 48 feature files, 5 fichiers bdd_*.rs, ~280 scenarios actifs
- **Frontend UI** : toutes les pages/composants (issue #206 -- FERME)
- **Documentation** : Sphinx + 22 fichiers RST legaux indexes
- **MCP Jalon 6** : crate ``koprogo-mcp`` + ``koprogo-node`` (MVP, demo mode)
- **Infrastructure** : K3s issues #266-268 en cours

Score conformite legale belge : **65%** (audit_conformite.rst)
Lacunes CRITIQUES identifiees : **3** (quorum, 2e convocation, procurations max 3)

1.2 Objectifs release 0.6.0
-----------------------------

1. **Fermer les 3 lacunes legales CRITIQUES** (conformite Art. 3.87)
2. **4 nouveaux Bounded Contexts** (BC14-BC17) : marketplace, AG visio,
   backoffice prestataires, AGE agile
3. **Score conformite cible** : 80%+ (de 65% a 80%)
4. **Issues GitHub** : #271-#279 + MCP #252-265 + K3s #266-268

=========================================================
2. WORK PACKAGES
=========================================================

2.1 WP-LEGAL — Corrections legales critiques (PRIORITE P0 — BLOQUANT)
-----------------------------------------------------------------------

**Estimation** : ~6h | **Prerequis** : aucun | **Issues** : #271, #272, #273 | **Statut** : ✅ TERMINÉ (commit f642fbb)

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - L1-A1
     - Quorum 50%+ : ajouter quorum_validated + quorum_pct a Meeting entity
     - 1.5h
     - ``domain/entities/meeting.rs``

   * - L1-A1b
     - Use case validate_quorum() + bloquer votes si quorum KO
     - 1h
     - ``use_cases/resolution_use_cases.rs``

   * - L1-A1c
     - Migration add_quorum_to_meetings.sql + tests unitaires
     - 0.5h
     - ``migrations/``

   * - L1-A2
     - ConvocationType::SecondConvocation (delai 15j, pas de quorum)
     - 1h
     - ``domain/entities/convocation.rs``

   * - L1-A2b
     - Use case schedule_second_convocation() auto-trigger
     - 0.5h
     - ``use_cases/convocation_use_cases.rs``

   * - L1-A3
     - Validation procurations max 3 par mandataire (exception 10%)
     - 1h
     - ``domain/entities/vote.rs``, ``resolution_use_cases.rs``

   * - L1-DOC
     - Mettre a jour matrice_conformite.rst : quorum + procurations -> CONFORME
     - 0.5h
     - ``docs/legal/matrice_conformite.rst``

**Definition of Done** :

- ``cargo test --lib`` : vert sur meeting, convocation, vote
- Erreur metier explicite si quorum non atteint
- Matrice conformite mise a jour (25/37 -> 27/37 CONFORME)

2.2 WP-BC15 — AG Visioconference (PRIORITE P0)
-----------------------------------------------

**Estimation** : ~12h | **Prerequis** : WP-LEGAL | **Issue** : #274 (cloture #237) | **Statut** : 🔄 EN COURS

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - B15-1
     - Entite AgSession (platform enum, video_url, status, quorum_remote_pct)
     - 2h
     - ``domain/entities/ag_session.rs``

   * - B15-2
     - Port AgSessionRepository (trait)
     - 0.5h
     - ``application/ports/ag_session_repository.rs``

   * - B15-3
     - Use cases (8 methodes : create, start, end, join, quorum combine)
     - 2h
     - ``application/use_cases/ag_session_use_cases.rs``

   * - B15-4
     - Implementation PostgreSQL
     - 1.5h
     - ``infrastructure/database/repositories/ag_session_repository_impl.rs``

   * - B15-5
     - Handlers REST (7 endpoints)
     - 1.5h
     - ``infrastructure/web/handlers/ag_session_handlers.rs``

   * - B15-6
     - Migration create_ag_sessions.sql
     - 0.5h
     - ``migrations/``

   * - B15-7
     - Enrichissement Convocation : has_video_option, video_platform, video_url
     - 1h
     - ``convocation.rs`` + migration

   * - B15-8
     - Frontend : indicateur visio dans la page AG
     - 1.5h
     - ``frontend/src/pages/meetings/``

   * - B15-9
     - Tests unitaires AgSession (entity + use cases)
     - 1h
     - ``#[cfg(test)]``

**Definition of Done** :

- calculate_combined_quorum() = presentiel + distanciel
- 7 endpoints operationnels
- Convocation affiche lien video si has_video_option = true

2.3 WP-BC17 — AGE Agile & Concertation (PRIORITE P0)
-----------------------------------------------------

**Estimation** : ~14h | **Prerequis** : WP-BC15 | **Issue** : #279

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - B17-1
     - Entite AgeRequest (co_signatories, total_shares_pct, status machine)
     - 2h
     - ``domain/entities/age_request.rs``

   * - B17-2
     - Use case add_cosignatory() avec calcul seuil 1/5 temps reel
     - 1.5h
     - ``application/use_cases/age_request_use_cases.rs``

   * - B17-3
     - Use case notify_syndic() + deadline 15j + auto-convocation si inactif
     - 2h
     - ``use_cases/age_request_use_cases.rs``

   * - B17-4
     - Implementation PostgreSQL + migration
     - 1.5h
     - ``infrastructure/database/repositories/``

   * - B17-5
     - Handlers REST (8 endpoints)
     - 1.5h
     - ``infrastructure/web/handlers/age_request_handlers.rs``

   * - B17-6
     - Convocation auto-generee par copros (lien BC15 + BC3)
     - 2h
     - ``use_cases/convocation_use_cases.rs``

   * - B17-7
     - Espace concertation pre-AGE (reutilise Poll module)
     - 1.5h
     - ``use_cases/poll_use_cases.rs`` (extension)

   * - B17-8
     - Tests unitaires AgeRequest (seuil 1/5, delai 15j, auto-convocation)
     - 2h
     - ``#[cfg(test)]``

**Definition of Done** :

- Seuil 1/5 calcule en temps reel depuis les quotes-parts enregistrees
- Auto-convocation generee si syndic inactif > 15j
- Option visioconference incluse par defaut dans convocation AGE

2.4 WP-BC16 — Backoffice Prestataires PWA (PRIORITE P1)
-------------------------------------------------------

**Estimation** : ~16h | **Prerequis** : WP-LEGAL | **Issue** : #275 (cloture #235)

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - B16-1
     - Entite ContractorReport (state machine : Draft->Validated)
     - 2h
     - ``domain/entities/contractor_report.rs``

   * - B16-2
     - Magic link JWT 72h : generation + validation endpoint
     - 1.5h
     - ``infrastructure/web/handlers/contractor_report_handlers.rs``

   * - B16-3
     - Use cases (10 methodes : submit, validate, request_corrections)
     - 2.5h
     - ``application/use_cases/contractor_report_use_cases.rs``

   * - B16-4
     - Implementation PostgreSQL + migration
     - 1.5h
     - ``infrastructure/database/repositories/``

   * - B16-5
     - Handlers REST (12 endpoints)
     - 2h
     - ``infrastructure/web/handlers/contractor_report_handlers.rs``

   * - B16-6
     - Trigger paiement automatique sur validation CdC
     - 1h
     - ``use_cases/contractor_report_use_cases.rs`` -> payment use case

   * - B16-7
     - Frontend PWA mobile-first (Camera API, Voice-to-text, offline IndexedDB)
     - 4h
     - ``frontend/src/pages/contractor/[token].astro``

   * - B16-8
     - Tests unitaires ContractorReport (transitions, trigger paiement)
     - 1.5h
     - ``#[cfg(test)]``

**Definition of Done** :

- Magic link JWT 72h genere et valide sans auth classique
- Workflow Draft -> Validated -> Payment declenche automatiquement
- PWA testee sur mobile (Chrome DevTools Device Mode)

2.5 WP-BC14 — Marketplace Corps de Metier + Evaluations L13 (PRIORITE P1)
-------------------------------------------------------------------------

**Estimation** : ~20h | **Prerequis** : WP-BC16 | **Issue** : #276

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - B14-1
     - Entite ServiceProvider (20 TradeCategory, certifications, rating_avg, slug)
     - 2.5h
     - ``domain/entities/service_provider.rs``

   * - B14-2
     - Entite ContractEvaluation (6 criteres JSON, global_score, is_legal_evaluation)
     - 2h
     - ``domain/entities/contract_evaluation.rs``

   * - B14-3
     - Use cases ServiceProvider (CRUD + verify + marketplace search)
     - 2h
     - ``application/use_cases/service_provider_use_cases.rs``

   * - B14-4
     - Use cases ContractEvaluation (CRUD + rapport L13 annuel)
     - 2.5h
     - ``application/use_cases/contract_evaluation_use_cases.rs``

   * - B14-5
     - Implementations PostgreSQL (2 repositories)
     - 2h
     - ``infrastructure/database/repositories/``

   * - B14-6
     - Handlers Marketplace public (sans auth) + evaluations
     - 2.5h
     - ``infrastructure/web/handlers/marketplace_handlers.rs``

   * - B14-7
     - Migrations (service_providers.sql + contract_evaluations.sql)
     - 1h
     - ``migrations/``

   * - B14-8
     - Auto-trigger evaluation sur Ticket::Closed (notification syndic)
     - 1h
     - ``use_cases/ticket_use_cases.rs`` (extension)

   * - B14-9
     - Frontend marketplace : liste prestataires + fiche publique
     - 3h
     - ``frontend/src/pages/marketplace/``

   * - B14-10
     - Tests unitaires (ServiceProvider + ContractEvaluation)
     - 1.5h
     - ``#[cfg(test)]``

**Definition of Done** :

- GET /marketplace/providers?trade=Plombier&postal_code=1000 fonctionne sans auth
- GET /buildings/:id/reports/contract-evaluations/annual retourne rapport L13
- Auto-trigger evaluation sur Ticket::Closed

2.6 WP-GUIDE — Guide Legal Contextuel UI (PRIORITE P1)
------------------------------------------------------

**Estimation** : ~10h | **Prerequis** : WP-BC15 | **Issue** : #277

.. list-table::
   :header-rows: 1
   :widths: 10 50 15 25

   * - ID
     - Tache
     - Effort
     - Fichiers

   * - G1
     - Generer legal_index.json depuis docs/legal/ (tous les codes L*, T*, G*, CP*, AG*)
     - 1.5h
     - ``infrastructure/legal_index.json`` (script de generation)

   * - G2
     - legal_handlers.rs : GET /legal/rules + /legal/ag-sequence + /legal/majority-for/:type
     - 2h
     - ``infrastructure/web/handlers/legal_handlers.rs``

   * - G3
     - LegalHelper.svelte : panneau contextuel flottant par page + role
     - 2.5h
     - ``frontend/src/components/LegalHelper.svelte``

   * - G4
     - AG Wizard 5 etapes (Astro + Svelte, backbone sequence_odj.rst)
     - 3h
     - ``frontend/src/pages/ag/wizard.astro``

   * - G5
     - Lien MCP tool legal_search (#254) -> legal_index.json
     - 1h
     - ``backend/koprogo-mcp/``

**Definition of Done** :

- LegalHelper affiche contenu contextuel sur page AG et page Resolution
- AG Wizard completement fonctionnel en 5 etapes
- GET /legal/ag-sequence retourne les 12 points de l'OdJ avec majorites

2.7 WP-BLOG — Articles de Blog Batiment (PRIORITE P2)
------------------------------------------------------

**Estimation** : ~22h | **Prerequis** : aucun | **Issue** : #278

18 articles RST dans ``docs/blog/batiment/`` (5 series thematiques).
Voir issue #278 pour la liste complete.

Effort par serie :

- Serie 0 (documents fondamentaux, 4 articles) : 4h
- Serie 1 (acteurs MOA, 3 articles) : 3h
- Serie 2 (corps de metier privatif/communs, 5 articles) : 6h
- Serie 3 (travaux d'ampleur, 3 articles) : 3.5h
- Serie 4 (best practices, 2 articles) : 2.5h
- Serie 5 (reglementations obligatoires EU/BE, 5 articles) : 6h (recherche web incluse)

**Definition of Done** :

- 18 fichiers RST + index.rst
- Chaque article : base legale verifiee + source externe citee
- RST valide (``make docs-serve`` sans erreur)

2.8 WP-MCP — MCP Tools AI Syndic (PRIORITE P1)
-----------------------------------------------

**Estimation** : ~30h | **Issues** : #252-265 (milestone Jalon 3 assigne)

Issues MCP en cours (Jalon 6 MVP depose) :
- #252-265 : 14 MCP tools pour l'assistant AI syndic
- Notamment : #258 travaux_qualifier, #261 documents_list, #254 legal_search (lie a WP-GUIDE)
- koprogo-mcp crate + koprogo-node (edge Raspberry Pi) deja implementes en demo mode

2.9 WP-K3S — Infrastructure K3s (PRIORITE P2)
----------------------------------------------

**Estimation** : ~20h | **Issues** : #266, #267, #268

Migration Docker Compose -> K3s (Kubernetes lightweight) :
- #266 : Helm charts KoproGo
- #267 : GitOps avec ArgoCD
- #268 : Observabilite K3s (Prometheus + Grafana)

=========================================================
3. CALENDRIER ET PRIORITES
=========================================================

.. list-table::
   :header-rows: 1
   :widths: 15 35 15 15 20

   * - Work Package
     - Description
     - Effort
     - Priorite
     - Prerequis

   * - WP-LEGAL
     - Corrections legales critiques A1-A3
     - ~6h
     - **P0 BLOQUANT**
     - aucun

   * - WP-BC15
     - AG Visioconference (AgSession + quorum combine)
     - ~12h
     - **P0**
     - WP-LEGAL

   * - WP-BC17
     - AGE Agile + concertation 1/5 quotites
     - ~14h
     - **P0**
     - WP-BC15

   * - WP-BC16
     - Backoffice prestataires PWA + magic link
     - ~16h
     - **P1**
     - WP-LEGAL

   * - WP-BC14
     - Marketplace corps de metier + evaluations L13
     - ~20h
     - **P1**
     - WP-BC16

   * - WP-GUIDE
     - Guide legal contextuel UI + AG Wizard
     - ~10h
     - **P1**
     - WP-BC15

   * - WP-MCP
     - MCP tools AI Syndic (#252-265)
     - ~30h
     - **P1**
     - independant

   * - WP-BLOG
     - 18 articles de blog batiment
     - ~22h
     - **P2**
     - independant

   * - WP-K3S
     - Infrastructure K3s (#266-268)
     - ~20h
     - **P2**
     - independant

**Total estime : ~150h**

Ordre d'execution recommande ::

   WP-LEGAL (P0) -> WP-BC15 (P0) -> WP-BC17 (P0)
                                           |
                                    WP-BC16 (P1) -> WP-BC14 (P1)
                                           |
                                    WP-GUIDE (P1)

   WP-MCP, WP-BLOG, WP-K3S : paralleles (independants)

=========================================================
4. NOUVEAUX BOUNDED CONTEXTS (resume)
=========================================================

.. list-table::
   :header-rows: 1
   :widths: 10 20 40 30

   * - BC
     - Nom
     - Entites principales
     - Ancre legale

   * - BC14
     - Marketplace & Evaluations
     - ServiceProvider, ContractEvaluation
     - Art. 3.89 §5 12° CC (L13 -- rapport evaluation contrats obligatoire AG)

   * - BC15
     - AG Visioconference
     - AgSession, Convocation (enrichie)
     - Art. 3.87 §1 CC (AG "physiquement ou a distance")

   * - BC16
     - Contractor Backoffice PWA
     - ContractorReport, magic link JWT
     - Best practice confiance + Art. 3.89 §5 4° (actes conservatoires)

   * - BC17
     - AGE Agile & Concertation
     - AgeRequest, concertation pre-AG
     - Art. 3.87 §2 al.2 CC (1/5 quotites peut convoquer AGE)

=========================================================
5. DEFINITION OF DONE GLOBALE RELEASE 0.6.0
=========================================================

- [ ] **Conformite legale** : score 65% -> 80%+ (matrice_conformite.rst)
- [ ] ``cargo test --lib`` : vert pour toutes les nouvelles entites
- [ ] ``cargo clippy -- -D warnings`` : zero warning
- [ ] **Routes actix-web** : routes specifiques enregistrees AVANT parametrees (critere MEMORY.md)
- [ ] **Magic link JWT** : ContractorReport accessible sans auth classique
- [ ] **Seuil 1/5** : AgeRequest.total_shares_pct calcule depuis unit_owners
- [ ] **Rapport L13** : GET /buildings/:id/reports/contract-evaluations/annual fonctionnel
- [ ] **AG Wizard** : 5 etapes completement guidees
- [ ] **18 articles RST** : valides Sphinx, sources citees
- [ ] **MCP legal_search** : utilise legal_index.json comme source de verite
- [ ] **K3s** : au moins un service deploye en staging

---

**Auteurs** : KoproGo Team + Claude Code
**Date** : 11 mars 2026
**Version** : 1.0
**Methode** : DDD-BDD-TDD (conforme CLAUDE.md)
