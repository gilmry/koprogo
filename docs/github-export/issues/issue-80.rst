===============================================================
Issue #80: feat: État Daté Generation for Property Transfers
===============================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical legal-compliance,pdf
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/80>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #017 - État Daté Génération
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 6-8 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Génération d'états datés pour mutations immobilières (Article 577-2 Code Civil belge). Un état daté est un document légal obligatoire pour toute vente de lot en copropriété.
   
   **Impact**: BLOQUE TOUTES LES VENTES DE LOTS sans ce document
   
   ## 🎯 Objectifs
   
   - [ ] Entity EtatDate (building_id, unit_id, reference_date, data JSONB, status)
   - [ ] Génération PDF conforme (16 sections légales requises)
   - [ ] Workflow: demande → génération (max 15 jours) → délivrance
   - [ ] Endpoints: POST /units/:id/etat-date, GET /etat-dates/:id/pdf
   - [ ] Historique complet: appels de fonds, paiements, travaux votés, litiges
   
   ## 📐 Contenu État Daté (16 sections légales)
   
   1. Identification immeuble et lot
   2. Quote-part charges ordinaires/extraordinaires
   3. Situation financière du propriétaire
   4. Montant provisions pour charges
   5. Solde créditeur/débiteur
   6. Travaux votés non encore payés
   7. Litiges en cours
   8. Assurance immeuble
   9. Règlement de copropriété
   10. PV dernières AG
   11. Budget prévisionnel
   12. Fonds de réserve
   13. Dettes/créances copropriété
   14. État d'avancement travaux
   15. Garanties et hypothèques
   16. Observations diverses
   
   ## 🔗 Dépendances
   
   **Dépend de**: #016 (Plan Comptable pour section financière)
   
   ## ✅ Critères d'Acceptation
   
   - PDF conforme législation belge
   - Délai génération max 15 jours (rappels si > 10j)
   - Tests E2E génération + validation contenu
   - Documentation procédure notaires
   - Multi-langue (FR/NL/DE)
   
   ---
   
   **Voir**: \`issues/critical/017-etat-date-generation.md\`

.. raw:: html

   </div>

