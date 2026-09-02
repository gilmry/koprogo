=================================================================
RFC 0002: Jumeau numérique du Code civil, Livre 3, chapitre copropriété
=================================================================

:RFC: 0002
:Auteur: Claude Opus 5 (rédaction) — @gilmry (arbitrage)
:Date: 2026-09-02
:Statut: Draft
:Type: Domain / Architecture
:Jalon: 0 (préparation v0.1.0)
:Voir aussi: ADR-0010 (acte de base), ADR-0011 (quorum), ADR-0012 (fonds), ADR-0045 (le dossier appartient à l'ACP)

Résumé (TL;DR)
==============

Le domaine doit devenir un **jumeau numérique** du chapitre « copropriété
forcée des immeubles bâtis » (Art. 3.84 à 3.100 du Code civil) : chaque règle
que la loi énonce a, dans le code, un endroit unique qui la porte, nommé comme
la loi le nomme, et un test qui la cite.

Trois décisions à trancher :

1. **Isoler le noyau légal** des deux autres contextes qui partagent
   aujourd'hui la même couche domaine (la plateforme SaaS, les fonctions de
   communauté). Soixante-dix entités à plat, sans frontière.
2. **Parler la langue de la loi** dans ce noyau (langage omniprésent), plutôt
   qu'un vocabulaire de gestion immobilière générique.
3. **Tenir un registre des invariants** indexé par article, qui permette de
   répondre à « que dit la loi, et où le code y répond ? » — et de mesurer la
   couverture **du côté de la loi**, pas du côté du code.

Contexte
========

Ce qui manque n'est pas du code
-------------------------------

Le domaine sait déjà faire des choses justes et bien testées : le double
quorum de l'Art. 3.87 § 5, l'exclusion des abstentions du calcul de majorité
(Art. 3.87 § 8), les fonds de roulement et de réserve (Art. 3.86 § 3), les
quotités de l'acte de base (Art. 3.85 § 1er).

Mais il est impossible, aujourd'hui, de répondre à la question inverse :
**quelles obligations légales le logiciel ne porte pas encore ?** On peut
lister les fonctions écrites ; on ne peut pas lister les articles couverts. Un
audit de conformité doit relire la loi et fouiller le code à la main.

Un jumeau numérique renverse le sens de lecture : la loi devient l'index.

Trois contextes dans une seule couche
-------------------------------------

Les soixante-dix entités de ``src/domain/entities/`` relèvent en réalité de
trois domaines qui n'ont ni les mêmes règles, ni les mêmes autorités, ni le
même rythme de changement :

.. list-table::
   :widths: 22 20 58
   :header-rows: 1

   * - Contexte
     - Autorité
     - Entités (extrait)
   * - **Copropriété** (noyau légal)
     - Code civil, Livre 3
     - ``acp``, ``unit``, ``unit_owner``, ``meeting``, ``resolution``,
       ``vote``, ``convocation``, ``budget``, ``call_for_funds``,
       ``charge_distribution``, ``journal_entry``, ``etat_date``,
       ``syndic_mandate``, ``board_member``, ``owner_contribution``
   * - **Plateforme**
     - RGPD, sécurité, contrat SaaS
     - ``user``, ``organization``, ``refresh_token``, ``magic_link``,
       ``two_factor_secret``, ``consent``, ``gdpr_*``, ``security_incident``
   * - **Communauté**
     - choix produit
     - ``achievement``, ``challenge``, ``local_exchange``, ``shared_object``,
       ``skill``, ``resource_booking``, ``energy_campaign``, ``iot_reading``

Le mélange a un coût déjà constaté : c'est parce que ``organization`` (notion
de plateforme) servait de clé d'accès aux pièces comptables (notion légale)
que le dossier de gestion appartenait au syndic et non à l'ACP — le défaut que
corrige l'ADR-0045. Une frontière absente se paie en règles qui fuient.

Le principe du jumeau
=====================

Trois règles, et rien de plus :

**1. Un article, un endroit.** Une obligation légale est portée par une seule
construction du domaine. Si deux endroits la vérifient, l'un des deux la
vérifiera un jour différemment.

**2. Le nom de la loi.** Ce que la loi appelle *quote-part*, *appel de fonds*,
*état daté*, *fonds de roulement*, le code l'appelle pareil. Les traductions
approximatives (``expense`` pour une charge commune, ``budget`` pour un budget
prévisionnel voté) créent une distance dans laquelle les erreurs
d'interprétation s'installent.

