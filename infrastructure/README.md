# KoproGo - Infrastructure

Déploiement automatisé de KoproGo sur OVH Public Cloud.

---

## 🚀 TL;DR

```bash
# Depuis la racine du projet
make setup-infra
```

**Durée** : ~20-30 minutes

---

## 📚 Documentation Complète

Toute la documentation d'infrastructure est centralisée dans [`docs/deployment/`](../docs/deployment/) :

### Guides

- **[Vue d'ensemble](../docs/deployment/index.md)** - Architecture, coûts, prérequis
- **[Configuration OVH](../docs/deployment/ovh-setup.md)** - Créer compte, utilisateur OpenStack, credentials API
- **[Terraform + Ansible](../docs/deployment/terraform-ansible.md)** - Détails techniques du provisionnement
- **[GitOps Auto-Update](../docs/deployment/gitops.md)** - Service systemd, mise à jour automatique
- **[Troubleshooting](../docs/deployment/troubleshooting.md)** - Résolution de problèmes

### Structure

```
infrastructure/
├── terraform/          # Provisionnement VPS
│   ├── main.tf
│   ├── .env           # Variables (gitignored)
│   └── load-env.sh    # Chargement variables
├── ansible/           # Configuration serveur
│   ├── playbook.yml
│   └── templates/
└── README.md          # Ce fichier
```

---

## 🛠️ Commandes Rapides

### Terraform

```bash
cd terraform
source ./load-env.sh

terraform init
terraform plan
terraform apply
terraform output vps_ip
```

### Ansible

```bash
cd ansible
ansible-playbook -i inventory.ini playbook.yml
```

---

## 📖 Ressources

- **Documentation complète** : [`docs/deployment/`](../docs/deployment/)
- **Lessons Learned** : [`LESSONS-LEARNED.md`](./LESSONS-LEARNED.md)
- **Makefile Guide** : [`docs/MAKEFILE_GUIDE.md`](../docs/MAKEFILE_GUIDE.md)

---

**KoproGo ASBL** 🚀
