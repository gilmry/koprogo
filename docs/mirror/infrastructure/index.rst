======================
Infrastructure - Index
======================


L'infrastructure KoproGo couvre le **déploiement**, **monitoring** et **sécurité**.

**Technologies** :
- **Ansible** : Automatisation déploiement VPS
- **Docker** : Conteneurisation (PostgreSQL, backend, frontend)
- **Prometheus + Grafana** : Monitoring métriques
- **Loki** : Agrégation logs
- **Suricata** : IDS (détection intrusions)
- **CrowdSec** : WAF collaboratif

**Fonctionnalités** :
- ✅ LUKS encryption at rest (AES-XTS-512)
- ✅ GPG encrypted backups (S3 off-site)
- ✅ fail2ban + SSH hardening
- ✅ Kernel hardening (sysctl)
- ✅ Security auditing (Lynis, rkhunter, AIDE)


Contenu
=======

.. toctree::
   :maxdepth: 2

   src/index

