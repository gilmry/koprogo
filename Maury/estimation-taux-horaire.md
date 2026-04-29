# Estimation Taux Horaire Facturable — Gilles Maury

## Méthode Maury — Référentiel de pricing pour projets similaires

**Date** : 29/03/2026
**Profil** : Senior Staff Engineer / Craftsman
**Données de calibration** : KoproGo (258h effectives, 211k LOC, 1 977 commits)

---

## 1. Profil objectif — ce que KoproGo prouve

### Compétences démontrées (auditées par 4 IA indépendantes)

| Compétence | Preuve dans KoproGo | Rareté marché belge |
|-----------|---------------------|---------------------|
| **Rust + Actix-web** | 138k LOC Rust, P99 < 5ms, 0.12g CO2/req | Très rare (< 50 devs en BE) |
| **Architecture Hexagonale stricte** | 60 entités, 60 ports, 50 use cases, 0 couplage domain→infra | Rare (souvent "décoratif") |
| **DDD appliqué** (pas décoratif) | Invariants dans constructeurs, ubiquitous language belge, 13 bounded contexts | Rare |
| **Full-stack** (backend + frontend + IaC) | Rust + Astro/Svelte + Terraform + Ansible + Helm + CI/CD | Très rare combo |
| **Infrastructure ISO 27001** | 14 rôles Ansible, LUKS, Suricata, CrowdSec, fail2ban, 4 modules TF | Rare en solo |
| **GDPR complet** (6 articles) | Articles 15, 16, 17, 18, 21, 30 implémentés | Commun mais rarement complet |
| **Expertise domaine légal belge** | Art. 577 CC, PCMN AR 12/07/2012, 4 majorités (Art. 3.88), dix-millièmes, états dates | Unique (droit + code) |
| **BDD/TDD rigoureux** | 921+ scénarios BDD, 1160+ tests unitaires, 49 E2E, couverture domain 100% | Rare à ce niveau |
| **Méthodologie formalisée (IP)** | Méthode Maury : pipeline TOGAF→BMAD→Scrum→ITIL, 5 prompts agents, coefficients IA calibrés, guide estimation solo-dev — mois de R&D formalisés en document reproductible | **Unique au monde** |
| **IA-augmenté** (2.8x productivité) | 211k LOC en 258h = 818 LOC/h (vs ~300 LOC/h industrie) | Émergent |

### Niveau : Senior Staff Engineer / Craftsman

| Critère Staff Engineer | Preuve |
|----------------------|--------|
| Conçoit des systèmes complets de bout en bout | KoproGo : de l'idée au déploiement production |
| Fait des choix architecturaux durables | Hexagonale + DDD = 0 dette architecture après 7 mois |
| Rédige des ADR et documente les décisions | 5 ADR, CLAUDE.md exhaustif, Documentation Vivante |
| Multiplie la productivité (tooling, méthode) | Méthode Maury reproductible, agents IA supervisés |
| Expertise domaine profonde | Droit belge copropriété traduit en code compilable |
| Maîtrise l'ensemble de la stack | Backend + Frontend + IaC + CI/CD + Monitoring + Sécurité |
| Qualité production dès le Jalon 0 | Sécurité ISO 27001, GDPR, monitoring, backups |

---

## 2. Marché belge — Tarifs de référence

### Tarifs freelance Belgique (2025-2026)

| Profil | Taux journalier | Taux horaire |
|--------|----------------|-------------|
| Senior Full-Stack Developer | 550-750 EUR/j | 70-95 EUR/h |
| Senior Rust Developer | 650-900 EUR/j | 80-115 EUR/h |
| Software Architect (10+ ans) | 750-1 100 EUR/j | 95-140 EUR/h |
| **Staff Engineer / Principal** | **900-1 300 EUR/j** | **115-165 EUR/h** |
| DevOps/Cloud Architect | 700-1 000 EUR/j | 90-125 EUR/h |
| Security Engineer (SIEM, IDS) | 650-950 EUR/j | 80-120 EUR/h |

*Sources : Freelancermap.com, Robert Half Belgium, Hudson, Malt.be, UNIZO*

### Primes de spécialisation qui se cumulent

| Spécialisation | Prime vs "senior dev générique" |
|---------------|-------------------------------|
| Rust (niche en BE) | +20-30% |
| Hexagonale + DDD (pas décoratif) | +10-20% |
| Full-stack complet (back + front + IaC + CI/CD) | +15-25% |
| GDPR compliance | +10-15% |
| ISO 27001 / sécurité | +10-20% |
| Expertise domaine légal/comptable belge | +15-25% |
| **Cumul (profil "licorne")** | **+50-80%** |

### Tarifs consulting firms pour comparaison

