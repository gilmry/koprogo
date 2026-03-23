# KoproGo K3s + GitOps Deployment Guide

**Issues**: #266, #267, #268
**Status**: Draft
**Date**: 2026-03-23
**Author**: KoproGo Team

## Overview

This document covers the complete deployment of KoproGo on a **K3s Kubernetes cluster** hosted on **OVH OpenStack**, with **GitOps automation** using **ArgoCD**.

### Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    ArgoCD (GitOps)                      │
│              Continuous Deployment from Git             │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│              K3s Kubernetes Cluster                     │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Namespace: koprogo                             │   │
│  │  ┌──────────────────┐  ┌──────────────────────┐│   │
│  │  │ Backend          │  │ Frontend             ││   │
│  │  │ (Rust Actix-web) │  │ (Astro + Svelte)     ││   │
│  │  │ 2 replicas       │  │ 2 replicas           ││   │
│  │  └──────────────────┘  └──────────────────────┘│   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Traefik Ingress Controller (K3s default)       │   │
│  │  - TLS via Let's Encrypt                        │   │
│  │  - Security headers (HSTS, CSP, etc.)          │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│        OVH OpenStack (EU: Gravelines/Roubaix)          │
│  ┌──────────────────────────────────────────────────┐  │
│  │  1x K3s Master (b2-7: 2 vCPU, 7GB RAM)          │  │
│  │  2x K3s Agents (b2-7: 2 vCPU, 7GB RAM each)     │  │
│  │  Private Network (10.0.0.0/24)                   │  │
│  │  Floating IP (public access to master)           │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Issue Breakdown

### Issue #266: K3s Provisioning with Terraform + OpenStack

**Objective**: Automate infrastructure provisioning for K3s cluster on OVH OpenStack using Terraform.

**Files**:
- `infrastructure/terraform/k3s-cluster.tf` - K3s cluster resources (master + agents)
- `infrastructure/terraform/k3s-variables.tf` - Input variables for K3s configuration
- `infrastructure/terraform/terraform.tfvars.example` - Example variable values

**Key Components**:
1. **OVH OpenStack Network**
   - Private network: 10.0.0.0/24
   - Router with external gateway (Ext-Net)
   - Subnet with DHCP enabled

2. **Security Group**
   - SSH (22): Worldwide access
   - K3s API (6443): Internal cluster only
   - HTTP/HTTPS (80/443): Public access (Ingress)
   - Flannel VXLAN (8472): Internal overlay network
   - Kubelet (10250): Internal cluster
   - Prometheus (9100): Internal monitoring

3. **Compute Instances**
   - **Master Node**: 1x instance (default: b2-7, 2 vCPU, 7GB RAM)
   - **Agent Nodes**: 2x instances (configurable, default: b2-7 each)
   - **Floating IP**: Public IP for master node (kubectl access)

4. **Cost Estimation** (OVH 2025 pricing, EUR):
   - Master: ~7€/month
   - 2x Agents: ~14€/month
   - **Total**: ~21€/month (3 x b2-7 instances)

**Usage**:

```bash
cd infrastructure/terraform

# Initialize Terraform backend
terraform init

# Plan provisioning
terraform plan -var-file=terraform.tfvars

# Apply (create infrastructure)
terraform apply -var-file=terraform.tfvars

# Export cluster info for Ansible
terraform output -json > k3s-cluster-info.json
```

**Environment Variables** (OVH API):
```bash
export OS_AUTH_URL="https://auth.cloud.ovh.net/v3"
export OS_USERNAME="<your-username>"
export OS_PASSWORD="<your-password>"
export OS_TENANT_NAME="<your-tenant>"
export OS_REGION_NAME="GRA"
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="<ovh-app-key>"
export OVH_APPLICATION_SECRET="<ovh-app-secret>"
export OVH_CONSUMER_KEY="<ovh-consumer-key>"
```

---

### Issue #267: K3s with Ansible (installation + hardening)

**Objective**: Install K3s on provisioned OpenStack instances with security hardening.

**Files**:
- `infrastructure/ansible/k3s-install.yml` - K3s installation & hardening playbook
- `infrastructure/ansible/inventory.ini` - Ansible inventory
- `infrastructure/ansible/group_vars/k3s_master/vault.yml` - Secrets (ansible-vault encrypted)
- `infrastructure/ansible/group_vars/k3s_agents/vault.yml` - Agent secrets

