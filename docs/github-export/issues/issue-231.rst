========================================================================================
Issue #231: R&D: Stratégie de scaling infrastructure - K8s timing et partitionnement DB
========================================================================================

:State: **OPEN**
:Milestone: No milestone
:Labels: track:infrastructure,priority:medium R&D
:Assignees: Unassigned
:Created: 2026-03-07
:Updated: 2026-03-22
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/231>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Contexte
   
   La roadmap prévoit une progression VPS → K3s → K8s selon le nombre de copropriétés.
   Cette R&D couvre le timing et les critères de migration.
   
   ## Objectifs de la R&D
   
   1. **Critères de migration VPS → K3s** :
      - Nombre de copros (seuil : 200-500)
      - Charge CPU/RAM (seuil : 70% sustained)
      - Nombre de workers backend simultanés
      - Taille de la base de données (seuil : 50GB)
   
   2. **Partitionnement PostgreSQL** :
      - Partitioning par ``organization_id`` (multi-tenant isolation)
      - Partitioning temporel (charges, paiements, IoT readings)
      - Connection pooling : PgBouncer configuration
      - Read replicas pour dashboard/analytics
   
   3. **K3s architecture** :
      - Cluster topology (1 master + 2 workers minimum)
      - Storage : Longhorn vs. NFS vs. local-path
      - Ingress : Traefik (déjà en place) vs. Nginx Ingress
      - Monitoring : Prometheus + Grafana (déjà en place) migration
      - Secrets management : Sealed Secrets vs. Vault
   
   4. **Cost optimization** :
      - VPS actuel : ~6€/mois (Hetzner CAX11)
      - K3s estimé : ~30-50€/mois (3 nœuds)
      - K8s managed : ~100-200€/mois (OVH Managed K8s)
      - Break-even : revenu par copro nécessaire
   
   5. **Zero-downtime deployment** :
      - Rolling updates
      - Blue-green deployment
      - Database migrations sans interruption (``pg_trgm``, zero-downtime DDL)
   
   ## Points de décision
   
   - [ ] Quand migrer VPS → K3s (nombre de copros seuil)
   - [ ] Partitionnement par tenant ou par temps
   - [ ] Provider cloud (Hetzner vs. OVH vs. Scaleway)
   - [ ] IaC tool (Terraform vs. Pulumi vs. Ansible)
   
   ## Estimation
   
   8-12h

.. raw:: html

   </div>

