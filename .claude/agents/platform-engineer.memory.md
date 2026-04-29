---
persona: platform-engineer
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `platform-engineer`

## Décisions architecturales acceptées

- **IaC Terraform** : modules dans `infrastructure/_shared/terraform/modules/` (ovh-vps, ovh-k3s, ovh-k8s, networking).
- **IaC Ansible** : roles dans `infrastructure/_shared/ansible/roles/` (common, security, k3s-master, monitoring, etc.).
- **Provider OVH** : `~> 0.51` (à pinner plus strict cf. findings).
- **Provider OpenStack** : `~> 2.1` (idem).
- **tfstate** : backend S3 (mais sans encrypt+lock+KMS actuellement — cf. findings).

## Findings audit 2026-04-29 à corriger en priorité

- 🔴 **SSH `0.0.0.0/0`** sur tous les masters K3s (`_shared/terraform/modules/ovh-k3s/main.tf:56`) — exposition brute-force directe sur 8 environnements.
- 🔴 **Backend tfstate sans encryption** (12 fichiers `backend.tf` identiques) — pas de `encrypt`, pas de `dynamodb_table` lock, pas de `kms_key_id`.
- 🔴 **SOPS `.sops.yaml`** utilise `age1placeholder_replace_with_your_public_key` — chiffrement factice.
- 🟠 **Outputs Terraform sans `sensitive = true`** sur kubeconfig, `master_ip_public`, `kubectl_command`.
- 🟠 **Pas de `prevent_destroy`** sur volumes Postgres prod ni instances K3s.
- 🟠 **Provider versions flottantes** (`~> 0.51`, `~> 2.1`) — bumps non contrôlés.

## Conventions IaC acceptées

- Terraform : `terraform fmt -check` + `terraform validate` + `tfsec` + `terraform plan` postés en commentaire de PR.
- Ansible : `ansible-lint` + `ansible-playbook --check --diff` (mode dry-run).
- Modules versionnés par tag git si extraits ; sinon source path-relative.
- Variables : tous secrets via Vault/SealedSecrets/SOPS, jamais inline `.tfvars` versionnés.

## Décisions en attente

- ADR : choix de SOPS-age (vraie clé) vs SealedSecrets vs ExternalSecrets+Vault pour le secret management runtime.
- RFC : restriction SSH `0.0.0.0/0` → CIDR list (admin + VPN + bastion).
- ADR : backend tfstate avec encrypt + DynamoDB lock + KMS key.
- ADR : `prevent_destroy = true` sur ressources data prod (volumes, RDS, etc.).

## Lessons learned

- (à enrichir après chaque sprint)

## Liens

- [`.claude/agents/platform-engineer.md`](platform-engineer.md)
- Issues : [#425](https://github.com/gilmry/koprogo/issues/425), [#429](https://github.com/gilmry/koprogo/issues/429)
- `infrastructure/SECURITY.md`, `infrastructure/_shared/terraform/`
