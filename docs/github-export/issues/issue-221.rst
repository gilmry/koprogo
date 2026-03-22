===============================================================================
Issue #221: R&D: Rôles Tenant & Guest - Modèle de participation communautaire
===============================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: enhancement,priority:medium community,R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-07
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/221>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'étude RBAC (``docs/RBAC_STUDY.rst`` sections 4-6) a détaillé l'intégration
   des rôles Tenant (locataire) et Guest (invité externe) dans l'écosystème KoproGo.
   
   **Issues liées**: #71, #72
   
   ## Problématiques R&D
   
   ### Tenant (Locataire)
   - **Modèle de bail** : table ``tenants`` avec ``lease_start/end``, ``unit_id``, ``owner_id``
   - **Accès charges** : limité aux charges du lot pour lequel le bail est actif
     - Si l'owner a 3 appartements, le tenant ne voit que les charges de SON lot
   - **Participation SEL** : système de crédits séparé (tenant_credits ≠ owner_credits)
   - **JWT scope** : ``tenant_unit_ids`` dans les claims pour filtrage backend
   
   ### Guest (Invité externe)
   - **Invitation model** : lien temporaire avec expiration (30-90 jours)
   - **Permissions granulaires** : ``can_participate_sel``, ``can_access_notices``, etc.
   - **GDPR** : suppression automatique des données à expiration
   - **Audit trail** : qui a invité, quand, pour quels modules
   
   ## Points de décision
   
   - [ ] Schéma SQL des tables ``tenants`` et ``guest_invitations``
   - [ ] Intégration avec le SEL existant (``owner_id`` → ``participant_id`` abstraction)
   - [ ] Mécanisme d'invitation (lien magique vs. code vs. approbation syndic)
   - [ ] Politique de rétention GDPR pour les guests expirés
   - [ ] Impact sur les composants frontend existants (filtrage par rôle)
   
   ## Livrables
   
   - Design document RST
   - Schéma SQL proposé
   - Impact analysis sur les modules existants (SEL, notices, skills, shared-objects)
   
   ## Estimation
   
   6-10h

.. raw:: html

   </div>

