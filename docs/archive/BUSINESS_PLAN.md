# KoproGo - Business Plan 2025-2028

**Version** : 1.0
**Date** : Janvier 2025
**Confidentiel** : Document stratégique

---

## Table des Matières

1. [Executive Summary](#executive-summary)
2. [Vision et Mission](#vision-et-mission)
3. [Produit et Proposition de Valeur](#produit-et-proposition-de-valeur)
4. [Analyse de Marché](#analyse-de-marché)
5. [Stratégie Commerciale](#stratégie-commerciale)
6. [Modèle Économique](#modèle-économique)
7. [Plan de Déploiement](#plan-de-déploiement)
8. [Projections Financières](#projections-financières)
9. [Équipe et Organisation](#équipe-et-organisation)
10. [Risques et Opportunités](#risques-et-opportunités)
11. [Besoins de Financement](#besoins-de-financement)
12. [Roadmap](#roadmap)

---

## Executive Summary

### Le Problème

Le marché de la gestion de copropriété en Europe et Afrique du Nord souffre de plusieurs problèmes majeurs :

1. **Lock-in technologique** : Migration difficile entre solutions (faiblesse #1 identifiée par ANGC 2024)
2. **Coûts élevés** : 15-30€/copropriété/mois en Europe
3. **Petites copropriétés négligées** : Segment 2-20 lots mal desservi
4. **Solutions obsolètes** : Technologies legacy (PHP, Java), UX datées
5. **Afrique du Nord sous-équipée** : Quasi-absence de solutions spécialisées

### La Solution

**KoproGo** est une plateforme SaaS moderne de gestion de copropriété, construite avec une architecture hexagonale (Rust + Actix-web + PostgreSQL + Astro) offrant :

- ⚡ **Performance extrême** : Latence P99 < 5ms (vs 50-200ms marché)
- 🔓 **Open-source & anti-lock-in** : Export total des données, API ouverte
- 💰 **Pricing disruptif** : 50-70% moins cher (3€/copro Europe, 0.50€/copro Maghreb)
- 🌱 **Éco-responsable** : < 0.5g CO2/requête
- 🚀 **Stack moderne** : Technologies de pointe vs legacy

### Marché Cible

**Dual-market strategy** :

- **Europe** (Phase 1) : Belgique → France → Luxembourg → Benelux
  - TAM : 5M+ copropriétés
  - Pricing : 15-49€/mois

- **Afrique du Nord** (Phase 1 bis) : Tunisie → Algérie → Maroc
  - TAM : 500k+ copropriétés (estimé)
  - Pricing : 5-15€/mois

**Total Addressable Market** : 5.5M copropriétés, 650M habitants

### Traction et Validation

**Produit** :
- ✅ MVP fonctionnel (backend + frontend)
- ✅ Architecture hexagonale complète
- ✅ Tests complets (Unit, Integration, BDD, E2E)
- ✅ CI/CD GitHub Actions
- ✅ Infrastructure K3s (Terraform + Ansible + Helm)

**Marché** :
- ✅ Analyse concurrentielle 5 pays (BE, FR, ES, IT, DE)
- ✅ Étude réglementaire complète
- ✅ Marché tunisien analysé (opportunité validée)
- ✅ Pricing strategy définie (dual-market)

### Projections Financières (3 ans)

| Année | Copropriétés | Clients | MRR | ARR | Coûts Infra | Marge |
|-------|--------------|---------|-----|-----|-------------|-------|
| **2025** | 120 | 25 | 400€ | 4,800€ | 60€ | 98% |
| **2026** | 800 | 150 | 3,500€ | 42,000€ | 720€ | 98% |
| **2027** | 3,000 | 500 | 15,000€ | 180,000€ | 3,240€ | 98% |

**Break-even** : Dès le 1er client (1 client = 15€/mois > 5€/mois infra)

### Besoins de Financement

**Seed Round : 50,000€**

- Développement produit : 20,000€ (6 mois dev full-time)
- Marketing & Sales : 15,000€
- Infrastructure & Ops : 5,000€
- Légal & Admin : 5,000€
- Runway : 5,000€

**Utilisation sur 12 mois** pour atteindre :
- 150 copropriétés actives
- 30 clients payants
- 500€ MRR
- Product-market fit validé

---

## Vision et Mission

### Vision

Devenir **la plateforme de référence** pour la gestion de copropriété en Europe et Afrique du Nord d'ici 2030, en combinant performance technique exceptionnelle, transparence open-source et accessibilité financière.

### Mission

**Démocratiser la gestion de copropriété** en offrant une solution performante, abordable et transparente qui élimine le lock-in technologique et redonne le contrôle des données aux utilisateurs.

### Valeurs

1. **Transparence** : Open-source, pas de lock-in, données exportables
2. **Performance** : Excellence technique (P99 < 5ms)
3. **Accessibilité** : Pricing abordable pour toutes les tailles
4. **Durabilité** : Éco-responsable (< 0.5g CO2/req)
5. **Fiabilité** : Architecture robuste, tests exhaustifs

---

## Produit et Proposition de Valeur

### Description du Produit

**KoproGo** est une plateforme SaaS complète de gestion de copropriété offrant :

#### Fonctionnalités Cœur (MVP)

1. **Gestion Immeubles**
   - Fiche complète (adresse, nombre de lots, année construction)
   - Documents attachés
   - Historique complet

2. **Gestion Lots/Units**
   - Numéro, étage, surface
   - Tantièmes
   - Liaison propriétaires

3. **Gestion Copropriétaires**
   - Informations personnelles (GDPR compliant)
   - Historique paiements
   - Communications

4. **Gestion Charges**
   - Création charges
   - Répartition automatique (tantièmes)
   - Suivi paiements
   - Relances automatiques

5. **Assemblées Générales**
   - Convocations
   - Ordres du jour
   - Procès-verbaux
   - Votes

6. **Documents**
   - Stockage centralisé
   - Catégorisation
   - Partage sécurisé

#### Fonctionnalités Avancées (Roadmap)

- **Paiements en ligne** : Intégration Stripe/PayPal
- **Notifications temps réel** : Email, SMS, push
- **Génération documents** : Contrats, quittances, PV
- **Reporting financier** : Tableaux de bord, exports comptables
- **Mobile app** : iOS/Android (Flutter)
- **IA intégrée** : Prédiction charges, détection anomalies

### Stack Technique

**Backend** :
- Rust 1.75+ (performance, sécurité)
- Actix-web 4.9 (framework web haute performance)
- PostgreSQL 15 (base relationnelle)
- SQLx (queries compile-time vérifiées)

**Frontend** :
- Astro 4.0 (SSG/SSR hybride)
- Svelte 4.0 (Islands Architecture)
- Tailwind CSS 3.0

**Infrastructure** :
- Docker + Docker Compose (dev)
- K3s (Kubernetes léger) (production)
- Traefik (reverse proxy + SSL)
- Prometheus + Grafana (monitoring)

### Différenciation Produit

| Critère | KoproGo | Concurrence |
|---------|---------|-------------|
| **Performance** | P99 < 5ms | 50-200ms |
| **Pricing** | 3€/copro | 15-30€/copro |
| **Lock-in** | Export total | Lock-in fort |
| **Open-source** | Oui | Non |
| **Stack** | Rust moderne | PHP/Java legacy |
| **Écologie** | < 0.5g CO2/req | Non mesuré |
| **Migration** | Import facile | Migration difficile |
| **API** | Ouverte, documentée | Limitée |

### Proposition de Valeur

#### Pour les Petites Copropriétés (2-20 lots)

> "Gérez votre copropriété comme un pro, sans le prix d'un pro"

- **Freemium généreux** : 1 copropriété gratuite
- **Prix abordables** : 15€/mois pour 5 copropriétés
- **Interface simple** : Pas besoin d'être expert
- **Support inclus** : Documentation + vidéos

#### Pour les Syndics Professionnels

> "La solution la plus rapide d'Europe, sans lock-in"

- **Performance extrême** : P99 < 5ms = productivité maximale
- **Scalabilité** : Gérez 100+ copropriétés sans ralentissement
- **API ouverte** : Intégrations personnalisées
- **Export total** : Vos données, votre contrôle
- **Pricing compétitif** : 2.45€/copro vs 20-30€ concurrent

#### Pour le Marché Maghreb

> "La première solution spécialisée copropriété pour l'Afrique du Nord"

- **Francophone** : Interface et support en français
- **Prix adaptés** : 0.50€/copro (vs 15-30€ solutions européennes)
- **Juridiquement compatible** : Code des Droits Réels
- **Infrastructure locale** : Hébergement adapté

---

## Analyse de Marché

### Taille du Marché

#### Europe

**Total Addressable Market (TAM)** :
- France : 740,000 copropriétés (~10M lots)
- Belgique : 100,000+ copropriétés
- Espagne : 850,000+ copropriétés
- Italie : 1,2M copropriétés
- Allemagne : 2M+ copropriétés
- **Total Europe** : ~5M copropriétés

**Serviceable Addressable Market (SAM)** :
- Marchés francophones : BE, FR, LU = 850,000 copropriétés
- Petites copropriétés (2-20 lots) : ~40% = 340,000 copropriétés

**Serviceable Obtainable Market (SOM) - 3 ans** :
- 0.5% SAM = 1,700 copropriétés
- Objectif conservateur : 3,000 copropriétés

#### Afrique du Nord

**Total Addressable Market (TAM)** :
- Tunisie : 50,000-100,000 copropriétés (estimation)
- Algérie : 200,000-300,000 copropriétés (estimation)
- Maroc : 150,000-200,000 copropriétés (estimation)
- **Total Maghreb** : ~500,000 copropriétés

**Serviceable Obtainable Market (SOM) - 3 ans** :
- 1% TAM Tunisie = 500-1,000 copropriétés
- Objectif conservateur : 300 copropriétés

### Valorisation du Marché

**Europe** :
- 850k copropriétés francophones × 20€/mois = **204M€ ARR**
- Notre objectif 3 ans (3k copros) = 0.35% part de marché

**Maghreb** :
- 500k copropriétés × 10€/mois = **60M€ ARR**
- Notre objectif 3 ans (300 copros) = 0.06% part de marché

**Marché total** : **264M€ ARR**

### Analyse Concurrentielle

#### Leaders Européens

**1. Vilogi** (France)
- **Position** : Leader SaaS français
- **Clients** : 1,000+ (estimation 20,000 copropriétés)
- **Pricing** : 20-30€/copro (estimation)
- **Forces** : 10+ ans expérience, multi-pays
- **Faiblesses** : UX datée, migration difficile

**2. Septeo ADB** (France)
- **Position** : Dominant marché français
- **Clients** : 2,300+ agences
- **Pricing** : 50-100€/mois (suite complète)
- **Forces** : Suite complète, forte présence
- **Faiblesses** : Cher, orienté grands comptes

**3. Copriciel** (Multi-pays)
- **Position** : International (FR, BE, CA, MA, DZ)
- **Clients** : Non divulgué
- **Pricing** : 15-30€/copro (estimation)
- **Forces** : Multi-pays, configurable
- **Faiblesses** : Migration difficile, lock-in

**4. Matera** (France)
- **Position** : SaaS moderne
- **Pricing** : 20-40€/copro (estimation)
- **Forces** : UX soignée, moderne
- **Faiblesses** : Marché français uniquement

#### Marché Belgique

- **Happy Syndic, Easy Syndic, Solvio** : Services traditionnels + outils internes
- **Peu de pure players SaaS** : Opportunité forte

#### Marché Tunisie

- **iMMOTECH** : N°1 local, focus agences immobilières (pas syndics)
- **Logis.tn** : Gestion immobilière générale
- **Absence totale** de solutions spécialisées copropriété/syndic

### Positionnement Concurrentiel

```
                 Prix
                  ↑
      Cher        |    Septeo ADB
                  |    (50-100€)
                  |
                  |    Vilogi, Copriciel
      Moyen       |    (20-30€)
                  |
                  |              KoproGo Europe
      Accessible  |              (15€/5 copros = 3€)
                  |
                  |    KoproGo Maghreb (5€/10 copros = 0.50€)
      Très bas    |
                  |
                  └──────────────────────────────────→
                       Fonctionnalités/Valeur

```

**Notre positionnement** : **Disrupteur high-value, low-cost**
- Fonctionnalités comparables aux leaders
- Performance supérieure (P99 < 5ms)
- Prix 50-70% inférieurs

---

## Stratégie Commerciale

### Segmentation Client

#### Segment 1 : Petites Copropriétés en Auto-gestion (Priorité 1)

**Profil** :
- 2-20 lots
- Syndic bénévole ou auto-gestion
- Budget limité (< 50€/mois)
- Besoin de simplicité

**Valeur pour nous** :
- Volume élevé (40% du marché)
- Churn faible (faible concurrence)
- Évangélistes (bouche-à-oreille)

**Stratégie d'acquisition** :
- SEO : "logiciel syndic gratuit", "gestion copropriété facile"
- Content marketing : Blog, guides pratiques
- Freemium agressif : 1 copropriété gratuite
- Partenariats associations de copropriétaires

#### Segment 2 : Syndics Professionnels Indépendants (Priorité 2)

**Profil** :
- Gèrent 10-50 copropriétés
- Recherchent efficacité opérationnelle
- Budget 200-500€/mois
- Sensibles à la performance

**Valeur pour nous** :
- ARPU élevé (49€/mois en moyenne)
- Contrats longs (1-3 ans)
- Références (portfolio visible)

**Stratégie d'acquisition** :
- LinkedIn Ads : Ciblage "syndic de copropriété"
- Démos personnalisées
- Argument performance : "P99 < 5ms = 10h gagnées/mois"
- Trial 30 jours avec import données concurrent

#### Segment 3 : Marché Tunisien (Priorité 1 bis)

**Profil** :
- Syndics professionnels émergents
- Sensibilité prix forte
- Besoin accompagnement
- Paiement virement/mobile money

**Valeur pour nous** :
- Marché vierge (first-mover advantage)
- Test stratégie low-cost
- Hub régional Maghreb

**Stratégie d'acquisition** :
- Facebook Ads Tunisie (CPC très bas)
- Partenariats agences immobilières locales
- Webinaires en français
- Beta gratuits (10 clients)

### Canaux d'Acquisition

#### Digital (70% budget)

**1. SEO (30% budget digital)**
- Mots-clés : "logiciel gestion copropriété", "syndic en ligne"
- Blog : 2 articles/semaine (guides pratiques)
- Backlinks : Annuaires immobiliers, forums

**2. Google Ads (25% budget digital)**
- Search : Mots-clés high-intent ("acheter logiciel syndic")
- Display : Retargeting
- Budget : 500€/mois → 50 leads qualifiés

**3. LinkedIn Ads (20% budget digital)**
- Ciblage : "Syndic de copropriété", "Gestionnaire immobilier"
- Content : Cas d'usage, comparatifs
- Budget : 300€/mois → 30 leads qualifiés

**4. Facebook Ads - Tunisie (15% budget digital)**
- Ciblage : Immobilier, syndics
- CPC : 0.10-0.20€ (vs 1-2€ Europe)
- Budget : 200€/mois → 100+ leads

**5. Content Marketing (10% budget digital)**
- Blog, vidéos YouTube
- Guides téléchargeables (lead magnets)
- Newsletter hebdomadaire

#### Partenariats (20% budget)

- **Associations de copropriétaires** : Affiliations, sponsoring événements
- **Notaires** : Recommandations lors ventes
- **Agences immobilières** : Co-marketing
- **Experts-comptables** : Intégrations

#### Sales Direct (10% budget)

- Démos personnalisées (syndics pro)
- Participation salons immobiliers
- Cold outreach LinkedIn (ciblé)

### Pricing Strategy

#### Europe (Belgique, France, Luxembourg)

**Plan Gratuit - "Starter"**
- 1 copropriété
- 10 lots maximum
- Fonctionnalités de base
- Support communautaire
- **Prix : 0€/mois**

**Plan "Auto-Syndic"**
- 5 copropriétés
- 50 lots par copropriété
- Toutes fonctionnalités
- Support email
- **Prix : 15€/mois** (3€/copro)
- **Target : Petites copropriétés**

**Plan "Pro"**
- 20 copropriétés
- Lots illimités
- Multi-utilisateurs (3)
- Support prioritaire
- Export avancé
- API access
- **Prix : 49€/mois** (2.45€/copro)
- **Target : Syndics professionnels**

**Plan "Enterprise"**
- 100+ copropriétés
- Lots illimités
- Multi-utilisateurs illimité
- Support dédié
- SLA garanti
- Déploiement on-premise (option)
- **Prix : Sur devis** (≥199€/mois)
- **Target : Grands syndics, bailleurs sociaux**

#### Maghreb (Tunisie, Algérie, Maroc)

**Plan Gratuit**
- 3 copropriétés (vs 1 Europe)
- **Prix : 0€/mois**

**Plan "Starter"**
- 10 copropriétés
- **Prix : 5€/mois** (0.50€/copro)

**Plan "Pro"**
- 30 copropriétés
- **Prix : 15€/mois** (0.50€/copro)

**Plan "Enterprise"**
- 100+ copropriétés
- **Prix : 50€/mois**

### Stratégie de Conversion

**Freemium → Paid** :
- Limite 1 copropriété (Europe) / 3 (Maghreb)
- Features avancées (reporting, exports) en paid
- Email nurturing : 1/semaine pendant 3 mois
- Taux de conversion cible : 15-20%

**Trial → Paid** :
- Trial 30 jours plan Pro
- Onboarding personnalisé
- Import données concurrent inclus
- Taux de conversion cible : 30-40%

**Upsell** :
- Auto-Syndic → Pro : Quand > 5 copropriétés
- Pro → Enterprise : Quand > 20 copropriétés
- Taux upsell cible : 20%/an

---

## Modèle Économique

### Structure de Coûts

#### Coûts Fixes Mensuels

**Infrastructure (OVH Cloud France)** :
- Année 1 : VPS OVH (1 vCPU / 2GB RAM) = 5€/mois
- Année 2 : VPS OVH (2 vCPU / 4GB RAM) = 15€/mois
- Année 3 : K3s OVH (dev) = 30€/mois
- Année 4 : K3s OVH (prod HA) = 270€/mois

**Services Cloud** :
- Frontend : Vercel (gratuit)
- Domain : 10€/an (~1€/mois)
- Email : SendGrid (gratuit jusqu'à 100 emails/jour)
- Monitoring : UptimeRobot (gratuit)

**Outils Dev** :
- GitHub : Gratuit (open-source)
- CI/CD : GitHub Actions (gratuit)

**Total Infrastructure Année 1** : **6€/mois** (72€/an)

#### Coûts Variables

**Support Client** :
- Email support : ~5min/ticket × 2 tickets/client/mois
- 100 clients = 16h/mois = 320€/mois (20€/h)
- **Coût support/client** : 3.20€/mois

**Paiements** :
- Stripe : 1.4% + 0.25€ par transaction
- Paiement moyen : 20€
- **Coût transaction** : 0.53€ (2.65%)

**Marketing/Acquisition** :
- CAC (Customer Acquisition Cost) cible : 50€
- Budget marketing : 1,000€/mois (Année 1)

#### Coûts Humains

**Année 1** (Bootstrap / Founder-led) :
- 1 Founder full-time (technique + product)
- Salaire : 0€ (equity)
- Freelance design : 2,000€ (one-time)

**Année 2** :
- 1 Founder full-time
- 1 Dev part-time (20h/semaine) : 2,000€/mois
- 1 Sales/Support part-time : 1,500€/mois
- **Total : 3,500€/mois**

**Année 3** :
- 2 Dev full-time : 8,000€/mois
- 1 Sales full-time : 3,000€/mois
- 1 Support : 2,000€/mois
- **Total : 13,000€/mois**

### Métriques Clés

**Lifetime Value (LTV)** :
```
LTV = ARPU × Gross Margin × (1 / Churn Rate)

Plan Auto-Syndic :
- ARPU : 15€/mois
- Gross Margin : 95% (14.25€)
- Churn : 5%/an (0.42%/mois)
- LTV = 14.25 / 0.0042 = 3,393€

Plan Pro :
- ARPU : 49€/mois
- Gross Margin : 95% (46.55€)
- Churn : 3%/an (0.25%/mois)
- LTV = 46.55 / 0.0025 = 18,620€
```

**Customer Acquisition Cost (CAC)** :
- Année 1 : 75€/client (organique + ads)
- Année 2 : 60€/client (SEO amélioration)
- Année 3 : 50€/client (word-of-mouth)

**LTV/CAC Ratio** :
- Auto-Syndic : 3,393€ / 75€ = **45:1** ✅
- Pro : 18,620€ / 75€ = **248:1** ✅

**Target sain : > 3:1** → Nous sommes excellents

**Payback Period** :
```
Payback = CAC / (ARPU × Gross Margin)

Auto-Syndic : 75€ / (15€ × 0.95) = 5.3 mois
Pro : 75€ / (49€ × 0.95) = 1.6 mois
```

**Target : < 12 mois** → Excellent

### Unit Economics

**Par client Plan Auto-Syndic (15€/mois)** :
```
Revenue            : 15.00€
- Infrastructure   : 0.06€ (5 copros × 0.012€ par copro)
- Support          : 3.20€
- Payment fees     : 0.43€
= Contribution     : 11.31€
- CAC amortized    : 1.25€ (75€ / 60 mois)
= Profit           : 10.06€/mois/client

Margin : 67%
```

**Par client Plan Pro (49€/mois)** :
```
Revenue            : 49.00€
- Infrastructure   : 0.24€ (20 copros × 0.012€)
- Support          : 5.00€ (plus de support)
- Payment fees     : 0.94€
= Contribution     : 42.82€
- CAC amortized    : 1.25€
= Profit           : 41.57€/mois/client

Margin : 85%
```

---

## Plan de Déploiement

### Timeline 3 Ans

#### T1 2025 : MVP Launch (Mois 1-3)

**Product** :
- ✅ MVP fonctionnel (déjà fait)
- Amélioration UX (design professionnel)
- Onboarding amélioré
- Documentation complète

**Go-to-Market** :
- Landing page optimisée (conversion)
- Blog : 10 articles SEO
- Beta lancée : 20 clients gratuits
- Feedback loops

**Résultats attendus** :
- 20 copropriétés actives
- 10 clients beta
- PMF initial validé

#### T2 2025 : Traction Europe (Mois 4-6)

**Product** :
- Paiements Stripe intégrés
- Exports comptables
- Mobile-responsive optimisé

**Go-to-Market** :
- Lancement public Belgique
- Google Ads : 500€/mois
- LinkedIn : 300€/mois
- Partenariats : 2 associations copropriétaires

**Résultats attendus** :
- 50 copropriétés actives
- 15 clients payants
- 300€ MRR

#### T3 2025 : Expansion Belgique + Test Tunisie (Mois 7-9)

**Product** :
- Notifications email automatiques
- Génération documents PDF
- Multi-devises (EUR, TND)

**Go-to-Market** :
- Scale Google/LinkedIn Ads : 1,500€/mois
- Lancement beta Tunisie : 10 clients
- SEO : Top 10 pour 5 mots-clés

**Résultats attendus** :
- 80 copropriétés (70 BE + 10 TN)
- 20 clients payants
- 400€ MRR

#### T4 2025 : Scale Belgique + France Prep (Mois 10-12)

**Product** :
- Mobile app (MVP)
- API publique documentée
- Intégrations comptables (Odoo, Sage)

**Go-to-Market** :
- Expansion France (soft launch)
- Partenariats comptables
- Content marketing : 2 articles/semaine

**Résultats attendus** :
- 120 copropriétés (100 BE + 15 TN + 5 FR)
- 25 clients payants
- 500€ MRR

**🎯 Objectifs Année 1 (2025)** :
- **120 copropriétés actives**
- **25 clients payants**
- **500€ MRR (6,000€ ARR)**
- **Product-Market Fit validé**

#### 2026 : Scale Multi-pays

**Q1-Q2** :
- Scale France : 20 nouveaux clients
- Tunisie : Lancement public
- Expansion Algérie (soft launch)
- Hiring : 1 dev + 1 support

**Q3-Q4** :
- Luxembourg launch
- Maroc soft launch
- Intégrations avancées
- Hiring : 1 sales

**🎯 Objectifs Année 2 (2026)** :
- **800 copropriétés**
- **150 clients payants**
- **3,500€ MRR (42,000€ ARR)**
- **Équipe : 4 personnes**

#### 2027 : Market Leadership

**H1** :
- Benelux complet (NL)
- Maghreb établi (TN, DZ, MA)
- Enterprise tier lancé
- Hiring : 3 personnes

**H2** :
- Espagne/Italie préparation
- Features IA (prédictions)
- Partenariats stratégiques
- Series A fundraising

**🎯 Objectifs Année 3 (2027)** :
- **3,000 copropriétés**
- **500 clients**
- **15,000€ MRR (180,000€ ARR)**
- **Équipe : 10 personnes**
- **Profitable**

---

## Projections Financières

### Hypothèses

**Acquisition** :
- CAC : 75€ (Année 1), 60€ (Année 2), 50€ (Année 3)
- Conversion freemium : 15%
- Conversion trial : 35%
- Growth rate : 30%/mois (Année 1), 15%/mois (Année 2), 10%/mois (Année 3)

**Retention** :
- Churn mensuel : 0.5% (excellent pour SaaS B2B)
- Churn annuel : ~6%
- Upsell rate : 20%/an

**Mix Clients** :
- 60% Auto-Syndic (15€/mois)
- 35% Pro (49€/mois)
- 5% Enterprise (200€/mois)
- ARPU moyen : 29€/mois

### Projections Détaillées - Année 1 (2025)

| Mois | Clients | Copros | MRR | Coûts | Profit | Cumul |
|------|---------|--------|-----|-------|--------|-------|
| Jan  | 5       | 15     | 75€ | 506€  | -431€  | -431€ |
| Fév  | 7       | 25     | 133€| 507€  | -374€  | -805€ |
| Mar  | 9       | 35     | 189€| 509€  | -320€  | -1,125€|
| Avr  | 11      | 45     | 245€| 711€  | -466€  | -1,591€|
| Mai  | 13      | 55     | 301€| 713€  | -412€  | -2,003€|
| Juin | 15      | 65     | 357€| 715€  | -358€  | -2,361€|
| Juil | 17      | 75     | 413€| 1,717€| -1,304€| -3,665€|
| Août | 19      | 85     | 469€| 1,719€| -1,250€| -4,915€|
| Sept | 21      | 95     | 525€| 1,721€| -1,196€| -6,111€|
| Oct  | 23      | 105    | 581€| 1,723€| -1,142€| -7,253€|
| Nov  | 24      | 115    | 637€| 1,724€| -1,087€| -8,340€|
| Déc  | 25      | 120    | 670€| 1,725€| -1,055€| -9,395€|

**Total Année 1** :
- Clients fin année : 25
- Copropriétés : 120
- MRR fin année : 670€
- ARR : 8,040€
- Coûts cumulés : 17,435€
- Perte cumulative : -9,395€

**Détail Coûts Année 1** :
- Infrastructure : 72€
- Marketing : 12,000€ (1,000€/mois)
- Design freelance : 2,000€
- Légal/Admin : 1,500€
- Divers : 1,863€

### Projections Détaillées - Année 2 (2026)

| Trimestre | Clients | Copros | MRR | Coûts | Profit | Cumul |
|-----------|---------|--------|-----|-------|--------|-------|
| Q1        | 50      | 250    | 1,450€ | 14,000€ | 3,350€ | -6,045€ |
| Q2        | 80      | 400    | 2,320€ | 16,000€ | 10,960€| 4,915€ |
| Q3        | 115     | 600    | 3,335€ | 18,000€ | 22,005€| 26,920€ |
| Q4        | 150     | 800    | 4,350€ | 20,000€ | 32,100€| 59,020€ |

**Total Année 2** :
- Clients fin année : 150
- Copropriétés : 800
- MRR fin année : 4,350€
- ARR : 52,200€
- Coûts annuels : 68,000€
- Profit : +59,020€ (cumulatif devient positif Q2)
- **Break-even atteint T2 2026**

**Détail Coûts Année 2** :
- Infrastructure : 180€
- Salaires (dev + support) : 42,000€
- Marketing : 20,000€
- Outils : 2,000€
- Divers : 3,820€

### Projections Détaillées - Année 3 (2027)

| Trimestre | Clients | Copros | MRR | Coûts | Profit | Cumul |
|-----------|---------|--------|-----|-------|--------|-------|
| Q1        | 250     | 1,200  | 7,250€ | 42,000€ | 44,750€| 103,770€ |
| Q2        | 350     | 1,800  | 10,150€| 45,000€| 76,450€| 180,220€ |
| Q3        | 425     | 2,400  | 12,325€| 48,000€| 98,975€| 279,195€ |
| Q4        | 500     | 3,000  | 14,500€| 51,000€| 122,000€| 401,195€ |

**Total Année 3** :
- Clients fin année : 500
- Copropriétés : 3,000
- MRR fin année : 14,500€
- ARR : 174,000€
- Coûts annuels : 186,000€
- Profit net : +342,175€ (cumulatif depuis création)

**Détail Coûts Année 3** :
- Infrastructure : 3,240€
- Salaires (6 personnes) : 156,000€
- Marketing : 18,000€
- Outils : 4,000€
- Divers : 4,760€

### Synthèse 3 Ans

| Métrique | 2025 | 2026 | 2027 |
|----------|------|------|------|
| **Clients** | 25 | 150 | 500 |
| **Copropriétés** | 120 | 800 | 3,000 |
| **MRR** | 670€ | 4,350€ | 14,500€ |
| **ARR** | 8,040€ | 52,200€ | 174,000€ |
| **Coûts** | 17,435€ | 68,000€ | 186,000€ |
| **Profit Annuel** | -9,395€ | +16,415€ | +160,575€ |
| **Cash Cumul** | -9,395€ | +7,020€ | +167,595€ |
| **Équipe** | 1 | 4 | 10 |

**Break-even** : T2 2026 (18 mois)

**Profitabilité** : T4 2026 (24 mois)

**ROI investissement 50k€** : 180% sur 3 ans

### Scénarios

#### Scénario Pessimiste (-30% growth)

| Année | Clients | MRR | ARR |
|-------|---------|-----|-----|
| 2025  | 18      | 470€| 5,640€ |
| 2026  | 105     | 3,045€| 36,540€ |
| 2027  | 350     | 10,150€| 121,800€ |

**Break-even** : Q4 2026 (24 mois)

#### Scénario Optimiste (+30% growth)

| Année | Clients | MRR | ARR |
|-------|---------|-----|-----|
| 2025  | 33      | 871€| 10,452€ |
| 2026  | 195     | 5,655€| 67,860€ |
| 2027  | 650     | 18,850€| 226,200€ |

**Break-even** : Q1 2026 (15 mois)

---

## Équipe et Organisation

### Fondateurs

**[Nom Fondateur]** - CEO & CTO
- Background : [À compléter]
- Responsabilités :
  - Product vision & roadmap
  - Architecture technique
  - Fundraising
  - Stratégie globale

### Organisation Année 1 (2025)

**Équipe Lean** :
- 1 Founder (full-time)
- 1 Designer freelance (ponctuel)

**Advisors** :
- Expert immobilier/syndic
- Expert SaaS/Growth

### Organisation Année 2 (2026)

**Équipe : 4 personnes**

- **CEO/CTO** : Founder (Product + Tech)
- **Senior Dev** : Backend/Infrastructure (part-time → full-time)
- **Support/Success** : Onboarding, support, retention
- **Sales/Marketing** : Acquisition, partnerships

### Organisation Année 3 (2027)

**Équipe : 10 personnes**

**Engineering (4)** :
- 1 CTO
- 2 Senior Devs (Backend + Frontend)
- 1 DevOps

**Product (2)** :
- 1 Product Manager
- 1 UX/UI Designer

**Go-to-Market (3)** :
- 1 Head of Sales
- 1 Sales Rep
- 1 Marketing Manager

**Operations (1)** :
- 1 Customer Success Manager

### Recrutement Timeline

| Rôle | Date | Salaire Annuel |
|------|------|----------------|
| Senior Dev (part-time) | T2 2026 | 24k€ |
| Support | T3 2026 | 18k€ |
| Sales | T4 2026 | 24k€ |
| Senior Dev 2 | T1 2027 | 48k€ |
| Product Manager | T2 2027 | 42k€ |
| DevOps | T3 2027 | 48k€ |
| Designer | T3 2027 | 36k€ |
| Head of Sales | T4 2027 | 48k€ |
| CSM | T4 2027 | 30k€ |

### Culture d'Entreprise

**Valeurs** :
- 🚀 **Excellence technique** : Code quality, performance
- 🔓 **Transparence** : Open-source, communication claire
- 🌱 **Sustainability** : Éco-responsabilité, long-term thinking
- 🤝 **Collaboration** : Feedback loops, team spirit
- 📚 **Learning** : Continuous improvement, knowledge sharing

**Remote-first** :
- Équipe distribuée Europe/Maghreb
- Async communication (pas de réunions inutiles)
- Autonomie et ownership

---

## Risques et Opportunités

### Risques Majeurs

#### 1. Risque Marché

**Risque** : Adoption lente, marché plus conservateur que prévu

**Probabilité** : Moyenne (30%)

**Impact** : Élevé

**Mitigation** :
- Freemium agressif (réduire friction adoption)
- Dual-market strategy (Europe + Maghreb)
- Import données concurrent (faciliter migration)
- Partenariats associations (crédibilité)

#### 2. Risque Concurrence

**Risque** : Réaction concurrents (baisse prix, copie features)

**Probabilité** : Élevée (60%)

**Impact** : Moyen

**Mitigation** :
- Avance technologique (P99 < 5ms difficile à copier)
- Open-source = barrière morale (concurrents propriétaires)
- Lock-in inversé (nos clients peuvent partir = confiance)
- Velocity produit élevée (Rust = rapidité développement)

#### 3. Risque Technique

**Risque** : Bugs critiques, downtime, perte données

**Probabilité** : Faible (15%)

**Impact** : Critique

**Mitigation** :
- Tests exhaustifs (Unit, Integration, E2E, BDD)
- CI/CD automatisé (GitHub Actions)
- Backups automatiques quotidiens
- Monitoring 24/7 (UptimeRobot, Prometheus)
- Incident response plan

#### 4. Risque Réglementaire

**Risque** : Changements réglementation copropriété (GDPR, lois locales)

**Probabilité** : Moyenne (40%)

**Impact** : Moyen

**Mitigation** :
- Veille juridique continue
- Architecture flexible (adaptation rapide)
- GDPR by design (privacy native)
- Conseiller juridique au board

#### 5. Risque Financement

**Risque** : Runway insuffisant, difficultés lever fonds

**Probabilité** : Moyenne (35%)

**Impact** : Critique

**Mitigation** :
- Bootstrap Année 1 (coûts très bas = 5€/mois infra)
- Break-even rapide (18 mois)
- Profitabilité Année 2 (indépendance financière)
- Métriques solides pour Series A (LTV/CAC 45:1)

### Opportunités

#### 1. Expansion Géographique Accélérée

**Trigger** : Traction forte Belgique/Tunisie

**Impact** : Expansion rapide France, Maghreb, Benelux

**Upside** : ARR 500k€+ dès Année 3

#### 2. Partenariats Stratégiques

**Trigger** : Intérêt grands acteurs immobiliers

**Opportunités** :
- White-label pour réseaux agences
- Intégration plateformes immobilières (SeLoger, Immoweb)
- Distribution via banques/assurances

**Upside** : Distribution massive, ARR 1M€+

#### 3. Features Premium (AI)

**Trigger** : Base installée 500+ clients

**Opportunités** :
- Prédiction charges (IA)
- Détection anomalies paiements
- Génération automatique documents juridiques
- Pricing tier "AI" : +50€/mois

**Upside** : ARPU +50%, marges supérieures

#### 4. Acquisition Concurrents

**Trigger** : Profitabilité établie, cash disponible

**Opportunités** :
- Acquérir solutions legacy (clients + tech)
- Migration clients vers KoproGo
- Consolidation marché

**Upside** : Accélération croissance, market leadership

---

## Besoins de Financement

### Seed Round : 50,000€

#### Allocation

**1. Développement Produit : 20,000€**
- 6 mois développement full-time
- Features prioritaires :
  - Paiements Stripe/PayPal
  - Génération documents PDF
  - Mobile app MVP
  - Notifications automatiques
  - API publique

**2. Marketing & Sales : 15,000€**
- Google Ads : 6,000€ (12 mois × 500€)
- LinkedIn Ads : 3,600€ (12 mois × 300€)
- Facebook Ads Tunisie : 2,400€ (12 mois × 200€)
- Content creation : 3,000€ (articles, vidéos)

**3. Infrastructure & Ops : 5,000€**
- Infrastructure 12 mois : 72€
- Outils (Stripe, SendGrid, etc.) : 500€
- Backups & monitoring : 200€
- Buffer : 4,228€

**4. Légal & Admin : 5,000€**
- Création société : 1,500€
- Contrats clients/fournisseurs : 1,500€
- Comptable année 1 : 1,200€
- Assurances : 800€

**5. Runway : 5,000€**
- Buffer imprévus

#### Timeline Utilisation

- **Mois 1-3** : Développement produit (7,000€)
- **Mois 4-6** : Launch + Marketing (10,000€)
- **Mois 7-9** : Scale marketing (15,000€)
- **Mois 10-12** : Growth + optimisation (13,000€)
- **Buffer** : 5,000€

### Métriques Success (12 mois)

**Objectifs avec 50k€ seed** :
- ✅ 150 copropriétés actives
- ✅ 30 clients payants
- ✅ 600€ MRR
- ✅ Product-Market Fit validé
- ✅ Prêt pour scale (Series A)

### Series A (Année 3) : 500,000€-1M€

**Timing** : T4 2027 (si trajectoire optimiste)

**Utilisation** :
- Hiring : 10 → 30 personnes
- Expansion Espagne/Italie
- R&D IA features
- Marketing scale : 50k€/mois

**Valorisation estimée** :
- ARR T4 2027 : 174k€
- Multiple SaaS : 5-10x ARR
- Valorisation : **870k€ - 1.74M€**

**Dilution** :
- Seed (50k€) : 10-15%
- Series A (750k€) : 20-25%
- Fondateurs post-Series A : 60-70%

---

## Roadmap

### Roadmap Produit

#### 2025 - MVP to Market

**Q1** :
- ✅ Onboarding UX amélioré
- Paiements Stripe
- Exports comptables (CSV, Excel)
- Notifications email automatiques

**Q2** :
- Génération documents PDF (PV, quittances)
- Multi-devises (EUR, TND)
- Tableau de bord analytics
- Mobile-responsive optimisé

**Q3** :
- API publique (v1)
- Webhooks
- Intégrations Zapier
- Mobile app (React Native MVP)

**Q4** :
- Paiements en ligne copropriétaires
- Portail copropriétaires
- Notifications SMS
- Intégrations comptables (Odoo, Sage)

#### 2026 - Scale & Features

**H1** :
- Mobile app complète (iOS + Android)
- Génération documents avancée (contrats, bilans)
- Reporting financier avancé
- Multi-utilisateurs avec rôles

**H2** :
- Features IA (prédictions charges)
- Détection anomalies paiements
- Chatbot support
- White-label option

#### 2027 - Market Leader

**H1** :
- Platform marketplace (intégrations tierces)
- Advanced analytics & BI
- Compliance automation (GDPR, lois locales)
- Enterprise features (SSO, SAML)

**H2** :
- AI document generation
- Predictive maintenance
- IoT integration (smart buildings)
- Blockchain proof-of-ownership

### Roadmap Géographique

#### 2025

- **T1** : Belgique (Bruxelles, Anvers, Liège)
- **T2** : Belgique nationale + Tunisie beta
- **T3** : Tunisie public launch
- **T4** : France (Paris, Lyon, soft launch)

#### 2026

- **H1** : France scale, Algérie soft launch
- **H2** : Luxembourg, Maroc soft launch

#### 2027

- **H1** : Pays-Bas, Maghreb établi
- **H2** : Espagne, Italie preparation

### Roadmap Équipe

#### 2025

- **T1** : Founder solo
- **T2** : + Designer freelance
- **T3** : Advisors board
- **T4** : Preparation hiring 2026

#### 2026

- **T1** : + Senior Dev (part-time)
- **T2** : Senior Dev full-time
- **T3** : + Support
- **T4** : + Sales

#### 2027

- **T1** : + Dev 2, + Product Manager
- **T2** : + DevOps
- **T3** : + Designer full-time
- **T4** : + Head of Sales, + CSM

**Équipe fin 2027** : 10 personnes

---

## Conclusion

### Pourquoi KoproGo va réussir

**1. Marché énorme et mal adressé**
- 5.5M copropriétés Europe + Maghreb
- 40% petites copropriétés négligées
- 264M€ TAM
- Croissance digitale sectorielle : 15%/an

**2. Produit disruptif**
- Performance 10x supérieure (P99 < 5ms)
- Prix 50-70% inférieurs
- Open-source = confiance
- Stack moderne vs legacy

**3. Stratégie commerciale solide**
- Dual-market (Europe mature + Maghreb émergent)
- Freemium agressif (acquisition low-cost)
- Unit economics excellents (LTV/CAC 45:1)
- Payback 1.6-5.3 mois

**4. Équipe exécution**
- Expertise technique Rust/SaaS
- Connaissance marché immobilier
- Culture produit et excellence

**5. Timing optimal**
- Digitalisation copropriétés post-COVID
- Afrique du Nord #1 e-gov (maturité tech)
- Open-source SaaS en croissance
- Sensibilité écologique croissante

### Prochaines Étapes

**Mois 1-3** :
1. Finaliser seed round (50k€)
2. Améliorer UX/design professionnel
3. Lancer beta publique Belgique (20 clients)

**Mois 4-6** :
4. Intégrer paiements Stripe
5. Scale marketing (Google + LinkedIn)
6. Atteindre 50 copropriétés

**Mois 7-12** :
7. Lancer Tunisie
8. Expansion France (soft launch)
9. Atteindre 120 copropriétés, 25 clients
10. Préparer Series A

---

**Contact** :

📧 Email : [votre-email]
🌐 Site : https://koprogo.com (à venir)
📱 LinkedIn : [profil]
💻 GitHub : https://github.com/gilmry/koprogo

---

**Annexes** :

- A : Analyse concurrentielle détaillée
- B : Projections financières complètes (spreadsheet)
- C : Pitch deck
- D : Documentation technique
- E : Lettres d'intention clients beta

---

*Document confidentiel - Ne pas distribuer sans autorisation*

**KoproGo** - Révolutionner la gestion de copropriété 🏢
