.. post:: 2026-02-28
   :tags: conformite, audit, droit-belge, rgpd, bdd, transparence
   :category: Conformite
   :author: KoproGo Team

Comment KoproGo prouve sa conformite juridique — et pourquoi vous pouvez le verifier
======================================================================================

KoproGo gere votre copropriete : vos charges, vos votes en assemblee generale,
vos donnees personnelles. C'est une responsabilite serieuse. Alors comment
savoir si le logiciel respecte vraiment la loi belge ?

Nous avons choisi la transparence totale. Voici comment ca marche, sans jargon.

Le probleme : comment prouver qu'un logiciel respecte la loi ?
---------------------------------------------------------------

Quand un syndic vous envoie une convocation d'assemblee generale, le Code Civil
belge exige un preavis de **15 jours minimum**. Quand une penalite de retard est
calculee, elle doit utiliser le **taux d'interet legal** (4.5% en 2026, pas un
chiffre invente). Quand vous demandez l'effacement de vos donnees personnelles,
le RGPD oblige a le faire.

Mais comment verifier qu'un logiciel applique ces regles correctement ?
La plupart des editeurs vous demandent de leur faire confiance. Chez KoproGo,
on a choisi une autre voie : **tout est ecrit noir sur blanc, lisible par tous**.

Notre methode : des scenarios en langage naturel
--------------------------------------------------

Nous utilisons une methode appelee **BDD** (Behavior-Driven Development,
ou "developpement guide par le comportement"). Le principe est simple :
avant d'ecrire le moindre code, on ecrit un **scenario** qui decrit ce
que le logiciel doit faire, en francais courant.

Voici un vrai exemple tire de notre code :

.. code-block:: gherkin

   Scenario: [Art. 3.87 §3] Convocation 15 jours pour TOUS types AG
     # Loi : "la convocation est communiquée quinze jours au moins
     #         avant la date de l'assemblée"
     Etant donne les types de convocation Ordinaire, Extraordinaire et Deuxieme
     Alors le delai minimum doit etre de 15 jours pour TOUS les types

Ce scenario est **lisible par n'importe qui** — pas besoin d'etre developpeur.
Et il est **executable** : a chaque modification du logiciel, la machine verifie
automatiquement que cette regle est toujours respectee.

Un fichier central de conformite
----------------------------------

Toutes les exigences legales belges applicables a KoproGo sont rassemblees dans
un seul fichier : ``legal_compliance.feature``. Ce fichier contient **37 scenarios**,
un par obligation legale.

Chaque scenario porte une etiquette de statut :

- **@conforme** : la regle est implementee et verifiee automatiquement
- **@manquant** : la regle n'est pas encore implementee (on le dit clairement)
- **@partiel** : la regle est partiellement implementee
- **@corrige** : une erreur a ete trouvee et corrigee lors de l'audit

Aujourd'hui, le score est de **25 sur 37** exigences conformes (67%).
Ce n'est pas 100%, et nous le disons ouvertement. C'est pourquoi KoproGo
reste en version 0.1.0 — nous ne passerons en production qu'une fois
les lacunes critiques corrigees.

Ce que l'audit a revele (et corrige)
--------------------------------------

En comparant notre code avec les textes de loi officiels belges, nous avons
decouvert quatre erreurs. Elles ont toutes ete corrigees :

**1. Delai de convocation** — Le code imposait 8 jours de preavis pour les
AG extraordinaires. L'Art. 3.87 §3 du Code Civil ne fait aucune distinction :
c'est **15 jours pour tous les types d'assemblee**. Corrige.

**2. Taux de penalite** — Le code appliquait 8% de penalite annuelle.
Le taux legal civil belge pour 2026 est de **4.5%** (publie au Moniteur belge).
Appliquer un taux superieur expose a une reduction d'office par le juge. Corrige.

**3. Delai etat date** — Le seuil de retard etait fixe a 10 jours.
L'Art. 3.94 prevoit **15 jours**. Corrige.

**4. Regle des 3 devis** — Le code presentait la regle des 3 devis pour
travaux >5000 EUR comme une "obligation legale". C'est en realite une
**bonne pratique professionnelle**, pas une loi. La terminologie a ete corrigee
dans 15 fichiers.

Les lacunes identifiees
------------------------

Trois manques critiques bloquent la mise en production :

1. **Quorum d'assemblee** (Art. 3.87 §5) : Le systeme permet de voter meme
   sans quorum de 50%. En droit belge, les decisions prises sans quorum sont
   nulles et contestables pendant 4 mois.

2. **Limite des procurations** (Art. 3.87 §7) : Aucune limite n'est imposee
   au nombre de procurations. La loi limite a 3 mandats par mandataire
   (sauf si le total ne depasse pas 10% des voix).

3. **Lien agenda-resolutions** (Art. 3.87 §2) : Des votes sur des sujets
   hors agenda sont possibles. La loi les rend nuls.

Pourquoi cette transparence compte
-------------------------------------

Dans la gestion de copropriete, les erreurs ont des consequences reelles :

- Une convocation envoyee trop tard ? L'assemblee est contestable.
- Un taux de penalite trop eleve ? Le juge le reduit d'office.
- Des donnees personnelles mal gerees ? L'APD peut sanctionner jusqu'a 20 millions d'euros.

En rendant notre code **auditable par tous**, nous permettons a chaque
coproprietaire, chaque syndic, chaque juriste de verifier par lui-meme que
KoproGo fait ce qu'il dit. Pas de boite noire, pas de "faites-nous confiance" :
**les preuves sont dans le code**.

Comment verifier par vous-meme
-------------------------------

Tout est public et accessible :

- **Les scenarios de conformite** : ouvrez le fichier
  ``backend/tests/features/legal_compliance.feature`` — chaque scenario
  cite l'article de loi, le fichier de code concerne, et le statut.

- **Les extraits de loi** : la section `Conformite Juridique <../legal/index.html>`_
  contient les textes de loi in extenso (Code Civil Art. 3.84 a 3.94,
  Arrete Royal PCMN, RGPD).

- **La matrice de conformite** : un tableau qui lie chaque exigence legale
  au code qui l'implemente et au test qui le prouve.

- **L'analyse des risques** : chaque risque juridique est documente avec
  sa probabilite, son impact et sa mitigation.

Prochaines etapes
------------------

La montee en version au-dela de 0.1.0 est conditionnee a la correction des
lacunes critiques (quorum, procurations, lien agenda). Nous publierons des
mises a jour regulieres.

Une revue par un juriste belge specialise en copropriete est prevue avant
toute mise en production.
