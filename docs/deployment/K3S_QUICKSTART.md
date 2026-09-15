# K3s + ArgoCD Quick Start Guide

**Issues**: #266, #267, #268
**Time Estimate**: 30-45 minutes
**Prerequisites**: Terraform, Ansible, kubectl, Git

## 5-Minute Overview

KoproGo deployment on K3s involves 3 steps:

1. **Provision Infrastructure** (Terraform) → OVH OpenStack
2. **Install K3s** (Ansible) → K3s cluster on instances
3. **Deploy with ArgoCD** (Git) → Continuous deployment

## Prerequisites

### Install Tools

```bash
# Terraform
curl -fsSL https://apt.releases.hashicorp.com/gpg | sudo apt-key add -
sudo apt-add-repository "deb [arch=amd64] https://apt.releases.hashicorp.com $(lsb_release -cs) main"
sudo apt update && sudo apt install terraform

# Ansible
sudo apt install ansible

# kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo mv kubectl /usr/local/bin/
chmod +x /usr/local/bin/kubectl

# ArgoCD CLI
curl -sSL -o argocd-linux-amd64 https://github.com/argoproj/argo-cd/releases/latest/download/argocd-linux-amd64
sudo install -m 555 argocd-linux-amd64 /usr/local/bin/argocd
rm argocd-linux-amd64
```

### OVH API Credentials

