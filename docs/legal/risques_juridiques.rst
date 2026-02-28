Analyse des Risques Juridiques
================================

.. contents:: Table des matieres
   :local:
   :depth: 2

Introduction
------------

Ce document analyse les risques juridiques lies a l'utilisation de KoproGo
en production, leurs consequences potentielles, et les mitigations en place ou a implementer.

Responsabilite du syndic (Art. 3.89 §6)
-----------------------------------------

**Base legale** : Le syndic est le mandataire de l'ACP. Sa responsabilite est double :

1. **Responsabilite contractuelle** (mandat) : obligation de moyens dans l'execution
   de ses missions (convocations, comptabilite, conservation documents).
2. **Responsabilite civile** (Art. 1382-1383 CC) : responsabilite en cas de faute
   de gestion causant un prejudice a l'ACP ou aux coproprietaires.

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Risque
     - Probabilite
     - Impact
     - Mitigation KoproGo
   * - Convocations hors delai (< 15j)
     - Faible
     - ELEVE (nullite AG)
     - Validation automatique 15j (CONFORME)
   * - Comptabilite non conforme
     - Faible
     - ELEVE (revocation)
     - PCMN 90 comptes + partie double (CONFORME)
   * - Absence etat date
     - Moyen
     - MOYEN (retard vente)
     - Entite EtatDate avec alertes 15j (CONFORME)
   * - Defaut de conservation
     - Faible
     - MOYEN (amendes)
     - Backups chiffres S3 (PARTIEL)

Non-conformite des assemblees generales
-----------------------------------------

**Risque principal** : Nullite des decisions de l'AG.

Les decisions prises en AG peuvent etre annulees par le juge de paix si :

1. **Absence de quorum** (Art. 3.87 §5) : Les decisions prises sans quorum de 50%+50%
   sont nulles. Tout coproprietaire peut agir en annulation dans les **4 mois**.

2. **Procurations irregulieres** (Art. 3.87 §7) : Si un mandataire represente plus
   de 3 coproprietaires (ou depasse 10% des voix), les votes concernes sont nuls.

3. **Decisions hors agenda** (Art. 3.87 §2) : Toute decision prise sur un point
   non inscrit a l'ordre du jour est nulle.

.. warning::

   **Lacunes critiques KoproGo** :

   - Pas de validation du quorum (votes possibles sans quorum)
   - Pas de limite des procurations (mandataire illimite)
   - Pas de lien agenda-resolutions (decisions hors agenda possibles)

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Risque
     - Probabilite
     - Impact
     - Statut
   * - Vote sans quorum
     - ELEVEE
     - CRITIQUE (nullite)
     - MANQUANT
   * - Procurations excessives
     - MOYENNE
     - CRITIQUE (nullite)
     - MANQUANT
   * - Decision hors agenda
     - MOYENNE
     - ELEVE (nullite)
     - MANQUANT
   * - Mauvaise majorite
     - FAIBLE
     - ELEVE (nullite)
     - CONFORME (choix manuel)

Taux d'interet de retard
--------------------------

**Base legale** : Le taux d'interet legal civil est publie annuellement au Moniteur belge
par Arrete Royal. Il s'agit du taux applicable par defaut aux retards de paiement
entre particuliers.

**Taux recents** :

- 2024 : 5.25%
- 2025 : 4.0%
- 2026 : 4.5%

**Risque** : Si le logiciel applique un taux superieur au taux legal (ex: 8% au lieu de 4.5%),
les penalites peuvent etre contestees en justice. Le juge peut :

- Reduire d'office les penalites excessives (Art. 1153 et 1231 CC)
- Annuler totalement les penalites si le taux est manifestement abusif

**Mitigation KoproGo** : Le taux a ete corrige de 0.08 (8%) a 0.045 (4.5%) dans cette release.
La constante ``BELGIAN_PENALTY_RATE`` dans ``payment_reminder.rs`` doit etre mise a jour
annuellement selon publication au Moniteur belge.

.. note::

   Il est recommande de rendre ce taux configurable par organisation plutot que de
   le hardcoder, afin de permettre une mise a jour sans deploiement.

