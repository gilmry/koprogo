=============================================
Gouvernance KoproGo : Solo Dev → Coopérative
=============================================

:Version: 2.1
:Auteur: Gilles Maury - Fondateur KoproGo ASBL
:Statut: Phase Bootstrap (Solo dev bénévole)
:Constitution ASBL: Quand Jalon 3 atteint (500-1,000 copros)
:Voir aussi: :doc:`VISION` | :doc:`MISSION` | :doc:`ECONOMIC_MODEL` | :doc:`ROADMAP_PAR_CAPACITES`

----

.. note::
   **Évolution progressive** : La gouvernance KoproGo évolue selon 4 phases alignées sur la maturité du projet. Ce document décrit l'ensemble du parcours et les structures applicables à chaque phase.

----

Table des Matières
==================

1. `Évolution de la Gouvernance (Par Jalons)`_
2. `Introduction & Vision`_
3. `Phase 1: Bootstrap (Jalons 0-1)`_
4. `Phase 2: Fondateurs (Jalons 2-3)`_
5. `Phase 3: ASBL (Jalons 4-5)`_
6. `Phase 4: Coopérative (Jalons 6+)`_
7. `Membres de l'ASBL`_
8. `Conseil d'Administration`_
9. `Assemblée Générale`_
10. `Mainteneurs Techniques`_
11. `Contributeurs Externes`_
12. `Processus de Décision`_
13. `Financements & Transparence`_
14. `Propriété Intellectuelle`_
15. `Code of Conduct & Modération`_
16. `Résolution de Conflits`_

----

Évolution de la Gouvernance (Par Jalons)
=========================================

Vue d'Ensemble
--------------

KoproGo adopte une **approche progressive** de gouvernance, évoluant d'un projet solo vers une structure coopérative démocratique. Cette évolution garantit la viabilité à chaque étape tout en préservant les valeurs fondamentales.

.. list-table:: Synthèse des 4 Phases
   :header-rows: 1
   :widths: 15 20 25 20 20

   * - Phase
     - Jalons
     - Statut Juridique
     - Gouvernance
     - Financement
   * - **Bootstrap**
     - Jalons 0-1 (10-100 copros)
     - Projet personnel
     - Solo dev (Gilles Maury)
     - Bénévolat + IA (65€/mois)
   * - **Fondateurs**
     - Jalons 2-3 (200-1,000 copros)
     - Association de fait
     - 2-3 fondateurs
     - Autofinancement (revenus 5€/mois)
   * - **ASBL**
     - Jalons 4-5 (1,000-5,000 copros)
     - ASBL belge
     - AG + CA élu
     - Cloud + PropTech 2.0
   * - **Coopérative**
     - Jalons 6+ (5,000+ copros)
     - Coopérative agréée
     - Sociétaires
     - Économie sociale

Progression par Capacités
--------------------------

**Phase 1 - Bootstrap (Jalons 0-1)** :

- **Jalon 0** ✅ : Architecture hexagonale, 73 endpoints API (10-20 early adopters)
- **Jalon 1** : LUKS encryption, backups GPG, GDPR basique → **50-100 copropriétés**
- **Revenus** : 2,400€/an (100 copros × 40% cloud × 5€ × 12 mois)
- **Force de frappe** : Solo dev + IA

**Phase 2 - Fondateurs (Jalons 2-3)** :

- **Jalon 2** : Facturation TVA, recouvrement, conseils syndical → **200-500 copropriétés**
- **Jalon 3** : Audit trails, RBAC avancé, rapports financiers → **500-1,000 copropriétés**
- **Revenus** : 12,000-24,000€/an
- **Déclencheur ASBL** : Quand Jalon 3 atteint (500-1,000 copros)
- **Force de frappe** : 1-2 devs temps partiel, préparation statuts ASBL

**Phase 3 - ASBL (Jalons 4-5)** :

- **Constitution ASBL** : Quand 500-1,000 copros atteints (Jalon 3 validé)
- **Première AG** : Élection CA (3-7 membres), vote statuts
- **Jalon 4** : Intégrations bancaires, exports comptables → **1,000-2,000 copropriétés**
- **Jalon 5** : Mobile app, API publique, internationalisation → **2,000-5,000 copropriétés**
- **Revenus** : 48,000-120,000€/an
- **Force de frappe** : 2-4 ETP, embauche 1er salarié

**Phase 4 - Coopérative (Jalons 6+)** :

