============================================================================================
Issue #82: feat: Board of Directors (Conseil de Copropriété) - LEGAL OBLIGATION >20 units
============================================================================================

:State: **CLOSED**
:Milestone: Jalon 2: Conformité Légale Belge 📋
:Labels: enhancement,phase:vps track:software,priority:critical legal-compliance,governance
:Assignees: Unassigned
:Created: 2025-11-01
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/82>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   # Issue #022 - Conseil de Copropriété
   
   **Priorité**: 🔴 CRITIQUE - BLOQUANT PRODUCTION  
   **Estimation**: 12-15 heures  
   **Phase**: VPS MVP (Nov 2025 - Mar 2026)  
   
   ## 📋 Description
   
   **OBLIGATION LÉGALE** pour immeubles >20 lots (Article 577-8/4 Code Civil belge). Le conseil de copropriété est un organe de contrôle du syndic, élu par l'AG.
   
   **Gap critique**: 0% implémenté actuellement - Bloque production pour >20 lots (majorité du marché belge)
   
   ## 🎯 Objectifs
   
   - [ ] Nouveau rôle BoardMember avec permissions spéciales
   - [ ] Entity BoardMember (user_id, building_id, position, mandate_start/end)
   - [ ] Entity BoardDecision (subject, decision_text, deadline, status)
   - [ ] Élections conseil (vote AG) avec mandats 1 an renouvelables
   - [ ] Dashboard conseil: suivi décisions AG + alertes retards syndic
   - [ ] Tracking délais: devis (30j), travaux votés (60j), PV (30j)
   - [ ] Rapports automatiques: semestriel + annuel pour AG
   - [ ] Trigger SQL: vérification incompatibilité syndic ≠ conseil
   
   ## 📐 Rôle BoardMember
   
   ```rust
   pub enum BuildingRole {
       SuperAdmin,
       Syndic,
       Accountant,
       BoardMember,  // NOUVEAU
       Owner,
   }
   
   pub struct BoardMember {
       pub id: Uuid,
       pub user_id: Uuid,
       pub building_id: Uuid,
       pub position: BoardPosition,  // President, Treasurer, Member
       pub mandate_start: DateTime,
       pub mandate_end: DateTime,
       pub elected_by_meeting_id: Uuid,
   }
   ```
   
   ## 🔗 Permissions BoardMember
   
   - Consulter tous documents copropriété
   - Demander comptes au syndic
   - Convoquer AG si syndic défaillant
   - Vérifier exécution décisions AG
   - Émettre recommandations
   
   ## ✅ Critères d'Acceptation
   
   - Rôle BoardMember opérationnel
   - Workflow élections + mandats
   - Dashboard suivi + alertes
   - Rapports semestriels/annuels automatiques
   - Tests BDD scenarios complets
   - Trigger incompatibilité syndic/conseil
   
   ---
   
   **BLOQUE**: Production pour tout immeuble >20 lots  
   **Voir**: \`issues/critical/022-conseil-copropriete.md\`

.. raw:: html

   </div>

