# Modèle Économique KoproGo

## 🎯 Philosophie : Solidarité et Prix Coûtant

KoproGo ASBL adopte un **modèle économique solidaire** basé sur la **mutualisation des coûts** et la **transparence absolue**. Notre objectif n'est pas le profit, mais la viabilité financière du projet au service de l'intérêt général.

### Principes Fondamentaux

1. **Prix d'entrée minimal** : 1€/mois par copropriété
2. **Quotas d'espace raisonnables** : Suffisants pour 90% des usages
3. **Dépassement à prix coûtant** : Coûts additionnels dilués avec toute la communauté
4. **Transparence totale** : Facture détaillée consultable par tous
5. **Option self-hosted gratuite** : Liberté totale pour les utilisateurs techniques

---

## 💰 Structure Tarifaire SaaS Cloud

### Offre Standard : 1€/mois TTC

**Inclus dans l'offre de base** :

| Ressource | Quota Standard | Usage Typique |
|-----------|----------------|---------------|
| **Stockage documents** | 500 MB | ~500 fichiers PDF/photos (1 MB moyen) |
| **Utilisateurs** | 50 | Suffisant pour immeuble 20-30 lots |
| **Requêtes API** | 100,000/mois | ~3,300 req/jour (~140 req/h) |
| **Bande passante** | 5 GB/mois | Téléchargement documents, consultation |
| **Backup automatique** | Quotidien | Rétention 7 jours |
| **Support** | Email (72h) | Documentation complète + forum |

**Pour qui ?**
- Petites et moyennes copropriétés (5-30 lots)
- Usage standard : gestion charges, assemblées, documents
- ~90% des utilisateurs restent dans les quotas de base

---

### Dépassement de Quotas : Prix Coûtant Mutualisé

**Philosophie** : Nous ne faisons **aucun profit** sur les dépassements. Les coûts additionnels sont calculés au **prix coûtant réel** et **mutualisés entre tous les utilisateurs** du cloud KoproGo.

#### Calcul du Prix Coûtant

Le prix coûtant est calculé mensuellement et communiqué publiquement :

**Formule** :
```
Prix coûtant = (Coût infrastructure total + Coût bande passante + Coût stockage S3) / Nombre total copropriétés cloud
```

**Exemple Octobre 2025** :
```
Infrastructure VPS OVH (d2-2): 7€/mois
Stockage S3 OVH (200 GB):     2€/mois (0.01€/GB)
Bande passante (500 GB):      0€ (inclus)
Support (bénévole):           0€
Total coûts:                  9€/mois

Nombre copropriétés cloud:    100
Prix coûtant de base:         0.09€/copro/mois
Marge ASBL (maintenance):     0.91€/copro/mois (91%)
```

#### Grille Tarifaire Dépassement (Prix Coûtant)

| Ressource | Coût Unitaire | Exemple Dépassement | Coût Additionnel |
|-----------|---------------|---------------------|------------------|
| **Stockage +100 MB** | 0.001€/GB/mois | 600 MB total | +0.10€/mois |
| **Utilisateurs +10** | 0€ | 60 users total | **Gratuit** |
| **Requêtes API +50k** | 0€ | 150k req/mois | **Gratuit** |
| **Bande passante +1 GB** | 0.002€/GB | 6 GB/mois | +0.02€/mois |

**Important** : Les quotas utilisateurs et requêtes API n'ont **aucun coût marginal** pour l'infrastructure, donc **aucun surcoût** en cas de dépassement.

#### Exemples Concrets

**Cas 1 : Copropriété 10 lots (usage léger)**
- Stockage : 200 MB (sous quota)
- Utilisateurs : 15 (sous quota)
- Requêtes : 30,000/mois (sous quota)
- **Coût total : 1.00€/mois** ✅

**Cas 2 : Copropriété 50 lots (usage normal)**
- Stockage : 800 MB *(+300 MB)*
- Utilisateurs : 80 *(+30 users, gratuit)*
- Requêtes : 180,000/mois *(gratuit)*
- **Coût total : 1.30€/mois** (1€ base + 0.30€ stockage)

**Cas 3 : Grande copropriété 100 lots (usage intensif)**
- Stockage : 2 GB *(+1.5 GB)*
- Utilisateurs : 150 *(+100 users, gratuit)*
- Requêtes : 500,000/mois *(gratuit)*
- Bande passante : 12 GB *(+7 GB)*
- **Coût total : 2.64€/mois** (1€ + 1.50€ stockage + 0.14€ BP)

