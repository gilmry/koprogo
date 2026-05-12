# Prompt ChatGPT - Comprendre la philosophie de travail de Gilles Maury

Copie-colle ce prompt dans ChatGPT (avec accès web activé) :

---

Tu es un analyste technique senior et consultant en stratégie IT. Je veux comprendre comment travaille **Gilles Maury**, consultant informatique basé à Bruxelles, en explorant son projet open source **KoproGo** sur GitHub : https://github.com/gilmry/koprogo

Ce projet est sa vitrine professionnelle. Il montre comment il aborde n'importe quel projet logiciel, même si ses missions de consulting ne sont pas toujours open source. La stack et les méthodes sont les mêmes.

**Explore le dépôt en profondeur** (README, CLAUDE.md, docs/, structure des dossiers, issues, milestones, commits, PR) et produis une analyse structurée sur les points suivants :

## 1. Architecture & Principes techniques
- Quelle architecture logicielle utilise-t-il ? (regarde la structure `backend/src/`)
- Comment applique-t-il le Domain-Driven Design (DDD) ?
- Comment sépare-t-il les responsabilités entre couches (domain, application, infrastructure) ?
- Quels choix de stack fait-il (Rust, Astro, Svelte, PostgreSQL) et pourquoi ces choix sont-ils révélateurs ?

## 2. Qualité & Rigueur
- Quelle est sa stratégie de tests ? (unit, intégration, BDD/Gherkin, E2E, benchmarks)
- Comment gère-t-il la sécurité ? (GDPR, encryption, WAF, 2FA, rate limiting...)
- Quels sont ses objectifs de performance ? (latence, throughput, mémoire)
- Comment utilise-t-il les outils de qualité de code ? (clippy, fmt, audit, coverage)

## 3. Philosophie de gestion de projet
- Comment organise-t-il sa roadmap ? (regarde docs/ROADMAP_PAR_CAPACITES.rst et les milestones GitHub)
- Sa devise semble être "on livre quand c'est prêt, pas selon des dates arbitraires" — comment cela se traduit-il concrètement ?
- Comment utilise-t-il les issues et les milestones GitHub pour suivre l'avancement ?
- Comment structure-t-il ses commits et ses branches ?

## 4. Conformité métier & expertise domaine
- Comment intègre-t-il les exigences légales belges (copropriété, PCMN comptable, GDPR) directement dans le code ?
- Est-ce que le code reflète une compréhension profonde du métier ou juste une implémentation technique ?
- Comment gère-t-il la complexité métier (workflow de votes, majorités qualifiées, tantièmes, SEL, convocations AG) ?

## 5. Vision long terme & scalabilité
- Comment le projet évolue-t-il de "fondations techniques" vers un produit complet ? (jalons 0 à 7)
- Comment prépare-t-il l'intégration future d'IoT, IA, blockchain sans sur-ingénierer le présent ?
- Quel est son engagement écologique (< 0.5g CO2/requête) et comment cela influence-t-il ses choix ?

## 6. Analyse critique : failles et axes d'amélioration
Sois honnête et constructif. Identifie :
- **Failles architecturales** : Y a-t-il des anti-patterns, du couplage excessif, des couches qui fuient, des responsabilités mal placées ?
- **Dette technique** : Des TODO/FIXME/HACK dans le code ? Du code mort ? Des dépendances obsolètes ou non maintenues ?
- **Couverture de tests** : Y a-t-il des modules critiques insuffisamment testés ? Des scénarios edge-case manquants ?
- **Sécurité** : Des failles potentielles (injection SQL, CORS mal configuré, secrets en dur, dépendances avec CVE connues) ? Regarde les `Cargo.toml`, les `.env.example`, les headers de sécurité.
- **Scalabilité** : Des goulots d'étranglement prévisibles (requêtes N+1, absence de cache, pagination manquante) ?
- **Documentation** : Des zones sous-documentées ? Des docs désynchronisées du code ?
- **DevOps** : Le CI/CD est-il robuste ? Y a-t-il des étapes manquantes (SAST, DAST, dependency scanning) ?
- **Frontend** : L'accessibilité (a11y) est-elle réellement implémentée ou juste documentée ? Le SSR/SSG est-il bien exploité ?

Pour chaque faille identifiée, évalue sa **sévérité** (critique/majeure/mineure) et propose une **recommandation concrète**.

