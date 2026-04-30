=====================================================
Assemblée générale — Types de majorités (Art. 3.88)
=====================================================

:Source principale: Code civil belge, Art. 3.88 (Loi du 04/02/2020, en vigueur 01/09/2021)
:Ancienne numérotation: Art. 577-7 (avant réforme)
:Dernière vérification: 28 mars 2026

.. note::

   La réforme du 4 février 2020 (entrée en vigueur 1er septembre 2021) a **assoupli**
   certaines majorités. Ce document reflète le droit en vigueur en 2026.

Quorum préalable (Art. 3.87 §5)
---------------------------------

Avant tout vote, l'assemblée doit être valablement constituée :

- **1ère convocation** : Au moins 50% des copropriétaires présents ou représentés
  ET détenant au moins 50% des quotes-parts
- **2ème convocation** : Aucun quorum requis — l'AG délibère valablement quel que soit
  le nombre de présents, mais les mêmes seuils de majorité s'appliquent aux votes

Les 4 types de majorité
=========================

1. Majorité absolue (>50%) — Art. 3.88 §1 (DÉFAUT)
-----------------------------------------------------

:Seuil: Plus de 50% des voix des copropriétaires **présents ou représentés**
:Base de calcul: Voix des présents/représentés, **abstentions exclues**
:Code KoproGo: ``MajorityType::Absolute``

**IMPORTANT** : Il n'existe PAS de "majorité simple" distincte en droit belge de la copropriété.
La majorité absolue est le régime de droit commun (par défaut). Les abstentions, votes
blancs et votes nuls sont **exclus** du décompte.

**Décisions concernées** (toute décision non attribuée à une majorité spéciale) :

.. list-table::
   :header-rows: 1
   :widths: 60 40

   * - Décision
     - Référence
   * - Approbation des comptes annuels et du budget prévisionnel
     - Art. 3.89 §5 15°-16°
   * - Nomination et révocation du syndic
     - Art. 3.89 §1
   * - Nomination et révocation du commissaire aux comptes
     - Art. 3.91
   * - Constitution du conseil de copropriété
     - Art. 3.90 §1
   * - Travaux imposés par la loi (sécurité incendie, ascenseur, PEB, isolation)
     - Art. 3.88 §1
   * - Entretien courant et réparations ordinaires des parties communes
     - Art. 3.88 §1
   * - Actes conservatoires et d'administration provisoire
     - Art. 3.89 §5 2°
   * - Contrats de fournitures régulières (nettoyage, jardinage, ascenseur)
     - Art. 3.89 §5 12°
   * - Autorisation d'ester en justice
     - Art. 3.88 §1
   * - Élection du président de l'assemblée
     - Pratique + ROI
   * - Décharge (quitus) du syndic
     - Jurisprudence
   * - Décharge du commissaire aux comptes
     - Art. 3.91
   * - Règlement d'ordre intérieur (si ne touche pas à l'acte de base)
     - Art. 3.88 §1

2. Majorité des 2/3 (≥66,67%) — Art. 3.88 §1, 1°
-----------------------------------------------------

:Seuil: Au moins 2/3 des voix
:Base de calcul: Voix des copropriétaires **présents ou représentés**
:Code KoproGo: ``MajorityType::TwoThirds``

**Décisions concernées** :

.. list-table::
   :header-rows: 1
   :widths: 60 40

   * - Décision
     - Référence
   * - Modification des statuts (jouissance, usage ou administration des parties communes)
     - Art. 3.88 §1, 1° a)
   * - Travaux affectant les parties communes (sauf travaux imposés par la loi)
     - Art. 3.88 §1, 1° b)
   * - Seuil de montant à partir duquel la mise en concurrence est obligatoire
     - Art. 3.88 §1, 1° c)
   * - Travaux sur parties privatives pour raisons techniques/économiques (motivation spéciale)
     - Art. 3.88 §1, 1° d)

3. Majorité des 4/5 (≥80%) — Art. 3.88 §1, 2°
-------------------------------------------------

:Seuil: Au moins 4/5 des voix
:Base de calcul: Voix des copropriétaires **présents ou représentés**
:Code KoproGo: ``MajorityType::FourFifths``

**Décisions concernées** :

