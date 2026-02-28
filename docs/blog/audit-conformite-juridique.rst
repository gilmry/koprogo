.. post:: 2026-02-28
   :tags: conformite, audit, droit-belge, rgpd
   :category: Conformite
   :author: KoproGo Team

Audit de conformite juridique : resultats et plan de remediation
=================================================================

Nous publions aujourd'hui les resultats de l'audit de conformite juridique
de KoproGo, mene en comparant le code source avec les textes de loi officiels belges.

Pourquoi cet audit ?
---------------------

KoproGo gere des donnees sensibles (financieres, personnelles) et implemente
des processus a valeur juridique (convocations AG, votes, comptabilite).
Avant toute mise en production, il est essentiel de verifier que le code
respecte les obligations legales applicables.

Methodologie
-------------

L'audit couvre quatre domaines :

1. **Droit de la copropriete** — Art. 3.84 a 3.94 du Code Civil belge
2. **Comptabilite** — AR du 12/07/2012 (PCMN pour associations de coproprietaires)
3. **RGPD** — Reglement UE 2016/679 + Loi belge du 30/07/2018
4. **Securite technique** — OWASP, chiffrement, infrastructure

Pour chaque exigence legale, nous avons verifie : le fichier de code concerne,
le scenario de test BDD qui le prouve, et le statut de conformite.

Resultats par domaine
----------------------

.. list-table::
   :header-rows: 1
   :widths: 35 15 50

   * - Domaine
     - Score
     - Commentaire
   * - Droit copropriete (Art. 3.84-3.94)
     - **70%**
     - Convocations OK, quorum/procurations manquants
   * - RGPD
     - **65%**
     - Articles 15-21 OK, documentation legale absente
   * - Comptabilite (PCMN)
     - **95%**
     - Quasi complet
   * - Securite technique
     - **90%**
     - Excellente infrastructure
   * - **GLOBAL**
     - **65%**
     - NON PRET pour production

Erreurs corrigees
------------------

L'audit a revele trois erreurs factuelles dans le code :

**1. Delai de convocation AG**

Le code imposait 8 jours de preavis pour les AG extraordinaires et les
deuxiemes convocations. L'Art. 3.87 §3 du Code Civil est clair :

   *"Sauf dans les cas d'urgence, la convocation est communiquee quinze jours
   au moins avant la date de l'assemblee."*

Ce delai de **15 jours** s'applique a **tous les types d'assemblee**, sans distinction.
Le code a ete corrige dans ``convocation.rs``.

**2. Taux d'interet de retard**

Le code appliquait un taux de penalite de 8% annuel. Le taux d'interet legal civil
belge pour 2026 est de **4.5%** (publie au Moniteur belge par Arrete Royal).
Appliquer un taux superieur expose a une reduction d'office par le juge
(Art. 1153 et 1231 CC). Le code a ete corrige dans ``payment_reminder.rs``.

**3. Delai de l'etat date**

Le seuil de detection de retard etait fixe a 10 jours. L'Art. 3.94 du Code Civil
prevoit un delai de **15 jours** pour une demande simple (30 jours pour une
demande par recommande du notaire). Le code a ete corrige dans ``etat_date.rs``.

**4. 3 devis "obligatoires"**

Le code et la documentation presentaient la regle des 3 devis pour travaux >5000 EUR
comme une "obligation legale belge". Aucun article du Code Civil n'impose cette regle.
C'est une **bonne pratique professionnelle** que nous maintenons dans l'application,
mais avec une terminologie corrigee.

Lacunes critiques
------------------

Trois lacunes critiques doivent etre corrigees avant la mise en production :

1. **Quorum AG** (Art. 3.87 §5) : Les votes sont possibles meme sans quorum de 50%.
   Les decisions prises sans quorum sont nulles et peuvent etre contestees sous 4 mois.

2. **Limitation des procurations** (Art. 3.87 §7) : Un mandataire peut representer
   un nombre illimite de coproprietaires. La loi impose un maximum de 3 mandats
   (avec exception si le total ne depasse pas 10% des voix).

3. **Lien agenda-resolutions** (Art. 3.87 §2) : Des decisions hors agenda peuvent
   etre enregistrees. Toute decision sur un point absent de l'ordre du jour est nulle.

Plan de remediation
--------------------

Le plan de remediation est organise en trois phases :

- **Phase 1 (critique)** : Quorum, procurations, 2eme convocation, lien agenda, documentation legale (~15 jours)
- **Phase 2 (elevee)** : Distribution PV, presets majorite, notification RGPD (~7 jours)
- **Phase 3 (moyenne)** : Snapshot tantiemes, cookies, pentest (~5 jours + externe)

Documentation complete
-----------------------

La section `Conformite Juridique <../legal/index.html>`_ de la documentation
contient :

- Les extraits de loi in extenso (Art. 3.84 a 3.94 CC, AR 12/07/2012)
- La matrice de conformite code-loi complete
- L'analyse des risques juridiques avec probabilites et impacts
- Le detail de la conformite RGPD et des sanctions APD

Prochaines etapes
------------------

La montee en version de KoproGo (au-dela de 0.1.0) est conditionnee a la
correction des lacunes critiques identifiees dans cet audit. Nous publierons
des mises a jour regulieres sur l'avancement de la remediation.

Une revue par un juriste belge specialise en copropriete est prevue avant
la mise en production.
