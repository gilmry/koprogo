Conformite RGPD et Autorite de Protection des Donnees (APD)
=============================================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Introduction
------------

Le Reglement General sur la Protection des Donnees (RGPD, Reglement UE 2016/679)
est directement applicable en Belgique. La **Loi du 30 juillet 2018** transpose
les dispositions nationales et cree l'Autorite de Protection des Donnees (APD).

KoproGo traite des donnees personnelles de coproprietaires (nom, email, telephone,
adresse, donnees financieres). A ce titre, le logiciel doit etre conforme au RGPD.

Articles RGPD implementes
--------------------------

Art. 15 — Droit d'acces
~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"La personne concernee a le droit d'obtenir [...] la confirmation que
des donnees a caractere personnel la concernant sont ou ne sont pas traitees et [...]
l'acces auxdites donnees."*

**Implementation** :

- Endpoint : ``GET /gdpr/export``
- Retourne toutes les donnees personnelles en format JSON
- Inclut : donnees utilisateur, donnees proprietaire, roles, logs d'audit
- Entite : ``GdprExport`` avec ``UserData``, ``OwnerData``, ``RelatedData``

**Statut** : CONFORME

Art. 16 — Droit de rectification
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"La personne concernee a le droit d'obtenir [...] la rectification
des donnees a caractere personnel la concernant qui sont inexactes."*

**Implementation** :

- Endpoint : ``PUT /gdpr/rectify``
- Permet de corriger : email, prenom, nom
- Validation : format email, champs non vides
- Audit : ``GdprDataRectified`` / ``GdprDataRectificationFailed``

**Statut** : CONFORME

Art. 17 — Droit a l'effacement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"La personne concernee a le droit d'obtenir [...] l'effacement, dans
les meilleurs delais, de donnees a caractere personnel la concernant."*

**Implementation** :

- Endpoint : ``DELETE /gdpr/erase``
- Verification prealable : ``GET /gdpr/can-erase`` (obligations legales en cours ?)
- Anonymisation : remplacement des donnees par des valeurs generiques
- Conservation du squelette pour obligations comptables (7 ans)

**Statut** : CONFORME

Art. 18 — Droit a la limitation du traitement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"La personne concernee a le droit d'obtenir [...] la limitation
du traitement [...]."*

**Implementation** :

- Endpoint : ``PUT /gdpr/restrict-processing``
- Champ : ``processing_restricted`` (boolean) + ``processing_restricted_at``
- Methode domain : ``User::restrict_processing()``
- Helper : ``User::can_process_data()`` pour verification

**Statut** : CONFORME

Art. 21 — Droit d'opposition (marketing)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"La personne concernee a le droit de s'opposer a tout moment [...]
au traitement des donnees a caractere personnel la concernant a des fins de prospection."*

**Implementation** :

- Endpoint : ``PUT /gdpr/marketing-preference``
- Champ : ``marketing_opt_out`` (boolean) + ``marketing_opt_out_at``
- Helper : ``User::can_send_marketing()``

**Statut** : CONFORME

Art. 30 — Registre des traitements
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"Chaque responsable du traitement [...] tient un registre des activites
de traitement effectuees sous sa responsabilite."*

**Implementation** :

- Table ``audit_logs`` avec : user_id, action, entity_type, entity_id, ip_address, user_agent
- 7 types d'evenements GDPR specifiques
- Retention calculee a 7 ans

**Statut** : CONFORME

Lacunes RGPD identifiees
--------------------------

Art. 13-14 — Politique de confidentialite
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : Le responsable du traitement doit informer les personnes concernees de :

- L'identite du responsable du traitement
- Les finalites du traitement
- Les bases legales
- Les destinataires des donnees
- La duree de conservation
- Les droits des personnes

**Statut** : ABSENT — Aucune politique de confidentialite n'est publiee.

**Risque** : Amende APD + plaintes des personnes concernees.

**Remediation** : Rediger et publier une politique de confidentialite FR/NL
accessible depuis le frontend.

Directive ePrivacy — Cookies
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

