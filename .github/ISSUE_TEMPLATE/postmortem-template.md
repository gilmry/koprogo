---
name: Postmortem
about: Rédaction post-incident — draft agent (Tier 2), signature humaine (Tier 1)
title: 'postmortem-YYYY-MM-DD-<incident>'
labels: ceremony, track:infrastructure
assignees: ''
---

<!--
Gabarit posé par l'issue #429 (couche L7). Le draft est du Tier 2 (rédaction,
logué dans docs/agent-activity/) ; la signature est du Tier 1 (un humain
valide et archive). Un postmortem non signé n'est pas clos.
-->

## Résumé

- **Incident lié** : #
- **Sévérité** : SEV1 / SEV2 / SEV3 / SEV4
- **Durée** (détection → résolution) :
- **Impact** (utilisateurs/tenants affectés, fonctionnalités indisponibles) :

## Timeline (UTC)

| Heure | Événement |
|-------|-----------|
| | Alerte déclenchée |
| | Diagnostic ouvert (issue incident) |
| | Plan approuvé par @<superviseur> |
| | Action exécutée |
| | Résolution confirmée |

## Ce qui s'est passé

## Cause racine

## Ce qui a bien fonctionné

## Ce qui a mal fonctionné

## Actions de suivi

| # | Action | Owner | Issue |
|---|--------|-------|-------|
| 1 | | | |

## Leçons — évolution méthode Maury (Phase H)

Si un pattern récurrent apparaît sur plusieurs postmortems, proposer une RFC
(`docs/rfc/`) plutôt que de le laisser en dette tribale.

## Signature (Tier 1 — obligatoire pour clore)

- [ ] Revu et signé par : @<superviseur>, le <date>.
- [ ] Archivé dans `docs/postmortem/YYYY-MM-DD-<incident>.md`.
