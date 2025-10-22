# KoproGo - Architecture Infrastructure

Documentation complète de l'architecture infrastructure pour KoproGo.

## 🎯 Objectifs Architecturaux

### Performance

- ⚡ **Latence P99** : < 5ms
- 🚀 **Throughput** : > 100k req/s
- 💾 **Memory** : < 128MB par pod
- 🌱 **CO2** : < 0.5g par requête

### Disponibilité

- 🎯 **SLA** : 99.9% (prod)
- 🔄 **RTO** : < 1 heure
- 💾 **RPO** : < 5 minutes
- 📊 **MTTR** : < 30 minutes

### Scalabilité

- **Horizontal** : Auto-scaling 2-50 pods
- **Vertical** : Node scaling manuel
- **Database** : Read replicas
- **Cache** : Distributed DragonflyDB

## 🏗️ Architecture Globale

### Vue d'ensemble

```
┌─────────────────────────────────────────────────────────┐
│                     Internet                             │
└────────────────────────┬────────────────────────────────┘
                         │
                    ┌────▼────┐
                    │   DNS   │
                    │ OVH     │
                    └────┬────┘
                         │
              ┌──────────▼──────────┐
              │  Load Balancer OVH  │
              │  (Layer 4/7)        │
              └──────────┬──────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
   ┌────▼────┐      ┌────▼────┐     ┌────▼────┐
   │  CP-1   │      │  CP-2   │     │  CP-3   │
   │  etcd   │◄────►│  etcd   │◄───►│  etcd   │
   │  6443   │      │  6443   │     │  6443   │
   └─────────┘      └─────────┘     └─────────┘
        │                │                │
   ┌────┴────────────────┴────────────────┴────┐
   │           K3s Cluster Network             │
   │         (10.42.0.0/16 Pod CIDR)          │
   └───────────────────┬──────────────────────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
   ┌────▼────┐    ┌────▼────┐   ┌────▼────┐
   │Worker-1 │    │Worker-2 │   │Worker-3 │
   │         │    │         │   │         │
   │Longhorn │◄──►│Longhorn │◄─►│Longhorn │
   │Storage  │    │Storage  │   │Storage  │
   └────┬────┘    └────┬────┘   └────┬────┘
        │              │              │
        └──────────────┼──────────────┘
                       │
              ┌────────▼────────┐
              │  Private vRack  │
              │  10.0.0.0/24    │
              └─────────────────┘
```

### Composants Principaux

#### 1. Control Plane (K3s Server)

- **Rôle** : Gestion du cluster Kubernetes
- **Composants** :
  - API Server (port 6443)
  - etcd (embedded, ports 2379-2380)
  - Controller Manager
  - Scheduler
- **HA** : 3 nodes en production (quorum etcd)
- **Taints** : NoSchedule (pas de workloads)

#### 2. Workers (K3s Agent)

- **Rôle** : Exécution des workloads applicatifs
- **Composants** :
  - Kubelet
  - Kube-proxy
  - Container runtime (containerd)
  - CNI (Flannel)
- **Storage** : Longhorn pour volumes persistants
- **Resources** : CPU/Memory requests & limits

#### 3. Longhorn Storage

- **Type** : Distributed block storage
- **Réplication** : 3 replicas par volume
- **Backend** : Volumes OVH attachés aux workers
- **Features** :
  - Snapshots automatiques
  - Backup vers S3
  - Volume encryption
  - CSI driver

#### 4. Ingress

- **Controller** : nginx-ingress
- **Type** : NodePort (30080, 30443)
- **TLS** : cert-manager + Let's Encrypt
- **Features** :
  - Rate limiting
  - CORS
  - Path rewriting
  - SSL termination

## 📡 Flux Réseau

### Requête HTTP(S)

```
Client
  │
  ├─► DNS Lookup (api.koprogo.io) → OVH Load Balancer IP
  │
  └─► HTTPS (443)
       │
       ├─► OVH Load Balancer
       │    │
       │    └─► Round-robin vers Workers (NodePort 30443)
       │         │
       │         └─► nginx-ingress-controller
       │              │
       │              └─► Service koprogo-api (ClusterIP)
       │                   │
       │                   └─► Pod koprogo-api
       │                        │
       │                        ├─► PostgreSQL (internal)
       │                        ├─► DragonflyDB (cache)
       │                        └─► MinIO (storage)
       │
       └◄── Response
```

