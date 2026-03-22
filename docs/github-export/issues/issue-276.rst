=========================================================================================================
Issue #276: feat: Marketplace corps de métier + enquêtes satisfaction (ancre L13 Art. 3.89 §5 12° CC)
=========================================================================================================

:State: **OPEN**
:Milestone: Jalon 3: Features Différenciantes 🎯
:Labels: enhancement,track:software finance,legal-compliance release:0.1.0
:Assignees: Unassigned
:Created: 2026-03-11
:Updated: 2026-03-14
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/276>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   **Art. 3.89 §5 12° CC** (L13) impose au syndic de soumettre à l'AG annuelle un rapport d'évaluation des contrats de fournitures régulières. Cette obligation légale crée une **participation contrainte** aux évaluations → données fiables → marketplace de confiance.
   
   L'ancrage légal transforme un simple annuaire en outil de conformité obligatoire.
   
   ## Entités
   
   ### ServiceProvider
   `company_name`, `trade_category` (enum 20 corps de métier), `specializations[]`, `service_zone_postal_codes[]`, `certifications[]` (VCA, Saber, BOSEC…), `ipi_registration`, `bce_number`, `rating_avg`, `reviews_count`, `is_verified`, `public_profile_slug`
   
   **20 TradeCategory** : Syndic, BureauEtude, Architecte, AssistantMaitreOeuvre, IngenieurStabilite, Plombier, Electricien, Chauffagiste, Menuisier, Peintre, Maconnerie, Etancheite, Ascensoriste, Jardinier, Nettoyage, Securite, Deboucheur, Couvreur, Carreleur, TechniquesSpeciales
   
   ### ContractEvaluation
   `service_provider_id`, `quote_id` (nullable), `ticket_id` (nullable), `evaluator_id`, `building_id`, `criteria` (JSON : qualite/delai/prix/communication/proprete/conformite_devis 0-5), `global_score`, `would_recommend`, `is_legal_evaluation` (true = rapport L13), `is_anonymous`
   
   **Déclenchement automatique** :
   - Ticket → status `Closed` → notification syndic (obligation légale Art. 3.89 §5 12°)
   
   ## Rapport L13 automatique
   `GET /buildings/:id/reports/contract-evaluations/annual` → PDF du rapport obligatoire pour l'AG
   
   ## Marketplace public (sans auth)
   - `GET /marketplace/providers?trade=Plombier&postal_code=1000&min_rating=4`
   - `GET /marketplace/providers/:slug` — profil public
   
   ## REST API (~16 endpoints)
   CRUD ServiceProvider + ContractEvaluation + rapport L13 + marketplace search
   
   ## Lien
   - Prérequis code : #275 (ContractorReport validé déclenche ContractEvaluation)
   - Ancre légale : L13 dans `docs/legal/syndic/missions_legales.rst`
   
   ## Definition of Done
   - [ ] ServiceProvider entity + marketplace public endpoints
   - [ ] ContractEvaluation entity avec 6 critères
   - [ ] Rapport L13 annuel généré via GET endpoint
   - [ ] Auto-trigger évaluation sur Ticket::Closed
   - [ ] 2 migrations créées

.. raw:: html

   </div>

