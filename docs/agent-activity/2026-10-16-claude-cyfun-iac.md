# Agent activity — 2026-10-16 — Durcissement IaC vers CyFun « Important »

**Persona :** claude (Tier 2 — diagnostic + proposal)

**Contexte :** Gilles a demandé une évaluation du niveau CyFun de l'IaC, puis
a validé l'implémentation des 5 premiers correctifs proposés, sur une branche
avec PR (`feature/cyfun-important-hardening`).

## Actions Tier 2 effectuées

- Audit de `infrastructure/` (Terraform, Ansible, compose, CI) par rapport aux
  contrôles CyFun 2023 Basic/Important.
- Correctifs proposés en PR (aucune application en environnement) :
  1. Terraform : 11 configs racines sur 12 ne parsaient pas (chemins,
     syntaxe HCL `;`, providers aliasés) ; chemins de rôles Ansible cassés
     dans les playbooks monosite.
  2. SSH restreint aux CIDR admin (security groups + ufw), variable
     validée, jamais `0.0.0.0/0`.
  3. State Terraform chiffré (`encrypt`) et verrouillé (`use_lockfile`),
     script de durcissement du bucket.
  4. Sauvegardes : clé publique générée hors ligne (plus de clé privée sur
     le serveur), copie hors site obligatoire (elle ne s'exécutait jamais),
     Object Lock, script non modifiable par l'utilisateur `koprogo`,
     métriques réellement collectées + alertes.
  5. CI : Trivy IaC + images, gitleaks serveur, SBOM/provenance, cosign,
     validation Terraform/Ansible de tous les environnements.
  6. Conteneurs : cap_drop/no-new-privileges/read_only, secrets obligatoires
     en production, `TOTP_ENCRYPTION_KEY` manquant sur le chemin IaC.

## Ce qui n'a PAS été exécuté (hors délégation / hors portée session)

- Aucun `terraform apply`, `ansible-playbook` sur un hôte réel,
  `docker compose up` en environnement, ni script de bucket (Tier 1).
- Aucune modification du code Rust (repli `TOTP_ENCRYPTION_KEY` par défaut
  dans `backend/src/main.rs` signalé, non modifié).
- `docker-compose.prod.yml` (pile démo, `deploy.sh`) non modifié.

## Vérification

- `terraform init -backend=false && terraform validate` : 12/12 racines OK ;
  validations de `admin_ssh_cidrs` testées (vide, `0.0.0.0/0`, CIDR invalide
  refusés ; IPv4+IPv6 acceptés).
- `ansible-playbook --syntax-check` : 12/12 playbooks ; `ansible-lint`
  (version CI 24.x) : aucune régression par rapport à `main`.
- Rôle backup : converge ×2 (idempotent) + verify sur Ubuntu 24.04 ; script
  rendu exécuté de bout en bout (off-site manquant → échec, nominal,
  déchiffrement hors ligne, sha256, artefact chiffré pour une autre clé
  refusé) ; `test-backup-restore.sh` vert.
- Pile compose durcie lancée sous Docker 29 avec les images ghcr réelles :
  conteneurs sains, routage 200 via Traefik en lecture seule, écriture rootfs
  backend refusée, `CapEff=0`/`NoNewPrivs=1` dans postgres ; config prod
  refuse les secrets manquants.
- `promtool check rules` (21 règles), `actionlint`, `shellcheck`, gitleaks
  sur la branche : OK.
