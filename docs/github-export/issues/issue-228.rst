=============================================================================
Issue #228: R&D: Marketplace prestataires - Modèle business et vérification
=============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:low R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/228>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'issue #95 prévoit un marketplace de prestataires. Cette R&D couvre le modèle
   business et les mécanismes de confiance.
   
   **Issue liée**: #95
   
   ## Objectifs de la R&D
   
   1. **Modèle business** :
      - Commission structure (% par transaction, abonnement, freemium)
      - Escrow (séquestre) pour travaux importants
      - Assurance responsabilité civile (vérification)
      - Facturation automatisée (TVA belge)
   
   2. **Onboarding prestataires** :
      - Vérification BCE/KBO (numéro d'entreprise)
      - Vérification assurance RC
      - Portfolio de réalisations
      - Certification par catégorie (plomberie, électricité, etc.)
      - GDPR : consentement explicite pour profil public
   
   3. **Intégration modules existants** :
      - Tickets → demande de devis automatique
      - Quotes → comparaison multi-prestataires (déjà implémenté)
      - Factures → paiement via Stripe (déjà implémenté)
      - Ratings → réputation (intégré dans quotes)
   
   4. **Taxonomie services** :
      - Catégorisation des corps de métier
      - Zones géographiques de couverture
      - Disponibilité (urgences 24/7 vs. horaires normaux)
      - Certifications (agréé gaz, RGIE, etc.)
   
   ## Points de décision
   
   - [ ] Modèle de revenus (commission vs. abonnement)
   - [ ] Vérification manuelle vs. automatisée (BCE API)
   - [ ] Responsabilité juridique (intermédiaire vs. marketplace)
   - [ ] Géolocalisation des prestataires
   
   ## Estimation
   
   10-15h

.. raw:: html

   </div>

