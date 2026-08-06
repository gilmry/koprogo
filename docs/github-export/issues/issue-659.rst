=====================================================================================
Issue #659: Auto-merge Dependabot sur feature/dev sans gate CI (merge inconditionnel)
=====================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: None
:Assignees: Unassigned
:Created: 2026-07-25
:Updated: 2026-07-25
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/659>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Les PR Dependabot sont **auto-mergées sur `feature/dev` sans qu'aucune CI ne soit exécutée au préalable**. C'est ainsi qu'un bump cassant le build (`printpdf 0.7 → 0.11` → `lopdf` ne compile plus) a été intégré sans signal bloquant.
   
   Le workflow [`dependabot-auto-merge.yml`](.github/workflows/dependabot-auto-merge.yml) **documente lui-même** sa condition de fonctionnement (lignes 11-14) :
   
   > « Branch protection sur `feature/dev` avec required status checks (au minimum : ci.yml, security.yml). **Sans cela, `--auto` merge immédiatement sans attendre la CI** — ce qui contredit la politique. »
   
   ## Cause
   
   Les deux pré-requis énoncés sont **absents** (vérifié 2026-07-25) :
   
   | Pré-requis du workflow | État réel |
   |---|---|
   | `ci.yml` déclenché sur PR vers `feature/dev` | ❌ `pull_request.branches` = main / dev / integration / staging / production — **pas `feature/**`** |
   | Branch protection + required status checks sur `feature/dev` | ❌ `GET /branches/feature/dev/protection` → `required_status_checks.contexts = []` |
   
   ⇒ `gh pr merge --auto --squash` merge **instantanément**, sans gate. La politique « auto-merge quand la CI est verte » (décision 2026-05-11) n'est donc **pas** appliquée en pratique : c'est un auto-merge inconditionnel.
   
   ### Couche 2 : le signal post-merge est noyé
   
   La CI tourne bien **en push** sur `feature/**` (post-merge). Sur `af29b54` : `CI Pipeline` ❌ · `Docker Build` ❌ · `Security Audit` ❌ · `Characterization E2E` ❌.
   
   Le break **a donc été signalé** — mais dans un tableau **rouge en permanence** (#634 frontend, #636 audit, #617 e2e). Un rouge de plus dans un fond rouge est indétectable. (Même famille que le piège tracé dans #524 : juger par job/scénario, pas par conclusion globale.)
   
   ## Recette proposée
   
   Options, à trancher (config repo + politique = décision humaine) :
   
   - **A. Gate dans le workflow** *(recommandé)* — faire attendre la CI **par le workflow lui-même** avant `gh pr merge` (job de vérification, ou `workflow_call` vers `ci.yml`). ✅ N'exige **aucune** branch protection sur `feature/dev` → pas de deadlock du buffer GitFlow. Rend enfin vrai le contrat « auto-merge quand vert ».
   - **B. Branch protection + ajout de `feature/dev` aux triggers `pull_request`** — conforme à l'intention d'origine du workflow, mais contraire au rôle de `feature/dev` comme buffer sans CI (risque de deadlock).
   - **C. Restreindre le scope** — auto-merge security/patch/minor, **major en revue humaine**. Amende la politique du 2026-05-11 (« tous les bumps, y compris breaking ») ; son rationale (faille non patchée > régression) reste valide, mais son coût est désormais mesuré.
   - **D. Assainir le fond rouge** (#634 / #636 / #617) — sans quoi aucune des options ci-dessus ne rend un nouveau break détectable.
   
   **Recommandation : A + D**, éventuellement complété par C pour les majors.
   
   ## Critères de sortie
   
   - [ ] Un bump Dependabot cassant le build est **bloqué avant merge** (test : PR volontairement cassante, ou rejeu du cas `printpdf 0.11`).
   - [ ] La politique effective correspond à la politique documentée en tête du workflow (ou le commentaire est corrigé pour refléter la réalité).
   - [ ] `feature/dev` reste sans branch protection bloquante (buffer GitFlow préservé) — ou décision explicite de changer ce choix.
   - [ ] Fond CI assaini pour que les nouveaux rouges soient visibles.
   
   ## Contexte
   
   Découvert le 2026-07-25 en investiguant le blocage build `lopdf`/`printpdf` (issue soeur). Aucun fichier modifié : workflow CI + politique = décision humaine.

.. raw:: html

   </div>

