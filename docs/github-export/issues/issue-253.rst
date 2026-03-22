===============================================
Issue #253: MCP: Auth JWT + matrice rôle/outil
===============================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,security track:mcp,release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/253>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Implémenter l'authentification JWT et le contrôle d'accès par rôle pour les outils MCP.
   
   ## Tâches
   
   - [ ] Créer `src/mcp/auth.rs` : extraction et validation du Bearer token JWT
   - [ ] Implémenter la matrice rôle × outil (voir README.md)
   - [ ] Vérifier le rôle actif de l'utilisateur (`user_roles` table existante)
   - [ ] Retourner erreur MCP standard si outil non autorisé pour le rôle
   - [ ] Support des niveaux d'accès (ex: copropriétaire voit uniquement son lot)
   - [ ] Tests unitaires pour chaque combinaison rôle/outil
   
   ## Matrice rôle × outil
   
   | Outil | Syndic | Copropriétaire | Locataire | Commissaire | CdC |
   |-------|--------|----------------|-----------|-------------|-----|
   | legal_search | oui | oui | oui | oui | oui |
   | majority_calculator | oui | oui | - | oui | oui |
   | copropriete_info | oui | oui (limité) | - | oui | oui |
   | list_coproprietaires | oui | oui | - | oui | oui |
   | ag_create | oui | - | - | - | - |
   | ag_quorum_check | oui | - | - | - | - |
   | ag_vote | oui | - | - | - | - |
   | ag_generate_pv | oui | - | - | - | - |
   | comptabilite_situation | oui | oui (son lot) | - | oui | oui |
   | appel_de_fonds | oui | - | - | - | - |
   | travaux_qualifier | oui | oui | - | - | oui |
   | transmission_lot_dossier | oui | oui (son lot) | - | - | - |
   | alertes_list | oui | oui | - | oui | oui |
   | documents_list | oui | oui (non privé) | oui (ROI) | oui | oui |
   | document_generate | oui | - | - | - | - |
   | energie_campagne_list | oui | oui | oui | - | oui |
   | energie_inscrire | oui | oui | oui | - | - |
   | energie_offre_personnalisee | oui | oui (son offre) | oui | - | - |
   | energie_comparer_tarif | oui | oui | oui | - | oui |
   | energie_ag_point | oui | - | - | - | - |
   
   > **Note** : Les locataires avec compteur individuel peuvent participer à l'achat groupé
   > via le rôle « occupant énergie », sans passer par le syndic.
   
   ## Spécification complète
   
   Voir `backend/koprogo-mcp/README.md` pour la matrice rôle × outil détaillée.
   
   ## Dépendances
   
   - Bloqué par #252 (serveur SSE)
   - Réutilise le middleware auth existant (`AuthenticatedUser`, `user_roles`)

.. raw:: html

   </div>

