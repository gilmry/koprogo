======================================================
Issue #263: MCP: Prompt système + intégration Claude
======================================================

:State: **OPEN**
:Milestone: Jalon 4: Automation & Intégrations 📅
:Labels: enhancement,track:mcp release:0.2.0
:Assignees: Unassigned
:Created: 2026-03-10
:Updated: 2026-03-15
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/263>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Description
   
   Configurer le prompt système de l'agent IA KoproGo et valider l'intégration bout en bout avec Claude via MCP.
   
   ## Prompt système
   
   ```
   Tu es l'assistant KoproGo, spécialisé dans la gestion de copropriétés en Belgique.
   
   Tu disposes d'outils MCP pour accéder aux données de la copropriété de l'utilisateur
   et à la base légale belge (Code civil art. 3.78-3.100, Code de déontologie IPI, etc.)
   
   Principes :
   1. Toujours citer la source légale (article + paragraphe) quand tu donnes un conseil
   2. Ne jamais prendre de décision à la place de l'utilisateur
   3. Alerter sur les risques juridiques (délais, majorités, incompatibilités)
   4. Adapter ton langage au rôle : pédagogique pour un syndic bénévole, technique pour un pro
   5. Répondre en français (Belgique)
   ```
   
   ## Cas d'usage à valider (E2E)
   
   1. **Syndic bénévole prépare sa première AG** : `copropriete_info` → `ag_create` → `legal_search` pour générer un OdJ conforme avec explications à chaque point
   2. **Copropriétaire conteste une décision** : `legal_search("CP07")` → délai 4 mois, conditions (préjudice personnel, décision irrégulière/frauduleuse/abusive)
   3. **Vente d'un appartement** : `transmission_lot_dossier` → distinction FR (remboursable) vs FdR (non remboursable)
   4. **Travaux urgents** : `travaux_qualifier` → confirmer art. 3.89 §5 2° → `document_generate("appel_de_fonds")`
   5. **Vérification de conformité** : `alertes_list` → mandat expiré ? RC à jour ? BCE ? Fonds de réserve ?
   
   ## Spécification complète
   
   Voir `backend/koprogo-mcp/README.md` pour le prompt système et les cas d'usage détaillés.
   
   ## Tâches
   
   - [x] ✅ Prompt système documenté dans le README MCP
   - [x] ✅ Cas d'usage prioritaires documentés
   - [ ] Configurer le prompt système dans le serveur MCP (`tools/list` response)
   - [ ] Valider chaque cas d'usage E2E avec Claude
   - [ ] Documenter les limitations et edge cases
   - [ ] Affiner le prompt selon les résultats des tests
   
   ## Dépendances
   
   - Bloqué par toutes les issues MCP tools (#252-#261, #265)

.. raw:: html

   </div>