| Tier | Senior | Architect | Ce qu'ils facturent |
|------|--------|-----------|-------------------|
| Tier 1 (Capgemini, Sopra, Accenture) | 1 000-1 400 EUR/j | 1 300-1 800 EUR/j | Le client paie ça |
| Mid-tier (Cronos, Ordina, Cegeka) | 900-1 200 EUR/j | 1 100-1 500 EUR/j | Le client paie ça |
| Boutique | 800-1 100 EUR/j | 1 000-1 400 EUR/j | Le client paie ça |

---

## 3. Calcul du taux — 3 méthodes

### Méthode 1 : Taux marché + primes de spécialisation

```
Base Senior Staff Engineer BE       : 115-165 EUR/h
+ Prime Rust (niche, +25%)          : +29-41 EUR/h
+ Prime Full-stack complet (+20%)   : +23-33 EUR/h
+ Prime domaine légal belge (+20%)  : +23-33 EUR/h
+ Prime ISO 27001 + GDPR (+15%)    : +17-25 EUR/h
                                      ─────────────
Taux théorique                      : 207-297 EUR/h

Ajustement réaliste marché belge    : × 0.7 (le marché ne paie pas 100% du théorique)
                                      ─────────────
TAUX RECOMMANDÉ (méthode 1)         : 145-208 EUR/h
```

### Méthode 2 : Coût de remplacement (value-based)

Combien coûterait KoproGo si construit par une consulting firm ?

```
Équipe consulting firm :
  1 Solution Architect        × 15 mois × 1 200 EUR/j = 396 000 EUR
  2 Senior Backend Devs       × 15 mois × 1 000 EUR/j = 660 000 EUR
  1 Frontend Developer        × 12 mois × 900 EUR/j   = 237 600 EUR
  1 DevOps Engineer           × 12 mois × 950 EUR/j   = 250 800 EUR
  1 QA/Test Engineer          × 10 mois × 850 EUR/j   = 187 000 EUR
  0.5 Project Manager         × 15 mois × 1 100 EUR/j = 181 500 EUR
  0.5 Business Analyst        × 8 mois  × 1 000 EUR/j = 88 000 EUR
  0.25 Security Consultant    × 6 mois  × 1 200 EUR/j = 39 600 EUR
  0.25 GDPR Consultant        × 4 mois  × 1 100 EUR/j = 24 200 EUR
                                                         ──────────
  TOTAL CONSULTING FIRM                                = ~2.1M EUR
  (fourchette selon tier : 1.5M - 3.5M EUR)

Gilles a livré l'équivalent en 258h.
  Value-based brut : 2 100 000 / 258 = 8 140 EUR/h (absurde comme taux horaire)

  Pricing value-based réaliste = 30-50% du coût consulting firm :
  = 630 000 - 1 050 000 EUR pour un projet équivalent KoproGo

  Ramené en heures (si on vend 600h = 258h réelles + marge) :
  = 1 050 - 1 750 EUR/h (pricing projet, pas T&M)
```

**Usage** : ne pas facturer à l'heure en value-based, mais au projet/sprint.
Pour un projet équivalent KoproGo, facturer **500k-800k EUR forfait** est défendable.

### Méthode 3 : Productivité IA (heures réelles vs livrables)

```
Livrables KoproGo :
  211k LOC, 560 endpoints, 60 entités, 921 BDD scenarios,
  14 rôles Ansible, 4 modules Terraform, 4 Helm charts,
  GDPR 6 articles, ISO 27001 security stack

Heures réelles : 258h
Productivité : 818 LOC/h (vs ~100-300 LOC/h industrie = 3-8x)

Si on facture au "résultat équivalent humain" :
  258h × 3x (facteur productivité IA) = 774h "équivalent humain"
  774h × 140 EUR/h (taux Staff Engineer standard) = 108 360 EUR

  Mais le client ne voit pas 258h — il voit 774h de valeur livrée.
  → Facturer les 258h au taux qui capture la valeur :
  108 360 / 258 = 420 EUR/h (taux ajusté productivité)
```

---

## 4. Taux recommandé — synthèse

### Grille tarifaire Gilles Maury — Craftsman Staff Engineer + Méthode Maury