Non-conformite RGPD
---------------------

**Sanctions** : Amendes APD jusqu'a 20M EUR ou 4% du CA mondial.
En pratique : 2 000 EUR - 600 000 EUR. Moyenne entreprise privee : ~18 000 EUR.

**Droit de recours** : Les personnes concernees peuvent :

1. Porter plainte aupres de l'APD (gratuit)
2. Exercer un recours judiciaire devant le tribunal de premiere instance
3. Reclamer des dommages et interets (prejudice materiel et moral)

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Risque
     - Probabilite
     - Impact
     - Statut
   * - Absence politique confidentialite
     - ELEVEE
     - MOYEN (amende APD)
     - MANQUANT
   * - Absence consentement cookies
     - ELEVEE
     - FAIBLE (avertissement)
     - MANQUANT
   * - Pas de DPA sous-traitants
     - MOYENNE
     - ELEVE (co-responsabilite)
     - MANQUANT
   * - Pas de notification violation 72h
     - FAIBLE (si pas de violation)
     - CRITIQUE (amende + responsabilite)
     - MANQUANT
   * - Droits des personnes (Art. 15-21)
     - FAIBLE
     - MOYEN
     - CONFORME

Non-conformite comptable (PCMN)
---------------------------------

**Risques** :

1. **Rejet des comptes par l'AG** : Si la comptabilite ne respecte pas le PCMN,
   l'AG peut refuser d'approuver les comptes. Le syndic est alors en defaut.

2. **Revocation du syndic** : Defaut grave pouvant justifier une revocation
   pour faute de gestion (Art. 3.89 §6).

3. **Contestation judiciaire** : Tout coproprietaire peut demander au juge de paix
   la designation d'un administrateur provisoire si le syndic est defaillant.

.. list-table::
   :header-rows: 1
   :widths: 25 25 25 25

   * - Risque
     - Probabilite
     - Impact
     - Statut
   * - Plan comptable non PCMN
     - FAIBLE
     - ELEVE
     - CONFORME (90 comptes)
   * - Absence comptabilite partie double
     - FAIBLE
     - ELEVE
     - CONFORME (JournalEntry)
   * - Conservation < 7 ans
     - FAIBLE
     - MOYEN
     - PARTIEL (backups OK, cron manquant)
   * - Conservation pieces < 10 ans
     - MOYENNE
     - MOYEN
     - A VERIFIER (politique S3)
   * - Absence rapports annuels
     - FAIBLE
     - ELEVE
     - CONFORME (bilan + compte resultats)

Conservation des documents
----------------------------

.. list-table::
   :header-rows: 1
   :widths: 30 20 50

   * - Type de document
     - Duree de conservation
     - Base legale
   * - Documents comptables (ecritures, journaux)
     - **7 ans**
     - AR 12/07/2012 + Code des societes
   * - Pieces justificatives tiers (factures, contrats)
     - **10 ans**
     - Prescription civile de droit commun
   * - PV d'assemblee generale
     - **10 ans** (recommande illimite)
     - Art. 3.87 §10
   * - Registre des coproprietaires
     - Duree de vie de l'ACP
     - Art. 3.86
   * - Donnees personnelles (RGPD)
     - **Duree necessaire au traitement**
     - Art. 5(1)(e) RGPD
   * - Logs d'audit
     - **7 ans** (recommande)
     - Art. 30 RGPD + PCMN

Synthese des risques par priorite
-----------------------------------

Priorite 1 — Critique (avant production)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

1. Validation du quorum pour les AG
2. Limitation des procurations (max 3 mandats + 10% voix)
3. Lien agenda-resolutions
4. Documentation legale (politique confidentialite, CGU)

Priorite 2 — Elevee (avant 100 utilisateurs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

5. Distribution PV sous 30 jours
6. Notification violation RGPD (Art. 33)
7. DPA sous-traitants

Priorite 3 — Moyenne (avant 500 utilisateurs)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

8. Consentement cookies (ePrivacy)
9. Snapshot tantiemes au debut de l'AG
10. Fenetre temporelle de vote
11. Test de penetration tiers
