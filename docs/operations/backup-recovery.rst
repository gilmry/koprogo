========================
Backup & Recovery Guide
========================

:Version: 1.0.0
:Date: 10 novembre 2025

📦 Stratégie de Sauvegarde
============================

**Composants sauvegardés** :

1. PostgreSQL (dumps quotidiens + WAL)
2. Uploads (/opt/koprogo/uploads)
3. Configuration (.env, secrets)
4. Monitoring (métriques 30d, logs 7d)

**Fréquence** :

- **Quotidien** : 2h du matin (faible charge)
- **Hebdomadaire** : Dimanche 3h (backup complet)
- **Mensuel** : 1er du mois 4h

**Rétention** :

- Local : 7 jours
- S3 : 90 jours → Glacier (12 mois)

🛠️ Scripts de Backup
======================

PostgreSQL
----------

.. code-block:: bash

   #!/bin/bash
   # /opt/koprogo/scripts/backup-postgres.sh

   DATE=$(date +%Y%m%d_%H%M%S)
   BACKUP_DIR="/var/backups/postgresql"
   S3_BUCKET="s3://koprogo-backups-prod/postgresql/"

   # Dump database
   sudo -u postgres pg_dump koprogo_db > "$BACKUP_DIR/koprogo_$DATE.sql"

   # Compress & encrypt
   gpg --encrypt --recipient "backup@koprogo.com" \
       "$BACKUP_DIR/koprogo_$DATE.sql"

   # Upload to S3
   aws s3 cp "$BACKUP_DIR/koprogo_$DATE.sql.gpg" "$S3_BUCKET"

   # Cleanup local (keep 7 days)
   find "$BACKUP_DIR" -name "*.sql.gpg" -mtime +7 -delete

Uploads
-------

.. code-block:: bash

   #!/bin/bash
   # /opt/koprogo/scripts/backup-uploads.sh

   DATE=$(date +%Y%m%d_%H%M%S)
   tar czf "/var/backups/uploads/uploads_$DATE.tar.gz" /opt/koprogo/uploads
   gpg --encrypt --recipient "backup@koprogo.com" "/var/backups/uploads/uploads_$DATE.tar.gz"
   aws s3 cp "/var/backups/uploads/uploads_$DATE.tar.gz.gpg" "s3://koprogo-backups-prod/uploads/"

🔄 Procédures de Restauration
===============================

Restaurer PostgreSQL
---------------------

.. code-block:: bash

   # 1. Lister backups disponibles
   aws s3 ls s3://koprogo-backups-prod/postgresql/

   # 2. Télécharger backup
   aws s3 cp s3://koprogo-backups-prod/postgresql/koprogo_20251110_020000.sql.gpg /tmp/

   # 3. Déchiffrer
   gpg --decrypt /tmp/koprogo_20251110_020000.sql.gpg > /tmp/restore.sql

   # 4. Restaurer
   sudo systemctl stop koprogo-api
   sudo -u postgres psql koprogo_db < /tmp/restore.sql
   sudo systemctl start koprogo-api

Restaurer Uploads
-----------------

.. code-block:: bash

   aws s3 cp s3://koprogo-backups-prod/uploads/latest.tar.gz.gpg /tmp/
   gpg --decrypt /tmp/latest.tar.gz.gpg | tar xz -C /opt/koprogo/uploads

📋 Vérification
================

Tests mensuels obligatoires
----------------------------

.. code-block:: bash

   # Test déchiffrement
   gpg --decrypt /var/backups/postgresql/latest.sql.gpg > /dev/null

   # Test restauration (DB temporaire)
   sudo -u postgres createdb koprogo_test
   gpg --decrypt /var/backups/postgresql/latest.sql.gpg | sudo -u postgres psql koprogo_test
   sudo -u postgres psql koprogo_test -c "SELECT COUNT(*) FROM buildings;"
   sudo -u postgres dropdb koprogo_test

---

**Version** : 1.0.0