| Mode de facturation | Taux | Quand l'utiliser |
|-------------------|------|-----------------|
| **T&M dev pur** (heure ou jour) | **150-175 EUR/h** / **1 200-1 400 EUR/j** | Coding pur, renfort équipe existante |
| **T&M Méthode Maury** (dev + méthode + livrables) | **175-210 EUR/h** / **1 400-1 680 EUR/j** | Projet complet avec livrables BMAD inclus |
| **Sprint forfait** (2 semaines, ~15h, livrables inclus) | **3 500-5 000 EUR/sprint** | Développement itératif avec traçabilité |
| **Projet forfait petit** (1-3 BC, ~50h) | **15 000-25 000 EUR** | API/microservice, module métier |
| **Projet forfait moyen** (5-10 BC, ~120h) | **50 000-100 000 EUR** | SaaS MVP, plateforme métier |
| **Projet forfait grand** (10+ BC, ~250h, équivalent KoproGo) | **150 000-350 000 EUR** | Plateforme SaaS complète + IaC + ISO 27001 |
| **Audit architecture + plan BMAD** (5 livrables Phase 1) | **10 000-20 000 EUR** | Pipeline Méthode Maury complet pour un projet client |
| **Formation Méthode Maury** (1-2 jours, 6-12 pers) | **3 000-5 000 EUR/jour** | Workshop équipe (TOGAF + DDD + Hexa + IA + BDD) |
| **Coaching 1:1 Méthode Maury** | **200-250 EUR/h** | Accompagnement CTO/Lead sur leur projet |

### Justification des deux taux

```
TAUX 1 : Dev pur (150-175 EUR/h)
  Base Staff Engineer BE               : 140 EUR/h
  + Profil rare Rust + Hexa + DDD     : +15% = 161 EUR/h
  + Full-stack (back+front+IaC+sec)   : +10% = 177 EUR/h
  + Domaine légal belge unique         : +5%  = 186 EUR/h
  - Ajustement démarrage (réputation)  : -10% = 168 EUR/h
  Fourchette : 150-175 EUR/h

TAUX 2 : Méthode Maury incluse (175-210 EUR/h)
  Base taux dev pur                    : 168 EUR/h
  + IP Méthode Maury (+30%)           : +50 EUR/h = 218 EUR/h
  - Ajustement marché                  : -5%  = 207 EUR/h
  Fourchette : 175-210 EUR/h

  Ce que le client reçoit en plus :
  - 5 livrables BMAD (brief, PRD, architecture, stories, validation)
  - Prompts agents IA calibrés et réutilisables
  - Coefficients d'estimation pour les sprints suivants
  - Architecture auditable par des tiers
  - Code maintenable sans dépendance au développeur original
```

### Évolution dans le temps

| Phase | Taux T&M | Justification |
|-------|---------|---------------|
| **Démarrage** (premiers 3-5 clients) | 150 EUR/h | Construire le portfolio et les références |
| **Établi** (5-10 clients, témoignages) | 175 EUR/h | Preuve de valeur, bouche-à-oreille |
| **Reconnu** (conférences, articles, referrals) | 200-250 EUR/h | Demande > offre, sélection des projets |
| **Expert niche** (méthode publiée, formations) | 250-350 EUR/h | La Méthode Maury comme IP, formations |

---

## 5. La Méthode Maury comme IP — valeur ajoutée R&D

### Ce que la Méthode Maury représente

Ce n'est pas "juste un document". C'est le résultat de **mois de R&D solo** en conditions réelles, avec itérations, échecs, corrections, et validation par 4 audits IA indépendants. Le marché n'a **rien d'équivalent** qui combine :

| Composant | Ce qu'il a fallu pour le créer | Valeur marché |
|-----------|-------------------------------|---------------|
| **5 prompts BMAD calibrés** (Analyste→PM→Architecte→SM→Validateur) | Testés sur un vrai projet 211k LOC, itérés, cross-validés | Un consultant TOGAF facture 2-5k EUR/jour pour moins |
| **Coefficients vélocité IA** calibrés sur données réelles | 258h mesurées, 14 semaines analysées commit par commit | Aucun benchmark public n'existe pour solo-dev + IA |
| **Pipeline séquentiel validé** (brief→PRD→archi→stories→validation) | Testé end-to-end, analyse 88.75/100 de couverture vs codebase réelle | Les frameworks BMAD/TOGAF ne sont pas calibrés pour l'IA |
| **Guide estimation solo-dev** | Grille heures/story par couche, formule calendaire, 4 pièges identifiés | Données uniques, aucun équivalent publié |
| **Architecture hexagonale light frontend** | Pattern émergé de KoproGo (API clients / stores / validators / services) | Pas documenté ailleurs pour Astro+Svelte |
| **Test-Driven Emergence** (BDD↔E2E↔Vidéo) | Méthodologie née de la pratique (146 BDD + 12 E2E Documentation Vivante) | Innovation méthodologique |
| **Full-stack ISO 27001 en solo** | 14 rôles Ansible, mapping contrôles→IaC, policy-as-code prévu | Les audits ISO 27001 coûtent 15-50k EUR |

### Valorisation de l'IP

