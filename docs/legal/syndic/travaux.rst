================================
Syndic — Travaux et entretien
================================

Sources principales
-------------------

- Code civil art. 3.88 §1, art. 3.89 §5 2°
- Code de déontologie IPI art. 73-75

Règles
------

T01 — Travaux urgents : syndic seul
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.89 §5 2°
- **Règle** : Le syndic peut exécuter seul les actes conservatoires et d'administration provisoire. Cela inclut les travaux absolument urgents et ceux nécessaires à l'entretien normal et à la conservation du bien.
- **KoproGo** : Workflow : qualification urgence + trace de la justification + exécution sans AG.

T02 — Travaux non-urgents : majorité 2/3
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.88 §1 1°b
- **Règle** : Tous travaux affectant les parties communes nécessitent une majorité des 2/3 des voix. Exception : les travaux conservatoires = majorité absolue.
- **KoproGo** : Calcul automatique de la majorité requise selon le type.

T03 — Mise en concurrence obligatoire
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.88 §1 1°c
- **Règle** : L'AG fixe le montant des marchés à partir duquel une mise en concurrence est obligatoire (majorité 2/3). Les actes conservatoires sont exclus de cette obligation.
- **KoproGo** : Alerte si devis unique au-dessus du seuil. Collecte de devis comparatifs.

T04 — Fournisseurs agréés
~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Déonto. art. 73
- **Règle** : Le syndic ne passe commande qu'à des fournisseurs disposant des agréations éventuellement requises par la loi ou la réglementation.
- **KoproGo** : Champ agréation obligatoire dans le répertoire fournisseurs.

T05 — Provision avant commande
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Déonto. art. 74
- **Règle** : Le syndic réclame une provision suffisante au commettant avant de commander des travaux ou services.
- **KoproGo** : Calcul de la provision nécessaire + appel de fonds spécial automatisé.

T06 — Fournisseurs liés au syndic
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Déonto. art. 75
- **Règle** : Le syndic ne peut pas passer commande avec des personnes ayant un lien familial ou juridique avec lui, sauf autorisation ou ratification de l'ACP dûment informée.
- **KoproGo** : Déclaration de conflit d'intérêts obligatoire + validation AG.

T07 — Travaux d'optimisation énergétique par un copropriétaire
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- **Source** : Art. 3.82 (depuis 2024)
- **Règle** : Un copropriétaire peut utiliser les parties communes pour améliorer énergétiquement son lot (pompes à chaleur, bornes de recharge, sauf panneaux solaires). Demande écrite et motivée au syndic ≥ 2 mois avant. L'ACP doit se prononcer dans les 2 mois (sinon approbation tacite).
- **KoproGo** : Formulaire de demande de travaux énergétiques avec timer 2 mois.
