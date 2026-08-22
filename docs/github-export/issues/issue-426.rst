=========================================================================================================
Issue #426: Doc — Archiver binaires root, dédoublonner docs/, désinfler CLAUDE.md, mettre à jour .claude/
=========================================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: documentation,track:infrastructure priority:high
:Assignees: Unassigned
:Created: 2026-04-29
:Updated: 2026-04-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/426>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Audit du dépôt 2026-04-29 : la documentation s'est accumulée au point de devenir plus toxique qu'utile (binaires lourds versionnés, doublons, summaries périmés, CLAUDE.md gonflé, doc Claude Code obsolète). Cette issue acte le nettoyage et les mises à jour à faire.
   
   Issue compagnon : meta-garde-fous IA (la doc obsolète `.claude/hooks.md` y est aussi listée).
   
   ---
   
   ## 1. Constats
   
   ### 1.1 Pollution du root — 4.5 Mo de binaires versionnés
   | Fichier | Taille |
   |---|---|
   | `Guide_Comptable_KoproGo.docx` | 867 Ko |
   | `Guide_Comptable_KoproGo.docx.pdf` | 1.5 Mo |
   | `Guide_Syndic_KoproGo.docx` | 854 Ko |
   | `Guide_Syndic_KoproGo.pdf` | 1.3 Mo |
   | `documentation_admin_koprogo.docx` | 573 Ko |
   | `rapport-tests-e2e-koprogo.docx` | ~25 Ko |
   
   **Problème** : binaires non-diffables, non-mergeables, gonflent les clones. Doivent vivre dans GitHub Releases ou un bucket S3, pas dans Git.
   
   ### 1.2 Brouillons personnels au root
   - `prompt-chatgpt-philosophie-gilles.md`
   - `prompt-gh-update-issues.md`
   - `prompt-sync-github-docs.md`
   
   **Problème** : documents de travail personnel — pas du projet livrable. À déplacer dans `docs/cowork/archive/` (déjà existant) ou supprimer après confirmation.
   
   ### 1.3 Scripts orphelins au root
   - `get_building_id.py`
   - `test_post.py`
   
   **Problème** : scripts ad-hoc qui devraient être dans `scripts/` ou supprimés.
   
   ### 1.4 Doublons documentaires dans `docs/`
   - `CONVOCATIONS_AG.rst` ↔ `CONVOCATIONS_SYSTEM.rst`
   - `NOTIFICATIONS_SYSTEM.rst` ↔ `NOTIFICATION_SYSTEM.rst`
   
   **Problème** : confusion sur la version canonique, drift entre les deux.
   
   ### 1.5 Summaries / inventories périmés au root
   - `IMPLEMENTATION_SUMMARY.md`
   - `INFRASTRUCTURE_DEPLOYMENT_SUMMARY.md`
   - `MCP_INTEGRATION_SUMMARY.md`
   - `FRONTEND_INVENTORY.md`
   
   **Problème** : snapshots à un instant T qui drift sans owner. Soit on les date/archive, soit on les supprime.
   
   ### 1.6 CLAUDE.md gonflé — 1 469 lignes, 92 "✅ NOUVEAU"
   - Mélange architecture, journal de release, marketing.
   - Claims faux : "137 k LOC" (réel : 29 726), "Jalon 0 ✅ COMPLÉTÉ" (74 issues open).
   - Listings d'endpoints API exhaustifs (devrait être auto-généré depuis utoipa).
   - Roadmap en double avec `docs/ROADMAP_PAR_CAPACITES.rst`.
   
   **Problème** : un fichier auto-loadé à chaque session Claude Code → chaque mensonge ou claim obsolète biaise tous les agents. Doit être ramené à < 300 lignes.
   
   ### 1.7 Doc Claude Code obsolète
   - `.claude/hooks.md` documente le format `pre-commit:` / `post-commit:` qui n'est plus le format actuel des hooks Claude Code (`PreToolUse` / `PostToolUse` / `Stop` / `UserPromptSubmit` / `SessionStart`).
   - `.claude/README.md` mentionne un dossier `templates/` qui n'existe pas.
   
   **Problème** : doc qui rassure sans rien protéger ni guider.
   
   ### 1.8 Recettes manquantes
   - Pas de `docs/AGENT_RECIPES.md` synthétisant les recettes (skills, hooks, commands).
   - Pas de `docs/cowork/STYLE_GUIDE.md` pour les conventions agent.
   - Pas de séparation claire CLAUDE.md / ROADMAP / API_REFERENCE.
   
   ---
   
   ## 2. Cause
   
   L'historique du repo montre une accumulation **sans ménage périodique** : chaque feature ajoute son `*_SYSTEM.rst`, son `IMPLEMENTATION_SUMMARY.md`, son `prompt-*.md` au root. Les binaires ont été ajoutés "pour partager rapidement" sans politique d'archivage. Aucun owner désigné pour la doc transverse.
   
   Lié au manque de garde-fous IA : aucun hook ne refuse l'ajout d'un binaire, aucun lint markdown ne signale un doublon de titre.
   
   ---
   
   ## 3. Plan d'action
   
   ### Phase 1 — Archive des binaires
   - [ ] Créer GitHub Release `docs-archive-v0.1.0` (ou bucket S3) avec les 5 binaires `.docx`/`.pdf`.
   - [ ] `git rm` les binaires du tronc (historique conservé pour compatibilité).
   - [ ] `.gitignore` : ajouter `*.docx`, `*.pdf` avec exception explicite si nécessaire.
   - [ ] Hook PreToolUse Write : refuser tout `*.docx`/`*.pdf` au commit (cf issue garde-fous IA).
   
   ### Phase 2 — Déplacements / suppressions
   - [ ] Déplacer les `prompt-*.md` du root vers `docs/cowork/archive/`.
   - [ ] Déplacer `get_building_id.py` et `test_post.py` vers `scripts/legacy/` (ou supprimer après confirmation owner).
   - [ ] Vérifier que `Maury/` au root a un README expliquant son rôle (sinon archiver).
   
   ### Phase 3 — Dédoublonnage
   - [ ] Audit `CONVOCATIONS_AG.rst` vs `CONVOCATIONS_SYSTEM.rst` : identifier canonique, fusionner contenu utile, supprimer doublon.
   - [ ] Idem `NOTIFICATIONS_SYSTEM.rst` vs `NOTIFICATION_SYSTEM.rst`.
   - [ ] Datage/archivage des `*_SUMMARY.md` du root (ou suppression si remplacés par documentation vivante).
   
   ### Phase 4 — Désinflation CLAUDE.md
   - [ ] Réduire CLAUDE.md à ≤ 300 lignes : architecture hexagonale, commandes, conventions de tests, philosophie, pointeurs vers docs/.
   - [ ] Extraire la roadmap → `docs/ROADMAP_PAR_CAPACITES.rst` (existant).
   - [ ] Extraire les listings d'endpoints → `docs/api/` (auto-généré utoipa, ou script `make docs-api`).
   - [ ] Supprimer les 92 "✅ NOUVEAU" et le storytelling marketing.
   - [ ] Régénérer les claims chiffrés via script (LOC mesurés via `tokei`, issues open/closed via `gh`, scenarios via grep `Scenario:`).
   
   ### Phase 5 — Doc Claude Code à jour
   - [ ] Remplacer `.claude/hooks.md` par `.claude/AGENT_GUARDRAILS.md` (produit par l'issue garde-fous IA).
   - [ ] Mettre à jour `.claude/README.md` (retirer mention `templates/` absent, lier vers les nouveaux skills/agents/commands).
   - [ ] Créer `docs/AGENT_RECIPES.md` : synthèse des skills/hooks/commands, comment les invoquer, exemples concrets de prompts gagnants.
   - [ ] Créer `docs/cowork/STYLE_GUIDE.md` : conventions de prompt, anti-patterns observés dans l'audit, points de validation humaine.
   
   ### Phase 6 — Hygiène continue
   - [ ] CI : action `markdown-link-check` pour détecter liens cassés.
   - [ ] CI : action qui refuse `*.docx`/`*.pdf` ajoutés.
   - [ ] Hook `PostToolUse Edit` markdown : warn si doublon de titre H1 détecté avec un autre fichier.
   - [ ] Cron mensuel : régénérer les claims chiffrés de CLAUDE.md.
   
   ---
   
   ## 4. Critères d'acceptation
   
   - [ ] Aucun binaire `.docx`/`.pdf` au root du tronc Git (`git ls-files | grep -iE '(docx|pdf)$'` doit être vide ou whitelist explicite).
   - [ ] Plus de doublons `*SYSTEM.rst` ↔ `*_SYSTEM.rst` dans `docs/`.
   - [ ] CLAUDE.md ≤ 300 lignes, claims chiffrés vérifiables.
   - [ ] `docs/AGENT_RECIPES.md` existe et liste skills/hooks/commands actifs.
   - [ ] `docs/cowork/STYLE_GUIDE.md` existe.
   - [ ] `.claude/hooks.md` supprimé ou redirige vers `AGENT_GUARDRAILS.md`.
   - [ ] Phases 1-5 mergées via PR séparées (chaque phase reviewable indépendamment).
   - [ ] Phase 6 : markdown-link-check et anti-binary action actifs en CI.
   
   ---
   
   ## 5. Liens
   
   - Issue compagnon : meta-garde-fous IA (constat audit + plan).
   - Audit conversation Claude Code 2026-04-29.
   - `docs/cowork/` (déjà existant pour journaux de prompts).
   
   ---
   
   🤖 Issue générée par Claude Opus 4.7 (1M context) après audit du dépôt.

.. raw:: html

   </div>