- **Déclencheur** : Quand 5,000 copros atteints (Jalon 5 validé)
- **Vote AG** : Consultation membres sur transformation coopérative
- Si approuvé : Conversion ASBL → Coopérative agréée
- Utilisateurs deviennent sociétaires (parts sociales)
- Gouvernance renforcée (1 personne = 1 voix)
- **Jalons 6-7** : PropTech 2.0 (IA, Blockchain, IoT, Carbon Credits)

----

Introduction & Vision
=====================

**KoproGo** est un projet open source de gestion de copropriété développé selon les principes d'**éthique, transparence et durabilité écologique**. Le projet évolue progressivement d'un projet solo vers une **coopérative** belge, garantissant un modèle économique au prix coûtant et une gouvernance démocratique.

Valeurs Fondamentales
----------------------

* 🌱 **Écologie** : Empreinte carbone minimale (< 0.5g CO₂/req, 96% réduction vs concurrents)
* 🔓 **Transparence** : Comptabilité publique, décisions ouvertes, roadmap communautaire
* 🤝 **Équité** : Prix fixe 5€/mois (baisse par vote AG selon surplus), pas d'actionnaires, réinvestissement 100%
* ⚖️ **Démocratie** : Gouvernance participative évolutive, vote des membres
* 🎓 **Pédagogie** : Documentation exhaustive, architecture exemplaire (hexagonale/DDD)

Objectif
--------

Rendre la gestion de copropriété **accessible, performante et respectueuse de l'environnement** via un modèle open source pérenne et une gouvernance démocratique progressive.

----

Phase 1: Bootstrap (Jalons 0-1)
===============================

Statut Juridique
----------------

**Projet personnel opensource** (pas de structure légale formelle)

Gouvernance
-----------

**Solo dev bénévole** : Gilmry

* Développement : 10-20h/semaine (side-project)
* Emploi temps plein ailleurs (sécurité financière)
* Décisions techniques : Solo (architecture hexagonale/DDD)
* Roadmap : Publique sur GitHub Projects

Objectifs Phase
---------------

* ✅ Validation MVP et product-market fit
* ✅ Premiers utilisateurs bêta (< 100 copropriétés)
* ✅ Documentation exhaustive (Sphinx RST)
* ✅ Architecture hexagonale solide
* ✅ Tests automatisés (unit, BDD, E2E)
* ✅ GitOps fonctionnel (déploiement < 3 min)

Financement
-----------

* **Bénévolat complet** : 0€ revenus
* Infrastructure : 7-15€/mois (VPS OVH perso)
* Temps investi : ~600-1000h sur 14 mois

Décisions
---------

* **Techniques** : Gilmry (Lead Dev)
* **Stratégiques** : Gilmry (Founder)
* **Feedback communauté** : GitHub Discussions (consultatif)

----

Phase 2: Fondateurs (Jalons 2-3)
=================================

Statut Juridique
----------------

**Association de fait** (2-3 fondateurs, pas de structure formelle)

Gouvernance
-----------

**Noyau fondateur** : 2-3 personnes

* Décisions par **consensus** entre fondateurs
* Spécialisations : Backend, Frontend, Community/Support
* Réunions hebdomadaires (synchronisation)
* Mainteneurs GitHub : Fondateurs + contributeurs actifs

Objectifs Phase
---------------

* ✅ Croissance 100 → 500 copropriétés
* ✅ Premiers revenus cloud (autofinancement)
* ✅ Équipe noyau stabilisée (2-3 fondateurs)
* ✅ Préparation statuts ASBL (rédaction, consultation juridique)
* ✅ Communauté contributeurs active (10+ contributeurs réguliers)

Financement
-----------

* **Revenus cloud** : 200-1,000 copros × 40% cloud × 5€ = 400-2,000€/mois
* **Autofinancement** : Couvre infrastructure + petits salaires ponctuels
* **Réinvestissement** : 100% excédents dans développement

Décisions
---------

* **Techniques** : Consensus fondateurs (Lead Dev = voix prépondérante si désaccord)
* **Stratégiques** : Vote fondateurs (majorité simple)
* **Roadmap** : Co-décidée avec communauté (GitHub Discussions)

Admission Fondateurs
--------------------

**Critères pour devenir co-fondateur** :

1. **Contribution significative** : 3+ mois de contributions régulières
2. **Compétences complémentaires** : Backend/Frontend/Community
3. **Alignement valeurs** : Engagement éthique, opensource, ASBL
4. **Disponibilité** : 10-20h/semaine minimum
5. **Consensus unanime** : Tous les fondateurs existants doivent approuver

