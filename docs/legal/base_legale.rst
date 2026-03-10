======================================
KoproGo — Base legale de reference
======================================

Ce repertoire rassemble l'ensemble des obligations legales et deontologiques applicables
a chaque role utilisateur de la plateforme KoproGo. Il sert de :

- **Source de verite** pour les regles metier implementees dans le code
- **Contexte** pour le serveur MCP (l'agent IA s'y refere pour guider les utilisateurs)
- **Base editoriale** pour le blog KoproGo

Sources juridiques
------------------

.. list-table::
   :header-rows: 1
   :widths: 10 40 50

   * - Code
     - Source
     - Reference
   * - CC
     - Code civil belge, Livre 3, Titre 1, Sous-titre 3
     - Art. 3.78 a 3.100
   * - DEONTO
     - Code de deontologie IPI
     - AR 29/06/2018 (MB 31/10/2018)
   * - LOI-AI
     - Loi organisant la profession d'agent immobilier
     - Loi du 11/02/2013
   * - BCE
     - Inscription du syndic a la BCE
     - AR 15/03/2017
   * - RGPD
     - Reglement general sur la protection des donnees
     - Reglement (UE) 2016/679
   * - AML
     - Loi anti-blanchiment
     - Loi du 18/09/2017
   * - CDE
     - Code de droit economique
     - Livre VI (pratiques du marche)
   * - CSA
     - Code des societes et des associations
     - Applicable a la liquidation ACP

Structure par role
------------------

::

   legal/
   ├── syndic/
   │   ├── mandat                  ← cycle de vie du mandat (M01-M11)
   │   ├── missions_legales        ← 18 missions art. 3.89 §5 (L01-L18)
   │   ├── deontologie_specifique  ← Titre III deonto. IPI (D01-D14)
   │   ├── deontologie_generale    ← Titre I deonto. IPI (G01-G14)
   │   └── travaux                 ← urgents vs non-urgents (T01-T07)
   ├── coproprietaire/
   │   └── droits_obligations      ← vote, charges, recours (CP01-CP15)
   ├── locataire/
   │   └── droits_obligations      ← information, observations (LO01-LO06)
   ├── commissaire/
   │   └── droits_obligations      ← audit, rapport, acces (CO01-CO05)
   ├── conseil-copropriete/
   │   └── droits_obligations      ← supervision, delegations (CC01-CC06)
   ├── acp/
   │   └── personnalite_juridique  ← patrimoine, justice, BCE (ACP01-ACP08)
   ├── notaire/
   │   └── transmission_lot        ← art. 3.94, delais (N01-N05)
   └── assemblee-generale/
       └── sequence_odj            ← ordre logico-juridique des points

Convention de nommage des regles
---------------------------------

Chaque regle est identifiee par un code : ``{PREFIXE}{NUMERO}``

.. list-table::
   :header-rows: 1
   :widths: 15 85

   * - Prefixe
     - Role/Domaine
   * - M
     - Mandat syndic
   * - L
     - Missions legales syndic
   * - D
     - Deontologie specifique syndic
   * - G
     - Deontologie generale
   * - T
     - Travaux
   * - AG
     - Assemblee generale
   * - CP
     - Coproprietaire
   * - LO
     - Locataire/occupant
   * - CO
     - Commissaire aux comptes
   * - CC
     - Conseil de copropriete
   * - ACP
     - Association des coproprietaires
   * - N
     - Notaire / transmission de lot
   * - F
     - Finance / comptabilite

Utilisation par le serveur MCP
-------------------------------

Le serveur MCP expose un outil ``legal_reference`` qui permet a l'agent IA de :

1. Chercher une regle par code (ex: ``AG09``)
2. Lister les regles par role (ex: ``coproprietaire``)
3. Identifier la majorite requise pour un type de decision
4. Citer la source legale exacte dans ses reponses
