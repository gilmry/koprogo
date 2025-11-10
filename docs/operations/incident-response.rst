========================
Incident Response Plan
========================

:Version: 1.0.0
:Date: 10 novembre 2025

🚨 Niveaux de Sévérité
========================

.. list-table::
   :header-rows: 1
   :widths: 10 30 30 30

   * - Niveau
     - Impact
     - Exemples
     - Response Time
   * - **P0 (Critical)**
     - Service down pour tous
     - API down, DB corruption, security breach
     - < 15 minutes
   * - **P1 (High)**
     - Fonctionnalité majeure down
     - Login fail, paiements bloqués
     - < 1 heure
   * - **P2 (Medium)**
     - Dégradation performance
     - Latence élevée, bugs non-bloquants
     - < 4 heures
   * - **P3 (Low)**
     - Problème mineur
     - Typo, UI glitch
     - < 24 heures

📞 Procédure de Réponse
========================

Phase 1 : Détection (0-5 min)
------------------------------

1. **Alerte reçue** (Alertmanager, user report, monitoring)
2. **Acknowledge** dans Slack #incidents
3. **Évaluer sévérité** (P0-P3)
4. **Créer incident ticket** (GitHub Issues avec label `incident`)

Phase 2 : Triage (5-15 min)
----------------------------

1. **Vérifier impact** :
   - Combien d'utilisateurs affectés ?
   - Quelles fonctionnalités down ?
   - Risque sécurité/données ?

2. **Escalader** si nécessaire :
   - P0 → Notifier Lead Engineer + CTO
   - P1 → Notifier astreinte SRE
   - P2/P3 → Équipe dev normale

3. **Communication** :
   - P0/P1 : Mettre à jour https://status.koprogo.com
   - Email utilisateurs si prolongé (> 1h)

Phase 3 : Résolution (variable)
--------------------------------

1. **Diagnostiquer** :
   - Consulter logs (Loki)
   - Vérifier métriques (Grafana)
   - Reproduire si possible

2. **Mitigation immédiate** :
   - Rollback si déploiement récent
   - Restart service si applicable
   - Activer mode dégradé si possible

3. **Fix permanent** :
   - Déployer hotfix
   - Tester en staging first
   - Déployer en production

Phase 4 : Post-Incident (< 48h)
--------------------------------

1. **Post-mortem meeting** (obligatoire pour P0/P1)
2. **Documentation** :
   - Timeline des événements
   - Root cause analysis (5 Whys)
   - Action items pour prévenir récurrence
3. **Mettre à jour runbooks**

🔐 Incidents de Sécurité
==========================

**Procédure spéciale** :

1. **Isolation** : Bloquer trafic si nécessaire
2. **Forensics** : Préserver logs, snapshots
3. **Notification GDPR** : CNIL sous 72h si breach
4. **Communication** : Transparent avec utilisateurs
5. **Investigation** : Root cause + patch vulnérabilités

📋 Templates
=============

Incident Report
---------------

.. code-block:: markdown

   # Incident Report - [YYYY-MM-DD]

   **Severity**: P0/P1/P2/P3
   **Duration**: HH:MM start → HH:MM resolved
   **Impact**: [Number of users/services affected]

   ## Timeline
   - 14:30 - Alert fired: API P99 > 5s
   - 14:32 - Incident acknowledged
   - 14:35 - Root cause identified: DB connection pool exhausted
   - 14:40 - Mitigation: Restarted API service
   - 14:45 - Resolved: Latency back to normal

   ## Root Cause
   Database connection pool size (10) insufficient under peak load.

   ## Action Items
   - [ ] Increase connection pool to 20
   - [ ] Add alert for connection pool usage > 80%
   - [ ] Load test with realistic traffic

Status Page Update
------------------

.. code-block:: text

   🔴 **Investigating** - We are currently investigating issues with login functionality.
   Posted: 2025-11-10 14:30 UTC

   🟡 **Identified** - The issue has been identified as a database connection problem.
   Posted: 2025-11-10 14:35 UTC

   🟢 **Resolved** - The login functionality has been restored.
   Posted: 2025-11-10 14:45 UTC

---

**Version** : 1.0.0
