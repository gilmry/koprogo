# RUNBOOK — KoproGo v0.1.0 sur VPS de production (Phase 1)

> **Statut** : rédigé Tier-2 (agent) à partir du code d'infrastructure existant
> (`infrastructure/monosite/vps/production/`, `infrastructure/_shared/`). **Aucune
> commande de ce document n'a été exécutée en session distante** (pas d'accès
> OVH/DNS réel ici) — à **relire et valider par un humain** avant le premier
> déploiement réel (WP-F4, DoD go-live). Périmètre : bêta privée fermée, VPS
> OVH + docker-compose (Phase 1 — cf. `docs/WBS_GO_LIVE_v0.1.0.md`).

## 1. Topologie

- **Infra as code** : `infrastructure/monosite/vps/production/terraform/` (module
  `../../../../_shared/terraform/modules/ovh-vps`, provider `ovh` + `openstack`).
- **Provisioning** : `infrastructure/monosite/vps/production/ansible/playbook.yml`
  — rôles `common`, `hardening`, `docker`, `security`, `monitoring`, `backup`,
  `gitops`, `dns` (chemins relatifs vers `infrastructure/_shared/ansible/roles/`).
- **Runtime** : `docker compose` avec
  `infrastructure/_shared/docker-compose/docker-compose.base.yml` +
  `infrastructure/monosite/vps/production/docker-compose.override.yml`
  (Traefik + Let's Encrypt HTTP-01, limites mémoire/CPU par service).
- **Déploiement continu** : poller `infrastructure/_shared/scripts/gitops-deploy.sh`
  en unit systemd, watch la branche `production` (`inventory.ini` :
  `koprogo_branch=production`).

## 2. Pré-requis humains (Tier 1 — non faisables en session agent)

1. Compte OVH + credentials API (`ovh_endpoint`, clés OpenStack) exportés dans
   l'environnement Terraform.
2. État Terraform distant : bucket S3 `koprogo-tfstate`,
   clé `monosite/vps/production/terraform.tfstate`, région `eu-west-0`
   (`terraform/backend.tf`) — vérifier que le bucket existe et est verrouillé
   (lock DynamoDB/équivalent) avant le premier `apply`.
3. DNS : `API_DOMAIN=api.koprogo.be` et `FRONTEND_DOMAIN=app.koprogo.be`
   (`.env.example`) pointés en A/AAAA vers l'IP du VPS (sortie Terraform
   `vps_ip`) ; ports 80/443 ouverts.