**Installation Steps**:

#### 1. Prepare Ansible Inventory

```bash
cd infrastructure/ansible

# Generate inventory from Terraform output
terraform output -json | python3 -c "
import sys, json
output = json.load(sys.stdin)
print('[k3s_master]')
master = output['k3s_cluster_info']['value']['master_hostname']
ip = output['k3s_cluster_info']['value']['master_ip_private']
print(f'{master} ansible_host={ip}')
print('\n[k3s_agents]')
for agent in output['k3s_cluster_info']['value']['agents']:
  for i, name in enumerate(agent.keys()):
    print(f'{name} ansible_host={agent[name]}')
" > inventory.ini
```

#### 2. Create Ansible Vault Secrets

```bash
# Encrypt K3s token (32-char random)
ansible-vault create group_vars/k3s_master/vault.yml

# Content:
vault_k3s_token: "<32-char-random-token>"
vault_ghcr_dockercfg: "<base64-encoded-ghcr-credentials>"
```

#### 3. Run K3s Installation

```bash
# Install K3s master
ansible-playbook -i inventory.ini k3s-install.yml \
  --limit k3s_master \
  --ask-vault-pass

# Install K3s agents (after master is ready)
ansible-playbook -i inventory.ini k3s-install.yml \
  --limit k3s_agents \
  --ask-vault-pass
```

**Security Hardening** (in k3s-install.yml):

1. **Kernel Hardening** (sysctl):
   - `net.ipv4.ip_forward=1` - Enable IP forwarding for Kubernetes
   - `net.bridge.bridge-nf-call-iptables=1` - iptables for bridge traffic
   - `kernel.unprivileged_userns_clone=0` - Disable unprivileged user namespaces
   - `kernel.kptr_restrict=2` - Hide kernel pointers
   - `kernel.sysrq=0` - Disable SysRq key

2. **K3s API Server** (--kube-apiserver-arg):
   - Audit logging: `audit-log-path=/var/log/k3s-audit.log`
   - Audit retention: `audit-log-maxage=7` days
   - Audit policy: `audit-policy-file=/var/lib/rancher/k3s/server/audit-policy.json`

3. **Kubelet** (--kubelet-arg):
   - Read-only port disabled: `read-only-port=0`
   - Event throttling: `event-record-qps=1`

4. **Network**:
   - Traefik disabled (use Traefik 2.x separately)
   - Local storage disabled (use persistent volumes)

#### 4. Retrieve Kubeconfig

```bash
# Copy kubeconfig from master
scp ubuntu@<master-ip>:/etc/rancher/k3s/k3s.yaml ~/.kube/k3s-config.yaml

# Update server IP to floating IP
sed -i 's/server:.*$/server: https:\/\/<floating-ip>:6443/' ~/.kube/k3s-config.yaml

# Test cluster access
export KUBECONFIG=~/.kube/k3s-config.yaml
kubectl get nodes
```

---

### Issue #268: GitOps pipeline with ArgoCD on K3s

**Objective**: Deploy KoproGo using GitOps with ArgoCD continuous deployment from Git.

**Files**:
- `k8s/` - Kubernetes manifests for KoproGo
  - `namespace.yaml` - koprogo namespace
  - `backend-deployment.yaml` - Rust API backend (2 replicas)
  - `backend-service.yaml` - ClusterIP service
  - `backend-serviceaccount.yaml` - RBAC for backend
  - `frontend-deployment.yaml` - Astro + Svelte frontend (2 replicas)
  - `frontend-service.yaml` - Frontend service
  - `frontend-serviceaccount.yaml` - Frontend RBAC
  - `ingress.yaml` - Traefik ingress with TLS
  - `kustomization.yaml` - Kustomize overlays
- `argocd/koprogo-app.yaml` - ArgoCD Application manifest

#### Step 1: Install ArgoCD

```bash
# Create argocd namespace
kubectl create namespace argocd

# Install ArgoCD
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Wait for services
kubectl rollout status deployment/argocd-server -n argocd

# Get initial admin password
kubectl get secret -n argocd argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d

# Port-forward to ArgoCD UI
kubectl port-forward -n argocd svc/argocd-server 8080:443 &
# Access: https://localhost:8080
```

#### Step 2: Configure GitHub Repository Access

