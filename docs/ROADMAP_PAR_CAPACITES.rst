=========================================================
KoproGo - Roadmap par Capacités
=========================================================

:Version: 6.0
:Modèle: Progression par jalons et métriques
:Auteur: Gilles Maury - Fondateur KoproGo ASBL
:Statut: Document de référence stratégique
:Dernière mise à jour: 15 mars 2026

.. note::

   **Mise à jour 2026-03-15** : Réorganisation des jalons et releases.
   MCP Tools (#252-265) et itsme (#48) repoussés de J3 vers J4.
   K3s infra (#266-268) déplacé vers J5.
   Schéma releases simplifié : v0.1.0 (J0-J3) / v0.2.0 (J4 partiel).
   WBS obsolètes supprimés. Voir `WBS_PROJET_COMPLET.rst <WBS_PROJET_COMPLET.rst>`_ pour le détail technique.

.. contents:: Table des matières
   :depth: 3
   :local:

=========================================================
Philosophie: Pas de Dates, Des Capacités
=========================================================

**Principe fondamental** : KoproGo avance quand les **conditions sont remplies**, pas selon un calendrier arbitraire.

Au lieu de promettre "Jalon 1 en décembre", nous disons:

**"Jalon 1 débloque 50-100 copropriétés quand Sécurité + GDPR basique sont implémentés"**

Pourquoi ce Modèle?
-------------------

**Dates fixes = pression artificielle**

* Stress d'équipe
* Qualité compromise
* Burnout fondateurs
* Promesses non tenues

**Capacités mesurables = progression saine**

* On livre quand c'est prêt
* Qualité préservée
* Équipe soutenable
* Confiance utilisateurs

=========================================================
Métriques de Progression: Les Deux Moteurs
=========================================================

Le succès de KoproGo se mesure par **deux métriques fondamentales**:

1. **Nombre de copropriétés hébergées**

   * Mesure: Adoption technique et proposition de valeur
   * Déclencheur: Features débloquent nouveaux paliers
   * Impact: Validation product-market fit

2. **Nombre de participants au projet**

   * Mesure: Contributeurs + sociétaires + bénévoles
   * Déclencheur: Vélocité de développement augmente
   * Impact: Viabilité communautaire

**Effet multiplicateur**: Plus de copropriétés → Plus de revenus → Plus de contributeurs → Plus de features → Plus de copropriétés → **Cercle vertueux**

Les Trois Moteurs d'Acquisition (Engouement)
---------------------------------------------

**L'engouement est notre moteur de croissance organique**. KoproGo peut être adopté de **trois façons différentes**, chacune alimentant les métriques de progression :

**1. Moteur Gestion** 🏗️ (Remplacement complet)

   * **Cible** : Petites copropriétés (< 50 lots), syndics bénévoles
   * **Proposition** : Self-hosted gratuit ou cloud 5€/mois
   * **Impact roadmap** : Valide product-market fit, génère revenus pour financer développement
   * **Lien technique** : Jalons 1-2 (GDPR, Conformité belge, Facturation)

**2. Moteur Communauté** 🤝 (Modules standalone)

   * **Cible** : Tous les immeubles (même avec syndic existant)
   * **Proposition** : SEL, Partage objets, Troc (compatible Vilogi/Apronet/Excel)
   * **Impact roadmap** : Adoption massive sans friction, viralité naturelle
   * **Lien technique** : Jalon 3 (Modules communautaires optionnels)

**3. Moteur Valeurs** 💚 (Sympathisants)

   * **Cible** : Individus alignés valeurs (écologie, opensource, solidarité)
   * **Proposition** : Cotisation 5€/mois = 1 voix AG (sans usage outils)
   * **Impact roadmap** : Communauté active, contributeurs, financement stable
   * **Lien technique** : Tous jalons (vote roadmap, allocation surplus)

**Synergie des 3 moteurs** :

.. code-block:: text

   Moteur Gestion → Revenus + Validation technique
            ↓
   Moteur Communauté → Adoption virale + Lien social
            ↓
   Moteur Valeurs → Contributeurs + Financement
            ↓
   ───────────→ Engouement collectif → Cercle vertueux

**Exemple concret** (Jalon 3 atteint) :

* **200 copros gestion complète** (40% cloud) → 4,000€/mois revenus
* **500 copros modules communautaires seuls** (20% cloud) → 2,500€/mois revenus
* **100 sympathisants cotisants** (sans usage) → 500€/mois revenus
* **50 contributeurs actifs** → Vélocité ×2-3

**Total** : 800 membres, 7,000€/mois, force de frappe 2-3 devs → Jalons 4-5 accessibles

**L'engouement remplace le marketing** : Pas de budget pub, croissance organique par recommandation (SEL viral, valeurs partagées, gestion performante).

=========================================================
⏱️ Force de Frappe: Tout Sera Fait, Mais QUAND?
=========================================================

Principe Fondamental: Progression Organique
--------------------------------------------

   **"Tout ce qui est dans cette roadmap sera livré tôt ou tard. La seule question est: QUAND?"**

   La réponse dépend de **la force de frappe collective**:
   - Plus de clients/membres → Plus de revenus → Plus de devs → Plus vite
   - Plus de partenaires → Plus de contributeurs → Plus de vélocité → Plus vite
   - Plus de communauté → Plus d'entraide → Moins de support → Plus vite

**Ce qui NE change PAS**: La séquence logique des jalons (on ne peut pas faire Jalon 3 avant Jalon 1)

**Ce qui CHANGE**: Le **rythme** auquel on passe d'un jalon à l'autre

Point de Départ Ultra-Réaliste (Aujourd'hui)
---------------------------------------------

**Configuration actuelle (Nov 2025)**:

.. code-block:: text

   👤 Équipe: 1 solo dev (Gilles)
   ⏰ Temps: 10-15h/semaine (side-project)
   💼 Emploi: Temps plein ailleurs (sécurité financière)
   💰 Budget infrastructure: 15€/mois max (VPS + domaine)
   🤖 Budget outils IA: 50€/mois min (tokens Claude/GPT)
   💸 Total payé de sa poche: 65€/mois
   👥 Clients: 0 (MVP en construction)
   📦 Revenus: 0€

   → Vélocité: 2-3 features/mois (simples)
   → Jalons: Séquentiels, selon disponibilité
   → Horizon: Tout sera fait, mais lentement

**Pourquoi 50€/mois en tokens IA?**

Les outils IA (Claude Code, GPT-4, Copilot) sont **stratégiques** pour compenser le manque de temps:

✅ **Accélération développement**: ×2-3 vélocité vs sans IA
✅ **Qualité code**: Architecture hexagonale maintenue, tests générés
✅ **Documentation**: Sphinx RST, guides utilisateur, API docs
✅ **Maturité technique**: Refactoring assisté, bonnes pratiques
✅ **Réduction bugs**: Reviews automatiques, suggestions sécurité

**Impact concret**:

.. code-block:: text

   1 solo dev (10h/semaine) SANS IA:
   → 1-2 features/mois, dette technique, docs limitées

   1 solo dev (10h/semaine) AVEC IA (50€/mois):
   → 2-3 features/mois, qualité pro, docs exhaustives
   → Équivalent à 2-3 devs juniors sans IA

**ROI**: 50€/mois IA = Économie 2-3 mois salaire dev (5,000-7,500€)

**Message clé**: C'est le point de départ. Chaque nouveau membre/client/partenaire **accélère** la vélocité ET rembourse l'investissement initial du fondateur.

Abaque d'Estimation: Comment la Vélocité Augmente
--------------------------------------------------

.. list-table:: Progression Force de Frappe (Modèle 5€/mois par copro cloud)
   :header-rows: 1
   :widths: 12 18 15 15 20 20

   * - Palier
     - Config Équipe
     - Budget/mois
     - Heures/sem
     - Vélocité
     - **Durée Jalon Moyen**
   * - **Jour 1**
     - Solo dev + IA (bénévole)
     - 65€/mois (15€ infra + 50€ IA)
     - 10-15h
     - **2-3 feat/mois**
     - **4-6 mois/jalon**
   * - **50 copros**
     - Solo + contributeurs
     - 100€ (50×40%×5€)
     - 20-30h
     - **5-8 feat/mois**
     - **2-3 mois/jalon**
   * - **100 copros**
     - Solo + 1 temps partiel
     - 200€ (insuffisant)
     - 30-40h
     - **8-12 feat/mois**
     - **6-8 semaines/jalon**
   * - **500 copros**
     - 2 devs temps partiel
     - 1,000€
     - 60-80h
     - **15-25 feat/mois**
     - **4-6 semaines/jalon**
   * - **2,000 copros**
     - 1-2 ETP
     - 4,000€
     - 140-160h
     - **30-50 feat/mois**
     - **2-4 semaines/jalon**
   * - **5,000 copros**
     - 3-4 ETP + R&D
     - 10,000€
     - 280-320h
     - **60-100 feat/mois**
     - **1-2 semaines/jalon**
   * - **10,000 copros**
     - 5-8 ETP + PropTech
     - 20,000€
     - 560+h
     - **100-200 feat/mois**
     - **< 1 semaine/jalon**

**Note**: Budget = Revenus cloud (copros × 40% cloud × 5€) - Coûts infra

Influence de la Communauté sur le Rythme
-----------------------------------------

**3 Leviers d'Accélération**:

1. **Clients/Membres** (revenus directs)

   .. code-block:: text

      100 copros cloud → 200€/mois → Solo dev continue bénévole
      500 copros cloud → 1,000€/mois → 1 dev temps partiel embauché
      2,000 copros cloud → 4,000€/mois → 1-2 ETP, vélocité ×5-10

   **Impact**: Financement direct du développement

2. **Partenaires** (ressources humaines)

   .. code-block:: text

      1 syndic partenaire → Feedback produit, beta-testing
      1 université partenaire → Stagiaires (2-3 mois), mémoires
      1 entreprise sponsor → Dev mis à disposition (1-2j/semaine)

   **Impact**: Force de frappe sans coût, expertise externe

3. **Contributeurs Open Source** (communauté)

   .. code-block:: text

      5 contributeurs réguliers → Traductions, bugfixes, docs
      10 contributeurs actifs → Features mineures, tests
      50+ communauté → Reviews, amélioration continue

   **Impact**: Distribution de la charge, qualité accrue

Scénarios de Progression Réalistes
-----------------------------------

**Scénario 1: Croissance Lente** (solo dev, peu de clients)

.. code-block:: text

   Année 1: Solo dev + IA, 50 copros → Jalon 1 livré
     Coûts: 65€/mois × 12 = 780€/an (poche du fondateur)
     Revenus: 50 × 40% × 5€ × 12 = 1,200€/an
     → Rembourse investissement initial + 420€ excédent

   Année 2: Solo dev + IA, 100 copros → Jalon 2 livré
     Revenus: 2,400€/an → Peut embaucher aide ponctuelle

   Année 3: 1 dev temps partiel, 500 copros → Jalon 3 livré
   Année 5: 2 ETP, 2,000 copros → Jalon 4 livré

   → Tout livré, mais sur 5 ans
   → Fondateur remboursé dès 50 copros (Année 1)

**Scénario 2: Croissance Organique** (communauté active)

.. code-block:: text

   Trimestre 1: Solo dev + 5 contributeurs, 50 copros → Jalon 1 livré
   Trimestre 3: 1 dev temps partiel, 200 copros → Jalon 2 livré
   Année 2: 1-2 ETP, 1,000 copros → Jalon 3 livré
   Année 3: 3-4 ETP, 5,000 copros → Jalon 4 livré

   → Tout livré en 3 ans

**Scénario 3: Croissance Rapide** (partenaires + financement)

.. code-block:: text

   Mois 3: Solo + 1 syndic partenaire + 10 contributeurs → Jalon 1 livré
   Mois 6: 2 devs temps partiel + université, 500 copros → Jalon 2 livré
   Année 1: 2 ETP + sponsor, 2,000 copros → Jalon 3 livré
   Année 2: 5 ETP + R&D, 10,000 copros → Jalon 4 livré

   → Tout livré en 2 ans

**Message clé**: La durée dépend de l'**oxygène** (clients, partenaires, contributeurs) apporté par la communauté.

Durée vs Qualité: Notre Engagement
-----------------------------------

   **"Nous livrons quand c'est prêt, pas selon un calendrier arbitraire."**

**Garanties qualité** (quel que soit le rythme):

✅ **Tests exhaustifs** (unit, BDD, E2E) avant chaque livraison
✅ **Sécurité d'abord** (GDPR, chiffrement, audits)
✅ **Documentation complète** (guides utilisateur, API)
✅ **Pas de dette technique** (architecture hexagonale maintenue)
✅ **Performance validée** (load tests avant production)

**Principe**: Une feature livrée lentement mais bien > Une feature rapide mais buggée

Paliers de Croissance Infrastructure
-------------------------------------

.. list-table::
   :header-rows: 1
   :widths: 15 20 15 15 15 20

   * - Palier
     - Capacité
     - Infrastructure
     - Coût/mois
     - Déclencheur
     - Temps Estimé
   * - **Nano**
     - 100 copros
     - VPS s1-2 (1c/2GB)
     - 6,30€
     - MVP lancé
     - Phase bootstrap
   * - **Micro**
     - 500 copros
     - VPS s1-4 (1c/4GB)
     - 17,70€
     - >80 copros
     - +3-6 mois
   * - **Small**
     - 1.500 copros
     - VPS b2-7 (2c/7GB)
     - 43,50€
     - >400 copros
     - +6-12 mois
   * - **Medium**
     - 3.000 copros
     - VPS b2-15 (4c/15GB)
     - 87€
     - >1.200 copros
     - +12-18 mois
   * - **Large**
     - 10.000+ copros
     - K8s cluster
     - 290€
     - >2.500 copros
     - +24-36 mois

**Principe clé**: L'infrastructure évolue **automatiquement** quand le seuil de 80% de capacité est atteint. Pas de date fixe, mais des **conditions mesurables**.

=========================================================
⚠️ PropTech 2.0: Modules Avancés Phase Tardive
=========================================================

Distinction IA Développement vs IA Produit
-------------------------------------------

**IA pour le développement** (50€/mois, dès Jour 1):

.. code-block:: text

   Claude Code, GPT-4, Copilot
   → Accélération développement ×2-3
   → Génération tests, refactoring assisté
   → Documentation Sphinx automatisée
   → **Utilisé PAR les devs, pas POUR les utilisateurs**

**IA Assistant Syndic** (Jalon 6+, nécessite 3-4 ETP):

.. code-block:: text

   Chatbot réglementaire copropriété
   → Répond aux questions des copropriétaires
   → Base de connaissance législation belge
   → Intégration GPT-4/Claude via OVH AI Endpoints
   → **Feature produit, pas outil dev**
   → Coût: +2€/mois par copropriété

Modules PropTech 2.0: Phase Leadership (Jalon 6+)
--------------------------------------------------

Les modules avancés suivants nécessitent **maturité technique ET équipe structurée (3-4+ ETP)**:

.. list-table::
   :header-rows: 1
   :widths: 20 30 20 30

   * - Module
     - Description
     - Jalon Recommandé
     - Prérequis Équipe
   * - **IoT Sensors**
     - Capteurs énergie/eau temps réel
     - **Jalon 6-7**
     - +IoT engineer, +DevOps
   * - **Blockchain Voting**
     - Votes AG immutables (Polygon)
     - **Jalon 7**
     - +Blockchain dev, audit sécurité
   * - **AI Assistant**
     - Chatbot réglementaire syndic
     - **Jalon 6**
     - +Data scientist, +MLOps
   * - **PSD2 Banking**
     - Réconciliation bancaire auto
     - **Jalon 6**
     - +FinTech expert, compliance
   * - **Carbon Credits**
     - Trading crédits carbone
     - **Jalon 7**
     - +Blockchain dev, legal

**Pourquoi Phase Tardive?**

❌ **Complexité technique**: Blockchain, IoT, ML nécessitent expertise pointue
❌ **Risque légal**: PSD2, GDPR data processing nécessitent compliance avancée
❌ **Dépendances**: Nécessitent base utilisateurs stable (>2,000 copros)
❌ **Coût R&D**: Budget R&D nécessite revenus >10,000€/mois
❌ **Support**: Features expérimentales nécessitent équipe dédiée

**Principe**: Livrons d'abord les features essentielles qui créent 80% de la valeur (Jalons 1-5), puis itérons sur les 20% différenciants (Jalons 6-7).

**Référence VISION.rst**: PropTech 2.0 modules optionnels (+1-2€/mois), pas inclus dans prix de base 5€/mois.

=========================================================
Jalons Produit: Features Débloquant la Croissance
=========================================================

Jalon 0: Fondations Techniques ✅
----------------------------------

**État**: Achevé (Automne 2025)

**Ce qui a été accompli**:

* ✅ Architecture hexagonale implémentée
* ✅ 73 endpoints API REST
* ✅ Tests E2E Playwright
* ✅ Load tests validés (99.74% success, 287 req/s)
* ✅ Documentation Sphinx publiée

**Capacité débloquée**: 10-20 early adopters (beta fermée)

**Conformité légale**: 30% (features CRUD de base)

**Ce que ça permet**:

* Valider l'architecture technique
* Tester avec premiers utilisateurs
* Identifier les besoins réels
* Construire une base solide pour la suite

Jalon 1: Sécurité & GDPR ⏳ (~83%)
------------------------------------

**Débloque**: 50-100 copropriétés (beta publique possible)

**État**: En cours - 83% complété (Mars 2026) — 15 fermées, 3 ouvertes

**Issues**:

* ✅ #39 : LUKS Encryption at-rest
* ✅ #40 : Backups automatisés GPG + S3
* ✅ #41 : Monitoring Prometheus + Grafana + Loki
* ✅ #42 : GDPR basique - export + effacement Articles 15 & 17
* ✅ #43 : Security hardening avancé - fail2ban, WAF/CrowdSec, IDS Suricata
* ✅ #78 : 2FA TOTP + rate limiting + JWT refresh tokens
* ✅ #90 : GDPR complémentaire - Articles 16, 18, 21
* ⏳ #271 : **CRITIQUE** — Quorum 50%+ validation AG (Art. 3.87 §5 CC)
* ⏳ #272 : **CRITIQUE** — 2e convocation si quorum non atteint (Art. 3.87 §5 CC)
* ⏳ #273 : Réduction vote mandataire (Art. 3.87 §7 CC) — ✅ done, à fermer

**Livrables complétés**:

* ✅ Données chiffrées au repos (LUKS AES-XTS-512)
* ✅ Backups quotidiens GPG + S3 (7j local, configurable S3)
* ✅ Monitoring complet (Prometheus, Grafana, Loki, Alertmanager)
* ✅ IDS Suricata + WAF CrowdSec + fail2ban
* ✅ Conformité GDPR Articles 15, 16, 17, 18, 21 & 30
* ✅ SSH hardening, kernel hardening, HSTS, CSP
* ✅ Login rate limiting (5 tentatives/15min)
* ✅ 2FA TOTP optionnel avec backup codes
* ✅ JWT refresh token rotation

**Livrables restants**:

* ⏳ Validation quorum 50%+ AG (wiring migration existante)
* ⏳ Workflow 2e convocation automatique
* ⏳ Vérification procurations (#273 — marqué done)

.. note::

   #48 (itsme/eID) déplacé en **Jalon 4** (mars 2026). Pas réaliste pour la 0.1.0.

**Conformité légale**: 40%

**Effort restant estimé**: ~6h (3 bugs legal)

Jalon 2: Conformité Légale Belge ✅
-------------------------------------

**Débloque**: 200-500 copropriétés (production ouverte)

**État**: COMPLÉTÉ (Novembre 2025 - Février 2026)

**Issues (toutes fermées)**:

* ✅ #79 : Plan Comptable Normalisé Belge PCMN (AR 12/07/2012, ~90 comptes pré-seedés)
* ✅ #73 : Workflow facturation complet (Draft → PendingApproval → Approved/Rejected, TVA 6%/12%/21%)
* ✅ #83 : Workflow recouvrement impayés (4 niveaux: Gentle → Formal → FinalNotice → LegalAction)
* ✅ #77 : Rapports financiers (Bilan + Compte de résultats)
* ✅ #80 : État Daté pour transferts immobiliers
* ✅ #81 : Budget annuel avec variance analysis
* ✅ #82 : Conseil de Copropriété (obligatoire >20 lots, dashboard)
* ✅ #29 : Validation quotes-parts (total = 100%, trigger PostgreSQL)
* ✅ #76 : Gestion documentaire complète (upload/download)
* ✅ #75 : Gestion complète AG (assemblées générales)
* ✅ #44 : Stratégie stockage documents
* ✅ #45 : Upload fichiers UI drag-and-drop

**Livrables complétés**:

* ✅ Comptabilité conforme arrêté royal 12/07/2012 (PCMN complet)
* ✅ Génération états datés
* ✅ Budgets prévisionnels avec variance analysis
* ✅ Dashboard conseil de copropriété avec alertes
* ✅ Relances automatiques 4 niveaux (pénalités taux légal belge 8%)
* ✅ Factures multi-lignes avec TVA belge
* ✅ Rapports financiers (bilan, compte de résultats, export CSV)
* ✅ Gestion documentaire complète

**Conformité légale**: 80%

**Bloquants levés**:

* ✅ **État daté**: Ventes de lots possibles
* ✅ **Conseil copropriété**: Copros >20 lots accessibles (60% du marché belge)
* ✅ **Comptabilité conforme**: Crédibilité auprès syndics professionnels

Jalon 3: Features Différenciantes ⏳ (~56%)
---------------------------------------------

**Débloque**: 500-1.000 copropriétés (différenciation marché)

**État**: En cours - 56% (Mars 2026) — 9 fermées, 7 ouvertes

**Release cible**: v0.1.0

**Issues fermées**:

* ✅ #46 : Voting Digital AG (3 types majorité, tantièmes/millièmes, procuration)
* ✅ #49 : Community Features 6 phases (SEL, Notice, Skills, Sharing, Booking, Gamification) — ~21,100 LOC
* ✅ #84 : Paiements en ligne (Stripe + SEPA, 38 endpoints)
* ✅ #51 : Sondages Board (4 types, anonymat, résultats)
* ✅ #99 : Community modules complets
* ✅ #133 : IoT Linky integration
* ✅ #134 : Work Reports & Technical Inspections
* ✅ #52/#91 : Devis entrepreneurs avec comparaison multi-entrepreneurs
* ✅ #96 : Energy campaigns (achat groupé)

**Issues ouvertes** (scope release 0.1.0):

* ⏳ #274 : BC15 — AG Visioconférence (backend ✅, BDD ✅, manque E2E + frontend)
* ⏳ #275 : BC16 — Backoffice prestataires PWA (backend ✅, BDD ✅, frontend ✅, manque E2E)
* ⏳ #276 : BC14 — Marketplace corps de métier + satisfaction (non implémenté)
* ⏳ #277 : Guide légal contextuel UI (LegalHelper, AG Wizard) (non implémenté)
* ⏳ #278 : Blog 18 articles RST (documentation)
* ⏳ #279 : BC17 — AGE Agile 1/5 quotités + concertation (backend ✅, BDD ✅, manque E2E + frontend)
* ⏳ #280 : Orchestrateur énergie neutre + CER (non implémenté)

.. note::

   **Mars 2026** : 13 issues MCP (#252-265) et #48 (itsme) déplacées vers **Jalon 4**.
   3 issues K3s infra (#266-268) déplacées vers **Jalon 5**.
   3 bugs legal (#271-273) restent dans **Jalon 1**.

**Livrables complétés**:

* ✅ Votes AG avec 3 types de majorité + tantièmes
* ✅ Sondages consultations rapides (4 types, anonymat)
* ✅ SEL monnaie locale (1h = 1 crédit, leaderboard)
* ✅ Bibliothèque objets + Réservation ressources + Skills
* ✅ Gamification (achievements, challenges, leaderboard)
* ✅ Paiements Stripe + SEPA avec méthodes stockées
* ✅ Work Reports & Technical Inspections
* ✅ IoT Linky + Energy Campaigns
* ✅ Frontend UI pour toutes les features fermées

**Livrables restants**:

* ⏳ **BC15 (P0, ~8h)** — AG Visioconférence + quorum combiné
* ⏳ **BC17 (P0, ~8h)** — AGE Agile : pétition 1/5 + auto-convocation
* ⏳ **BC16 (P1, ~6h)** — Backoffice prestataires (E2E manquant)
* ⏳ **BC14 (P1, ~20h)** — Marketplace prestataires + satisfaction
* ⏳ **WP-GUIDE (P1, ~10h)** — Guide légal contextuel UI
* ⏳ **WP-ENERGY (P1, ~16h)** — Orchestrateur énergie neutre
* ⏳ **WP-BLOG (P2, ~22h)** — 18 articles blog

**Conformité légale**: 80%

**Effort restant estimé**: ~90h (voir `WBS_RELEASE_0_1_0.md <WBS_RELEASE_0_1_0.md>`_)

Jalon 4: Automation & Intégrations ⏳ (~50%)
----------------------------------------------

**Débloque**: 1.000-2.000 copropriétés (scalabilité)

**État**: En cours - 50% (Mars 2026) — 14 fermées, 14 ouvertes

**Release cible**: v0.2.0

**Issues fermées**:

* ✅ #88 : Convocations AG automatiques (délais légaux 15j/8j, tracking email, rappels J-3)
* ✅ #89 : Carnet d'Entretien Digital
* ✅ #90 : GDPR complet Articles 16, 18, 21
* ✅ #85 : Ticketing maintenance (6 états, 5 priorités, SLA auto)
* ✅ #86 : Notifications multi-canal (Email, SMS, Push, In-App, 22 types)
* ✅ #91 : Devis travaux + comparaison multi-entrepreneurs (scoring belge)
* ✅ #92 : Page publique syndic SEO-friendly
* ✅ #110 : Groupements d'achat énergie
* ✅ #64, #65 : GDPR Articles 21, 16/18 (implémentation K3s)
* ✅ #67 : Documentation GDPR
* ✅ #71, #72 : Rôles étendus + RBAC (étude)
* ✅ #93 : Accessibilité WCAG 2.1 AA

**Issues ouvertes** (scope release 0.2.0):

* ⏳ #252 : MCP — Serveur SSE + JSON-RPC handler
* ⏳ #253 : MCP — Auth JWT + matrice rôle/outil
* ⏳ #254 : MCP — legal_search + majority_calculator
* ⏳ #255 : MCP — copropriete_info + list_coproprietaires
* ⏳ #256 : MCP — ag_create + ag_quorum_check + ag_vote + ag_generate_pv
* ⏳ #257 : MCP — comptabilite_situation + appel_de_fonds
* ⏳ #258 : MCP — travaux_qualifier
* ⏳ #259 : MCP — transmission_lot_dossier
* ⏳ #260 : MCP — alertes_list
* ⏳ #261 : MCP — documents_list + document_generate
* ⏳ #262 : MCP — Indexation base légale docs/legal/
* ⏳ #263 : MCP — Prompt système + intégration Claude
* ⏳ #265 : MCP — énergie (campagne_list, offre, comparer_tarif)
* ⏳ #48 : Authentification forte itsme/eID pour votes AG

**Livrables complétés**: voir issues fermées ci-dessus

**Livrables restants**:

* ⏳ **MCP AI Syndic** (13 issues, ~100h) — Serveur SSE, auth JWT, 10+ tools métier
* ⏳ **Auth itsme/eID** (#48, ~20h) — Intégration pour votes AG sécurisés

**Conformité légale**: 95%

**Effort restant estimé**: ~120h (MCP + itsme)

Jalon 5: Mobile & API Publique ⏳ (~29%)
------------------------------------------

**Débloque**: 2.000-5.000 copropriétés (expansion)

**État**: 29% (Mars 2026) — 2 fermées, 5 ouvertes

**Issues ouvertes**: #87 (PWA), #97 (BI), #98 (Mobile), #266-268 (K3s infra)

**Features**:

* PWA mobile responsive
* API publique v1 documentée (OpenAPI)
* Multi-langue NL/FR/DE/EN complet
* Intégrations comptables (Winbooks, Exact)
* K3s infrastructure (Terraform + Ansible + ArgoCD)
* Analytics & Dashboards

**Livrables**:

* Progressive Web App installable
* SDK Python/JS/PHP
* Webhooks pour événements
* Export Winbooks/Exact Online
* Digest hebdomadaire personnalisé
* KPIs syndic temps réel

**Conformité légale**: 100%

**Ce que ça débloques**:

* **Écosystème**: API publique → développeurs tiers
* **Intégrations**: Winbooks/Exact → syndics professionnels
* **Mobile**: PWA → adoption copropriétaires
* **International**: Multi-langue → expansion EU

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Équipe recommandée
   * - **Équipe projet**
     - 80-120h
     - 14-18 semaines
     - +Mobile dev +API architect
   * - **Communauté active**
     - 200+h
     - 6-8 semaines
     - 8 tracks parallèles

**Note**: Ce jalon nécessite une équipe structurée (mobile + API expertise)

Jalon 6: Intelligence & Expansion (PropTech 2.0 🚀)
---------------------------------------------------

**Débloque**: 5.000-10.000 copropriétés (leadership)

.. warning::
   **PropTech 2.0 Zone**: Ce jalon contient modules avancés nécessitant **maturité technique complète + équipe 3-4 ETP minimum**. Voir section "PropTech 2.0: Modules Avancés Phase Tardive" pour détails.

**Features avancées (PropTech 2.0)**:

* ⚠️ IA Assistant Syndic (GPT-4/Claude via OVH AI Endpoints)
* ⚠️ API Bancaire PSD2 (réconciliation auto, compliance FinTech)
* ⚠️ IoT Sensors (capteurs énergie/eau temps réel, MQTT + TimescaleDB)
* Marketplace Services Locaux
* Prédictions budgétaires (ML, modèles ARIMA)
* Multi-region (Benelux, adaptation législative)

**Livrables**:

* Chatbot réglementaire copropriété (base de connaissance législation belge)
* Import transactions bancaires automatique (API PSD2)
* Capteurs IoT déployables (énergie, eau, température)
* Annuaire prestataires vérifiés
* Modèles ML prévisions charges (historique 2+ ans)
* Adaptation législation NL/LU

**Ce que ça débloques**:

* **IA Syndic**: Réponses réglementaires instantanées (+2€/mois par copro)
* **PSD2**: Réconciliation bancaire automatique (gain 4-6h/mois syndic)
* **IoT**: Détection fuites eau, surconsommation énergie (+1€/mois par copro)
* **Marketplace**: Économie de plateforme (commissions prestataires)
* **Expansion**: Benelux → 3M copropriétés potentielles

**Prérequis CRITIQUES**:

✅ Base utilisateurs stable (>2,000 copros) pour feedback features avancées
✅ Revenus >10,000€/mois pour financer R&D PropTech
✅ Équipe structurée: +Data scientist, +IoT engineer, +FinTech expert, +MLOps
✅ Budget infrastructure IoT (MQTT broker, TimescaleDB, edge devices)
✅ Compliance PSD2 (DSP2, agrément FSMA Belgique)

**Effort estimé selon force de travail**:

.. list-table::
   :header-rows: 1
   :widths: 30 25 25 20

   * - Configuration
     - Heures/semaine
     - Durée estimée
     - Équipe
   * - **Équipe élargie**
     - 200+h
     - 18-24 semaines
     - 3-4 ETP: +Data scientist, +IoT engineer, +MLOps

**Note**: Jalon ambitieux nécessitant maturité produit + équipe spécialisée. **Ne PAS démarrer avant Jalon 5 complet** et revenus >10k€/mois.

Jalon 7: Platform Economy (PropTech 2.0 🔬)
-------------------------------------------

**Débloque**: 10.000+ copropriétés (scale planétaire)

.. warning::
   **PropTech 2.0 Expérimental**: Ce jalon contient features blockchain et trading carbone nécessitant **équipe 10-15 ETP + audits sécurité externes**.

**Vision long terme (PropTech 2.0)**:

* SDK multi-langages pour développeurs (Python, JavaScript, PHP, Ruby)
* Store modules tiers (marketplace plugins)
* ⚠️ **Blockchain Voting** (votes AG immutables, Polygon RPC)
* ⚠️ **Carbon Credits Trading** (tokenisation économies CO₂)
* White-label pour fédérations (multi-tenant SaaS)
* Interopérabilité EU (API standards CEN/CENELEC)

**Livrables**:

* SDK client libraries (pip, npm, composer, gem)
* Plugin marketplace avec revenue sharing
* Smart contracts Polygon pour votes AG (audit Trail of Bits)
* Carbon tokenization (ERC-20, vérifiable blockchain)
* White-label deployment automation (Terraform)
* Compliance GDPR multi-juridiction (EU 27)

**Ce que ça débloques**:

* **Écosystème complet**: Développeurs tiers créent valeur (+20-50 modules/an)
* **Blockchain Immutabilité**: Votes AG auditables éternellement (conformité ISO)
* **Carbon Economy**: Trading 840 tonnes CO₂/an économisées (mission ASBL)
* **Expansion EU**: France, Espagne, Italie (3M+ copros)
* **Leadership**: Référence européenne PropTech ESS

**Prérequis CRITIQUES**:

✅ Organisation mature (10-15 ETP, processus qualité ISO)
✅ Revenus >50,000€/mois pour financer R&D blockchain + audits sécurité
✅ Équipe blockchain: +Blockchain dev, +Smart contract auditor, +Legal compliance
✅ Budget audits sécurité externes (50-100k€/audit Trail of Bits)
✅ Agrément trading carbone (FSMA Belgique, AMF France)

**Effort estimé**: Organisation mature, 10-15 ETP, 24-36 semaines

**Note**: Jalon expérimental. **Blockchain Voting** et **Carbon Trading** nécessitent conformité réglementaire complexe (MiCA EU, DSP3). Ne démarrer que si surplus ASBL > 100k€/an.

=========================================================
Progression Conditionnelle: Force de Travail
=========================================================

Le Facteur Multiplicateur
--------------------------

**Principe clé**: La vélocité dépend de la force de travail disponible.

.. list-table::
   :header-rows: 1
   :widths: 20 15 20 15 30

   * - Configuration
     - Heures/sem
     - Jalons/an
     - Coût RH
     - Financement
   * - **Solo**
     - 10-20h
     - 2-3
     - 0€
     - Bénévolat
   * - **Duo**
     - 40-60h
     - 4-5
     - 18k€
     - Premiers revenus
   * - **Équipe**
     - 80-120h
     - 6-8
     - 60k€
     - Revenus cloud
   * - **Communauté**
     - 200+h
     - 10-12
     - 120k€
     - Surplus ASBL

**Scénarios de Croissance Réalistes**

**Scénario 1: Bootstrap Solo** (Gilles Maury bénévole, 10-20h/sem)

.. code-block:: text

   Mois 0-3 : Jalon 1 (Sécurité)
     → 50-100 copros débloquées
     → Premiers revenus: 250-500€/mois (5€ × 50-100)

   Mois 4-9 : Jalon 2 (Conformité)
     → 200-500 copros débloquées
     → Revenus: 1.000-2.500€/mois (5€ × 200-500)
     → Peut embaucher 1 dev temps partiel

   Mois 10-18 : Jalon 3 (Différenciation)
     → 500-1.000 copros
     → Revenus: 2.500-5.000€/mois (5€ × 500-1.000)
     → Duo fondateurs possibles

   Total: 18-24 mois pour atteindre 1.000 copropriétés

**Scénario 2: Duo Fondateurs** (40-60h/sem total)

.. code-block:: text

   Mois 0-2 : Jalon 1 (Sécurité)
     → 50-100 copros
     → Revenus: 250-500€/mois (5€ × 50-100)

   Mois 3-5 : Jalon 2 (Conformité)
     → 200-500 copros
     → Revenus: 1.000-2.500€/mois (5€ × 200-500)

   Mois 6-9 : Jalon 3 (Différenciation)
     → 500-1.000 copros
     → Revenus: 2.500-5.000€/mois (5€ × 500-1.000)

   Mois 10-14 : Jalon 4 (Automation)
     → 1.000-2.000 copros
     → Revenus: 5.000-10.000€/mois (5€ × 1.000-2.000)
     → Équipe 3-5 personnes possible

   Total: 14-18 mois pour atteindre 2.000 copropriétés

**Scénario 3: Équipe Projet** (80-120h/sem total)

.. code-block:: text

   Mois 0-1 : Jalon 1 (Sécurité)
     → 50-100 copros

   Mois 2-3 : Jalon 2 (Conformité)
     → 200-500 copros

   Mois 4-6 : Jalon 3 (Différenciation)
     → 500-1.000 copros

   Mois 7-9 : Jalon 4 (Automation)
     → 1.000-2.000 copros

   Mois 10-14 : Jalon 5 (Mobile + API)
     → 2.000-5.000 copros
     → Revenus: 10.000-25.000€/mois (5€ × 2.000-5.000)
     → Organisation 10+ personnes

   Total: 14-18 mois pour atteindre 5.000 copropriétés

**Principe**: Chaque scénario est valide. La rapidité dépend des ressources, pas de promesses marketing.

Effet Boule de Neige
---------------------

**Phase 1: Traction Initiale** (0-100 copros)

* Bénévolat fondateurs
* Feedback qualitatif
* Itérations rapides
* Coûts minimaux (< 10€/mois)

**Phase 2: Premiers Revenus** (100-500 copros)

* Revenus: 500-2.500€/mois (prix fixe 5€ × copros)
* Autofinancement partiel
* Contributions OSS augmentent
* Infrastructure scale up (20€/mois)

**Phase 3: Équipe Viable** (500-1.000 copros)

* Revenus: 2.500-5.000€/mois (5€ × copros)
* 1er emploi temps partiel possible
* Communauté active (10+ contributeurs)
* Infrastructure mature (50€/mois)

**Phase 4: Organisation Stable** (1.000-5.000 copros)

* Revenus: 5.000-25.000€/mois (5€ × copros)
* Équipe 3-10 personnes
* ASBL structurée
* Infrastructure scalable (100-300€/mois)
* **AG peut voter baisse tarifaire** si surplus > 25%

**Phase 5: Leadership ESS** (5.000+ copros)

* Revenus: 25.000+€/mois (5€ × copros, ou moins si AG vote baisse)
* Organisation 10-15 ETP
* Coopérative possible
* Expansion européenne
* **Prix réévalué démocratiquement** selon surplus

=========================================================
Vision Holistique: Tout S'Articule
=========================================================

L'Architecture des Dépendances
-------------------------------

**Pourquoi cette séquence de jalons?**

.. code-block:: text

   Jalon 0: Fondations
     ↓ Nécessaire pour
   Jalon 1: Sécurité
     ↓ Débloque confiance → Beta publique
     ↓ Sans ça: Pas d'utilisateurs réels
   Jalon 2: Conformité
     ↓ Débloque légalité → Production ouverte
     ↓ Sans ça: Illégal pour copros >20 lots (60% marché)
   Jalon 3: Différenciation
     ↓ Débloque impact social → Viralité
     ↓ Sans ça: Clone des concurrents
   Jalon 4: Automation
     ↓ Débloque efficacité → Scale
     ↓ Sans ça: Support non soutenable
   Jalon 5: Mobile + API
     ↓ Débloque écosystème → Expansion
     ↓ Sans ça: Platform monocanal
   Jalon 6-7: Intelligence & Platform
     ↓ Leadership européen ESS PropTech

**Chaque jalon est une fondation pour le suivant.**

Les Trois Piliers Interconnectés
---------------------------------

**1. Technique → Économique → Social**

.. code-block:: text

   Architecture Rust (Technique)
     ↓ Permet
   Infrastructure 99% moins chère (Économique)
     ↓ Permet
   Prix fixe 5€/mois (Social, voté démocratiquement)
     ↓ Permet
   Adoption massive (Technique)
     → Cercle vertueux
     → Si surplus > 25%, AG peut voter baisse (ex: 5€ → 4€)

**2. Conformité → Crédibilité → Adoption**

.. code-block:: text

   GDPR + PCN Belge (Conformité)
     ↓ Donne
   Crédibilité syndics pro (Crédibilité)
     ↓ Génère
   Bouche-à-oreille (Adoption)
     ↓ Finance
   Nouvelles features (Conformité++)
     → Cercle vertueux

**3. Communauté → Impact → Viralité**

.. code-block:: text

   Features SEL + Partage (Communauté)
     ↓ Créent
   Lien social + Économie 750k€ (Impact)
     ↓ Génèrent
   Témoignages utilisateurs (Viralité)
     ↓ Attirent
   Nouveaux membres (Communauté++)
     → Cercle vertueux

**L'interconnexion est la clé de la viabilité.**

Métriques de Succès Holistiques
--------------------------------

**Au lieu de KPIs 2030, des paliers mesurables:**

.. list-table::
   :header-rows: 1
   :widths: 20 20 20 20 20

   * - Palier
     - Copros
     - Participants
     - Impact CO₂
     - Économie
   * - **Validation**
     - 100
     - 10
     - -2t/an
     - 20k€/an
   * - **Viabilité**
     - 500
     - 50
     - -15t/an
     - 100k€/an
   * - **Impact**
     - 1.000
     - 100
     - -107t/an
     - 350k€/an
   * - **Leadership**
     - 2.000
     - 200
     - -214t/an
     - 750k€/an
   * - **Référence**
     - 5.000
     - 500
     - -840t/an
     - 2,35M€/an

**Chaque palier valide la proposition de valeur à une échelle supérieure.**

=========================================================
Avantages du Modèle par Capacités
=========================================================

Pour l'Équipe
-------------

✅ **Pas de pression calendaire**

* On livre quand c'est prêt
* Qualité préservée
* Burnout évité

✅ **Planification réaliste**

* Effort estimé selon force disponible
* Scenarios multiples (solo/duo/équipe)
* Flexibilité assumée

✅ **Motivation intrinsèque**

* Objectifs mesurables
* Progrès visible
* Impact concret

Pour les Utilisateurs
---------------------

✅ **Transparence totale**

* Pas de promesses vides
* Conditions de déblocage claires
* Dashboard progression public

✅ **Participation possible**

* Contributeurs accélèrent les jalons
* Feedback oriente les priorités
* Communauté co-construit

✅ **Confiance renforcée**

* Livraisons quand prêt = qualité
* Pas de "coming soon" éternel
* Roadmap honnête

Pour les Investisseurs / Subventions
-------------------------------------

✅ **Risques maîtrisés**

* Progression par paliers validés
* Métriques objectives
* Pas de promesses irréalistes

✅ **Scalabilité prouvée**

* Chaque palier démontre viabilité
* Infrastructure évolue avec adoption
* Modèle économique validé à chaque étape

✅ **Impact mesurable**

* CO₂ évité calculé par palier
* Économies générées documentées
* Social impact tracé

=========================================================
Plan d'Action: Prochaines Étapes (Février 2026)
=========================================================

**Progression spectaculaire**: Jalons 0-2 complétés, Jalons 3-4 à ~80%.

Priorité 1: Compléter Jalon 1 - Sécurité restante
---------------------------------------------------

**Objectif**: Débloquer beta publique (50-100 copropriétés)

**Complété** ✅:

* [x] LUKS Encryption at-rest
* [x] Backups GPG + S3
* [x] Monitoring Prometheus + Grafana + Loki
* [x] fail2ban + CrowdSec WAF + Suricata IDS
* [x] GDPR complet (Articles 15, 16, 17, 18, 21, 30)
* [x] SSH/kernel hardening, headers sécurité
* [x] Login rate limiting (5/15min)

**Restant** ⏳:

* [ ] Rate limiting global (Redis) - Issue #78
* [ ] JWT refresh token rotation - Issue #78
* [ ] 2FA optionnel TOTP - Issue #78
* [ ] Intégration itsme® - Issue #48 (en attente inscription)

**Effort restant**: 2-4 semaines (solo)

Priorité 2: Compléter Jalon 3 - Features restantes
----------------------------------------------------

**Objectif**: Finir contractor backoffice + PDF generation

**Complété** ✅: Voting, SEL, Payments, Polls, Community (6 phases), Frontend complet, Work Reports & Technical Inspections REST API (Issue #134)

**Restant** ⏳:

* [ ] Contractor Backoffice frontend - Issue #52
* [ ] PDF Generation étendue (PV AG, relevé charges, contrats) - Issue #47

**Effort restant**: 2-3 semaines (solo)

Priorité 3: Compléter Jalon 4 - GDPR, RBAC & Accessibilité
------------------------------------------------------------

**Objectif**: Documentation GDPR, Rôles organisationnels, RBAC + Conformité European Accessibility Act

**Restant** ⏳:

* [ ] Documentation GDPR & revue qualité - Issue #67
* [ ] Rôles Organization Admin / Building Manager - Issue #71
* [ ] RBAC granulaire (matrice droits dynamique) - Issue #72
* [ ] Audit WCAG 2.1 AA - Issue #93
* [ ] Corrections accessibilité (contraste, ARIA, navigation clavier, lecteur écran)

**Effort restant**: 4-6 semaines (solo)

Priorité 4: Qualité & Tests
----------------------------

**Issues ouvertes transversales**:

* [ ] Corriger 200+ erreurs compilation E2E tests - Issue #158
* [ ] Tests Playwright frontend - Issue #69
* [ ] Frontend tracking issue - Issue #197

Ensuite: Attaquer Jalon 5 (Mobile & API)
------------------------------------------

**Conditions de début**: Jalons 1-4 complétés

* [ ] PWA - Issue #87
* [ ] API publique v1 documentée
* [ ] Multi-langue NL/FR/DE/EN

=========================================================
Conclusion: Progression Saine et Soutenable
=========================================================

Ce Que Nous Promettons
-----------------------

✅ **Qualité avant vitesse**

* Chaque jalon est solide
* Tests exhaustifs
* Documentation complète

✅ **Transparence totale**

* Conditions de déblocage publiques
* Progression mesurable
* Pas de dates artificielles

✅ **Flexibilité assumée**

* Vélocité dépend de la force de travail
* Scenarios multiples (solo/équipe/communauté)
* Aucune pression calendaire

Ce Que Nous NE Promettons PAS
------------------------------

❌ **Dates fixes**

* "Livré en décembre" → Fausse promesse
* Préférons: "Livré quand sécurité validée"

❌ **Roadmap rigide**

* Priorités peuvent changer selon feedback
* Jalons peuvent fusionner/splitter
* Communauté influence direction

❌ **Croissance à tout prix**

* Privilégions qualité sur quantité
* Pas de dette technique
* Équipe soutenable

Notre Engagement
----------------

**"Nous livrons quand c'est prêt, pas quand le calendrier le dit."**

Chaque jalon débloque des capacités mesurables. Chaque palier valide la viabilité. Chaque feature crée de l'impact réel.

**La patience et la qualité battent toujours la vitesse et les promesses vides.**

=========================================================
Documents de Référence
=========================================================

* :doc:`VISION` - Vision macro et problème sociétal
* :doc:`MISSION` - Mission holistique et valeurs
* :doc:`ECONOMIC_MODEL` - Viabilité économique
* :doc:`GOVERNANCE` - Structure ASBL évolutive
* :doc:`PERFORMANCE_REPORT` - Validation technique

---

*Roadmap par Capacités KoproGo v5.0 - Mise à jour Février 2026*
*Modèle de progression mesurable et soutenable*
*Contact : contact@koprogo.com - GitHub : github.com/gilmry/koprogo*