4. `ACME_EMAIL` valide dans `.env` de prod (Let's Encrypt HTTP-01).
5. Secrets réels générés (jamais les valeurs `changeme-*` de `.env.example`) :
   `POSTGRES_PASSWORD`, `JWT_SECRET` (≥32 car.), `S3_ACCESS_KEY`/`S3_SECRET_KEY`.
6. Clé GPG de sauvegarde (`gpg_backup_email`, rôle `backup`) générée et sa
   clé privée exportée/sauvegardée **hors du VPS** (sinon restore impossible
   en cas de perte totale de la machine).
7. `ansible/inventory.ini` : remplacer `<VPS_IP>` par l'IP réelle post-`terraform apply`.

## 3. Provisioning initial (une fois)

```bash
# 1. Plan Terraform (agent peut fournir le plan, HUMAIN applique)
cd infrastructure/monosite/vps/production/terraform
terraform init
terraform plan -out=tfplan
# revue humaine du plan avant apply
terraform apply tfplan        # HUMAIN — deny agent, cf. .claude/settings.json

# 2. Provisioning Ansible (HUMAIN exécute)
cd ../ansible
ansible-playbook -i inventory.ini playbook.yml
```

Vérifications post-playbook (rôle par rôle) :

- `hardening` : LUKS disque de données, fail2ban actif, SSH par clé seule,
  kernel hardening (sysctl).
- `security` : Suricata/CrowdSec actifs (`systemctl status suricata crowdsec`).
- `docker` : `docker compose version` fonctionne, utilisateur `koprogo` dans
  le groupe `docker`.
- `monitoring` : stack `infrastructure/_shared/monitoring/docker-compose.monitoring.yml`
  up — Prometheus `:9090`, Grafana `:3001` (interne `3000`), Loki `:3100`,
  Alertmanager `:9093`, node-exporter `:9100`, postgres-exporter `:9187`,
  cadvisor `:8082`. **Ne pas exposer ces ports publiquement** — accès via
  tunnel SSH ou VPN seulement (non filtré par Traefik dans l'override actuel :
  à vérifier/fermer au firewall si besoin, cf. rôle `hardening`).
- `backup` : `gpg --list-keys "$GPG_BACKUP_EMAIL"` renvoie la clé ; cron
  `KoproGo Daily Encrypted Backup` à 02:00 installé (`crontab -l` en root).
- `gitops` : unit systemd du poller installée (nom exact = voir rôle
  `infrastructure/_shared/ansible/roles/gitops/`).
- `dns` : rôle applicatif DNS (vérifier son contenu si utilisé en Phase 1 —
  le go-live Phase 1 s'appuie sur DNS externe manuel, cf. §2.3).

## 4. Bring-up applicatif (poller GitOps)

Le poller tourne via `gitops-deploy.sh` (unit systemd installée par le rôle
`gitops`) :

```bash
# Variables lues par le script (cf. infrastructure/_shared/scripts/gitops-deploy.sh) :
#   TOPOLOGY=vps ENV_NAME=production BRANCH=production REPO_DIR=<clone VPS>
BRANCH=production ENV_NAME=production TOPOLOGY=vps \
  infrastructure/_shared/scripts/gitops-deploy.sh watch
```

- `watch` : boucle infinie, `git fetch` sur `BRANCH` toutes les
  `CHECK_INTERVAL` (défaut 180s), si nouveau SHA → `deploy`.
- `deploy` : tag image = `${BRANCH}-$(git rev-parse --short=7 HEAD)`
  (ex. `production-a1b2c3d`), `docker compose pull` avec retry (10× / 90s si
  `manifest unknown` — laisse le temps à `docker-build-push.yml` de publier
  l'image), fallback `${BRANCH}-latest` si le tag exact n'apparaît jamais,
  puis `docker compose up -d`.
- Logs : `/var/log/koprogo-gitops-production.log` (topologie `vps` — chemin
  différent en topologie `local`, cf. script).
- Commandes manuelles : `gitops-deploy.sh status` (état des containers +
  SHA courant) ; `gitops-deploy.sh deploy` (déploiement one-shot, sans boucle) ;
  `gitops-deploy.sh logs` (tail du log).

## 5. Déploiement standard (jour-à-jour)

Le poller gère le déploiement automatiquement dès qu'un commit atterrit sur
la branche `production` (promotion `feature/dev` → `dev` → `production`, cf.
workflows `.github/workflows/promote-*.yml`). Aucune action manuelle requise
en fonctionnement normal — surveiller `gitops-deploy.sh status` ou le log.

## 6. Rollback

Le poller n'a pas de commande `rollback` dédiée : le rollback se fait en
inversant le commit sur la branche `production`, ce qui redéclenche un
déploiement au prochain cycle du poller (ou immédiatement via `deploy`
manuel) :

```bash
# Sur le poste HUMAIN (jamais push --force — cf. lignes rouges CLAUDE.md) :
git checkout production
git revert <sha-du-commit-cassé>       # merge commit, ne réécrit pas l'historique
git push origin production

# Sur le VPS (immédiat, sans attendre le prochain cycle watch) :
BRANCH=production ENV_NAME=production TOPOLOGY=vps \
  infrastructure/_shared/scripts/gitops-deploy.sh deploy
```

Le tag d'image redevient `production-<sha-revert>` ; comme `docker compose
pull` retente sur `manifest unknown`, s'assurer que `docker-build-push.yml`
a bien reconstruit l'image pour ce nouveau SHA avant de forcer un `deploy`
manuel (sinon le script bascule sur `production-latest` après 10 tentatives
= ~15 min).

**Drill rollback (à faire une fois avant tag v0.1.0, DoD F3/F4)** : déployer
un commit volontairement cassé (ex. sur une branche de test dédiée, jamais
`production`), constater l'échec, `revert`, re-déployer, confirmer le retour
au comportement sain. Documenter le résultat (durée, points de friction) dans
un commentaire d'issue ou un log `docs/agent-activity/`.

## 7. Sauvegarde & restauration (GPG + S3)

Script : `infrastructure/_shared/ansible/roles/backup/templates/backup-encrypted.sh.j2`
(déployé sur le VPS en `${koprogo_dir}/scripts/backup-encrypted.sh`, cron
quotidien 02:00).

Contenu d'une sauvegarde :

- `koprogo_<DATE>.sql.gz.gpg` — dump PostgreSQL complet, gzip puis chiffré GPG.
- `minio-metadata_<DATE>.json.gz.gpg` — métadonnées MinIO (liste objets),
  chiffré GPG (best-effort — skip si `mc` absent ou credentials manquants).
- `.env_<DATE>.gpg` — sauvegarde chiffrée du `.env` de prod.
- Sync S3 (`s3cmd`, best-effort — skip proprement si non configuré) vers
  `s3://${s3_backup_bucket}/postgres/` et `.../minio/`.
- Rétention locale : 7 jours (`cleanup_old_backups`).
- **Auto-test intégré** : le script déchiffre + vérifie le dernier backup
  (`gunzip -t`) à chaque exécution et `exit 1` si corrompu — surveiller
  `/var/log/koprogo-backup.log` (cron) pour détecter un échec silencieux.

Restauration PostgreSQL :

```bash
gpg --decrypt koprogo_YYYYMMDD_HHMMSS.sql.gz.gpg | gunzip | \
  docker exec -i koprogo-postgres psql -U koprogo koprogo_db
```

Restauration depuis S3 (si le VPS est reconstruit à neuf) :

```bash
s3cmd get s3://<s3_backup_bucket>/postgres/koprogo_<DATE>.sql.gz.gpg .
# puis la commande de restauration ci-dessus
```

**Pré-requis restore** : la clé privée GPG (`gpg_backup_email`) doit être
importée sur la machine qui restaure — **elle n'est jamais recréée
automatiquement** par le rôle `backup` (qui ne fait que `gpg --gen-key` si
aucune clé n'existe déjà, ce qui génère une **nouvelle** clé incapable de
déchiffrer d'anciennes sauvegardes). Conserver la clé privée hors du VPS
(coffre-fort secrets, HSM, ou au minimum un stockage séparé chiffré).

**Drill restore (à faire une fois avant tag v0.1.0, DoD F3/F4)** : sur un
environnement de test (jamais la prod), restaurer le dernier backup chiffré
et vérifier l'intégrité des données restaurées (comptage de lignes clé,
cohérence comptable Decimal). Documenter RTO/RPO observés.

## 8. Vérification TLS (WP-F2)

Aucun nouveau pipeline à créer — Traefik + Let's Encrypt HTTP-01 sont déjà
câblés dans `docker-compose.override.yml` (`certificatesresolvers.letsencrypt`,
challenge HTTP sur l'entrypoint `web`, redirection http→https automatique).
Après DNS + ports 80/443 ouverts + `ACME_EMAIL` renseigné :

```bash
curl -I https://api.koprogo.be/api/v1/health   # 200, cert valide
curl -I http://api.koprogo.be                   # 301/308 → https
```

## 9. Endpoints de santé & logs

- Backend : `GET /api/v1/health` (vérifier le handler exact dans
  `backend/src/infrastructure/web/handlers/` si le chemin diffère).
- Logs applicatifs : `docker compose -f docker-compose.base.yml -f
docker-compose.override.yml logs -f backend` (ou `frontend`).
- Logs GitOps : `/var/log/koprogo-gitops-production.log`.
- Logs backup : `/var/log/koprogo-backup.log`.
- Observabilité : Grafana `:3001` (dashboards node/postgres/cadvisor via les
  exporters listés en §3) — accès à restreindre (tunnel SSH/VPN), jamais
  exposé publiquement sans authentification en Phase 1.

## 10. Références

- WBS complet & DoD go-live : `docs/WBS_GO_LIVE_v0.1.0.md` (Track F/G).
- Sécurité runtime : `infrastructure/SECURITY.md`.
- Script GitOps : `infrastructure/_shared/scripts/gitops-deploy.sh`.
- Rôles Ansible : `infrastructure/_shared/ansible/roles/{hardening,security,monitoring,backup,gitops}/`.
- Phase 2 (k3s/ArgoCD, SOPS/age, Vault, Velero) : hors périmètre de ce
  runbook — cf. WBS §"Phase 2 (post-v0.1.0)".
