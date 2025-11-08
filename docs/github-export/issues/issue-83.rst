===========================================================================
Issue #83: feat: Payment Recovery Workflow (Workflow Recouvrement Impayés)
===========================================================================

:State: **CLOSED**
:Milestone: Phase 1: VPS MVP + Legal Compliance
:Labels: enhancement,phase:vps track:software,priority:critical finance,automation
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-08
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/83>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #023 - Workflow Recouvrement Impayés
   
   **Priorité**: 🔴 CRITIQUE  
   **Estimation**: 6-8 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Workflow automatisé de relances impayés avec 3 niveaux (J+15 aimable, J+30 ferme, J+60 mise en demeure légale).
   
   **Impact business**: Réduction impayés 30-50% via automatisation
   
   ## 🎯 Objectifs
   
   - [ ] Entity PaymentReminder (expense_id, owner_id, level, sent_date, status)
   - [ ] 3 niveaux: FirstReminder, SecondReminder, FormalNotice
   - [ ] Génération PDF lettres (templates par niveau + langue)
   - [ ] Cron job quotidien: détection impayés + envoi automatique
   - [ ] Calcul pénalités retard (taux légal belge 8% annuel)
   - [ ] Workflow: email → PDF lettre recommandée → procédure huissier
   - [ ] Dashboard syndic: vue impayés + historique relances
   
   ## 📐 Workflow Relances
   
   1. **J+15: First Reminder** (Aimable)
      - Email automatique
      - Ton courtois
      - Rappel montant + échéance
   
   2. **J+30: Second Reminder** (Ferme)
      - Email + PDF lettre
      - Ton plus ferme
      - Mention pénalités
   
   3. **J+60: Formal Notice** (Mise en demeure)
      - Lettre recommandée
      - Ton juridique
      - Pénalités calculées
      - Prépare procédure huissier
   
   ## 📐 Calcul Pénalités
   
   Taux légal belge: 8% annuel
   ```rust
   penalite = montant_impaye * 0.08 * (jours_retard / 365)
   ```
   
   ## ✅ Critères d'Acceptation
   
   - 3 templates PDF lettres (FR/NL/DE/EN)
   - Cron job relances automatique
   - Calcul pénalités conforme législation
   - Tests E2E workflow complet
   - Dashboard suivi impayés
   
   ---
   
   **Voir**: \`issues/critical/023-workflow-recouvrement.md\`

.. raw:: html

   </div>

