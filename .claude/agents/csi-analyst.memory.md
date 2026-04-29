---
persona: csi-analyst
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `csi-analyst`

## Sources de métriques

| Source | Métriques fournies | Owner |
|---|---|---|
| Prometheus / Grafana | SLO trend, latency P99, error rate | `sre-platform` |
| Alertmanager history | MTTR, MTBF | `sre-platform` |
| GH issues `incident-*` (fermées) | Incidents count, root causes distribution | `sre-platform` |
| GH PRs mergées | Deploy frequency, change failure rate (DORA) | `devops-engineer` |
| `docs/postmortem/*` | Action items follow-up | `sre-platform` |
| GH Discussions Q&A | Support volume, question types, satisfaction | `support-agent` |
| `docs/agent-activity/*` | Activity report tous personas | `documentation-writer` |
| GH labels / milestones | Velocity, sprint completion rate | `scrum-master-X` |
| `docs/metrics/token-budget-*` | Token budget trend | `documentation-writer` |

## Cibles DORA + SRE (à benchmarker mensuel)

| Métrique | Cible Elite | Cible Medium | KoproGo actuel |
|---|---|---|---|
| Deploy frequency | multiple/jour | weekly | (à mesurer S1) |
| Lead time for change | < 1 jour | 1 sem - 1 mois | (à mesurer S1) |
| MTTR | < 1h | < 1 jour | (à mesurer après 1er incident) |
| Change failure rate | < 5 % | 16-30 % | (à mesurer post-prod) |
| SLO availability | ≥ 99.9 % | ≥ 99 % | (cible 99.5 % initial cf. sre-platform) |
| Latency P99 | < 500ms | < 1s | (cible ≤ 500ms cf. sre-platform) |
| Support resolution time | < 4h | < 24h | (à mesurer après 1er Q&A) |
| Doc coverage Q&A | ≥ 80 % | ≥ 50 % | (à mesurer après 1er mois Q&A) |

## CSI reports historiques

(vide — premier rapport mensuel attendu fin mai 2026)

## Improvement opportunities détectées

(vide — top-3 à identifier dans le 1er CSI report)

## RFCs évolution Maury (Phase H) proposées par moi

(vide — première proposable après 1er PI Inspect & Adapt)

## Conventions report acceptées

- Format CSI report : header frontmatter + Executive summary + Métriques DORA+SLO tableau + Top-3 opportunities + Lessons incidents + Lessons support + Maury evolution candidates.
- Toujours **trend vs période précédente** explicité (↑ +X %, ↓ -Y %, → stable).
- Toujours **comparaison DORA quartile** (Elite/High/Medium/Low).
- **Top-3 only** pour improvement opportunities — pas de paralysie par liste exhaustive.
- **Honnêteté** : pas de spin positive sur métriques mauvaises.

## Lessons learned

- (à enrichir après chaque CSI cycle)

## Liens

- [`.claude/agents/csi-analyst.md`](csi-analyst.md)
- Issues : [#427](https://github.com/gilmry/koprogo/issues/427), [#428](https://github.com/gilmry/koprogo/issues/428), [#429](https://github.com/gilmry/koprogo/issues/429)
- DORA references : *Accelerate* (Forsgren/Humble/Kim), *State of DevOps Report*.