----

Phase 3: ASBL (Jalons 4-5)
==========================

Statut Juridique
----------------

**ASBL belge** (Association Sans But Lucratif, loi 27 juin 1921 réformée 2019)

**Constitution prévue** : Quand Jalon 3 atteint (500-1,000 copros, revenus stabilisés)

Avantages ASBL
--------------

* ✅ **But non lucratif** : Pas de distribution de bénéfices, réinvestissement intégral
* ✅ **Personnalité juridique** : Autonomie contractuelle, patrimoniale, judiciaire
* ✅ **Fiscalité avantageuse** : Exonération impôt sociétés sur activités conformes
* ✅ **Crédibilité** : Structure légale reconnue, gage de sérieux
* ✅ **Protection fondateurs** : Responsabilité limitée

Constitution ASBL
-----------------

**Étapes** :

1. **Rédaction statuts** (objet social, gouvernance) - 2-4 semaines - 0€
2. **Acte authentique** (notaire) - 1 jour - ~250€
3. **Publication Moniteur belge** - 2-4 semaines - ~200€
4. **Numéro BCE** (entreprise) - Automatique - 0€
5. **Compte bancaire ASBL** - 1 semaine - 0-10€/mois

**Coût total** : ~450-500€ + 1-2 mois

**Siège social** : Bruxelles, Belgique

Objet Social
------------

    "L'association a pour objet la **promotion de l'accès démocratique aux outils numériques de gestion de copropriété**, par le développement, la maintenance et la diffusion de logiciels libres et open-source, ainsi que la fourniture de services d'hébergement et de support à prix coûtant.

    L'ASBL poursuit un but d'**intérêt général** et d'**éducation populaire** en :

    * Rendant accessible la technologie de gestion immobilière à tous
    * Favorisant la transparence par l'open-source
    * Formant des bénévoles aux pratiques professionnelles
    * Réduisant l'empreinte écologique de l'hébergement numérique
    * Promouvant le lien social via des modules communautaires optionnels"

Gouvernance ASBL
----------------

**Assemblée Générale (AG)** :

* **Fréquence** : Annuelle minimum (AGO)
* **Compétences** : Vote budget, élection CA, roadmap stratégique, modification statuts
* **Vote** : 1 membre = 1 voix
* **Quorum** : 50% membres présents (AGO), 2/3 (AGE)

**Conseil d'Administration (CA)** :

* **Composition** : 3-7 administrateurs élus
* **Mandats** : 3 ans renouvelables
* **Rôles** : Président, Vice-président, Trésorier, Secrétaire, Admins techniques
* **Réunions** : Trimestrielles minimum
* **Décisions** : Majorité simple

**Membres ASBL** :

* **Principe** : Client cloud = Membre automatiquement
* **Cotisation** : **5€/mois** (60€/an) - Identique au prix cloud
* **Catégories** :
  * Membres actifs cloud (5€/mois) : Accès cloud + Vote AG
  * Membres actifs self-hosted (5€/mois) : Vote AG uniquement
  * Self-hosted gratuit (0€) : Usage libre, pas de vote
* **Droits** : Vote AG (1 membre = 1 voix), éligibilité CA, accès rapports financiers

Objectifs Phase
---------------

* ✅ Croissance 500 → 5000 copropriétés
* ✅ Viabilité financière long terme
* ✅ Embauche 1-2 salariés (développement)
* ✅ Communauté mature (50-100 contributeurs)
* ✅ Reconnaissance utilité publique (si éligible)

Financement
-----------

* **Revenus cloud/membres** : 500-5000 copros × 5€ = 2,500-25,000€/mois (60,000-300,000€/an)
* **Services B2B** : Formation, support premium, intégrations
* **Subventions** : Horizon Europe, Digital Wallonia, Innoviris
* **Dons** : Particuliers et entreprises (optionnel)

.. note::
   Le prix de 5€/mois peut **baisser par vote AG** selon le surplus disponible (ex: 5€ → 4€ → 3€)

Décisions
---------

* **Stratégiques** : Vote AG (budget, statuts, roadmap majeure)
* **Opérationnelles** : CA (partnerships, embauches, investissements)
* **Techniques** : Lead Maintainer + mainteneurs (architecture, stack)

----

Phase 4: Coopérative (Jalons 6+)
=================================

Statut Juridique
----------------

**Coopérative agréée** (transformation optionnelle si vote AG favorable)

**Décision** : Soumise à l'Assemblée Générale 2029 (vote 2/3 requis)