### Communication Interne

- **Pod-to-Pod** : CNI Flannel (VXLAN overlay)
- **Pod-to-Service** : kube-proxy (iptables)
- **Inter-node** : Private vRack (10.0.0.0/24)
- **External** : NAT gateway

### Ports & Protocoles

#### Control Plane

| Port       | Protocol | Purpose                  |
| ---------- | -------- | ------------------------ |
| 6443       | TCP      | K3s API Server           |
| 2379-2380  | TCP      | etcd client/peer         |
| 10250      | TCP      | Kubelet API              |
| 10251      | TCP      | kube-scheduler           |
| 10252      | TCP      | kube-controller-manager  |

#### Workers

| Port       | Protocol | Purpose                  |
| ---------- | -------- | ------------------------ |
| 10250      | TCP      | Kubelet API              |
| 30000-32767| TCP      | NodePort Services        |
| 80/443     | TCP      | Ingress HTTP/HTTPS       |

## 💾 Storage Architecture

### Hierarchy

```
┌─────────────────────────────────────────┐
│        Applications (Pods)              │
└────────────────┬────────────────────────┘
                 │ PVC
┌────────────────▼────────────────────────┐
│     Longhorn Storage Class              │
│     (default, replica=3)                │
└────────────────┬────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼──────┐  ┌───────▼──────┐
│ Worker Node 1│  │ Worker Node 2│
│              │  │              │
│ /dev/sdb     │  │ /dev/sdb     │
│ 200GB        │  │ 200GB        │
└──────────────┘  └──────────────┘
        │                 │
        └────────┬────────┘
                 │
        ┌────────▼────────┐
        │   OVH Object    │
        │   Storage (S3)  │
        │   (Backups)     │
        └─────────────────┘
```

### Volumes Types

1. **Local** : Données temporaires (emptyDir)
2. **Longhorn** : Données persistantes (PVC)
3. **S3** : Backups et archives (MinIO/OVH)

### Backup Strategy

- **Database** : pg_dump quotidien → S3
- **Volumes** : Longhorn snapshots → S3
- **etcd** : Snapshot toutes les 6h
- **Rétention** : 30 jours (dev), 90 jours (prod)

## 🔐 Sécurité

### Defense in Depth

```
┌──────────────────────────────────────────┐
│  Layer 7: Application Security          │
│  - JWT Authentication                    │
│  - Input Validation                      │
│  - Rate Limiting                         │
└──────────────┬───────────────────────────┘
               │
┌──────────────▼───────────────────────────┐
│  Layer 6: Pod Security                   │
│  - Security Context (non-root)           │
│  - Read-only filesystem                  │
│  - Drop capabilities                     │
│  - Network Policies                      │
└──────────────┬───────────────────────────┘
               │
┌──────────────▼───────────────────────────┐
│  Layer 5: Kubernetes RBAC               │
│  - Service Accounts                      │
│  - Role-based access                     │
│  - Least privilege                       │
└──────────────┬───────────────────────────┘
               │
┌──────────────▼───────────────────────────┐
│  Layer 4: Network Security              │
│  - Network Policies                      │
│  - TLS everywhere                        │
│  - Encrypted overlay (Flannel)           │
└──────────────┬───────────────────────────┘
               │
┌──────────────▼───────────────────────────┐
│  Layer 3: OS Security                   │
│  - UFW Firewall                          │
│  - Fail2ban                              │
│  - SSH hardening                         │
│  - Audit logging                         │
└──────────────────────────────────────────┘
```

### Secrets Management

- **Kubernetes Secrets** : Encrypted at rest
- **Sealed Secrets** : Encrypted in Git
- **External Secrets** : HashiCorp Vault (TODO)
- **Rotation** : Automated rotation (TODO)

## 📊 Observabilité

### Metrics Pipeline

```
┌─────────────────────────────────────┐
│         Application Pods            │
│    (expose /metrics endpoint)       │
└───────────────┬─────────────────────┘
                │
        ┌───────▼──────────┐
        │  Prometheus      │
        │  (Scrape every   │
        │   15 seconds)    │
        └───────┬──────────┘
                │
        ┌───────▼──────────┐
        │   Grafana        │
        │  (Dashboards &   │
        │   Alerts)        │
        └──────────────────┘
```

### Logs Pipeline

