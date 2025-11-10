===================================
Disaster Recovery Plan
===================================

:Version: 1.0.0
:Date: 10 novembre 2025
:Public: SRE, DevOps, SysAdmin
:RTO: 4 heures (Recovery Time Objective)
:RPO: 24 heures (Recovery Point Objective)

📋 Vue d'ensemble
=================

Ce runbook décrit les procédures de récupération en cas de **défaillance critique** des services KoproGo.

**Scénarios couverts** :

1. ✅ Corruption de base de données PostgreSQL
2. ✅ Perte de serveur OVH (panne matérielle, incendie datacenter)
3. ✅ Corruption du filesystem (données applicatives)
4. ✅ Attaque ransomware / intrusion malveillante
5. ✅ Suppression accidentelle de données

🎯 Objectifs de Récupération
==============================

.. list-table::
   :header-rows: 1
   :widths: 30 20 20 30

   * - Service
     - RTO
     - RPO
     - Priorité
   * - API Backend
     - 2h
     - 24h
     - Critique
   * - Base de données
     - 4h
     - 24h
     - Critique
   * - Frontend (statique)
     - 1h
     - 0h (CDN)
     - Haute
   * - Monitoring
     - 8h
     - 7d
     - Moyenne

📊 Architecture de Sauvegarde
===============================

**Composants sauvegardés** :

1. **PostgreSQL** : Dumps quotidiens + WAL archiving
2. **Uploads** : Fichiers utilisateurs (documents, factures)
3. **Configuration** : Variables d'environnement, secrets
4. **Monitoring** : Métriques Prometheus (30 jours), logs Loki (7 jours)

**Stratégie 3-2-1** :

- **3** copies : Production + Backup local + Backup S3
- **2** médias : Disque local (LUKS chiffré) + Cloud (S3 chiffré)
- **1** copie hors site : OVH Object Storage (Gravelines) ou AWS S3

**Rétention** :

- Backups quotidiens : **7 jours** (local)
- Backups hebdomadaires : **4 semaines** (S3)
- Backups mensuels : **12 mois** (S3, Glacier)

📦 Backups Existants
=====================

Emplacement des sauvegardes
----------------------------

**Local (VPS)** :

- **PostgreSQL dumps** : ``/var/backups/postgresql/``
- **Uploads** : ``/var/backups/uploads/``
- **Config** : ``/var/backups/config/``

**S3 (Off-site)** :

- **Bucket** : ``koprogo-backups-prod``
- **Région** : ``eu-west-3`` (Paris) ou ``eu-north-1`` (Stockholm)
- **Chiffrement** : GPG avec clé publique/privée (4096 bits RSA)
- **Lifecycle** : Transition vers Glacier après 90 jours

Vérification des backups
-------------------------

**Commande quotidienne** (cron) :

.. code-block:: bash

   #!/bin/bash
   # /opt/koprogo/scripts/verify-backups.sh

   # Vérifier présence backup PostgreSQL < 25h
   LATEST_PG_BACKUP=$(find /var/backups/postgresql -name "*.sql.gpg" -mtime -1 | wc -l)
   if [ "$LATEST_PG_BACKUP" -eq 0 ]; then
       echo "❌ CRITICAL: No PostgreSQL backup in last 24h"
       exit 1
   fi

   # Vérifier backup S3
   aws s3 ls s3://koprogo-backups-prod/postgresql/ --recursive | tail -n 5

   # Tester déchiffrement (sans restaurer)
   gpg --decrypt /var/backups/postgresql/latest.sql.gpg > /dev/null 2>&1
   if [ $? -eq 0 ]; then
       echo "✅ Backup decryption test successful"
   else
       echo "❌ CRITICAL: Backup decryption failed"
       exit 1
   fi

**Alertes** : Slack/Email via Alertmanager si échec

🚨 Scénarios de Récupération
==============================

Scénario 1 : Corruption PostgreSQL
-----------------------------------

**Symptômes** :

- Erreurs "could not read block" dans les logs
- Requêtes SQL qui échouent avec "invalid page header"
- `pg_dump` échoue avec erreurs de corruption

**Procédure de récupération** :

