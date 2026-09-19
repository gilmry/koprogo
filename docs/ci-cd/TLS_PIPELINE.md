# Pipeline TLS dev/integration/staging — DNS-01 OVH (issue #453)

> Story C9.1 : un développeur qui rejoint le projet doit pouvoir tester les
> cookies `Secure`, les Service Workers et les flux OAuth avec un certificat
> **réellement signé par une CA publique**, pas un contournement local par dev.
> Let's Encrypt ne valide pas `*.local` : d'où DNS-01 via l'API OVH, seule voie
> pour des wildcards non-prod sur un domaine public (`koprogo.com`).

**Hors scope** : la production. Le cluster prod gère son certificat via
`cert-manager` + ACME directement (issue séparée + ADR si le scope évolue).

## Vue d'ensemble

```
OVH DNS (koprogo.com)  →  lego (DNS-01)  →  SOPS/age (chiffrement)
                                                    ↓
                          infrastructure/_shared/secrets/<env>/tls.enc.yaml
                                                    ↓
        dev: pull + déchiffrement manuel   |  integration/staging: ArgoCD
```

Un seul TLD chez OVH, un wildcard par environnement
(`*.dev.koprogo.com`, `*.integration.koprogo.com`, `*.staging.koprogo.com`),
émis et renouvelé par [`.github/workflows/certs-renew.yml`](../../.github/workflows/certs-renew.yml).

## Statut au moment de l'écriture de ce document

Ce pipeline a été implémenté par un agent IA (story/453) dont les outils sont
**bloqués par construction** sur toute action Tier 1 (garde-fous
`.claude/settings.json` + `.claude/hooks/`). Trois catégories de travail restent
donc entièrement à faire par un humain avant la première exécution réelle :

| # | Action | Pourquoi bloqué pour l'agent |
|---|---|---|
| 1 | Créer les enregistrements DNS chez OVH | Accès console OVH, hors dépôt |
| 2 | Créer l'application OVH `koprogo-acme-ci` + Consumer Key scopé | Accès console OVH, hors dépôt |
| 3 | Créer l'environment GitHub `certs-issuance` + ses 5 secrets | `gh secret set` est *deny* pour l'agent (`.claude/settings.json`), création d'environment = UI GitHub |
| 4 | Générer la clé age et l'ajouter à `infrastructure/_shared/secrets/.sops.yaml` | `Write`/`Edit` sur `infrastructure/_shared/secrets/**` et `.sops.yaml` sont *deny* pour l'agent (hook `pretool-deny-secret-write.sh`) |

Le workflow lui-même **contourne le point 4** en dérivant la clé publique age
depuis le secret privé à chaque run (`age-keygen -y`), donc il fonctionne sans
attendre la mise à jour de `.sops.yaml` — mais un humain devrait quand même
ajouter une `creation_rule` pour `tls.enc.yaml` afin qu'un `sops -d` local ne
nécessite pas de repasser `--age` à la main. Voir §Setup ci-dessous.

## Setup initial (Tier 1, manuel humain — CRITICAL.md règle 11)

> Aucune de ces étapes ne peut être faite par un agent IA : mutation de
> systèmes externes (OVH) ou de secrets (GitHub). À faire une fois par un
> mainteneur avec accès admin OVH + GitHub.

### 1. Enregistrements DNS chez OVH

| Record | Type | Valeur | Note |
|---|---|---|---|
| `dev.koprogo.com` | A | `127.0.0.1` | Docker Desktop (loopback) |
| `*.dev.koprogo.com` | A | `127.0.0.1` | |
| `integration.koprogo.com` | A | `<IP runner CI>` | TBD selon stack CI retenue |
| `*.integration.koprogo.com` | A | `<IP runner CI>` | |
| `staging.koprogo.com` | A | `<IP VPS staging>` | TBD |
| `*.staging.koprogo.com` | A | `<IP VPS staging>` | |

L'apex `koprogo.com` et `www` restent gérés par la prod, hors scope ici.

### 2. Application OVH `koprogo-acme-ci`

1. https://api.ovh.com/createToken/ → créer une application dédiée.
2. Consumer Key scopé **uniquement** à `/domain/zone/koprogo.com/*`
   (GET/PUT/POST/DELETE + refresh). Jamais de scope global.
3. Noter la date de création → rotation annuelle planifiée (voir §Rotation).

### 3. Environment GitHub `certs-issuance`

