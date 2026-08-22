=====================================================================================
Issue #355: refactor(infra): Restructuration IaC — repo séparé, tests, policy-as-code
=====================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: enhancement,track:software track:infrastructure,priority:high
:Assignees: Unassigned
:Created: 2026-03-29
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/355>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   Issue parente pour la restructuration complète de l'infrastructure. Décompose le travail identifié dans l'analyse BMAD (Maury/analyse-temporelle-bmad-vs-reel.md).
   
   L'infra représente 52% des commits du projet mais était traitée comme 3 stories M dans le plan BMAD original.
   
   ## Sous-issues
   
   ### Tests IaC (voir #354)
   Terraform validate, ansible-lint, molecule, conftest ISO 27001
   
   ### Structure repo infra
   - [ ] Documenter la structure `koprogo-infra-restructure` (README actualisé)
   - [ ] Synchroniser les Makefiles (`Makefile.infra`) avec les deux repos
   - [ ] Définir la stratégie de versioning infra (tags semver indépendants ?)
   
   ### CI/CD infra dédié
   - [ ] Workflow `infra-lint.yml` dans le repo infra
   - [ ] Workflow `infra-deploy.yml` (terraform plan → review → apply)
   - [ ] ArgoCD sync status dans les PR comments
   
   ### Documentation
   - [ ] Mettre à jour `infrastructure/SECURITY.md` avec les résultats des tests
   - [ ] Mapping ISO 27001 → tests automatisés (tableau de couverture)
   - [ ] Runbooks ITIL : incident, changement, release (mis à jour)
   
   ## Relation avec Méthode Maury
   
   Cette issue comble l'angle mort identifié dans le pipeline BMAD v1 :
   - Plan initial : 3 stories M dans Sprint 0
   - Réalité : 1 033 commits, 18.7k LOC, 14 rôles Ansible, 4 modules Terraform
   - Correction v2 : IaC + CI/CD traités comme couches à part entière
   
   ## Priorité
   
   **HIGH** — L'infra non testée est la plus grosse dette technique du projet. Bloque la confiance pour le passage en production (Jalon 1).

.. raw:: html

   </div>

