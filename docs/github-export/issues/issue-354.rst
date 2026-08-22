=================================================================================================================
Issue #354: refactor(infra): Tests IaC manquants — terraform validate, ansible-lint, molecule, conftest ISO 27001
=================================================================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: track:software,track:infrastructure priority:high,security testing
:Assignees: Unassigned
:Created: 2026-03-29
:Updated: 2026-03-29
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/354>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   L'analyse BMAD vs codebase réelle (Maury/analyse-temporelle-bmad-vs-reel.md) révèle que l'infrastructure représente **52% des commits totaux** (1 033 sur 1 977) mais dispose de **0 tests automatisés**.
   
   ### Chiffres infra actuels
   - **920 commits** dans `koprogo-infra-restructure` + 113 dans le repo principal
   - **18 770 LOC IaC** (Terraform 989, Ansible 3 033, Helm 949, Scripts 4 902, CI/CD 841, Monitoring 1 085, Kustomize 352)
   - **236 fichiers** (39 Terraform, 47 Ansible YAML, 21 templates J2, 23 Helm, 23 Kustomize, 36 scripts, 6 workflows, 16 monitoring, 20 Dockerfiles)
   - **14 rôles Ansible** (security, monitoring, backup, k3s-master, k3s-agent, argocd, vault, velero, pgo, dns, common, docker, gitops, hardening)
   - **4 modules Terraform** (ovh-vps, ovh-k3s, ovh-k8s, networking)
   - **4 Helm charts** (koprogo, monitoring, vault, velero)
   
   ### Dette technique identifiée
   
   La boucle TDD est **incomplète** : le backend a 100% de couverture domain, 819+ BDD scenarios, 49 E2E tests — mais l'infra a **0 tests**.
   
   ## Tâches
   
   ### Phase 1 : Linting et validation statique
   - [ ] `terraform fmt -check` + `terraform validate` dans CI (workflow `ci.yml`)
   - [ ] `ansible-lint` sur les 14 rôles Ansible
   - [ ] `yamllint` sur tous les YAML (Ansible, Helm, Kustomize, docker-compose)
   - [ ] `shellcheck` sur les 36 scripts shell
   - [ ] `helm lint` sur les 4 charts
   
   ### Phase 2 : Policy-as-Code ISO 27001
   - [ ] `conftest` avec politiques OPA pour ISO 27001 contrôles A.5-A.8
   - [ ] Vérifier : LUKS activé (A.8.24), fail2ban configuré (A.8.7), logs centralisés (A.8.15)
   - [ ] Vérifier : Suricata IDS actif (A.8.16), CrowdSec WAF (A.8.7)
   - [ ] Vérifier : SSH hardening (A.8.9), kernel hardening (A.8.9)
   
   ### Phase 3 : Tests d'infrastructure
   - [ ] `molecule` pour tester les rôles Ansible (au minimum : security, monitoring, common)
   - [ ] `terratest` ou `terraform plan` automatisé pour les modules Terraform
   - [ ] Tests de backup/restore automatisés (GPG + S3)
   - [ ] Smoke tests post-déploiement (health checks, endpoints monitoring)
   
   ### Phase 4 : Intégration CI
   - [ ] Nouveau workflow `.github/workflows/infra-test.yml`
   - [ ] Exécution sur push dans `infrastructure/` ou `koprogo-infra-restructure`
   - [ ] Bloquant pour merge (comme le CI applicatif)
   
   ## Critères de succès
   - [ ] CI passe avec tous les linters infra
   - [ ] Politiques OPA ISO 27001 vérifient les 9 contrôles mappés
   - [ ] Au moins 3 rôles Ansible testés avec molecule
   - [ ] Backup/restore test automatisé passe
   
   ## Références
   - `Maury/analyse-temporelle-bmad-vs-reel.md` section 3.5
   - `Maury/Méthode Maury.md` sections 12-13 (IaC + CI/CD)
   - `infrastructure/SECURITY.md` (mapping ISO 27001)

.. raw:: html

   </div>