Pourquoi une Coopérative ?
---------------------------

**Avantages vs ASBL** :

* ✅ **Sociétariat** : Utilisateurs deviennent sociétaires (parts sociales)
* ✅ **Gouvernance renforcée** : 1 personne = 1 voix (démocratisation totale)
* ✅ **Subventions** : Éligibilité économie sociale et solidaire
* ✅ **Implication** : Utilisateurs co-propriétaires du projet
* ✅ **Pérennité** : Structure coopérative non délocalisable

**Inconvénients** :

* ⚠️ **Complexité** : Gestion sociétaires plus lourde
* ⚠️ **Coûts admin** : Registre parts sociales, AG plus complexes
* ⚠️ **Réglementation** : Conformité économie sociale (contraintes)

**Condition sine qua non** : Vote favorable Assemblée Générale 2029

Gouvernance Coopérative
------------------------

**Structure** :

* **Sociétaires** : Tout utilisateur KoproGo peut devenir sociétaire (part sociale ~25-50€)
* **AG Coopérative** : 1 sociétaire = 1 voix (démocratie totale)
* **CA Coopérative** : Élu par sociétaires (5-10 membres)
* **Direction opérationnelle** : Directeur/rice salarié(e) + équipe

**Objectifs 2030+** :

* 10,000+ copropriétés utilisatrices
* 500-1000 sociétaires actifs
* 5-10 salariés permanents
* Écosystème plugins communautaires

----

Membres de l'ASBL
=================

*(Applicable Phase 3: ASBL)*

Qui Peut Devenir Membre ?
--------------------------

L'ASBL KoproGo est **ouverte** aux catégories suivantes :

1. **Utilisateurs actifs** : Syndics, gestionnaires utilisant KoproGo en production
2. **Sponsors/Donateurs** : Particuliers ou organisations soutenant financièrement
3. **Organisations** : Copropriétés, fédérations, associations
4. **Contributeurs** : Développeurs, rédacteurs, traducteurs (contribution significative)
5. **Sympathisants** : Toute personne adhérant aux valeurs

Processus d'Admission
----------------------

**Pour clients cloud** : Adhésion **automatique** lors de la souscription

1. **Souscription cloud** : 5€/mois → Devient membre ASBL automatiquement
2. **Droits immédiats** : Accès cloud + Vote AG (1 voix)
3. **Notification** : Email de bienvenue avec droits AG

**Pour membres self-hosted** (qui souhaitent voter) :

1. **Candidature** : Formulaire en ligne ou email
2. **Examen CA** : Vérification alignement valeurs
3. **Vote CA** : Majorité simple
4. **Cotisation** : 5€/mois (60€/an) pour droit de vote
5. **Droits** : Vote AG, pas d'accès cloud (autonomie self-hosted)

Droits des Membres
-------------------

* ✅ Participation et vote AG (1 membre = 1 voix)
* ✅ Éligibilité au CA
* ✅ Accès rapports financiers
* ✅ Participation discussions stratégiques
* ✅ Tarifs préférentiels services premium

----

Conseil d'Administration
=========================

*(Applicable Phase 3: ASBL et Phase 4: Coopérative)*

Composition
-----------

* **Nombre** : 3-7 administrateurs (ASBL), 5-10 (Coopérative)
* **Mandats** : 3 ans renouvelables
* **Révocation** : Vote AG (majorité 2/3)

Rôles
-----

.. list-table::
   :header-rows: 1

   * - Rôle
     - Responsabilités
   * - **Président(e)**
     - Représentation légale, convocation réunions
   * - **Vice-président(e)**
     - Suppléance, coordination projets spéciaux
   * - **Trésorier/ère**
     - Comptabilité, budgets, rapports financiers
   * - **Secrétaire**
     - PV, archives, correspondance officielle
   * - **Admins techniques**
     - Supervision développement, architecture

Réunions du CA
--------------

* **Fréquence** : Trimestrielle minimum
* **Convocation** : Email/GitHub 7 jours avant
* **Quorum** : Majorité présents
* **Décisions** : Majorité simple
* **PV** : Publics (sauf décisions confidentielles)

----

Assemblée Générale
==================

*(Applicable Phase 3: ASBL et Phase 4: Coopérative)*

Types
-----

* **AGO** (Ordinaire) : Annuelle obligatoire
* **AGE** (Extraordinaire) : Sur demande CA ou 20% membres

Compétences
-----------