1. Visit [OVH Manager Public Cloud](https://www.ovh.com/manager/public-cloud/)
2. Navigate to **API** → **Application Keys**
3. Create new application and consumer keys
4. Export environment variables:

```bash
export OVH_ENDPOINT="ovh-eu"
export OVH_APPLICATION_KEY="<app-key>"
export OVH_APPLICATION_SECRET="<app-secret>"
export OVH_CONSUMER_KEY="<consumer-key>"

export OS_AUTH_URL="https://auth.cloud.ovh.net/v3"
export OS_USERNAME="user@example.com"
export OS_PASSWORD="<password>"
export OS_TENANT_NAME="<project-name>"
export OS_REGION_NAME="GRA"
```

### SSH Key Pair

```bash
# Generate SSH key
ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa -N ""

# Verify
ls -l ~/.ssh/id_rsa*
```

## Step 1: Provision Infrastructure (5 minutes)

```bash
cd infrastructure/terraform

# Copy example config
cp terraform.tfvars.example terraform.tfvars

# Edit terraform.tfvars with your values
vim terraform.tfvars

# Initialize Terraform
terraform init

# Plan (preview)
terraform plan

# Apply (create infrastructure)
terraform apply
# Type "yes" to confirm

# Show outputs
terraform output
```

**Expected Output**:
```
k3s_master_ip_private = "10.0.0.10"
k3s_master_ip_public = "195.154.x.x"
k3s_agents_ips = { "koprogo-agent-1" = "10.0.0.11", ... }
```

**Cost**: ~21€/month for 3x b2-7 instances

## Step 2: Install K3s (10-15 minutes)

### Generate Ansible Inventory

```bash
cd infrastructure/ansible

# Copy example inventory
cp inventory.example.ini inventory.ini

# Edit with actual IPs from Terraform output
vim inventory.ini
```

### Create Ansible Vault Secrets

```bash
# Generate K3s token (32-char random)
K3S_TOKEN=$(openssl rand -base64 32)

# Create vault file
ansible-vault create group_vars/k3s_master/vault.yml

# Content (paste and save):
vault_k3s_token: "$K3S_TOKEN"
vault_ghcr_dockercfg: "<base64-encoded-ghcr-config>"
# To generate GHCR credentials:
# echo -n 'USERNAME:TOKEN' | base64
```

### Run Ansible Playbook

```bash
# Install K3s master (will prompt for vault password)
ansible-playbook -i inventory.ini k3s-install.yml \
  --limit k3s_master \
  --ask-vault-pass

# Wait 2-3 minutes, then install agents
ansible-playbook -i inventory.ini k3s-install.yml \
  --limit k3s_agents \
  --ask-vault-pass

# Verify all nodes are ready
ansible all -i inventory.ini -m shell -a "k3s --version"
```

### Get Kubeconfig

```bash
# Copy from master
scp ubuntu@10.0.0.10:/etc/rancher/k3s/k3s.yaml ~/.kube/k3s-config.yaml

# Update server IP (use floating IP)
sed -i 's|127.0.0.1|195.154.x.x|g' ~/.kube/k3s-config.yaml

# Verify connectivity
export KUBECONFIG=~/.kube/k3s-config.yaml
kubectl get nodes
```

**Expected Output**:
```
NAME             STATUS   ROLES                  AGE
koprogo-master   Ready    control-plane,master   2m
koprogo-agent-1  Ready    <none>                 1m
koprogo-agent-2  Ready    <none>                 1m
```

## Step 3: Deploy with ArgoCD (10 minutes)

### Install ArgoCD

```bash
# Create namespace
kubectl create namespace argocd

# Install
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Wait for services
kubectl rollout status deployment/argocd-server -n argocd -w

# Get admin password
ARGOCD_PASSWORD=$(kubectl get secret -n argocd argocd-initial-admin-secret \
  -o jsonpath="{.data.password}" | base64 -d)
echo "ArgoCD admin password: $ARGOCD_PASSWORD"

# Port-forward to UI
kubectl port-forward -n argocd svc/argocd-server 8080:443
```

Access ArgoCD at: https://localhost:8080
- Username: `admin`
- Password: (from above)

### Deploy KoproGo

```bash
# Create koprogo namespace
kubectl create namespace koprogo

# Apply ArgoCD application
kubectl apply -f argocd/koprogo-app.yaml

# Watch sync
argocd app wait koprogo --timeout 300

# Or: kubectl get application -n argocd -w
```

### Verify Deployment

```bash
# Check pods
kubectl get pods -n koprogo
kubectl logs -n koprogo deployment/koprogo-backend
kubectl logs -n koprogo deployment/koprogo-frontend

# Check ingress
kubectl get ingress -n koprogo
kubectl describe ingress -n koprogo

# Test health endpoints
curl https://api.koprogo.be/health
curl https://app.koprogo.be/
```

## Update Deployment (GitOps)

Deployment is now GitOps-enabled. To update:

```bash
# 1. Update k8s manifests or container image tag
vim k8s/kustomization.yaml

# 2. Commit and push
git add k8s/
git commit -m "chore: Update image to v1.2.3"
git push origin main

# 3. ArgoCD syncs automatically (3-minute poll interval)
# Monitor: kubectl get application -n argocd -w
```

## Troubleshooting

### K3s master not responding

```bash
# SSH to master and check service
ssh ubuntu@195.154.x.x
sudo systemctl status k3s
sudo journalctl -u k3s -n 50

# Check disk space
df -h /var/lib/rancher/k3s
```

### Pods pending

```bash
# Check node resources
kubectl describe node koprogo-master

# Check pod events
kubectl describe pod <pod-name> -n koprogo
```

### ArgoCD stuck syncing

```bash
# Check repo connection
kubectl logs -n argocd deployment/argocd-repo-server | tail -20

# Force refresh
argocd app refresh koprogo
```

## Cost Breakdown

| Component | Count | Unit Price | Total |
|-----------|-------|------------|-------|
| b2-7 Instance | 3 | 7€/mo | 21€ |
| Network traffic | - | (included) | - |
| **Monthly Total** | | | **21€** |

**Scaling**:
- Add agent: `k3s_agent_count = 3` → 28€/month
- Upgrade to b2-15: `k3s_master_flavor = "b2-15"` → 42€/month

## Next Steps

1. Configure DNS (app.koprogo.be, api.koprogo.be)
2. Enable HTTPS certificates via Traefik + Let's Encrypt
3. Setup monitoring (Prometheus + Grafana)
4. Configure backups for PostgreSQL
5. Implement network policies
6. Add CI/CD pipeline for image builds

See [K3S_GITOPS_DEPLOYMENT.md](./K3S_GITOPS_DEPLOYMENT.md) for complete guide.

## Support

- K3s Issues: https://github.com/k3s-io/k3s/issues
- ArgoCD Issues: https://github.com/argoproj/argo-cd/issues
- KoproGo Issues: https://github.com/gilmry/koprogo/issues
