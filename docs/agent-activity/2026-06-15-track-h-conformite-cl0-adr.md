# Agent activity — 2026-06-15 — Track H Conformité légale, passe CL0 (ADR)

- **Persona** : Winston (architecture) / Maury BMAD
- **Tier** : 2 (rédaction doc / ADR, autorisée par `validation.md` signée v1.0)
- **Story** : CL0 / H0-ADR (issue #618)
- **Branche** : feature/dev

## Réalisé

Rédaction des 3 ADR (statut **Proposed**, acceptation @gilmry au merge) issus du dossier Maury `track-h-conformite-legale/` signé v1.0 :

1. `docs/adr/0010-acte-de-base-conformite-copropriete.md` — modèle hybride (acte de base sur ACP + sous-total bloc), conformité 2 niveaux, gates ACP-level. H16 associations partielles différé v0.2.0 (D6).
2. `docs/adr/0011-quorum-double-et-suspension-vote.md` — double quorum têtes+quotités (Art. 3.87 §5), suspension vote lots démembrés/indivis (Art. 3.87 §1), gates votes (proxy 3/10%).
3. `docs/adr/0012-fonds-reserve-roulement.md` — fonds réserve ≥5% + roulement obligatoires (loi 2019, Art. 3.86 §3), comptes distincts, budget f64→Decimal.

Chaque ADR sourcé (liens Code civil / juridique).

## Suite (Gantt par passe)

Passe **CL1** (socle conformité ACP, code) : H4 migration `acps.total_tantiemes` → H5 `Acp::assert_conformant` → H6 `AcpRepository::find_by_id_with_metrics` → H7 bascule des 4 gates building→ACP (reprend `git stash@{0}` + résout le 400-vs-422 du spec H2).

## Tier 1 pendant
Acceptation des 3 ADR (statut Proposed → Accepted) par @gilmry au merge.
