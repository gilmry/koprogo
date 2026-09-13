---
name: Incident
about: Diagnostic + plan d'action pour une alerte runtime (Tier 2 diagnostic, Tier 1 exécution)
title: 'incident-YYYY-MM-DD-<alertname>'
labels: ceremony, priority:high, track:infrastructure
assignees: ''
---

<!--
Gabarit posé par l'issue #429 (modèle Tier 1/Tier 2). Le persona `sre-platform`
remplit les sections DIAGNOSTIC et PLAN — c'est du Tier 2, logué dans
docs/agent-activity/. Le superviseur humain remplit APPROBATION avant toute
exécution — c'est du Tier 1, ça n'a pas eu lieu sans une trace ici
(réponse à ce commentaire, run de workflow_dispatch, ou approbation
d'environment GH).
-->

## Alerte

- **Alert ID / nom** :
- **Sévérité** : SEV1 / SEV2 / SEV3 / SEV4
- **Déclenché le** (UTC) :
- **Source** : Alertmanager / observation manuelle / rapport support
- **Fenêtre de logs/métriques jointe** :

## Diagnostic (Tier 2 — agent `sre-platform`)

**Hypothèses de cause racine :**

1.

**Blast radius (composants/tenants affectés) :**

**Éléments consultés** (logs, métriques, historique ArgoCD, issues liées, runbook) :

## Plan d'action proposé (Tier 2 — proposition seule, jamais exécutée par l'agent)

| # | Action | Risque | Type de mutation (si applicable) |
|---|--------|--------|-----------------------------------|
| 1 | | | |

## Approbation (Tier 1 — obligatoire avant toute exécution)

- [ ] Le plan a été relu par un humain.
- [ ] Décision : ✅ approuvé tel quel / ✏️ corrigé (voir commentaire) / ⛔ ignoré/escaladé.
- **Mode de validation** (cocher un) :
  - [ ] Réponse à ce commentaire par @<superviseur>
  - [ ] `workflow_dispatch` GH — run : <lien>
  - [ ] Approbation GH Environment — run : <lien>
- **Exécuté par** (toujours un humain, jamais l'agent) :
- **Trace d'exécution** (PR mergée / run de workflow / commande + audit log) :

## Postmortem

- [ ] Draft rédigé par l'agent (Tier 2) — section ci-dessous.
- [ ] Revu et signé par un humain (Tier 1).
- [ ] Archivé dans `docs/postmortem/YYYY-MM-DD-<alertname>.md`.

<details>
<summary>Draft postmortem</summary>

Voir `.github/ISSUE_TEMPLATE/postmortem-template.md` pour la structure complète.

</details>