* ✅ Vote budget annuel et approbation comptes
* ✅ Élection/révocation administrateurs
* ✅ Modification statuts (2/3)
* ✅ Dissolution (4/5)
* ✅ Validation roadmap stratégique
* ✅ Partenariats majeurs

Modalités de Vote
-----------------

* **1 membre = 1 voix** (égalité stricte)
* **Vote à distance** : Autorisé (visioconf, formulaire)
* **Procurations** : Max 3 par membre présent
* **Quorum** : 50% (AGO), 2/3 (AGE)

----

Mainteneurs Techniques
======================

*(Applicable toutes phases)*

Rôle
----

Les **mainteneurs** ont les **droits d'écriture** GitHub et assurent :

* Review et merge Pull Requests
* Gestion releases et versioning
* Supervision architecture hexagonale/DDD
* Résolution bugs critiques
* Mentorat nouveaux contributeurs

Comment Devenir Mainteneur ?
-----------------------------

**Nomination par CA** (ou Lead Dev en Phase 1-2) sur proposition Lead Maintainer

**Critères** :

* Contributions régulières et de qualité
* Maîtrise architecture hexagonale + Rust
* Respect Code of Conduct
* Disponibilité reviews (engagement minimum)

----

Contributeurs Externes
======================

*(Applicable toutes phases)*

Contributions Bienvenues
-------------------------

* 💻 Code (Rust, Astro/Svelte)
* 📚 Documentation (guides, traductions)
* 🧪 Tests (unit, BDD, E2E)
* 🎨 Design (UI/UX, logos)
* 🐛 Rapports bugs et suggestions

Developer Certificate of Origin (DCO)
--------------------------------------

**Obligation** : Tous les commits doivent être signés avec DCO

.. code-block:: bash

   git commit -s -m "feat: add amazing feature"

**Signification** : En signant, vous certifiez :

1. Avoir écrit ce code OU avoir le droit de le soumettre
2. Accepter publication sous AGPL-3.0
3. Comprendre que la contribution est publique et permanente

**Pourquoi DCO ?** Moins contraignant qu'un CLA, utilisé par Linux, Git, GitLab, Docker.

Processus de Contribution
--------------------------

1. **Fork** le dépôt
2. **Créer branche** (feature/, fix/, docs/)
3. **Développer** (architecture hexagonale)
4. **Tester** (make test)
5. **Commit avec DCO** (git commit -s)
6. **Pull Request** (description complète)
7. **Review** (feedback 48-72h)
8. **Merge** si approuvé

----

Processus de Décision
=====================

Décisions de Développement
---------------------------

**Modèle hybride** : Lead Dev/CA + Communauté

**Features Majeures** (breaking changes, nouveaux modules) :

1. **Proposition** : GitHub Discussion
2. **Votes communauté** : 2 semaines
3. **Analyse technique** : Lead + mainteneurs
4. **Avis AG** : Si impact financier/stratégique (Phase 3+)
5. **Décision CA** : Vote final
6. **Roadmap** : Ajout officiel

**Features Mineures** (bug fixes, améliorations UI) :

1. **GitHub Issue**
2. **Triage mainteneurs**
3. **Implémentation**
4. **Review + Merge**

Décisions Techniques
--------------------

* **Lead Maintainer** : Pouvoir décision architecture
* **Désaccord mainteneurs** : Vote CA technique
* **Changements majeurs** (migration langage) : Vote AG requis (Phase 3+)

----

Financements & Transparence
============================

Sources de Revenus
------------------

.. list-table::
   :header-rows: 1

   * - Source
     - Description
     - Phase Activation
   * - **SaaS Cloud/Membres**
     - 5€/mois par copropriété cloud (= cotisation membre)
     - Phase 2 (2026)
   * - **Membres self-hosted**
     - 5€/mois pour droit de vote AG (optionnel)
     - Phase 3 (2027)
   * - **Dons**
     - Liberapay, Open Collective (optionnel)
     - Toutes phases
   * - **Services B2B**
     - Formation, support, consulting
     - Phase 3 (2027)
   * - **Subventions**
     - EU, régions (Horizon, NGI)
     - Phase 3 (2027)

Transparence Financière
------------------------

**Engagement** :

* ✅ Rapports financiers annuels publics (site + GitHub)
* ✅ Budget prévisionnel voté AG et publié
* ✅ Dépenses détaillées par catégorie
* ✅ Comptes certifiés (si seuils dépassés)

**Exemple de Rapport** :

