============================
Vision de Koprogo
============================

:Version: 4.0
:Date: 8 novembre 2025
:Voir aussi: :doc:`ROADMAP_INTEGREE_2025_2030` | :doc:`MISSION` | :doc:`ECONOMIC_MODEL`

Le Problème : Des Charges qui Montent en Flèche
================================================

Aujourd'hui, les charges de copropriété montent en flèche : **trente à cinquante pour cent au-dessus du raisonnable**.

Pourquoi ? Parce que chaque immeuble négocie seul—sans poids, sans données. Résultat : on laisse filer des millions, alors qu'en regroupant nos **cinq millions de logements**, on pourrait taper du poing ensemble et diviser les factures par deux.

Pas de magie, juste du collectif.

Notre Solution : Reprendre le Contrôle Ensemble
===============================================

On ne va pas attendre que le marché se corrige. On prend le problème à bras-le-corps :

✅ **Logiciel open source** (AGPL-3.0)
  Code public sur GitHub, auditable par tous, aucune surprise

✅ **Hébergé chez OVH pour trente-trois euros le mois**
  Infrastructure 100% française (Gravelines), souveraineté des données garantie

✅ **CO₂ quasi nul** (0.12g par requête)
  96% de réduction vs solutions actuelles, datacenter bas carbone

✅ **Un modèle associatif qui partage tout**
  ASBL belge, les économies reviennent aux usagers, pas à des actionnaires

Pas de patron, pas de commission cachée : les dividendes ? Ils reviennent aux usagers, pas à un conseil d'admin.

Gouvernance Transparente
========================

**Le vote ? Toujours en assemblée générale, comme avant.**

Mais si tu veux, on t'offre la **blockchain—optionnel, audité, transparent**. Pas pour remplacer les voisins, juste pour ceux qui préfèrent voir où va l'argent comme sur un relevé de banque.

* **1 membre ASBL = 1 voix** (pas de pondération par capital)
* **Conseil d'Administration élu** par l'Assemblée Générale
* **Budgets votés collectivement** (allocation surplus)
* **Transparence comptable trimestrielle** (rapports publics)

Le pouvoir est chez les usagers, pas chez des actionnaires.

L'Impact : Des Chiffres, Pas des Rêves
======================================

En 2030, on vise :

