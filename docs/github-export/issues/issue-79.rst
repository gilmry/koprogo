============================================================================
Issue #79: feat: Belgian Accounting Chart (Plan Comptable Normalisé Belge)
============================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical finance,legal-compliance
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-13
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/79>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #016 - Plan Comptable Normalisé Belge
   
   **Priorité**: 🔴 CRITIQUE - PRIORITÉ #1  
   **Estimation**: 8-10 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   Implémenter un plan comptable normalisé conforme aux exigences belges (arrêté royal 12/07/2012). Le système actuel utilise des catégories basiques non conformes.
   
   **Gap identifié**: 0% implémenté - Obligation légale belge
   
   ## 🎯 Objectifs
   
   - [ ] Créer enum AccountCode avec classes 4, 5, 6, 7
   - [ ] Migration SQL pour account_code dans expenses table
   - [ ] Use cases génération bilan comptable
   - [ ] Use cases compte de résultat
   - [ ] Endpoints /financial/balance-sheet, /income-statement
   - [ ] Frontend rapports comptables avec drill-down
   
   ## 📐 Plan Comptable
   
   ### Classe 4: Créances et Dettes
   - 40xx: Fournisseurs
   - 41xx: Copropriétaires
   - 44xx: TVA
   
   ### Classe 5: Trésorerie
   - 50xx: Compte courant
   - 51xx: Fonds de réserve
   - 52xx: Placements
   
   ### Classe 6: Charges
   - 6000: Assurance
   - 6010: Entretien
   - 6020: Utilities
   - 6030: Chauffage
   - 6040: Nettoyage
   
   ### Classe 7: Produits
   - 7000: Appels de fonds ordinaires
   - 7100: Appels de fonds extraordinaires
   
   ## 🔗 Dépendances
   
   **BLOQUE**: #017 (État Daté), #018 (Budget), #003 (Rapports)
   
   ## ✅ Critères d'Acceptation
   
   - Bilan comptable conforme AR 12/07/2012
   - Compte de résultat conforme
   - Migration données existantes
   - Tests unitaires + E2E
   - Documentation PCN
   
   ---
   
   **Voir**: \`issues/critical/016-plan-comptable-belge.md\`

.. raw:: html

   </div>