.. code-block:: text

   Rapport Financier 2028

   Revenus
   - Membres cloud : 96,000€ (1,600 copros × 5€/mois × 12)
   - Membres self-hosted : 2,400€ (40 membres × 5€/mois × 12)
   - Services B2B : 15,000€
   - Subventions : 10,000€
   Total : 123,400€

   Dépenses
   - Infrastructure : 4,200€
   - Salaires (1.5 ETP) : 60,000€
   - Admin/compta : 3,000€
   - Communication : 2,000€
   Total : 69,200€

   Résultat : +54,200€ (44% marge)

   Décision AG 2029:
   → Surplus > 25% pendant 2 trimestres
   → Vote baisse tarifaire: 5€ → 4€/mois (adoptée 87% pour)

Principe Prix Coûtant
----------------------

* **Objectif** : Couvrir frais réels, pas générer profit
* **Excédents** : Réinvestis (développement, infra, réserves)
* **Ajustement** : Si excédents structurels, baisse prix ou amélioration services

----

Propriété Intellectuelle
=========================

Droits d'Auteur
---------------

* **Code source** : Détenu par ASBL (dès Phase 3) ou fondateurs (Phase 1-2)
* **Contributions externes** : Restent propriété auteurs SAUF si DCO signé (licence AGPL accordée)
* **Transition** : Fondateurs cèdent droits patrimoniaux à ASBL lors constitution

Licence Open Source
-------------------

* **Licence** : **AGPL-3.0** (GNU Affero General Public License v3.0)
* **Pourquoi AGPL** : Copyleft fort pour SaaS (modifications restent opensource même en mode hébergé)
* **Modification licence** : Uniquement par vote AG 2/3 (Phase 3+)

  * Licences acceptables : GPL-3.0, EUPL-1.2, autres copyleft
  * Licences interdites : Propriétaire, permissives (MIT, Apache)

Marques
-------

* **Nom "KoproGo"** : Déposé par ASBL (protection)
* **Logo** : Propriété ASBL, usage libre CC BY-SA 4.0
* **Usage commercial** : Autorisé si mention "Propulsé par KoproGo"

----

Code of Conduct & Modération
=============================

Code of Conduct
---------------

Le projet adopte le **Contributor Covenant v2.1**.

**Principes** :

* Respect, bienveillance, inclusivité
* Tolérance zéro : harcèlement, discrimination, toxicité
* Sanctions graduées : avertissement → ban temporaire → ban permanent

Signalement
-----------

* **Email** : abuse@koprogo.com (traité sous 48h)
* **Anonymat** : Possible via formulaire web
* **Confidentialité** : Garantie

----

Résolution de Conflits
=======================

Conflits Techniques
-------------------

1. **Discussion ouverte** : GitHub Discussion
2. **Lead Maintainer tranche** : Basé sur principes hexagonaux/DDD
3. **Escalade CA** : Si désaccord persistant
4. **Fork autorisé** : Licence AGPL le permet

Conflits Interpersonnels
-------------------------

1. **Médiation informelle** : Mainteneurs
2. **Médiation formelle** : CA ou Comité Modération
3. **Sanctions** : Selon Code of Conduct

----

Révision de la Gouvernance
===========================

Ce document évolue avec le projet :

* **Fréquence** : Revue annuelle AG (Phase 3+)
* **Révision majeure** : Tous les 3-5 ans ou croissance significative
* **Vote requis** : 2/3 en AG

**Processus de Modification** :

1. GitHub Discussion (30 jours feedback)
2. Révision CA
3. Vote AG
4. Publication nouvelle version

----

Contacts
========

* **Email général** : contact@koprogo.com
* **Code of Conduct** : abuse@koprogo.com
* **GitHub** : https://github.com/gilmry/koprogo
* **Discussions** : https://github.com/gilmry/koprogo/discussions

----

Historique des Versions
========================

.. list-table::
   :header-rows: 1

   * - Version
     - Date
     - Changements
   * - 2.0
     - 2 nov. 2025
     - Ajout évolution Solo→Fondateurs→ASBL→Coop, conversion RST
   * - 1.0
     - 31 oct. 2025
     - Version initiale (Markdown)

----

**Voir aussi** :

* :doc:`ROADMAP_PAR_CAPACITES` - Roadmap stratégique complète avec jalons gouvernance
* :doc:`VISION` - Vision et modèle communautaire
* :doc:`MISSION` - Mission et gouvernance évolutive
* :doc:`ECONOMIC_MODEL` - Modèle économique et évolution structure

**KoproGo** - Gouvernance transparente pour un projet éthique et durable 🌱