💰 **Neuf millions d'euros d'économies annuelles**
  * 8M€ logiciels propriétaires (80-95% de réduction)
  * 750k€ économie circulaire via SEL (Système d'Échange Local)
  * 600k€ consommation évitée (partage d'objets)

🌍 **Huit cent quarante tonnes de CO₂ en moins**
  * 50 tonnes infrastructure optimisée (Rust + datacenter France)
  * 790 tonnes features communautaires (partage, réutilisation, achats groupés)
  * Dépassement +57% vs objectif initial (-534t)

⚡ **Performance prouvée**
  * Latence P99 < 1s (752ms mesuré)
  * 287 req/s en charge soutenue
  * 0.12g CO₂/requête (vs 11.5g concurrents)

Pas des rêves : **des chiffres qu'on teste depuis octobre vingt-cinq**.

Le Pouvoir du Collectif : 5 Millions de Logements
==================================================

Notre force ? **Le nombre**.

En Belgique et en Europe, les copropriétés représentent :
* **Belgique** : 1,5 million de copropriétés
* **Europe** : 150 millions de personnes vivent en copropriété

Quand on négocie ensemble, on a du poids :
* Meilleurs prix fournisseurs (énergie, assurances, travaux)
* Mutualisation des coûts techniques (€33/mois pour tous)
* Partage des innovations (ce qui marche chez l'un profite à tous)

**Un immeuble seul ne pèse rien. Cinq millions ensemble changent la donne.**

Technologies au Service du Bien Commun
======================================

**Stack Technique** :

* **Backend** : Rust 1.83 + Actix-web 4.9 + PostgreSQL 15
* **Frontend** : Astro 4.x + Svelte 4.x (PWA offline-first)
* **Infrastructure** : Terraform + Ansible + GitOps (OVH Cloud)
* **Architecture** : Hexagonale (DDD) avec tests exhaustifs

**Hébergement 100% France** :

* VPS OVH s1-2 @ Gravelines (datacenter 60g CO₂/kWh)
* PropTech optionnel (IA, Blockchain, IoT) sur infrastructure mutualisée
* Backups chiffrés GPG + stockage S3 OVH
* Monitoring temps réel (Prometheus + Grafana)

**PropTech 2.0** (modules optionnels, activables par copropriété) :

* **AI Assistant** : OVH AI Endpoints @ Gravelines (+2€/mois si activé)
* **Blockchain Voting** : Polygon RPC sur VPS (+1€/mois si activé)
* **IoT Sensors** : MQTT + TimescaleDB (+1€/mois si activé)
* **Energy Platform** : Achats groupés énergie (gratuit, mission ASBL)

Le Modèle Économique : Plus de Monde = Moins Cher pour Tous
===========================================================

**Le Problème des SaaS Classiques**

Les solutions propriétaires facturent **50-500€/mois par copropriété**, avec des marges captées par les actionnaires (70-90%). Les économies d'échelle profitent uniquement aux investisseurs, jamais aux usagers.

**La Révolution KoproGo : Tarif Dégressif**

KoproGo inverse le modèle : **plus de participants = prix baisse pour tous**.

.. list-table:: Grille Tarifaire Dégressive
   :header-rows: 1
   :widths: 30 30 40

   * - Participants
     - Prix Facturé
     - Économie vs Départ
   * - 0-500 copros
     - **1.00€/mois**
     - Référence
   * - 500-1,000
     - **0.80€/mois**
     - **-20%**
   * - 1,000-2,000
     - **0.60€/mois**
     - **-40%**
   * - 2,000-5,000
     - **0.40€/mois**
     - **-60%**
   * - 5,000-10,000
     - **0.20€/mois**
     - **-80%**
   * - 10,000+
     - **0.10€/mois**
     - **-90%**

**Chaque palier se déclenche automatiquement** dès que le nombre de participants est atteint.

**Exemple concret** : Tu rejoins en 2026 avec 100 copropriétés (1€/mois). En 2030, 5,000 copropriétés utilisent KoproGo → Ton tarif est **automatiquement baissé à 0.40€/mois (-60%)** grâce aux 4,900 nouveaux participants. Tu n'as rien fait, tu bénéficies mécaniquement de l'échelle.

**Surplus réinvesti par vote AG** :

* 40-50% : Développement features (vote priorités communauté)
* 20-30% : R&D PropTech (IA, IoT, Blockchain)
* 20-30% : Réserve légale (sécurité financière)
* 0-20% : Baisse tarifaire anticipée (si surplus > 25%)

**Le surplus ne va jamais dans la poche d'actionnaires. Il revient à la communauté.**

Transparence Radicale
====================

KoproGo publie en temps réel sur ``https://koprogo.com/transparency`` (accessible sans login) :

* Nombre de participants actifs
* Coûts infrastructure réels
* Prix coûtant calculé vs Prix facturé
* Surplus généré (€ et %)
* Prochain palier dégressif (countdown)
* Historique baisses tarifaires

**Rapports trimestriels PDF** téléchargeables :

* Bilan comptable complet
* Détail coûts par poste
* Allocation surplus (décisions AG)
* Roadmap investissements

**Pas de commission cachée, pas de magie : juste des chiffres.**

Pourquoi une ASBL ?
===================

**ASBL (Association Sans But Lucratif)** belge :

* **Aucun actionnaire, aucun profit personnel**
* **Excédents réinvestis dans le projet** (développement, R&D, communauté)
* **Gouvernance démocratique** : 1 membre = 1 voix
* **Décisions collectives** en Assemblée Générale
* **Transparence comptable** obligatoire (rapports publics)

**Opensource (AGPL-3.0)** :

* Code source public sur GitHub
* Contributions communautaires bienvenues
* Audits de sécurité publics
* Fork autorisé si dérive du projet

**Pas de patron. Pas de dividendes. Juste le bien commun.**

Évolution de la Gouvernance
===========================

**Progression naturelle** (voir :doc:`ROADMAP_INTEGREE_2025_2030`) :

#. **Phase Bootstrap (2025)** : Solo dev bénévole, validation MVP
#. **Phase Fondateurs (2026)** : 2-3 fondateurs, constitution ASBL
#. **Phase ASBL (2027-2029)** : Gouvernance démocratique, Assemblées Générales
#. **Phase Coopérative (2030+)** : Transformation en coopérative (optionnel si communauté le souhaite)

**Transition vers coopérative possible** si la communauté vote pour :

* Utilisateurs deviennent sociétaires
* Parts sociales symboliques (1€)
* Gouvernance renforcée (1 sociétaire = 1 voix maintenu)

La Proposition
==============

**Tu viens ?**

En rejoignant KoproGo, tu ne prends pas un abonnement SaaS. Tu rejoins un mouvement :

✅ **Économies réelles** : 80-95% vs logiciels propriétaires
✅ **Impact écologique** : -840 tonnes CO₂/an collectivement
✅ **Pouvoir de négociation** : 5 millions de logements ensemble
✅ **Transparence totale** : Comptabilité publique trimestrielle
✅ **Gouvernance démocratique** : 1 membre = 1 voix
✅ **Prix dégressif** : Plus on est, moins cher c'est

**Pas de magie. Juste du collectif. Juste du bon sens.**

En 2030, on vise neuf millions d'euros d'économies annuelles et huit cent quarante tonnes de CO₂ en moins.

**Pas des rêves : des chiffres qu'on teste depuis octobre vingt-cinq.**

----

**Voir aussi** :

* :doc:`ROADMAP_INTEGREE_2025_2030` - Roadmap stratégique complète 2025-2030
* :doc:`ECONOMIC_MODEL` - Modèle économique ASBL et viabilité financière
* :doc:`MISSION` - Mission et valeurs fondamentales