```bash
# Create SSH key for GitHub (if not exists)
ssh-keygen -t rsa -b 4096 -f ~/.ssh/argocd-github -C "argocd@koprogo"

# Add public key to GitHub repository Deploy Keys
cat ~/.ssh/argocd-github.pub

# Create Kubernetes secret for GitHub credentials
kubectl create secret generic github-credentials \
  -n argocd \
  --from-file=ssh-privatekey=~/.ssh/argocd-github

# Test connection
kubectl debug -n argocd -it deployment/argocd-repo-server -- \
  ssh -T git@github.com
```

#### Step 3: Create KoproGo ArgoCD Application

```bash
# Apply KoproGo application manifest
kubectl apply -f argocd/koprogo-app.yaml

# Verify application is created
kubectl get application -n argocd
kubectl describe application koprogo -n argocd

# View sync status
argocd app get koprogo --refresh
```

#### Step 4: Configure Git Webhook (Optional)

For immediate push-to-deploy (without waiting for ArgoCD polling):

```bash
# In GitHub repository Settings > Webhooks:
# - Payload URL: https://<argocd-domain>/api/webhook
# - Content type: application/json
# - Events: Push events
# - Secret: <generate-random-secret>

# Configure ArgoCD webhook secret
kubectl patch configmap -n argocd argocd-cmd-params-cm \
  -p '{"data": {"webhook.github.secret": "<secret>"}}'
```

#### Step 5: Monitor Deployment

```bash
# Watch ArgoCD sync progress
argocd app wait koprogo --timeout 300

# Check pod status
kubectl get pods -n koprogo -w

# View logs
kubectl logs -n koprogo deployment/koprogo-backend
kubectl logs -n koprogo deployment/koprogo-frontend

# Verify ingress
kubectl get ingress -n koprogo
kubectl describe ingress koprogo-ingress -n koprogo
```

---

## GitOps Workflow

### Push-to-Deploy Flow

```
1. Developer commits code
   └─> git push origin feature-branch

2. GitHub triggers webhook
   └─> ArgoCD controller receives notification

3. ArgoCD polls Git repository
   └─> Detects new commit on main branch

4. ArgoCD syncs Kubernetes manifests
   └─> Applies changes via kubectl

5. K3s reconciles cluster state
   └─> Pulls new container images
   └─> Rolls out Deployment updates
   └─> Services remain available (RollingUpdate)

6. Monitoring/alerting detects issues
   └─> Slack/email notifications
```

### Deployment Process

#### 1. Update Container Image Tag

```bash
# Update image tag in k8s/kustomization.yaml
cd k8s
sed -i 's/newTag: latest/newTag: v1.2.3/' kustomization.yaml

git add kustomization.yaml
git commit -m "chore: Update KoproGo image to v1.2.3"
git push origin main
```

#### 2. ArgoCD Automatic Sync (3 minutes)

```bash
# Poll interval: 3 minutes (default)
# Sync options:
# - CreateNamespace=true: Create koprogo namespace if missing
# - PrunePropagationPolicy=foreground: Delete in proper order
# - PruneLast=true: Delete resources after rollout
```

#### 3. Verify Deployment

```bash
# Check sync status
kubectl get application -n argocd koprogo

# View deployed resources
kubectl get all -n koprogo

# Test frontend
curl https://app.koprogo.be/

# Test API
curl https://api.koprogo.be/api/v1/health
```

---

## Monitoring & Logging

### K3s Observability Stack

```
┌──────────────────────────────────────────┐
│        Prometheus (Metrics)               │
│  - K3s components                        │
│  - Pod/node metrics                      │
│  - Retention: 30 days                    │
└──────────┬───────────────────────────────┘
           │
┌──────────▼────────────────────────────────┐
│        Grafana (Visualization)             │
│  - K3s cluster dashboard                 │
│  - Application performance metrics       │
│  - Alert status                          │
└──────────┬───────────────────────────────┘
           │
┌──────────▼────────────────────────────────┐
│    Loki (Log Aggregation)                 │
│  - Container logs (koprogo namespace)    │
│  - Retention: 7 days                     │
│  - Searchable via Grafana                │
└──────────┬───────────────────────────────┘
           │
┌──────────▼────────────────────────────────┐
│   Alertmanager (Alerting)                 │
│  - CPU/memory usage                      │
│  - Pod restarts                          │
│  - Deployment failures                   │
│  - Targets: Slack, email                 │
└──────────────────────────────────────────┘
```

### Access Monitoring