1. https://github.com/gilmry/koprogo/settings/environments
2. **New environment** → nom : `certs-issuance`
3. **Required reviewers** → ajouter au moins un mainteneur
4. **Deployment branches** : restreindre à `main` (le workflow tourne sur le
   contenu de `main`, indépendamment de la branche applicative qui progresse)
5. Ajouter les secrets d'environment :
   - `OVH_ENDPOINT` = `ovh-eu`
   - `OVH_APPLICATION_KEY`
   - `OVH_APPLICATION_SECRET`
   - `OVH_CONSUMER_KEY`
   - `SOPS_AGE_KEY` (clé **privée** age, format `AGE-SECRET-KEY-1...`)

### 4. Générer la paire de clés age et compléter `.sops.yaml`

```bash
age-keygen -o /tmp/koprogo-certs.age.key
# Public key: age1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

- La **clé privée** (`AGE-SECRET-KEY-1...`) va uniquement dans le secret GitHub
  `SOPS_AGE_KEY` (étape 3) et dans un canal sécurisé pour les devs qui doivent
  déchiffrer en local (1Password partagé ou équivalent — **jamais** dans le
  dépôt, jamais dans un fichier `.env`).
- La **clé publique** (`age1...`) peut être committée sans risque. Ajouter à
  `infrastructure/_shared/secrets/.sops.yaml` une règle :

  ```yaml
  creation_rules:
    # ... règles existantes ...
    - path_regex: infrastructure/_shared/secrets/.*/tls\.enc\.yaml$
      age: >-
        age1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
  ```

  Note : le fichier `.sops.yaml` actuel utilise encore
  `age1placeholder_replace_with_your_public_key` pour ses autres règles — un
  chantier plus large que #453 (cf. finding `platform-engineer.memory.md`).
  Ne remplacer ici que ce qui concerne `tls.enc.yaml`, sans toucher aux autres
  `creation_rules` sauf décision explicite séparée.

### 5. Premier run

`gh workflow run certs-renew.yml` (déclenche les 3 environnements, chacun
attend l'approbation `certs-issuance`) ou cibler un seul environnement via
l'input `environments: dev`.

## Le workflow `certs-renew.yml`

| Étape | Fait |
|---|---|
| `lego` (image `goacme/lego`, tag épinglé — voir §Dette assumée) | Émet le certificat DNS-01 via l'API OVH |
| `check-cert-expiry.sh` | Décide s'il faut alerter (échec de renouvellement, ou marge < 60j) — logique testée par `check-cert-expiry.test.sh` |
| SOPS + age (clé publique dérivée du secret à chaque run) | Chiffre `tls.crt`/`tls.key` dans un manifest `Secret` Kubernetes |
| `git commit` + `gh pr create` | Committe `infrastructure/_shared/secrets/<env>/tls.enc.yaml` sur une branche dédiée, ouvre une PR — **jamais de merge automatique** (review humaine obligatoire, cf. Tier 2 avec PR review dans #453) |
| `gh issue create`/`comment`/`edit` | Alerte en cas d'échec, escalade `priority:high` au 2e échec consécutif — **substitut** à la notification Slack/email demandée par l'issue (voir §Dette assumée) |

Triggers : `workflow_dispatch` (manuel, ciblable par environnement) et
`schedule` bi-mensuel (`0 3 1,15 * *`, cf. cadence 90j des certs lego).

## Distribution par environnement

| Env | Cible | Méthode |
|---|---|---|
| `dev` | Docker Desktop local | Pull du dépôt + `sops -d` local (voir §Onboarding) |
| `integration` | Cluster k3s integration | ArgoCD (`argocd-vault-plugin` ou `helm-secrets` — à câbler dans une PR de suivi une fois le cluster disponible, hors scope strict de #453) |
| `staging` | Cluster k3s staging | Idem ArgoCD |
| `prod` | Cluster prod | Hors scope — `cert-manager` direct |

> `integration`/`staging` : le câblage ArgoCD précis dépend du cluster k3s
> cible (pas encore décrit ailleurs dans le repo — `infrastructure/_shared/argocd/`
> existe mais ne référence pas encore ce Secret). À faire dans une PR de suivi
> quand le cluster integration/staging existe réellement (cf. #429 runtime ops).

## Onboarding un nouveau dev (< 5 min, Docker Desktop)

Prérequis : avoir reçu la clé privée age via le canal sécurisé de l'équipe
(1Password ou équivalent — jamais par email/Slack en clair).

```bash
git clone git@github.com:gilmry/koprogo.git && cd koprogo
export SOPS_AGE_KEY="AGE-SECRET-KEY-1..."   # reçue via canal sécurisé
sops -d infrastructure/_shared/secrets/dev/tls.enc.yaml > /tmp/koprogo-tls-dev.yaml
kubectl --context docker-desktop apply -f /tmp/koprogo-tls-dev.yaml
shred -u /tmp/koprogo-tls-dev.yaml || rm -f /tmp/koprogo-tls-dev.yaml
```

Le certificat `*.dev.koprogo.com` est alors utilisable par le reverse proxy
Traefik de la stack `make dev` (cf. CLAUDE.md, ADR 0050 ports décalés).

## Rotation du Consumer Key OVH (annuelle)

1. Créer un nouveau Consumer Key scopé identique (§Setup point 2) **avant**
   d'invalider l'ancien (chevauchement pour éviter une coupure).
2. Mettre à jour le secret `OVH_CONSUMER_KEY` dans l'environment
   `certs-issuance` (UI GitHub, Tier 1 humain).
3. Déclencher un `workflow_dispatch` manuel pour valider que le nouveau
   Consumer Key fonctionne.
4. Révoquer l'ancien Consumer Key côté OVH.
5. Noter la date de rotation (prochaine échéance : +1 an).

## Incident : révocation d'un certificat compromis (Tier 1, humain)

1. Révoquer via `lego --dns ovh --domains "..." revoke` (exécution manuelle
   humaine, **jamais** via ce workflow automatisé).
2. Ouvrir une issue `priority:critical` documentant la compromission.
3. Ré-émettre via `workflow_dispatch` ciblé sur l'environnement concerné une
   fois la cause traitée.

## Dette assumée par cette implémentation

Décisions prises pendant l'implémentation de #453, à valider ou corriger par
un humain :

- **Pas de `:latest`, mais pas encore de digest sha256** pour l'image
  `goacme/lego` : épinglée sur le tag `v4.20.4`. L'agent qui a écrit le
  workflow n'avait pas d'accès réseau sortant pour résoudre et vérifier le
  digest courant. À finaliser par un humain ou via Dependabot/Renovate
  (digest pinning des images Docker).
- **Pas de Slack/email** : l'issue #453 demande une "notification
  Slack/email si renew fail 2x consécutifs", mais aucune intégration Slack
  n'existe dans ce dépôt (vérifié : aucun webhook, aucun secret Slack). Le
  workflow utilise `gh issue create`/`comment`/`edit` à la place, cohérent
  avec CRITICAL.md règle #6 ("Tout dans GitHub"). Si une intégration Slack
  existe déjà ailleurs (canal ops non documenté ici), l'ajouter en plus est
  un simple step supplémentaire dans `certs-renew.yml`.
- **Fermeture des issues d'alerte non automatisée** : sur succès après un
  échec, le workflow **commente** l'issue existante mais ne la ferme pas
  (CRITICAL.md règle 11 : fermeture d'issue = Tier 1 humain systématique).
- **Câblage ArgoCD pour integration/staging non fait** : aucun cluster k3s
  integration/staging n'est décrit dans `infrastructure/_shared/argocd/`
  aujourd'hui à câbler sur ce Secret — dépend d'une infra qui n'existe pas
  encore concrètement (cf. #429).
- **`.sops.yaml` non modifié par cet agent** : bloqué par
  `pretool-deny-secret-write.sh`. Le workflow fonctionne sans (dérivation de
  la clé publique à la volée), mais §Setup point 4 reste à faire pour le
  confort des devs qui déchiffrent en local.

## Definition of Done — état après cette implémentation

- [x] Workflow `.github/workflows/certs-renew.yml` écrit, matrice des 3 envs,
      environment `certs-issuance`, alerte GitHub native
- [x] `check-cert-expiry.sh` + tests `@happy @negative @edge @security`
      (`infrastructure/_shared/scripts/check-cert-expiry.test.sh`)
- [x] Documentation runbook (ce fichier)
- [ ] Records DNS créés chez OVH — **Tier 1 humain**
- [ ] Application OVH + Consumer Key créés — **Tier 1 humain**
- [ ] Environment GitHub `certs-issuance` + 5 secrets créés — **Tier 1 humain**
- [ ] `.sops.yaml` complété avec la vraie clé publique age — **Tier 1 humain**
- [ ] Premier run réel testé (2 cycles `workflow_dispatch`) — dépend des
      points précédents
- [ ] Câblage ArgoCD integration/staging — dépend de l'existence du cluster
