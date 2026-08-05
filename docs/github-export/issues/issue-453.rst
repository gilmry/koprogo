========================================================================
Issue #453: Pipeline TLS dispatch dev/integration/staging via OVH DNS-01
========================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:infrastructure,priority:medium security
:Assignees: Unassigned
:Created: 2026-04-30
:Updated: 2026-04-30
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/453>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Constat
   
   Pas de pipeline automatisé pour émettre/renouveler des certificats TLS valides (signés par une CA publique) sur les environnements non-prod (`dev`, `integration`, `staging`). Conséquences :
   
   - Tests E2E, OAuth flows, Service Workers, cookies `Secure` : impossibles ou contournés en dev avec certs auto-signés / hosts file.
   - Divergence dev ↔ prod : la prod aura un cert ACME, les autres envs des bricolages locaux différents par dev.
   - Pas de pattern reproductible pour onboarder un nouveau dev sur Docker Desktop ou un nouveau cluster k3s integration/staging.
   
   ## Cause
   
   - Let's Encrypt / ZeroSSL ne signent que des domaines réels validables publiquement → impossible avec `*.local` + `/etc/hosts`.
   - Pas de zone DNS publique configurée pour les sous-domaines non-prod de `koprogo.com`.
   - Pas de tooling commun pour distribuer les certs émis vers les clusters cibles (Docker Desktop local, k3s integration, k3s staging).
   
   ## Recette proposée
   
   **Architecture cible** : un seul TLD `koprogo.com` chez OVH, wildcards par environnement, ACME DNS-01 via OVH API, certs chiffrés SOPS/age committés dans le repo, distribution via ArgoCD (k3s) ou pull manuel (Docker Desktop).
   
   ### DNS (records à créer manuellement chez OVH, une fois)
   
   | Record | Type | Valeur | Note |
   |---|---|---|---|
   | `dev.koprogo.com` | A | `127.0.0.1` | résolution publique vers loopback (Docker Desktop) |
   | `*.dev.koprogo.com` | A | `127.0.0.1` | wildcard pour sous-services dev |
   | `integration.koprogo.com` | A | `<IP runner CI>` | TBD selon stack CI |
   | `*.integration.koprogo.com` | A | `<IP runner CI>` | |
   | `staging.koprogo.com` | A | `<IP VPS staging>` | TBD |
   | `*.staging.koprogo.com` | A | `<IP VPS staging>` | |
   
   Apex `koprogo.com` + `www` : géré par la prod (cert-manager dans cluster prod, hors scope de cette issue).
   
   ### Credentials OVH (création manuelle, Tier 1)
   
   - Application OVH dédiée : `koprogo-acme-ci`
   - Consumer Key scopé **uniquement** aux routes `/domain/zone/koprogo.com/*` (GET/PUT/POST/DELETE record + refresh)
   - Validity : illimité avec **rotation annuelle** planifiée
   
   ### Secrets GitHub (env `certs-issuance` avec required reviewers = humain)
   
   - `OVH_ENDPOINT` = `ovh-eu`
   - `OVH_APPLICATION_KEY`
   - `OVH_APPLICATION_SECRET`
   - `OVH_CONSUMER_KEY`
   - `SOPS_AGE_KEY` (clé privée age pour chiffrer les certs émis)
   
   ### Workflow GitHub Action
   
   - Fichier : `.github/workflows/certs-renew.yml`
   - Triggers :
     - `workflow_dispatch` (manuel, première exécution + rerun)
     - `schedule: cron '0 3 1,15 * *'` (bi-mensuel, 03:00 UTC, laisse 2 fenêtres avant expiration 90j)
   - Job matrix : `[dev, integration, staging]`
   - Environment : `certs-issuance` → required reviewers
   - Steps :
     1. Install `lego` (provider OVH natif)
     2. `lego --dns ovh --domains "*.<env>.koprogo.com" --domains "<env>.koprogo.com" --email ops@koprogo.com --accept-tos run`
     3. Chiffrement immédiat avec SOPS + age (clé pub committée, clé priv en secret)
     4. Commit dans `infrastructure/_shared/secrets/<env>/tls.enc.yaml` via PR auto
     5. Notification Slack/email si échec
   - Aucun secret en clair écrit (cf. règle critique #1, hooks PreToolUse)
   
   ### Distribution
   
   | Env | Cible | Méthode |
   |---|---|---|
   | `dev` | Docker Desktop local | Pull repo, déchiffrement SOPS + application manuelle du Secret K8s |
   | `integration` | Cluster k3s integration | ArgoCD + `argocd-vault-plugin` ou `helm-secrets` |
   | `staging` | Cluster k3s staging | Idem ArgoCD |
   | `prod` | Cluster prod | **Hors scope** : cert-manager + ACME directement dans le cluster prod |
   
   ## Critères d'acceptation
   
   - [ ] Records DNS `*.<env>.koprogo.com` créés chez OVH pour `dev`, `integration`, `staging`
   - [ ] Application OVH `koprogo-acme-ci` créée avec Consumer Key scopé minimal
   - [ ] Secrets GH ajoutés à un environment `certs-issuance` avec required reviewers
   - [ ] Workflow `.github/workflows/certs-renew.yml` opérationnel sur les 3 envs
   - [ ] Cert wildcard `*.<env>.koprogo.com` valide < 60j d'expiration en permanence sur les 3 envs
   - [ ] Rotation auto fonctionnelle (testée via `workflow_dispatch` manuel sur 2 cycles)
   - [ ] Alerte Slack/email si renew fail 2x consécutifs
   - [ ] Documentation `docs/ci-cd/TLS_PIPELINE.md` : runbook + procédure rotation OVH consumer key
   - [ ] Test : un dev fresh peut récupérer le cert dev en < 5 min sur Docker Desktop (procédure documentée)
   
   ## Tier d'autorisation
   
   - Setup initial (création secrets GH, application OVH, premier workflow run) = **Tier 1** (humain valide systématiquement)
   - Rotation auto bi-mensuelle = **Tier 2** mais via PR auto qui demande review humain avant merge dans `infrastructure/_shared/secrets/`
   - Révocation cert (incident) = **Tier 1** (humain)
   
   ## Lignes rouges
   
   - Aucun `*.pem`, `*.key`, `tls.crt` en clair commité (cf. #425, hooks bloquants)
   - Pas de Consumer Key OVH avec scope global (uniquement zone `koprogo.com`)
   - Pas de `:latest` sur l'image `lego` dans le workflow (digest pinning)
   - Pas d'extension du scope vers la prod sans issue séparée + ADR
   
   ## Liens
   
   - Garde-fous : #425
   - Runtime ops / Tier 1-2 : #429
   - Doc cible : `docs/ci-cd/TLS_PIPELINE.md` (à créer)

.. raw:: html

   </div>

