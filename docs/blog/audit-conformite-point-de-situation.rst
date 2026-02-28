.. post:: 2026-02-28
   :tags: conformite, roadmap, audit, transparence
   :category: Conformite
   :author: KoproGo Team

Audit de conformite : ou en est KoproGo ?
==========================================

Aujourd'hui, nous avons realise un **audit complet de conformite juridique** de
la plateforme. Ce billet est destine a vous, sympathisants et curieux, pour vous
donner une vision claire et honnete de l'etat du projet. Pas de marketing, pas
de promesses en l'air : des faits.

Pourquoi un audit maintenant ?
------------------------------

KoproGo suit une **roadmap par capacites** : on ne promet pas de dates, on
avance quand les conditions sont remplies. Avant d'ouvrir la beta publique,
nous devions repondre a une question simple :

   *Sommes-nous prets a accueillir de vraies coproprietes ?*

L'audit a passe au crible quatre domaines : le droit belge de la copropriete
(Art. 577 du Code Civil), le RGPD, la comptabilite belge (PCMN), et la
securite technique. Voici le resultat.

Le score global : 65%
---------------------

Soyons directs : **65%, ce n'est pas suffisant pour la production**. Mais
derriere ce chiffre se cachent des realites tres differentes selon les
domaines :

.. list-table::
   :header-rows: 1
   :widths: 40 15 45

   * - Domaine
     - Score
     - Commentaire
   * - Comptabilite belge (PCMN)
     - **95%**
     - Quasi complet, conforme a l'arrete royal
   * - Securite technique
     - **90%**
     - Infrastructure solide, chiffrement, monitoring
   * - Droit de la copropriete
     - **70%**
     - Solide mais lacunes sur les AG
   * - RGPD
     - **65%**
     - Articles 15-21 implementes, docs legales absentes
   * - Documentation legale
     - **15%**
     - Le maillon faible : CGU, politique cookies, mentions legales

Ce qui marche bien
------------------

La base technique est solide. Concretement :

**Comptabilite** -- Le Plan Comptable Minimum Normalise belge est implemente
avec 90+ comptes pre-configures, les trois taux de TVA (6%, 12%, 21%), le
bilan, le compte de resultats, et un workflow complet de facturation.

**Securite** -- Les donnees sont chiffrees au repos (LUKS), les sauvegardes
quotidiennes sont chiffrees (GPG), le monitoring est en place (Prometheus,
Grafana), et la detection d'intrusion fonctionne (Suricata + CrowdSec). Le
rate limiting protege les comptes contre le brute-force.

**Conformite copropriete** -- Les convocations d'AG respectent les delais
legaux (15 jours pour les AG ordinaires, 8 jours pour les extraordinaires), les
quotes-parts sont validees a 100% par un trigger PostgreSQL, le recouvrement
d'impayes suit 4 niveaux d'escalade au taux legal belge, et les informations
du syndic sont publiquement accessibles comme l'exige la loi.

**RGPD** -- Les 5 droits fondamentaux sont implementes : acces (Art. 15),
rectification (Art. 16), effacement (Art. 17), limitation (Art. 18), et
opposition au marketing (Art. 21). Chaque action est tracee dans un registre
d'audit conforme a l'Article 30.

Ce qu'il reste a faire
----------------------

L'audit a identifie des lacunes claires, organisees en trois phases :

**Corrections critiques** (avant la beta publique) :

- **Quorum des AG** -- Actuellement, un vote peut avoir lieu meme sans quorum.
  C'est contraire a l'Art. 577-6 par. 5 du Code Civil.
- **Limite des procurations** -- Un mandataire peut representer un nombre
  illimite de coproprietaires. La loi limite a 3 mandats maximum.
- **Lien agenda-resolutions** -- Des decisions hors ordre du jour sont
  techniquement possibles. En droit, elles seraient nulles.
- **Documentation legale** -- Politique de confidentialite, CGU, mentions
  legales et politique cookies : tout est a rediger.

**Corrections importantes** (avant 100 utilisateurs) :

- Distribution automatique des proces-verbaux sous 30 jours
- Presets de majorite par type de decision (pour guider les syndics)
- Enforcement des 3 devis obligatoires pour les travaux >5.000 EUR

**Ameliorations** (avant 500 utilisateurs) :

- Snapshot des tantiemes au debut de chaque AG
- Fenetre temporelle de vote
- Signatures numeriques pour les proces-verbaux
- Consentement cookies sur le frontend

Ou en est la roadmap par capacites ?
-------------------------------------

Pour rappel, KoproGo progresse par **jalons** et non par dates. Chaque jalon
debloque un palier d'adoption :

.. list-table::
   :header-rows: 1
   :widths: 30 15 15 40

   * - Jalon
     - Etat
     - Conformite
     - Ce que ca debloque
   * - **J0** Fondations
     - 100%
     - 30%
     - 10-20 early adopters (beta fermee)
   * - **J1** Securite & GDPR
     - ~75%
     - 40%
     - 50-100 copros (beta publique)
   * - **J2** Conformite legale
     - 100%
     - 80%
     - 200-500 copros (production)
   * - **J3** Features differenciantes
     - ~70%
     - 90%
     - 500-1.000 copros (differenciation)
   * - **J4** Automation
     - ~88%
     - 95%
     - 1.000-2.000 copros (scalabilite)

Vous remarquerez un paradoxe : les Jalons 3 et 4 sont plus avances que le
Jalon 1. C'est normal. La progression n'est pas lineaire parce que certaines
features communautaires (SEL, gamification, sondages) ont ete developpees en
parallele de l'infrastructure de securite. Les jalons ne sont pas des portes
sequentielles rigides -- ce sont des **seuils de maturite** dans des domaines
differents.

Ce que l'audit change concretement
-----------------------------------

L'audit confirme la priorite du **Jalon 1** : tant que les corrections
critiques sur les AG et la documentation legale ne sont pas en place, nous ne
pouvons pas ouvrir la beta publique en toute serenite.

Le plan d'action recommande represente environ **15 jours de developpement**
pour les corrections critiques. A la velocite actuelle (1 solo dev + IA,
10-15h/semaine), cela represente 4 a 6 semaines de travail.

Pourquoi ca vous concerne
-------------------------

Si vous etes sympathisant, vous payez 5 EUR/mois pour soutenir un projet qui
refuse les raccourcis. Cet audit est exactement cette philosophie en action :

- On ne sort pas en production avec 65% de conformite.
- On identifie les lacunes avec precision.
- On les corrige dans l'ordre de criticite.
- On vous dit ou on en est, sans embellir.

Chaque cotisation finance directement le developpement. Chaque nouveau
membre accelere la velocite. L'audit d'aujourd'hui est la preuve que votre
soutien sert a construire quelque chose de solide, pas a livrer vite et mal.

La suite
--------

Les prochains billets de blog vous tiendront informes de l'avancement des
corrections. Le rapport d'audit complet est disponible dans la
`documentation technique <https://doc.koprogo.com/AUDIT_CONFORMITE_JURIDIQUE.html>`_.

Si vous avez des questions, rendez-vous sur les
`discussions GitHub <https://github.com/gilmry/koprogo/discussions>`_.

   *"Nous livrons quand c'est pret, pas selon un calendrier arbitraire."*