.. code-block:: bash

   # 1. Arrêter PostgreSQL immédiatement
   sudo systemctl stop postgresql

   # 2. Sauvegarder l'état actuel (forensic)
   sudo tar czf /tmp/postgres-corrupted-$(date +%Y%m%d-%H%M%S).tar.gz /var/lib/postgresql/15/main

   # 3. Identifier le backup le plus récent
   LATEST_BACKUP=$(ls -t /var/backups/postgresql/*.sql.gpg | head -n 1)
   echo "Latest backup: $LATEST_BACKUP"

   # 4. Déchiffrer le backup
   gpg --decrypt $LATEST_BACKUP > /tmp/koprogo_restore.sql

   # 5. Recréer la base de données
   sudo -u postgres dropdb koprogo_db
   sudo -u postgres createdb koprogo_db

   # 6. Restaurer le dump
   sudo -u postgres psql koprogo_db < /tmp/koprogo_restore.sql

   # 7. Vérifier intégrité
   sudo -u postgres psql koprogo_db -c "SELECT COUNT(*) FROM buildings;"

   # 8. Redémarrer PostgreSQL
   sudo systemctl start postgresql

   # 9. Tester l'API
   curl http://localhost:8080/health

**Temps estimé** : 2-4 heures (selon taille DB)

**Perte de données** : Jusqu'à 24h (RPO)

Scénario 2 : Perte Totale du Serveur
-------------------------------------

**Symptômes** :

- Serveur inaccessible (ping fail, SSH timeout)
- Datacenter OVH signale panne matérielle
- Aucune réponse API/Frontend

**Procédure de récupération** :

.. code-block:: bash

   # 1. Provisionner nouveau serveur OVH (Terraform)
   cd infrastructure/terraform
   terraform apply -var="server_name=koprogo-prod-new"

   # 2. Installer stack de base (Ansible)
   cd ../ansible
   ansible-playbook -i inventory.ini playbook.yml

   # 3. Récupérer backups depuis S3
   aws s3 sync s3://koprogo-backups-prod/postgresql/ /var/backups/postgresql/
   aws s3 sync s3://koprogo-backups-prod/uploads/ /var/backups/uploads/

   # 4. Restaurer PostgreSQL (voir Scénario 1)
   # ...

   # 5. Restaurer uploads
   sudo mkdir -p /opt/koprogo/uploads
   sudo tar xzf /var/backups/uploads/latest.tar.gz -C /opt/koprogo/uploads

   # 6. Restaurer configuration
   sudo cp /var/backups/config/.env /opt/koprogo/backend/.env

   # 7. Déployer application (Docker Compose)
   cd /opt/koprogo
   docker-compose up -d

   # 8. Mettre à jour DNS (A record vers nouvelle IP)
   # Via interface OVH ou Cloudflare

   # 9. Vérifier services
   curl https://api.koprogo.com/health
   curl https://app.koprogo.com

**Temps estimé** : 4-6 heures

**Perte de données** : Jusqu'à 24h (RPO)

Scénario 3 : Suppression Accidentelle de Données
-------------------------------------------------

**Symptômes** :

- Utilisateur signale : "Mes données ont disparu"
- Logs audit montrent ``DELETE`` inattendu
- Vérification manuelle confirme absence de données

**Procédure de récupération** :

.. code-block:: bash

   # 1. Identifier l'heure de suppression (audit logs)
   sudo -u postgres psql koprogo_db -c \
       "SELECT * FROM audit_logs WHERE action='DELETE' AND entity_type='Building' ORDER BY timestamp DESC LIMIT 10;"

   # 2. Choisir le backup AVANT la suppression
   # Exemple : suppression à 14h30, utiliser backup de 02h00 (même jour)
   BACKUP_FILE="/var/backups/postgresql/koprogo_20251110_020000.sql.gpg"

   # 3. Créer DB temporaire pour extraction
   sudo -u postgres createdb koprogo_temp

   # 4. Restaurer dans DB temp
   gpg --decrypt $BACKUP_FILE | sudo -u postgres psql koprogo_temp

   # 5. Extraire les données perdues
   sudo -u postgres psql koprogo_temp -c \
       "COPY (SELECT * FROM buildings WHERE id='<building-uuid>') TO '/tmp/lost_building.csv' CSV HEADER;"

   # 6. Réinsérer dans DB production
   sudo -u postgres psql koprogo_db -c \
       "INSERT INTO buildings SELECT * FROM temp_buildings_import ON CONFLICT DO NOTHING;"

   # 7. Vérifier avec l'utilisateur
   curl -H "Authorization: Bearer <token>" https://api.koprogo.com/api/v1/buildings/<building-uuid>

   # 8. Nettoyer DB temporaire
   sudo -u postgres dropdb koprogo_temp

**Temps estimé** : 30 minutes - 2 heures

**Perte de données** : Minimale (données entre backup et suppression)

Scénario 4 : Attaque Ransomware
--------------------------------

**Symptômes** :

- Fichiers chiffrés avec extension ``.locked`` ou ``.encrypted``
- Note de rançon dans ``/tmp/README_DECRYPT.txt``
- PostgreSQL inaccessible
- Monitoring/alertes saturés

**Procédure de récupération** :

.. code-block:: bash

   # 1. ISOLATION IMMÉDIATE
   sudo iptables -A INPUT -j DROP    # Bloquer toute entrée
   sudo iptables -A OUTPUT -j DROP   # Bloquer toute sortie
   # Exception : SSH depuis IP admin uniquement

   # 2. ANALYSE FORENSIC
   sudo find / -name "*.locked" -o -name "*.encrypted" | head -n 20
   sudo journalctl --since "1 hour ago" | grep -i "encrypted\|ransom\|locked"

   # 3. NE PAS PAYER LA RANÇON

   # 4. Récupération depuis backups S3 (hors ligne, non compromis)
   # Depuis une machine PROPRE (pas le serveur infecté)
   aws s3 sync s3://koprogo-backups-prod/ /mnt/recovery/

   # 5. Provisionner NOUVEAU serveur (ne PAS réutiliser l'infecté)
   # Voir Scénario 2 (Perte totale serveur)

   # 6. Enquête de sécurité
   # - Identifier vecteur d'attaque (logs Suricata, fail2ban)
   # - Patcher vulnérabilités
   # - Changer TOUS les mots de passe et secrets
   # - Révoquer toutes les clés SSH

   # 7. Notifier autorités et utilisateurs (GDPR)
   # - CNIL (72h max)
   # - Email tous les utilisateurs

**Temps estimé** : 8-24 heures (forensics + récupération)

**Perte de données** : Jusqu'à 24h (RPO)

🧪 Tests de Récupération
=========================

Tests trimestriels obligatoires
--------------------------------

**Q1, Q2, Q3, Q4** : Simuler un scénario de DR complet

**Procédure de test** :

.. code-block:: bash

   # 1. Créer environnement de test (isolé de prod)
   cd infrastructure/terraform
   terraform apply -var="environment=dr-test"

   # 2. Récupérer backup S3 le plus récent
   aws s3 cp s3://koprogo-backups-prod/postgresql/latest.sql.gpg /tmp/

   # 3. Restaurer dans environnement de test
   gpg --decrypt /tmp/latest.sql.gpg | psql <test-db-url>

   # 4. Déployer application
   docker-compose -f docker-compose.test.yml up -d

   # 5. Vérifier fonctionnalités critiques
   curl http://test-server/health
   curl http://test-server/api/v1/buildings

   # 6. Mesurer temps de récupération
   # Objectif : < 4h RTO

   # 7. Documenter résultats
   echo "DR Test $(date): RTO actual = 3h 15min ✅" >> /docs/dr-test-log.txt

   # 8. Détruire environnement de test
   terraform destroy -var="environment=dr-test"

**Checklist de validation** :

- [ ] Backup déchiffré avec succès
- [ ] Base de données restaurée complètement
- [ ] API répond à ``/health``
- [ ] Authentification fonctionnelle (login)
- [ ] Au moins 1 requête CRUD réussie par endpoint principal
- [ ] RTO < 4h respecté
- [ ] Aucune corruption de données détectée

📞 Contacts d'Urgence
======================

**Équipe technique** :

- **SRE Lead** : +32 XXX XX XX XX (astreinte 24/7)
- **DBA** : +32 XXX XX XX XX
- **Security Lead** : +32 XXX XX XX XX

**Fournisseurs** :

- **OVH Support** : +33 9 72 10 10 07 (24/7)
- **AWS Support** : https://console.aws.amazon.com/support (Enterprise plan)

**Légal** :

- **Avocat** : +32 XXX XX XX XX
- **CNIL (GDPR)** : https://www.cnil.fr/

**Communication** :

- **Slack #incident** : https://koprogo.slack.com/archives/incident
- **Email incidents** : incidents@koprogo.com
- **Status page** : https://status.koprogo.com

🔒 Sécurité des Clés GPG
==========================

Gestion des clés de chiffrement
--------------------------------

**Clé publique** (chiffrement backups) :

- Stockée sur serveur : ``/root/.gnupg/pubring.kbx``
- Backup copie publique dans repo Git (sécurisé)

**Clé privée** (déchiffrement backups) :

- **NE PAS stocker sur serveur de production** (risque ransomware)
- Stockée hors ligne :
  - Yubikey (SRE Lead)
  - Coffre-fort physique (siège ASBL)
  - Password manager équipe (Bitwarden/1Password)

**Rotation des clés** : Annuelle (janvier)

.. code-block:: bash

   # Générer nouvelle paire de clés
   gpg --full-generate-key --expert

   # Exporter clé publique
   gpg --export -a "KoproGo Backup <backup@koprogo.com>" > koprogo-backup-2026.asc

   # Mettre à jour sur serveur
   scp koprogo-backup-2026.asc root@prod-server:/root/.gnupg/
   ssh root@prod-server "gpg --import /root/.gnupg/koprogo-backup-2026.asc"

   # Tester chiffrement
   echo "test" | gpg --encrypt --recipient "backup@koprogo.com" | gpg --decrypt

📋 Checklist Post-Incident
============================

Après chaque récupération
--------------------------

- [ ] **Documenter incident** : Date, cause, procédure suivie, temps réel
- [ ] **Post-mortem meeting** : Équipe technique + stakeholders
- [ ] **Identifier cause racine** : 5 Whys analysis
- [ ] **Mettre à jour runbook** : Améliorer procédures
- [ ] **Notifier utilisateurs** : Email transparent + compensation si applicable
- [ ] **Améliorer monitoring** : Ajouter alertes préventives
- [ ] **Tester à nouveau** : Re-tester procédure améliorée dans 30 jours

---

**Version** : 1.0.0 | **Dernière mise à jour** : 10 novembre 2025 | **Prochain test DR** : Janvier 2026