**3. La citation est dans le test.** Un invariant sans test qui le nomme n'est
pas un invariant, c'est une intention. Le test porte le numéro d'article ; il
devient la preuve consultable par un juriste qui ne lit pas le Rust.

Registre des invariants
=======================

L'artefact central. Une entrée par obligation computable, indexée par article.
État au 2026-09-02, établi contre le texte coordonné (Justel) et non contre
une source secondaire.

.. list-table::
   :widths: 14 46 12 28
   :header-rows: 1

   * - Article
     - Obligation
     - État
     - Où / ce qui manque
   * - 3.84 al. 4
     - Associations partielles possibles dès 20 lots ; compétence limitée aux
       parties communes particulières
     - absent
     - reporté v0.2.0 (ADR-0010)
   * - 3.85 § 1er al. 2
     - Les quotités sont fixées par l'acte de base ; leur somme est le
       dénominateur
     - **couvert**
     - ``Acp::is_conformant`` (ADR-0010)
   * - 3.85 § 3, 3°
     - Le ROI fixe la période annuelle de quinze jours de l'AG ordinaire
     - absent
     - aucune notion de fenêtre statutaire
   * - 3.86 § 1er
     - La personnalité juridique naît de deux conditions cumulatives
       (indivision née + transcription)
     - absent
     - ``Acp`` n'a pas d'état juridique ; une ACP non transcrite est
       manipulable comme une autre
   * - 3.86 § 1er al. 4
     - Tous les documents émanant de l'ACP mentionnent son numéro d'entreprise
     - partiel
     - le champ existe ; les exports PDF ne le portent pas tous
   * - 3.86 § 3
     - Patrimoine = au minimum fonds de roulement + fonds de réserve, sur des
       comptes distincts au nom de l'ACP
     - **couvert**
     - ``Acp`` (ADR-0012)
   * - 3.86 § 3 al. 4
     - Fonds de réserve obligatoire à cinq ans de la réception provisoire ;
       contribution annuelle ≥ 5 % des charges ordinaires de l'exercice
       précédent ; dérogation aux 4/5
     - partiel
     - ``reserve_fund_waived`` existe ; ni l'échéance des cinq ans ni le
       plancher des 5 % ne sont calculés
   * - 3.86 § 3 al. 7
     - Le syndic communique, **lors de l'appel de fonds**, la part affectée au
       fonds de réserve
     - absent
     - ``CallForFunds`` ne distingue pas la part de réserve
   * - 3.86 § 3 al. 8
     - Usufruit : les titulaires de droits réels sont **solidairement** tenus
       des charges
     - absent
     - l'indivision est modélisée, la solidarité ne l'est pas
   * - 3.86 § 4
     - Exécution sur le patrimoine de chaque copropriétaire au prorata des
       quotes-parts de vote
     - absent
     - hors périmètre applicatif à ce stade
   * - 3.87 § 2
     - AG sur requête de copropriétaires détenant ≥ 1/5 des parts ; convocation
       dans les trente jours
     - absent
     - aucune requête d'AG modélisée
   * - 3.87 § 3
     - Convocation quinze jours au moins avant, sauf urgence ; recommandé sauf
       accord écrit individuel
     - partiel
     - le délai est appliqué ; le mode d'envoi accepté par écrit n'est pas tracé
   * - 3.87 § 5
     - Double quorum, et seconde AG après quinze jours au moins
     - **couvert**
     - ``AgSession`` (ADR-0011)
   * - 3.87 § 6
     - Une voix par quotité
     - **couvert**
     - ``Vote``
   * - 3.87 § 7
     - Trois procurations au maximum, sauf si le total reste ≤ 10 % des voix ;
       nul ne vote pour plus de voix que la somme des autres présents
     - absent
     - ``Vote::is_proxy_vote`` existe, aucun plafond n'est vérifié
   * - 3.87 § 8
     - Majorité absolue des présents ; abstentions, blancs et nuls exclus du
       calcul
     - **couvert**
     - ``Resolution::calculate_result``
   * - 3.87 § 9
     - Conflit d'intérêts : un prestataire de l'ACP ne vote pas sur sa propre
       mission
     - absent
     - rien dans le domaine
   * - 3.87 § 10
     - PV signé par le président, le secrétaire et les copropriétaires présents
     - partiel
     - le PV est produit ; les signatures ne sont pas un état du domaine
   * - 3.87 § 12
     - PV consigné au registre et transmis dans les trente jours
     - absent
     - le délai n'est pas suivi
   * - 3.88
     - Majorités qualifiées par nature de décision
     - partiel
     - les seuils (2/3, 4/5, unanimité) existent ; leur **affectation** par
       type de décision n'est pas vérifiée contre l'article
   * - 3.89 § 1er
     - Mandat de syndic de trois ans au plus
     - **couvert**
     - ``SyndicMandate``
   * - 3.89 § 5, 5°
     - Relevé des dettes au notaire dans les trente jours
     - partiel
     - le délai est documenté, le suivi n'existe pas
   * - 3.89 § 5, 7°
     - Transmission de l'ensemble du dossier de gestion au successeur dans les
       trente jours
     - **en cours**
     - ``PieceDeGestion`` / ``perimetre_du_mandataire`` (ADR-0045)
   * - 3.90
     - Conseil de copropriété
     - partiel
     - ``board_member`` existe ; sa compétence légale n'est pas modélisée
   * - 3.91
     - Commissaire aux comptes
     - absent
     - une seule occurrence dans le domaine
   * - 3.94 § 1er / § 2
     - État daté : quinze jours (demande simple), trente jours (notaire,
       recommandé) — jours **calendaires**
     - **couvert**
     - ``EtatDate`` (tranché contre le texte primaire)
   * - 3.95
     - Arriérés en cas de transmission d'un lot
     - partiel
     - l'état daté porte les arriérés ; la règle d'imputation ne l'est pas

