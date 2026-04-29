---
persona: sre-platform
created: 2026-04-29
last_updated: 2026-04-29
---

# Mémoire persistante — `sre-platform`

> Cette mémoire est lue à chaque invocation du persona pour assurer la continuité inter-sprints.

## Décisions architecturales acceptées

- **Stack observability** : Prometheus + Grafana + Loki + Alertmanager (déjà en place cf. CLAUDE.md "Monitoring Endpoints").
- **Sécurité périmètre** : Suricata IDS + CrowdSec WAF + fail2ban + AIDE (cf. `infrastructure/SECURITY.md`).
- **RBAC k8s agent** : ServiceAccount `koprogo-agent-readonly` avec `get/list/watch` uniquement (à provisionner S1 #429).
- **Incident response trigger** : alertmanager webhook → repository_dispatch GH → workflow `incident-investigate.yml` (à câbler S4 #429).

## SLO targets actuels (à formaliser dans `docs/safe/slo.md`)

| SLO | Target | DORA quartile équivalent |
|---|---|---|
| Availability backend | 99.5 % (initial conservative) | High |
| Latency P99 backend | ≤ 500ms (cible réaliste, pas 5ms) | High |
| Error rate (5xx) | < 0.5 % sur 1h | High |
| MTTR | < 1h | Elite si < 1h |
| Backup RPO/RTO | 24h / 4h | — |

**Note historique** : la cible "P99 < 5ms" affichée dans CLAUDE.md jusqu'au 2026-04-29 était irréaliste pour une stack web SaaS. Corrigée en 500ms (cf. CLAUDE.md trim commit `23cfbea`).

## Findings audit 2026-04-29 (à investiguer une fois prod)

- ⚠️ ArgoCD application `koprogo-frontend-prod` montrait des drifts dans des audits précédents (à reproduire en environnement réel).
- ⚠️ Pas de SLO formels avant cette session — base à constituer.
- ⚠️ `xpack.security.enabled: false` sur Elasticsearch (cf. #425) — logs actuels accessibles sans auth = exfil possible.

## Conventions response acceptées

- **Severity** : SEV1 (full outage) / SEV2 (degradation) / SEV3 (minor) / SEV4 (cosmetic).
- **Diagnostic format** : timeline + hypothèses ranked (HIGH/MEDIUM/LOW confidence) + plan d'action ordonné par risque.
- **Postmortem** : draft par l'agent, signé par humain, archivé `docs/postmortem/YYYY-MM-DD-<incident>.md`.
- **Lessons** : remontent dans CSI mensuel (csi-analyst) + potentielles RFC évolution Maury (Phase H).

## Décisions en attente

- ADR : RBAC ServiceAccount `koprogo-agent-readonly` exact (qui valide la liste des verbes ?).
- RFC : SLO targets formalisés avec error budget calculation.
- ADR : alertmanager rules SLO-based (pas threshold-based) — qui maintient les seuils ?

## Lessons learned

- (à enrichir après chaque incident)

## Liens

- [`.claude/agents/sre-platform.md`](sre-platform.md)
- [`.claude/AGENT_GUARDRAILS.md`](../AGENT_GUARDRAILS.md)
- Issues : [#427](https://github.com/gilmry/koprogo/issues/427), [#429](https://github.com/gilmry/koprogo/issues/429)
- `infrastructure/SECURITY.md`, `infrastructure/_shared/monitoring/`