**Comparaison avec concurrent propriétaire** :
- Solution SaaS classique : 200-500€/mois pour 100 lots
- **KoproGo : 2.64€/mois** (soit **99% d'économie**)

---

### Transparence Comptable : Facture Publique

Chaque mois, l'ASBL publie un **rapport financier public** détaillant :

1. **Coûts infrastructure réels** (factures OVH)
2. **Nombre de copropriétés hébergées**
3. **Utilisation ressources** (stockage, BP, CPU)
4. **Prix coûtant calculé**
5. **Répartition revenus** (maintenance, développement, réserves)

**Accès** : Tableau de bord public sur [koprogo.be/transparence](https://koprogo.be/transparence)

**Format** :
```markdown
## Rapport Financier Octobre 2025

### Coûts Infrastructure
- VPS OVH (d2-2): 7.00€
- S3 OVH (200 GB): 2.00€
- DNS OVH: 0.10€
- Total: 9.10€

### Revenus
- 100 copropriétés × 1€: 100.00€
- Dépassements stockage: 15.00€
- Total: 115.00€

### Affectation Excédent (105.90€)
- Réserve sécurité (6 mois): 54.60€ (50%)
- Développement (salaires futurs): 32.76€ (30%)
- Infrastructure K3s (Phase 2): 10.92€ (10%)
- Fonds urgence: 7.62€ (10%)

### Statistiques
- Uptime: 99.94%
- Latency P99: 3.2ms
- CO2: 0.12g/req
- Support tickets: 3 (résolus en 48h)
```

---

## 🆓 Option Self-Hosted : Gratuit et Souverain

### Avantages Self-Hosted

**Coût : 0€ (uniquement coût serveur)**

| Avantage | Description |
|----------|-------------|
| **Gratuit à vie** | Aucun frais de licence, aucun abonnement |
| **Souveraineté totale** | Données sous votre contrôle exclusif |
| **Personnalisation** | Modification code source (AGPL-3.0) |
| **Pas de limites** | Stockage, utilisateurs, requêtes illimités |
| **GitOps automatique** | Mises à jour sécurité en 3 minutes |

### Prérequis Techniques

**Infrastructure minimale** :
- VPS : 1 vCPU, 2 GB RAM, 40 GB SSD (~7€/mois OVH)
- OS : Ubuntu 22.04 LTS
- Compétences : Terminal Linux, Git, Docker

**Installation automatique** :
```bash
git clone https://github.com/gilmry/koprogo.git
cd koprogo
make setup-infra  # Terraform + Ansible (20-30 min)
```

### Capacité Self-Hosted

Un VPS à 7€/mois peut héberger :
- **1,000-1,500 copropriétés** (charge légère)
- **50,000-100,000 utilisateurs**
- **Stockage local** : 40 GB (40,000 documents)
- **Performance** : P99 < 5ms maintenue

**Mutualisation possible** :
Un syndic gérant 100 copropriétés peut les héberger toutes sur un seul VPS à 7€/mois, soit **0.07€/copro/mois** (93% moins cher que le cloud KoproGo).

---

## 📊 Modèle Hybride : 20/80

### Répartition Prévue

**Objectif 2028** :
- **20% cloud KoproGo** : 400 copropriétés × 1.20€ = 480€/mois
- **80% self-hosted** : 1,600 copropriétés × 0€ = 0€

**Revenus cloud** financent :
1. **Développement** : 1 développeur temps partiel (2j/semaine)
2. **Infrastructure** : VPS + S3 + DNS
3. **Support** : Documentation, forum, email
4. **Réserves** : 6 mois de fonctionnement

### Évolution Tarifs avec l'Échelle

**Plus de copropriétés = Prix plus bas**

| Année | Copros Cloud | Coût Infra | Prix/Copro |
|-------|-------------|------------|------------|
| **2025** | 100 | 10€/mois | 1.00€ |
| **2026** | 400 | 20€/mois | 0.70€ |
| **2028** | 1,000 | 30€/mois | 0.50€ |
| **2030** | 2,000 | 40€/mois | 0.40€ |

**Mécanisme** : Chaque année, l'Assemblée Générale ASBL vote pour :
- **Baisser le prix de base** (si réserves suffisantes)
- **Améliorer les quotas** (plus de stockage inclus)
- **Investir dans de nouvelles features**

---

## 🌱 Impact Écologique du Modèle

### Comparaison Carbone

**Solution classique (SaaS WordPress)** :
- Serveur dédié par client : 50W × 8760h = 438 kWh/an
- Datacenter standard : 438 kWh × 0.3 kg CO2/kWh = **131 kg CO2/an**

**KoproGo Cloud (mutualisé)** :
- VPS partagé : 10W / 1,000 copros = 0.01W par copro
- Datacenter bas carbone (GRA11) : 0.01W × 8760h × 0.06 kg CO2/kWh = **0.0053 kg CO2/an**
- **Réduction : 99.996%** 🌱

### Politique Green IT

1. **Datacenter bas carbone** : OVH GRA11 (60g CO2/kWh vs 300g moyenne)
2. **Mutualisation maximale** : 1,000+ copros sur 1 VPS
3. **Architecture Rust** : 10x moins de CPU que Python/Node.js
4. **Progressive Web App** : Cache local, moins de requêtes réseau
5. **Backup intelligent** : Déduplication, compression

---

## 💼 Viabilité Financière ASBL

### Budget Prévisionnel 2025-2030

**Hypothèses conservatrices** :
- Croissance : 100 copros (2025) → 2,000 copros (2030)
- Répartition : 20% cloud, 80% self-hosted
- Prix moyen cloud : 1.20€/mois (avec dépassements)

| Année | Copros Cloud | Revenus/an | Coûts Infra | Développement | Excédent |
|-------|-------------|------------|-------------|---------------|----------|
| **2025** | 20 | 288€ | 120€ | 0€ (bénévole) | +168€ |
| **2026** | 80 | 1,152€ | 240€ | 0€ (bénévole) | +912€ |
| **2027** | 200 | 2,880€ | 360€ | 1,200€ (0.5 ETP) | +1,320€ |
| **2028** | 400 | 5,760€ | 480€ | 2,400€ (1 ETP) | +2,880€ |
| **2030** | 1,000 | 14,400€ | 600€ | 3,600€ (1.5 ETP) | +10,200€ |

**Réserves cumulées 2030** : ~15,000€ (soit 25 mois de fonctionnement)

### Scénarios de Crise

**Scénario 1 : Chute revenus cloud (-50%)**
- Impact : Réduction développement à 0.5 ETP
- Solution : Appel communauté, campagne dons

**Scénario 2 : Augmentation coûts infra (+100%)**
- Impact : Augmentation prix 1€ → 1.50€
- Vote Assemblée Générale requis

**Scénario 3 : Pic usage (×10)**
- Impact : Migration K3s anticipée (Phase 2)
- Financement : Réserves cumulées

---

## 🤝 Contribution Communautaire

### Modèles de Contribution

Au-delà de l'abonnement cloud, les utilisateurs peuvent contribuer :

1. **Contributions code** : Nouvelles features, bug fixes
2. **Traductions** : i18n (nl, fr, de, en, es, it)
3. **Documentation** : Tutoriels, guides, vidéos
4. **Support communautaire** : Forum, Discord, GitHub Issues
5. **Dons ponctuels** : Financement features spécifiques

### Programme "Copropriété Sponsor"

**Pour les grandes copropriétés (100+ lots)** :
- Sponsorship : 100€/an
- Avantages :
  - Logo sur site web KoproGo
  - Priorité support (email 24h)
  - Influence roadmap (vote features)
  - Quota cloud étendu (10 GB stockage)

---

## 📈 Comparaison Concurrence

### Marché Solutions Propriétaires

| Solution | Prix/mois | Stockage | Support | Souveraineté | CO2/an |
|----------|-----------|----------|---------|--------------|--------|
| **Vilogi** | 200-500€ | 5-50 GB | Phone 9-18h | ❌ Cloud US | ~50 kg |
| **Apronet** | 150-300€ | 10 GB | Email 48h | ❌ Cloud FR | ~40 kg |
| **Homeasy** | 100-200€ | 2 GB | Chatbot | ❌ Cloud BE | ~30 kg |
| **KoproGo Cloud** | **1-3€** | 0.5-∞ GB | Email 72h | ✅ EU/Local | **0.005 kg** |
| **KoproGo Self-Hosted** | **0€** | ∞ | Communauté | ✅ Total | **0.001 kg** |

**Économie moyenne** : **1,600-9,500€/an par copropriété** (soit 95-99% de réduction)

---

## 🎯 Conclusion : Économie Solidaire

Le modèle économique de KoproGo ASBL repose sur :

✅ **Prix d'entrée minimal** : 1€/mois pour 90% des copropriétés
✅ **Dépassements à prix coûtant** : Transparence absolue, zéro profit
✅ **Mutualisation communautaire** : Coûts dilués entre tous
✅ **Option gratuite self-hosted** : Liberté et souveraineté totales
✅ **Transparence comptable** : Factures publiques mensuelles
✅ **Gouvernance démocratique** : Assemblée Générale vote les prix
✅ **Viabilité long terme** : Réserves 6-12 mois garanties

**Notre engagement** : Le coût de KoproGo ne dépassera **jamais** 5€/mois par copropriété, quel que soit le succès du projet. Tout excédent sera réinvesti dans le développement, la communauté, ou redistribué via baisse de prix.

---

**Prochaine section** : [Business Plan Complet](BUSINESS_PLAN_BOOTSTRAP.md) - Stratégie 2025-2028, projections, équipe
