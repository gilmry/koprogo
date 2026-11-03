# 2026-11-03 — Jarvis : lot 6, documentation alignée sur le code

Persona : agent Jarvis, lettre de mission v0.1.0, lot 6. Documentation et
en-têtes uniquement, aucun code.

## Faits relevés (lecture du code et des migrations, 03/11)

- `buildings.organization_id` supprimée, `buildings.acp_id` obligatoire
  (`20260601040000_buildings_acp_id_not_null.sql`).
- `acps.organization_id` nullable, `ON DELETE SET NULL`
  (`20260601010000_create_acps.sql`).
- RLS désactivé depuis `20250103000002_disable_rls_policies.sql`.
- Contrôle d'accès applicatif : `verify_*_org_access`
  (`middleware/scope_guard.rs`).

## Corrections

| Fichier | Correction |
|---|---|
| `backend/index.rst` | section multi-tenancy réécrite, exemple de schéma `buildings` avec `acp_id` |
| `ROLE_PERMISSIONS_MATRIX.rst` | scoping décrit tel qu'il est (gardes applicatives) |
| `RBAC_STUDY.rst` | note « étude de conception, pas le code actuel » |
| `user-guides/MULTI_ROLE_SUPPORT.md`, `MULTI_OWNER_SUPPORT.md` | note sur le modèle d'accès |
| `user-guides/accountant-guide.rst` | « isolées par organization_id » remplacé |
| `OWNER_MODEL_REFACTORING.rst` | marqué historique |
| `adr/0003` | note de correction (même règle que les ADR 0001, décision du PO du 31/10) |
| `adr/0053` | statut « non implémentée au 29/10/2026 » |
| `legal/GDPR_COMPLIANCE_CHECKLIST.md` | isolation décrite sans RLS |

Déjà faits par #1050 : bandeau de statut, 99,74 %, 0,12 g, `MISSION.rst` RLS.