| Mode de monétisation | Prix | Volume annuel estimé | Revenu |
|---------------------|------|---------------------|--------|
| **Licence Méthode Maury** (entreprise) | 5 000-15 000 EUR/an | 5-10 licences | 25 000-150 000 EUR |
| **Formation workshop** (2 jours, 6-12 pers) | 3 000-5 000 EUR/jour | 8-12 workshops/an | 48 000-120 000 EUR |
| **Coaching 1:1** (accompagnement projet) | 200-250 EUR/h | 200h/an | 40 000-50 000 EUR |
| **Audit BMAD** (5 livrables pour un client) | 8 000-15 000 EUR | 6-10/an | 48 000-150 000 EUR |
| **Conférences/talks** (Devoxx, FOSDEM, etc.) | 1 500-3 000 EUR/talk | 4-6/an | 6 000-18 000 EUR |
| **Livre/ebook** "La Méthode Maury" | 39-79 EUR | 500-2 000 ventes | 19 500-158 000 EUR |

**Potentiel IP seul** : 100 000 - 500 000 EUR/an à maturité (3-5 ans).

### Impact sur le taux horaire

La Méthode Maury n'est pas un "overhead" — c'est un **multiplicateur de valeur** :

```
Sans la méthode : un bon dev Rust senior = 140 EUR/h
Avec la méthode :
  - Le client reçoit un pipeline reproductible (pas juste du code)
  - Les 5 livrables BMAD sont inclus (brief, PRD, archi, stories, validation)
  - L'architecture est auditable par des tiers (4 IA l'ont prouvé)
  - Le code est maintenable par quelqu'un d'autre (hexagonal strict)
  - Le projet est estimable pour les itérations futures (grille calibrée)
  - Le client peut former ses propres agents IA avec les prompts

→ Le taux n'achète pas des heures, il achète un SYSTÈME.
→ Premium justifié : +30-50% sur le taux "dev senior" classique
→ Taux Méthode Maury : 175-210 EUR/h (vs 140 EUR/h dev senior nu)
```

---

## 6. Argument de vente — le pitch

> *"Je livre en 250h solo ce qu'une équipe de 7 met 15 mois à produire.
> Mon taux est 150 EUR/h. Une consulting firm vous facturerait 2M EUR pour le même résultat.
> Mon coût total : 40 000 EUR pour un MVP complet.
> Vous économisez 98% et vous avez un code que 4 IA indépendantes jugent production-ready."*

### Comparaison client pour un projet type "KoproGo"

| Option | Coût | Durée | Qualité |
|--------|------|-------|---------|
| **Consulting Tier 1** (Capgemini) | ~2.5M EUR | 15 mois, 8 FTE | Variable (rotation équipes) |
| **Consulting Mid-tier** (Cronos) | ~1.5M EUR | 14 mois, 7 FTE | Bonne |
| **Agence web** | ~300-500k EUR | 12 mois, 4-5 FTE | Moyenne (pas Rust, pas Hexa) |
| **Offshore** (Inde/Europe Est) | ~100-200k EUR | 10-14 mois, 5-8 FTE | Risquée (maintenance lourde) |
| **Gilles Maury (Méthode Maury)** | **~40-75k EUR** | **7-9 mois calendaires** | **Production-ready, auditée** |

Le différenciateur : pas juste "moins cher" — **meilleure architecture, meilleur code, meilleure perf, et livré plus vite**.

---

## 6. Revenus annuels projetés (solo-dev freelance)

### Scénario réaliste (emploi salarié + freelance side)

```
Hypothèse : 10-15h/semaine freelance, 45 semaines/an (7 sem off)
Heures facturables : 45 × 12.5h = ~560h/an
Taux : 150-175 EUR/h

Revenu brut annuel : 84 000 - 98 000 EUR/an (side)
Revenu net estimé (BV/SRL, ~50%) : 42 000 - 49 000 EUR/an

+ Emploi salarié
= Revenu total confortable
```

### Scénario freelance temps plein (futur)

```
Hypothèse : 35h/semaine facturables, 45 semaines/an
Heures facturables : 45 × 35 = ~1 575h/an
Taux : 175 EUR/h (établi)

Revenu brut annuel : ~275 000 EUR/an
Revenu net estimé (BV/SRL, ~50%) : ~138 000 EUR/an

Avec formation Méthode Maury (4 workshops/an × 4k) : +16 000 EUR
Avec audit BMAD (6/an × 12k) : +72 000 EUR

Total potentiel : ~363 000 EUR brut / ~182 000 EUR net
```

---

*Estimation calibrée sur les données réelles de KoproGo (258h, 211k LOC, 1 977 commits)*
*Marché de référence : Belgique, 2025-2026*