## 7. Valorisation économique du projet

### 7.1 Estimation de la valeur de développement
En te basant sur les métriques observables du dépôt (lignes de code, nombre d'entités, endpoints, migrations, tests, infrastructure), estime :
- **Le volume de travail** en jours-homme (J/H) qu'un développeur senior aurait besoin pour produire ce résultat from scratch, en considérant : architecture, code, tests, documentation, infrastructure, CI/CD
- **Le coût de reconstruction** si une entreprise devait refaire ce projet à zéro avec une équipe classique (précise le nombre de profils nécessaires : backend, frontend, DevOps, QA, architecte, DBA)
- **La valeur en tant qu'asset logiciel** : que vaudrait cette codebase si elle était vendue ou licenciée ?

### 7.2 TJM (Taux Journalier Moyen) minimum justifiable
En te basant sur :
- Le niveau de compétence démontré (Rust, architecture hexagonale, DDD, sécurité, conformité légale, DevOps)
- La rareté du profil sur le marché belge/européen (développeur Rust + expertise métier copropriété + GDPR + comptabilité belge)
- Les benchmarks du marché freelance en Belgique et en Europe (2024-2025)

Estime :
- Le **TJM minimum** que ce profil devrait facturer (justifie avec des comparables marché)
- Le **TJM de positionnement optimal** (le sweet spot entre compétitivité et valorisation juste)
- La **fourchette haute** pour des missions critiques (fintech, legaltech, projets réglementés)
- Compare avec les TJM moyens de profils équivalents : architecte logiciel senior, développeur Rust senior, consultant DDD/hexagonal

### 7.3 Valeur commerciale du produit KoproGo
Évalue le potentiel commercial :
- **Marché adressable** : combien de copropriétés en Belgique ? En Europe francophone ? Quel est le prix moyen des solutions concurrentes (Syndic Manager, Cotoit, Matera, ImmoBelge) ?
- **Avantages concurrentiels** identifiables dans le code (performance Rust, conformité belge native, GDPR by design, SEL communautaire, gamification)
- **Revenus potentiels** : en SaaS (par copropriété/mois), quel pricing et quel ARR (Annual Recurring Revenue) pourrait-on viser avec 100, 500, 1000, 5000 copropriétés ?
- **Barrières à l'entrée** pour un concurrent qui voudrait répliquer cette stack : temps, coût, expertise nécessaire

### 7.4 Comparaison : faire appel à Gilles vs. une ESN (SSII)
Compare concrètement ce que ça coûterait de :

**Option A — Gilles Maury (freelance/consultant)**
- Coût estimé pour reproduire le scope actuel de KoproGo
- Délai estimé
- Avantages : expertise directe, pas d'overhead managérial, vision produit, continuité technique
- Risques : bus factor, disponibilité

**Option B — ESN classique (Accenture, Sopra Steria, CGI, NRB, Realdolmen...)**
- Équipe type nécessaire (nombre de profils, séniorité)
- TJM moyen par profil en ESN
- Coût total estimé (inclure overhead : management, coordination, turnover, montée en compétence)
- Délai estimé (avec les réalités d'une ESN : staffing, onboarding, turnover)
- Risques : turnover équipe, perte de contexte métier, dilution de la qualité, dépendance contractuelle

**Synthèse comparative** : tableau récapitulatif avec coût total, délai, qualité attendue, risques, et ROI pour le client.

## 8. Ce que ça dit de lui comme collaborateur
En synthèse, déduis de tout cela :
- Son niveau de rigueur et d'autonomie
- Sa capacité à comprendre et modéliser un domaine métier complexe
- Son approche de la documentation et de la transparence
- Ce qu'un client ou un employeur peut attendre en termes de livrables, de communication et de qualité
- Les types de projets pour lesquels il serait le plus pertinent
- **Son positionnement marché** : où se situe-t-il par rapport aux profils disponibles sur le marché belge/européen ?

**Important** : Base ton analyse uniquement sur ce que tu trouves dans le dépôt GitHub. Cite des fichiers, des issues ou des commits spécifiques quand c'est possible pour appuyer tes observations. Pour les estimations financières, utilise les benchmarks de marché publics (études Malt, Freelance.com, Hays, Robert Half, CodinGame pour les salaires Rust).

---
