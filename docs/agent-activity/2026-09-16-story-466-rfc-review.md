---
date: 2026-09-16
persona: platform-engineer
session: story-466-rfc-topology-review
tier: 2 # diagnostic + proposal écrit dans la RFC ; aucune mutation, aucun push
---

# Activity log — story/466 — revue RFC 0001 2026-09-16

## Contexte

Reprise de l'issue #466 (RFC GitOps multi-environnement) sur la branche
`story/466`. Avant d'écrire du code, vérification de l'état réel du dépôt par
rapport à la RFC 0001 déjà rédigée le 2026-05-01 (774 lignes, statut
`Draft`), pour éviter de dupliquer un travail déjà fait par d'autres agents
lors de la fusion des 28 branches (commit `b801bb4c`).

## Constat (Tier 2 — diagnostic)

- La RFC 0001 (`docs/governance/rfc/0001-gitops-multi-environment-strategy.rst`)
  existe déjà, complète, et documente l'**alternative F** (hybride
  symétrique) comme retenue.
- L'essentiel du plan d'implémentation de la RFC est **déjà mergé** dans
  `feature/dev` (visible sur `story/466`) : 4 branches `infra-*` créées,
  ApplicationSet refactoré (generator `koprogo-infra` → branches `infra-*`),
  `.github/workflows/ci-infra.yml` complet (kustomize/helm/kubeconform/
  ApplicationSet render/smoke test kind).
- **Écart trouvé** : 5 workflows de promotion/back-sync
  (`promote-infra-dev-to-integration.yml`,
  `promote-infra-integration-to-staging.yml`,
  `promote-infra-staging-to-prod.yml`, `backsync-infra-prod.yml`, plus
  référence croisée dans `ci-infra.yml`) citent tous « RFC #466 (Alternative
  D — release train + back-sync) » — un modèle **différent** de
  l'alternative F documentée dans la RFC 0001, et qui de plus entre en
  collision avec la lettre D **déjà utilisée dans cette même RFC** pour
  désigner « 2 repos apps/infra » (rejetée).
- Recherche de la décision réelle : `gh issue view 466` a été refusé par la
  politique d'approbation de l'environnement (accès réseau/API GitHub non
  disponible dans cette session) — impossible de vérifier si un commentaire
  a effectivement renommé/remplacé l'alternative F par un modèle D distinct
  après la rédaction de la RFC.

## Action Tier 2 (fichiers modifiés, aucun push)

Ajout d'une section « Écart constaté entre cette RFC et l'implémentation
mergée (2026-09-16) » dans `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst`,
juste avant « Processus Revue RFC ». Elle :

- documente précisément les 5 fichiers en désaccord et la nature du
  désaccord (cascade infra-only à base de tags `infra-rc-v*` et
  back-sync vers `main`, vs. cascades C1/C2/C3 documentées ici) ;
- explique pourquoi ce n'est **pas** à l'agent de trancher (décision de
  topologie réservée à l'humain, cf. critère de sortie #466 : « un humain
  aura signé la décision, label `accepted` ») ;
- liste ce qui reste vrai indépendamment du choix F/D (branches infra-*,
  refactor ApplicationSet, ci-infra.yml, aucune mutation prod autonome) ;
- demande explicitement au mainteneur de confirmer quel modèle fait foi et
  de mettre à jour le statut de la RFC en conséquence.

## Ce qui n'a pas été fait (délibérément, hors scope Tier 2)

- Pas de changement de statut de la RFC (reste `Draft`).
- Pas de réécriture des workflows `promote-*`/`backsync-*`.
- Pas de validation sur cluster réel (« 8 Applications saines ») : nécessite
  soit un cluster de test, soit d'attendre la résolution de l'écart
  F/D ci-dessus — valider un modèle dont on ne sait pas s'il est le bon
  n'apporterait pas d'information utile.
- Pas de CODEOWNERS pour `infrastructure/monosite/k3s/production/` (listé
  dans le plan RFC mais non créé) : dépend aussi de la résolution F/D
  (les chemins/branches protégés diffèrent selon le modèle retenu).

## Décision reportée (Tier 1 humain requis)

**Bloquant** : confirmer sur l'issue #466 si le modèle réellement en
production de fait (workflows mergés, alternative « D — release train +
back-sync ») remplace l'alternative F de la RFC 0001, ou si les workflows
doivent être corrigés pour suivre F. Sans cette confirmation, la RFC ne peut
pas passer en `Accepted` et l'issue #466 ne peut pas être fermée sans risquer
de valider une topologie qui n'a jamais été formellement décidée par
personne.

## Liens

- Issue #466 : https://github.com/gilmry/koprogo/issues/466
- RFC 0001 : `docs/governance/rfc/0001-gitops-multi-environment-strategy.rst`
- Log précédent : `docs/agent-activity/2026-05-01-platform-engineer.md`
- Workflows en écart : `.github/workflows/promote-infra-dev-to-integration.yml`,
  `promote-infra-integration-to-staging.yml`, `promote-infra-staging-to-prod.yml`,
  `backsync-infra-prod.yml`
