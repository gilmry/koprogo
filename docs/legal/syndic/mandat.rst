=====================================
Syndic — Mandat et désignation
=====================================

Sources principales
-------------------

- Code civil art. 3.89 §1 à §9
- Code de déontologie IPI art. 76, 82-85
- AR 15/03/2017 (BCE)

Règles
------

M01 — Désignation du syndic
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §1 al.1
- **Règle** : Le syndic est nommé par l'AG ou, à défaut, par le juge à la requête d'un copropriétaire ou de tout tiers ayant un intérêt.
- **KoproGo** : Workflow d'onboarding avec sélection du mode de désignation (AG / juge / statuts).

M02 — Durée maximale du mandat
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §1 al.3
- **Règle** : Le mandat ne peut excéder 3 ans. Pas de reconduction tacite. Le renouvellement nécessite une décision expresse de l'AG.
- **KoproGo** : Alerte automatique 3 mois avant expiration + inscription automatique du renouvellement à l'OdJ.

M03 — Renouvellement par décision expresse
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §1 al.4
- **Règle** : Le renouvellement requiert une décision expresse de l'AG à la majorité absolue. Le simple fait de ne pas renouveler ne donne pas droit à indemnité.
- **KoproGo** : Vérification que le point « renouvellement du mandat » figure bien à l'OdJ.

M04 — Engagements limités à la durée du mandat
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §1 al.5
- **Règle** : Le syndic ne peut souscrire d'engagement excédant la durée de son mandat sauf décision expresse de l'AG.
- **KoproGo** : Alerte si un contrat fournisseur a une durée qui dépasse la fin du mandat en cours.

M05 — Contrat écrit obligatoire
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §1 al.2 + Déonto. art. 76
- **Règle** : La relation syndic/ACP est régie par un contrat écrit comprenant la liste des prestations forfaitaires et complémentaires avec leurs rémunérations. Toute prestation non mentionnée ne peut donner lieu à rémunération sauf décision AG.
- **KoproGo** : Template de contrat intégré + vérification de la liste des prestations.

M06 — Révocation par l'AG
~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §7
- **Règle** : L'AG peut révoquer le syndic à tout moment (majorité absolue). Le syndic peut réclamer une indemnité si la révocation est abusive.
- **KoproGo** : Workflow de révocation : point OdJ obligatoire, PV avec motivation.

M07 — Continuité en cas de démission
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §7 + Déonto. art. 15
- **Règle** : Le syndic démissionnaire doit continuer à exécuter ses tâches jusqu'à ce que l'AG l'ait remplacé.
- **KoproGo** : Suivi de statut « en cours de remplacement » jusqu'à nomination du successeur.

M08 — Transfert du dossier au successeur
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §5 7°
- **Règle** : Le syndic sortant transmet l'intégralité du dossier (comptabilité, sinistres, clés, etc.) au successeur. En l'absence de successeur, au président de la dernière AG.
- **KoproGo** : Checklist de transition générée automatiquement.

M09 — Inventaire détaillé de transition
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Déonto. art. 85
- **Règle** : Le syndic sortant et le successeur établissent un inventaire détaillé des pièces transmises.
- **KoproGo** : Module d'inventaire de transition signé par les deux parties.

M10 — Incompatibilités de rôles
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §9
- **Règle** : Le syndic ne peut être en même temps membre du conseil de copropriété ni commissaire aux comptes au sein de la même ACP.
- **KoproGo** : Validation d'incompatibilité de rôles à la configuration des membres.

M11 — Inscription à la BCE
~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : AR 15/03/2017 + Art. 3.89 §5 17°
- **Règle** : Le syndic (professionnel ou bénévole) doit être inscrit à la BCE pour chaque ACP gérée, avant le début de mission ou dans les 8 jours ouvrables si désigné moins de 8 jours avant.
- **KoproGo** : Rappel BCE à l'onboarding + suivi du statut d'inscription.