```bash
# Prometheus metrics
kubectl port-forward -n monitoring svc/prometheus 9090:9090

# Grafana dashboards
kubectl port-forward -n monitoring svc/grafana 3000:3000
# Default: admin/admin

# Loki logs
kubectl port-forward -n monitoring svc/loki 3100:3100

# K3s server logs
journalctl -u k3s -f

# K3s agent logs
journalctl -u k3s-agent -f
```

---

## Cost Optimization

### OVH Instance Sizing

| Flavor | vCPU | RAM | Storage | Price/mo |
|--------|------|-----|---------|----------|
| b2-7   | 2    | 7GB | 40GB    | ~7€      |
| b2-15  | 4    | 15GB| 80GB    | ~14€     |
| b2-30  | 8    | 30GB| 160GB   | ~28€     |

**Recommended**:
- **Production**: 1x b2-15 (master) + 3x b2-15 (agents) = ~56€/month
- **Staging**: 1x b2-7 (master) + 2x b2-7 (agents) = ~21€/month
- **Development**: 1x b2-7 (single-node K3s) = ~7€/month

### Resource Limits

```yaml
# backend-deployment.yaml
resources:
  requests:
    memory: "128Mi"  # Minimum for pod scheduling
    cpu: "100m"
  limits:
    memory: "512Mi"  # Kill if exceeds
    cpu: "500m"

# frontend-deployment.yaml
resources:
  requests:
    memory: "64Mi"
    cpu: "50m"
  limits:
    memory: "256Mi"
    cpu: "200m"
```

---

## Security Checklist

- [x] Network isolation (10.0.0.0/24 private)
- [x] K3s API server hardened (audit logging, read-only port=0)
- [x] Kubelet hardened (read-only port=0, event throttling)
- [x] Ingress TLS (Let's Encrypt)
- [x] Security headers (HSTS, CSP, X-Frame-Options)
- [x] RBAC (service accounts with minimal permissions)
- [x] Pod security context (runAsNonRoot, readOnlyRootFilesystem)
- [x] Network policies (optional: restrict pod-to-pod traffic)
- [x] Container registry authentication (ghcr-secret)
- [x] Secrets management (Kubernetes Secrets + ArgoCD)
- [ ] **TODO**: Implement Network Policies (Issue TBD)
- [ ] **TODO**: Implement Pod Security Policies (Issue TBD)
- [ ] **TODO**: Configure OWASP WAF (CrowdSec integration) (Issue TBD)

---

## Troubleshooting

### K3s Master Won't Start

```bash
# Check service status
systemctl status k3s

# View logs
journalctl -u k3s -n 100 -f

# Verify disk space
df -h /var/lib/rancher/k3s

# Check port availability
lsof -i :6443
```

### Pods Stuck in Pending

```bash
# Check node resources
kubectl describe node <node-name>

# Check pod events
kubectl describe pod <pod-name> -n koprogo

# Check image pull secrets
kubectl get secrets -n koprogo
```

### ArgoCD Sync Failures

```bash
# View application status
argocd app get koprogo

# Check repository connectivity
kubectl logs -n argocd deployment/argocd-repo-server

# Verify kubeconfig
kubectl auth can-i get deployments --as=system:serviceaccount:argocd:argocd-application-controller -n koprogo
```

### Ingress Not Routing Traffic

```bash
# Verify ingress object
kubectl get ingress -n koprogo
kubectl describe ingress koprogo-ingress -n koprogo

# Test Traefik controller
kubectl get pods -n kube-system | grep traefik

# Check TLS certificate
kubectl get secret koprogo-tls -n koprogo -o yaml
```

---

## Related Issues

- **Issue #39-#43**: Security & Monitoring (LUKS, monitoring stack, IDS)
- **Issue #78**: 2FA Authentication (TOTP setup)
- **Issue #80**: Database Backups (PostgreSQL backup strategy)
- **Issue #100**: Performance Optimization (P99 < 5ms target)

---

## References

- [K3s Documentation](https://docs.k3s.io/)
- [Terraform OpenStack Provider](https://registry.terraform.io/providers/terraform-provider-openstack/openstack/)
- [ArgoCD Documentation](https://argo-cd.readthedocs.io/)
- [OVH Cloud Documentation](https://docs.ovh.com/gb/en/public-cloud/)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/security/pod-security-standards/)

---

**Last Updated**: 2026-03-23
**Next Review**: 2026-04-23
