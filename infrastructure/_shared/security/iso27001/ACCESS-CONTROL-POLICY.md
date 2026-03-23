# Politique de Controle d'Acces - KoproGo

## ISO 27001:2022 - A.8.2, A.8.3, A.8.5

### 1. Roles applicatifs

| Role | Description | Permissions |
|------|-------------|-------------|
| SuperAdmin | Administration plateforme | Tout |
| Syndic | Gestionnaire copropriete | CRUD batiments, charges, AG, votes |
| Owner | Coproprietaire | Lecture ses lots, vote, paiement |
| Accountant | Comptable | PCMN, ecritures, rapports financiers |
| Contractor | Prestataire | Devis, rapports travaux (magic link) |
| BoardMember | Membre conseil | Dashboard, decisions, suivi |

### 2. Acces infrastructure

| Acces | Methode | MFA | Audit |
|-------|---------|-----|-------|
| SSH serveurs | Cle ED25519 uniquement | N/A | auditd |
| ArgoCD UI | OIDC / admin password | Oui | ArgoCD logs |
| Grafana | Admin password | Non | Grafana audit |
| Kibana | Basic auth | Non | ES audit |
| Vault | Token / K8s auth | Oui | Vault audit |
| GitHub | SSH key + PAT | Oui (org) | GitHub audit log |

### 3. Principe du moindre privilege

- K8s: ServiceAccounts par composant, NetworkPolicies
- DB: User `koprogo` avec permissions limitees au schema
- S3: Bucket policy private, pas de public access
- Docker: Non-root containers (UID 1000/101)