```
┌─────────────────────────────────────┐
│         Application Pods            │
│    (stdout/stderr logs)             │
└───────────────┬─────────────────────┘
                │
        ┌───────▼──────────┐
        │    Promtail      │
        │  (Log collector) │
        └───────┬──────────┘
                │
        ┌───────▼──────────┐
        │      Loki        │
        │  (Log aggre-     │
        │   gation)        │
        └───────┬──────────┘
                │
        ┌───────▼──────────┐
        │    Grafana       │
        │  (Log query &    │
        │   visualization) │
        └──────────────────┘
```

### Metrics Collectées

- **Infrastructure** : CPU, Memory, Disk, Network
- **Kubernetes** : Pods, Nodes, Deployments
- **Application** : Request rate, latency, errors
- **Database** : Connections, queries, replication
- **Business** : Users, transactions, revenue
- **CO2** : Energy consumption (Kepler)

## 🔄 CI/CD Pipeline

```
┌──────────────┐
│   Git Push   │
│  (main branch)│
└──────┬───────┘
       │
┌──────▼────────┐
│ GitHub Actions│
│               │
│ 1. Build      │
│ 2. Test       │
│ 3. Scan       │
└──────┬────────┘
       │
┌──────▼────────┐
│  Push Image   │
│  to Registry  │
└──────┬────────┘
       │
┌──────▼────────┐
│ Helm Upgrade  │
│  (Rolling)    │
└──────┬────────┘
       │
┌──────▼────────┐
│  Smoke Tests  │
└───────────────┘
```

## 💰 Coût Infrastructure

### Production (mensuel)

| Composant          | Quantité | Unitaire | Total  |
| ------------------ | -------- | -------- | ------ |
| Control Plane b2-15| 3        | 30€      | 90€    |
| Worker b2-30       | 3        | 60€      | 180€   |
| Load Balancer      | 1        | 20€      | 20€    |
| Storage (600GB)    | 1        | 30€      | 30€    |
| Backup S3          | 1        | 10€      | 10€    |
| **Total**          |          |          | **330€**|

### Optimisations Possibles

- Spot instances pour staging
- Reserved instances (-15%)
- Auto-shutdown dev la nuit
- Compression backups

## 📈 Scaling Strategy

### Horizontal Pod Autoscaling

```yaml
Min Replicas: 2
Max Replicas: 50
Target CPU: 70%
Target Memory: 80%
```

### Vertical Node Scaling

Manuel, basé sur monitoring :
- Ajouter worker si CPU > 80% sustained
- Utiliser node taints/affinity

### Database Scaling

- **Vertical** : Upgrade instance size
- **Horizontal** : Read replicas
- **Sharding** : TODO (ScyllaDB migration)

## 🚨 Disaster Recovery

### Scenarios

1. **Single Node Failure**
   - Auto-healing : K8s reschedule pods
   - RTO : < 5 minutes
   - RPO : 0

2. **Control Plane Failure**
   - HA etcd : Quorum maintenu
   - RTO : < 1 minute
   - RPO : 0

3. **Complete Cluster Loss**
   - Restore from Terraform + backups
   - RTO : < 2 hours
   - RPO : < 1 hour

4. **Data Corruption**
   - Restore from S3 backup
   - RTO : < 1 hour
   - RPO : < 24 hours

### Recovery Procedures

Voir [RUNBOOK.md](./RUNBOOK.md) (TODO)

## 🔮 Roadmap Infrastructure

### Q1 2025

- [ ] ScyllaDB pour high-velocity data
- [ ] DragonflyDB cluster (3 nodes)
- [ ] MinIO distributed (4 nodes)
- [ ] GitOps avec ArgoCD

### Q2 2025

- [ ] Service Mesh (Linkerd)
- [ ] External Secrets Operator
- [ ] Automated scaling policies
- [ ] Multi-region (GRA + SBG)

### Q3 2025

- [ ] Zero-downtime deployments
- [ ] Blue/Green environments
- [ ] Chaos Engineering (Chaos Mesh)
- [ ] Cost optimization automation

### Q4 2025

- [ ] Multi-cloud (OVH + AWS backup)
- [ ] Edge locations
- [ ] Advanced observability (traces)
- [ ] Self-healing automation

---

**Maintenu par** : KoproGo DevOps Team
**Dernière mise à jour** : 2024-01-15
