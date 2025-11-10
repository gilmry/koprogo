===================================
Monitoring & Observability Runbook
===================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Stack: Prometheus + Grafana + Loki + Alertmanager

📊 Stack de Monitoring
=======================

**Composants** :

- **Prometheus** (http://vps-ip:9090) : Métriques time-series (30 jours)
- **Grafana** (http://vps-ip:3001) : Dashboards et visualisation
- **Loki** : Agrégation de logs (7 jours)
- **Alertmanager** (http://vps-ip:9093) : Gestion des alertes
- **Node Exporter** : Métriques système (CPU, RAM, disk, network)
- **PostgreSQL Exporter** : Métriques database
- **Backend `/metrics`** : Métriques applicatives (Actix-web)

🔍 Métriques Clés
==================

API Backend
-----------

**Métriques exposées** : ``http://localhost:8080/metrics``

.. code-block:: text

   # Requêtes HTTP
   http_requests_total{method="GET",path="/api/v1/buildings",status="200"} 1234
   http_request_duration_seconds_bucket{le="0.005"} 980
   http_request_duration_seconds_bucket{le="0.01"} 1150

   # Erreurs
   http_requests_errors_total{status="500"} 3

   # Database
   db_connections_active 8
   db_query_duration_seconds_sum 45.2

Système (Node Exporter)
------------------------

.. code-block:: promql

   # CPU
   100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)

   # RAM
   (node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) / node_memory_MemTotal_bytes * 100

   # Disk
   (node_filesystem_size_bytes - node_filesystem_avail_bytes) / node_filesystem_size_bytes * 100

   # Network
   rate(node_network_receive_bytes_total[5m])
   rate(node_network_transmit_bytes_total[5m])

PostgreSQL
----------

.. code-block:: promql

   # Connexions actives
   pg_stat_activity_count

   # Taille DB
   pg_database_size_bytes{datname="koprogo_db"}

   # Transactions
   rate(pg_stat_database_xact_commit{datname="koprogo_db"}[5m])
   rate(pg_stat_database_xact_rollback{datname="koprogo_db"}[5m])

🚨 Alertes Critiques
=====================

Alertes configurées (Alertmanager)
-----------------------------------

**Haute priorité** (PagerDuty + Slack + SMS) :

1. **API Down** : ``up{job="koprogo-api"} == 0`` pendant 1 minute
2. **Database Down** : ``up{job="postgresql"} == 0`` pendant 30 secondes
3. **Disk Full** : ``node_filesystem_avail_bytes / node_filesystem_size_bytes < 0.1`` (< 10%)
4. **High Error Rate** : ``rate(http_requests_errors_total[5m]) > 10`` (> 10 erreurs/s)
5. **Memory Exhaustion** : ``node_memory_MemAvailable_bytes < 104857600`` (< 100MB disponible)

**Priorité moyenne** (Slack uniquement) :

6. **Slow API** : ``histogram_quantile(0.99, http_request_duration_seconds) > 1`` (P99 > 1s)
7. **High CPU** : ``avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) < 0.2`` (> 80% usage)
8. **Database Connections High** : ``pg_stat_activity_count > 80`` (> 80 connexions)

**Procédure d'alerte** :

1. **Notification reçue** (Slack/SMS)
2. **Accéder à Grafana** pour visualiser
3. **Consulter logs Loki** pour contexte
4. **Appliquer runbook** correspondant (voir ci-dessous)
5. **Documenter résolution** dans #incidents
6. **Post-mortem** si critique

📈 Dashboards Grafana
=======================

Dashboards essentiels
----------------------

**1. API Overview** :

- Requêtes/seconde par endpoint
- Latence P50/P95/P99
- Taux d'erreurs (2xx, 4xx, 5xx)
- Top endpoints les plus lents

**2. System Health** :

- CPU, RAM, Disk, Network
- Nombre de processus
- Load average (1m, 5m, 15m)
- Disk I/O

**3. PostgreSQL Performance** :

- Connexions actives/idle
- Cache hit ratio (devrait être > 90%)
- Taille DB + croissance
- Slow queries (> 1s)

**4. Business Metrics** :

- Nouveaux utilisateurs/jour
- Dépenses créées/validées
- Taux de paiement à temps
- Utilisateurs actifs (DAU/MAU)

🔧 Runbooks par Alerte
========================

API Down
--------

**Alerte** : ``up{job="koprogo-api"} == 0``

.. code-block:: bash

   # 1. Vérifier si le service tourne
   ssh root@prod-server "systemctl status koprogo-api"

   # 2. Consulter logs récents
   ssh root@prod-server "journalctl -u koprogo-api -n 100 --no-pager"

   # 3. Redémarrer si crash
   ssh root@prod-server "systemctl restart koprogo-api"

   # 4. Vérifier health endpoint
   curl http://prod-server:8080/health

   # 5. Si échec persistant : disaster recovery
   # Voir docs/operations/disaster-recovery.rst

**Causes fréquentes** :

- Panic Rust (bug application)
- Database unreachable
- OOM killer (mémoire insuffisante)

Database Down
-------------

**Alerte** : ``up{job="postgresql"} == 0``

.. code-block:: bash

   # 1. Vérifier statut PostgreSQL
   ssh root@prod-server "systemctl status postgresql"

   # 2. Consulter logs
   ssh root@prod-server "tail -n 100 /var/log/postgresql/postgresql-15-main.log"

   # 3. Tenter redémarrage
   ssh root@prod-server "systemctl restart postgresql"

   # 4. Si corruption : disaster recovery
   # Voir docs/operations/disaster-recovery.rst (Scénario 1)

Disk Full
---------

**Alerte** : ``node_filesystem_avail_bytes / node_filesystem_size_bytes < 0.1``

.. code-block:: bash

   # 1. Identifier ce qui consomme l'espace
   ssh root@prod-server "du -sh /* | sort -h | tail -n 20"

   # 2. Nettoyer logs anciens
   ssh root@prod-server "find /var/log -name '*.log' -mtime +30 -delete"
   ssh root@prod-server "journalctl --vacuum-time=7d"

   # 3. Nettoyer Docker
   ssh root@prod-server "docker system prune -af --volumes"

   # 4. Nettoyer backups locaux
   ssh root@prod-server "find /var/backups -name '*.gpg' -mtime +7 -delete"

   # 5. Si insuffisant : étendre volume ou migrer vers serveur plus grand

Slow API (P99 > 1s)
--------------------

.. code-block:: bash

   # 1. Identifier endpoints lents (Grafana)
   # Dashboard "API Overview" → Panel "Slowest Endpoints"

   # 2. Consulter logs applicatifs
   ssh root@prod-server "grep 'duration_ms' /var/log/koprogo/api.log | awk '{if ($5 > 1000) print}'"

   # 3. Vérifier slow queries PostgreSQL
   ssh root@prod-server "sudo -u postgres psql koprogo_db -c \"SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;\""

   # 4. Ajouter index si requêtes non optimisées
   # Créer migration SQLx

   # 5. Si charge trop élevée : scale horizontalement
   # Ajouter instance load-balanced

📝 Logs (Loki)
===============

Requêtes LogQL utiles
---------------------

**Erreurs backend** :

.. code-block:: logql

   {job="koprogo-api"} |= "ERROR" | json

**Requêtes lentes** :

.. code-block:: logql

   {job="koprogo-api"} | json | duration_ms > 1000

**Échecs d'authentification** :

.. code-block:: logql

   {job="koprogo-api"} |= "401" | json | path="/api/v1/auth/login"

**Database errors** :

.. code-block:: logql

   {job="postgresql"} |= "ERROR" | logfmt

🔐 Sécurité Monitoring
========================

Intrusion Detection (Suricata)
-------------------------------

**Logs** : ``/var/log/suricata/eve.json``

**Alertes** :

- SQL Injection attempts
- XSS attempts
- Path traversal
- Port scanning

**Dashboard Grafana** : "Security Events"

fail2ban
--------

**Jails actifs** :

- SSH brute-force (10 tentatives → ban 10 minutes)
- API abuse (100 req/min → ban 1 heure)
- PostgreSQL brute-force

**Vérifier bans** :

.. code-block:: bash

   sudo fail2ban-client status sshd
   sudo fail2ban-client status api-abuse

CrowdSec
--------

**Community threat intelligence** :

.. code-block:: bash

   # Lister décisions actives
   sudo cscli decisions list

   # Métriques
   sudo cscli metrics

📞 Contacts Escalation
=======================

**Niveaux** :

1. **L1 - Monitoring automatique** : Alertmanager → Slack #alerts
2. **L2 - Astreinte SRE** : PagerDuty → +32 XXX XX XX XX
3. **L3 - Lead Engineer** : +32 XXX XX XX XX (incidents critiques uniquement)
4. **L4 - CTO** : +32 XXX XX XX XX (disaster recovery, incidents de sécurité)

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025