Lecture : **9 couverts, 9 partiels, 11 absents** sur les obligations
computables relevées. Ce sont des capacités à assembler, pas des régressions :
le projet n'a encore rien livré à de vrais copropriétaires.

Ce que ça change concrètement
=============================

- Le noyau légal devient un module distinct (``domain/copropriete/``), sans
  dépendance vers la plateforme ni vers la communauté. La dépendance ne va que
  dans l'autre sens.
- Chaque invariant du registre existe comme une construction nommée du
  domaine, avec un test qui cite son article dans son nom.
- Le registre ci-dessus devient exécutable : un test parcourt la liste
  déclarée et échoue si un invariant annoncé couvert n'a plus de test associé.
  C'est ce qui empêche le jumeau de se désynchroniser en silence.
- Un rapport de conformité se génère depuis le registre, et s'adresse à un
  juriste, pas à un développeur.

Questions ouvertes (à trancher)
===============================

**Q1 — Jusqu'où va le jumeau ?** Le chapitre entier (3.84 à 3.100, y compris
dissolution et liquidation), ou seulement ce qu'un syndic exécute au
quotidien ? Modéliser la dissolution d'une ACP a un coût réel pour un
événement rare.

**Q2 — Renommer ?** Parler la langue de la loi implique de renommer des
entités existantes (``Expense`` → charge commune, ``Budget`` → budget
prévisionnel). Le gain est la disparition d'une distance d'interprétation ; le
coût est une reprise large, y compris du contrat OpenAPI et du frontend. On
peut aussi ne l'appliquer qu'au code neuf et laisser converger.

**Q3 — Le français dans le code ?** Le domaine est aujourd'hui en anglais, la
loi est en français et en néerlandais. Nommer ``quote_part`` plutôt que
``quota`` rapproche du texte mais rompt l'homogénéité du dépôt. Les modules
récents (``dossier_de_gestion``, ``PieceDeGestion``) ont déjà fait ce choix
localement, sans qu'il ait été tranché.

**Q4 — Quelle version fait foi ?** La loi est modifiée. Le jumeau doit dater
sa source (ici : texte coordonné Justel consulté le 2026-09-02) et prévoir ce
qui se passe quand elle change.

Alternatives écartées
=====================

- **Documenter la conformité à part** (un fichier d'audit relu à la main).
  C'est l'état actuel : ``docs/AUDIT_CONFORMITE_JURIDIQUE.md`` existe et se
  périme dès que le code bouge, parce que rien ne les relie.
- **Un moteur de règles configurable.** Séduisant, mais la loi n'est pas
  paramétrique : ses règles ont des exceptions rédigées en prose. Un moteur
  générique déplacerait la complexité dans une configuration que personne ne
  saurait relire.
- **Ne rien isoler et se fier aux tests.** Les tests existants sont bons et
  n'ont pourtant pas empêché le dossier de gestion d'appartenir au syndic :
  ils vérifient ce qu'on a pensé à vérifier. Une frontière de contexte
  interdit une classe entière d'erreurs au lieu d'en attraper des instances.
