---
name: SLO KoproGo
owner: sre-platform (persona simulé, cf. .claude/agents/sre-platform.md)
status: draft — à valider par un humain avant toute alerte bloquante en prod
---

# SLO / SLI — KoproGo

Formalisé dans le cadre de l'issue [#429](https://github.com/gilmry/koprogo/issues/429)
(critère d'acceptation §7 : « SLO formalisés dans `docs/safe/slo.md` »).

**v0.1.0 n'est pas en production** — ces SLO sont écrits *avant* la mise en
ligne, pour que la première alerte réelle s'évalue contre une cible déjà
posée, pas contre un chiffre inventé sous pression (cf. CLAUDE.md, `.claude/rules/CRITICAL.md` règle 10).

## Cibles

| SLO | Cible | Source de mesure |
|---|---|---|
| **Disponibilité** | ≥ 99.5 % (conservateur, initial) | Prometheus uptime probe sur `/health` |
| **Latence P99** (endpoints critiques) | < 500 ms | `http_request_duration_seconds` (histogram Actix) |
| **Taux d'erreur** | < 0.5 % de 5xx sur 1h glissante | `rate(http_requests_errors_total[1h]) / rate(http_requests_total[1h])` |
| **Backup RPO** | ≤ 24h | Cadence cron backup (à valider par test restore) |
| **Backup RTO** | ≤ 4h | Temps mesuré lors d'un restore drill (cf. `docs/ops/2026-09-04-drill-f3-restauration.md`) |

La cible latence P99 remplace la cible historique 5 ms de `CLAUDE.md`
(reconnue irréaliste — cf. #429 §6). `CLAUDE.md` porte déjà la valeur
corrigée (500 ms) ; ce document en est la source détaillée.

## Écart connu avec le runbook existant

`docs/operations/monitoring-runbook.rst` définit aujourd'hui une alerte
« Slow API » sur `P99 > 1s`, plus permissive que la cible SLO ci-dessus
(500 ms). Ce document **ne modifie pas** le seuil d'alerte Alertmanager
existant — changer un seuil d'alerte en prod est une action Tier 1 (elle
change ce qui réveille un humain), pas un simple journal de decision. Faire
converger l'alerte vers la cible SLO est une action de suivi à valider par un
humain, pas un sous-produit silencieux de cette issue.

De même, `docs/operations/incident-response.rst` classe la sévérité en
P0–P3 ; le pipeline `#429` (workflow `prepare-incident-context.yml`, gabarit
`incident-template.md`) utilise SEV1–SEV4. Les deux échelles coexistent
aujourd'hui dans le dépôt ; les faire converger est aussi une décision
humaine, pas quelque chose que ce document tranche unilatéralement.

## Alertmanager

Les règles d'alerte basées sur ces SLO restent à câbler en prod (aucun
système n'est en ligne — cf. règle 10). Quand la première pile de
monitoring prod existera, les seuils ci-dessus doivent piloter les règles
Alertmanager, pas l'inverse.

## Error budget

- Budget d'erreur mensuel dérivé de la cible disponibilité : `(1 - 0.995) × 30j ≈ 3h36` d'indisponibilité tolérée par mois.
- Le burn rate du budget alimente le pipeline CSI mensuel (`docs/csi/`, cf. #429 §6bis) tenu par `csi-analyst`.

## Revue

Ces cibles sont un point de départ conservateur, pas un engagement contractuel
externe. `sre-platform` les révise à la lumière des premiers mois de
production réelle et propose des ajustements via RFC si les cibles s'avèrent
trop strictes ou trop laxistes.