La Directive ePrivacy (2002/58/CE), transposee en Belgique par la Loi du 13/06/2005,
exige un **consentement prealable** pour les cookies non essentiels.

**Statut** : ABSENT — Pas de banniere de consentement cookies.

**Remediation** : Implementer une banniere de consentement cookies dans le frontend (Phase 3).

Art. 28 — DPA sous-traitants
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"Le traitement par un sous-traitant est regi par un contrat [...]
qui definit l'objet et la duree du traitement, la nature et la finalite du traitement."*

Les sous-traitants de KoproGo incluent :

- **Stripe** : paiements (donnees de carte, IBAN)
- **AWS S3** : stockage documents (fichiers potentiellement personnels)
- **Fournisseur email** : envoi de convocations et notifications

**Statut** : ABSENT — Aucun DPA (Data Processing Agreement) n'est en place.

**Remediation** : Signer des DPA avec chaque sous-traitant avant mise en production.

Art. 33 — Notification de violation
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

**Texte** : *"En cas de violation de donnees a caractere personnel, le responsable
du traitement en notifie la violation a l'autorite de controle competente dans les
meilleurs delais et, si possible, 72 heures au plus tard apres en avoir pris connaissance."*

**Statut** : ABSENT — Pas de procedure formelle de notification de violation.

**Remediation** : Etablir un plan de reponse aux incidents avec :

1. Detection et classification de l'incident
2. Notification APD sous 72h
3. Notification des personnes concernees si risque eleve
4. Documentation de la violation

Art. 32 — Securite du traitement
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Le chiffrement des donnees personnelles au repos en base de donnees n'est pas
implemente. Seul le chiffrement disque (LUKS) est en place.

**Statut** : PARTIEL — Chiffrement disque OK, pas de chiffrement au niveau des colonnes.

Sanctions APD belge
--------------------

L'APD (Autorite de Protection des Donnees) peut infliger les sanctions suivantes :

.. list-table::
   :header-rows: 1
   :widths: 30 70

   * - Plafond legal
     - Details
   * - **20 000 000 EUR** ou **4% CA mondial**
     - Plafond maximum RGPD pour violations les plus graves
   * - 2 000 EUR - 600 000 EUR
     - Fourchette typique des amendes APD en pratique
   * - ~18 000 EUR
     - Amende moyenne pour entreprises privees belges

La Chambre Contentieuse de l'APD est competente pour :

- Prononcer des amendes administratives
- Ordonner la cessation du traitement
- Ordonner la mise en conformite avec delai
- Prononcer des astreintes

**Loi belge du 30/07/2018** : Definit les pouvoirs de l'APD, les procedures
de plainte, et les voies de recours.

Matrice de conformite RGPD
----------------------------

.. list-table::
   :header-rows: 1
   :widths: 25 15 30 30

   * - Article
     - Statut
     - Implementation
     - Action requise
   * - Art. 15 (Acces)
     - CONFORME
     - ``GET /gdpr/export``
     - —
   * - Art. 16 (Rectification)
     - CONFORME
     - ``PUT /gdpr/rectify``
     - —
   * - Art. 17 (Effacement)
     - CONFORME
     - ``DELETE /gdpr/erase``
     - —
   * - Art. 18 (Limitation)
     - CONFORME
     - ``PUT /gdpr/restrict-processing``
     - —
   * - Art. 21 (Opposition)
     - CONFORME
     - ``PUT /gdpr/marketing-preference``
     - —
   * - Art. 30 (Registre)
     - CONFORME
     - Table ``audit_logs``
     - —
   * - Art. 13-14 (Information)
     - ABSENT
     - —
     - Politique de confidentialite FR/NL
   * - Art. 28 (Sous-traitants)
     - ABSENT
     - —
     - DPA Stripe, AWS, email
   * - Art. 32 (Securite)
     - PARTIEL
     - LUKS disque, pas colonnes
     - Chiffrement colonnes sensibles
   * - Art. 33 (Violations)
     - ABSENT
     - —
     - Plan de reponse incidents
   * - Directive ePrivacy
     - ABSENT
     - —
     - Banniere cookies