.. list-table::
   :header-rows: 1
   :widths: 60 40

   * - Décision
     - Référence
   * - Modification des statuts non couverte par les 2/3 (y compris répartition des charges)
     - Art. 3.88 §1, 2° a)
   * - Changement de destination (affectation) de l'immeuble ou d'une partie
     - Art. 3.88 §1, 2° b)
   * - Reconstruction de l'immeuble ou restauration en cas de destruction partielle
     - Art. 3.88 §1, 2° c)
   * - Acquisition d'immeubles destinés à devenir parties communes
     - Art. 3.88 §1, 2° d)
   * - Aliénation de parties communes immobilières (vente, baux emphytéotiques, droits réels)
     - Art. 3.88 §1, 2° e)
   * - Modification du caractère commun ou privatif de parties de l'immeuble
     - Art. 3.88 §1, 2° f)
   * - Création d'associations partielles sans personnalité juridique
     - Art. 3.88 §1, 2° g)
   * - Décision de NE PAS constituer le fonds de réserve obligatoire
     - Art. 3.88 §1, 2° h)
   * - Démolition et reconstruction (justifiée par sécurité, salubrité, ou coût > valeur)
     - Art. 3.88 §1, 2° i)

**Recours judiciaire** : Si les 4/5 ne sont pas atteints mais que la proposition a obtenu
au moins 4/5 des voix des **présents ou représentés**, l'assemblée peut saisir le Juge de Paix
pour surpasser la minorité de blocage si le refus est jugé abusif (abus de droit).

4. Unanimité (100%) — Art. 3.88 §1, 3°
-----------------------------------------

:Seuil: 100% de TOUTES les voix
:Base de calcul: **TOUS les tantièmes** (y compris les copropriétaires absents)
:Code KoproGo: ``MajorityType::Unanimity``

.. warning::

   L'unanimité est la seule majorité calculée sur la **totalité** des quotes-parts,
   pas seulement les présents/représentés. C'est un blocage quasi-absolu.

**Décisions concernées** :

.. list-table::
   :header-rows: 1
   :widths: 60 40

   * - Décision
     - Référence
   * - Modification de la répartition des quotités de copropriété (pourcentages de propriété)
     - Art. 3.88 §1, 3°
   * - Reconstruction totale de l'immeuble
     - Art. 3.88 §1, 3°

**Exception** (réforme 2020) : L'unanimité n'est plus requise lorsque l'AG, à la majorité
légalement requise (2/3 ou 4/5), décide de travaux ou actes qui entraînent **nécessairement**
une modification des quotités. Dans ce cas, la modification des quotités suit automatiquement.

Règles spéciales
=================

Plafonnement du vote (Art. 3.87 §7)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

Nul ne peut prendre part au vote, même comme mandataire, pour un nombre de voix
supérieur à la somme des voix de tous les autres copropriétaires présents ou représentés.

En pratique : si un copropriétaire détient >50% des tantièmes, ses voix sont plafonnées
à 50% moins une voix, et les voix des autres sont recalculées proportionnellement.

Procurations (Art. 3.87 §7)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- Maximum 3 procurations par mandataire
- Exception : un mandataire peut détenir plus de 3 procurations si le total des voix
  qu'il représente (propres + mandats) ne dépasse pas 10% du total des voix
- Le syndic ne peut pas être mandataire (Art. 3.89 §9)
- Les procurations sont nominatives et spécifiques à une AG

Évolutions de la réforme 2020
================================

La loi du 4 février 2020 (en vigueur 1er septembre 2021) a modifié plusieurs seuils
par rapport à l'ancien Art. 577-7 :

.. list-table::
   :header-rows: 1
   :widths: 40 20 20 20

   * - Décision
     - Avant (Art. 577-7)
     - Après (Art. 3.88)
     - Impact
   * - Travaux sur parties privatives (technique/économique)
     - 3/4 (75%)
     - 2/3 (66,67%)
     - Assoupli
   * - Seuil mise en concurrence
     - Non prévu
     - 2/3
     - Nouveau
   * - Démolition/reconstruction (sécurité/salubrité)
     - Unanimité
     - 4/5 (80%)
     - Assoupli
   * - Override judiciaire (Juge de Paix)
     - Non prévu
     - 4/5 des présents
     - Nouveau
   * - Modification quotités suite à travaux votés
     - Unanimité
     - Suit la majorité du vote principal
     - Assoupli

Impact sur KoproGo
====================

L'enum ``MajorityType`` dans ``backend/src/domain/entities/resolution.rs`` doit être :

.. code-block:: rust

   #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
   #[serde(rename_all = "snake_case")]
   pub enum MajorityType {
       /// >50% des présents/représentés (hors abstentions) — DÉFAUT
       Absolute,
       /// ≥2/3 des présents/représentés — Art. 3.88 §1, 1°
       TwoThirds,
       /// ≥4/5 des présents/représentés — Art. 3.88 §1, 2°
       FourFifths,
       /// 100% de TOUS les tantièmes (y compris absents) — Art. 3.88 §1, 3°
       Unanimity,
   }

Le calcul de résultat doit :

1. **Exclure les abstentions** du décompte pour Absolute, TwoThirds, FourFifths
2. Pour l'unanimité : calculer contre la **totalité** des tantièmes (pas les présents)
3. Appliquer le plafonnement à 50% (Art. 3.87 §7) avant le calcul de majorité
