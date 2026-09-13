# language: fr
Fonctionnalité: Validation des quotes-parts de propriété (Bug #B6)
  En tant que syndic
  Je veux que le système bloque les quotes-parts dépassant 100%
  Afin de garantir la conformité légale belge (Art. 577-2 §4 Code Civil)

  Contexte:
    Étant donné un immeuble "Résidence Parc" avec 10 lots
    Et un lot "Appartement 1A" dans cet immeuble

  Scénario: Ajout d'un copropriétaire avec quote-part valide
    Étant donné aucun copropriétaire actif sur le lot
    Quand j'ajoute le copropriétaire "Jean Dupont" avec 50% de quote-part
    Alors l'ajout est accepté
    Et le total des quotes-parts est 50%

  Scénario: Ajout d'un deuxième copropriétaire complétant les 100%
    Étant donné le copropriétaire "Jean Dupont" possède 50% du lot
    Quand j'ajoute le copropriétaire "Marie Martin" avec 50% de quote-part
    Alors l'ajout est accepté
    Et le total des quotes-parts est 100%

  Scénario: Blocage quand le total dépasserait 100%
    Étant donné le copropriétaire "Jean Dupont" possède 70% du lot
    Quand j'ajoute le copropriétaire "Marie Martin" avec 50% de quote-part
    Alors l'ajout est refusé avec le message "Total ownership would exceed 100%"

  Scénario: Blocage de la modification si le nouveau total dépasse 100%
    Étant donné le copropriétaire "Jean Dupont" possède 50% du lot
    Et le copropriétaire "Marie Martin" possède 50% du lot
    Quand je modifie la quote-part de "Jean Dupont" à 60%
    Alors la modification est refusée avec le message "Total ownership would exceed 100%"

  Scénario: Modification acceptée si le nouveau total ne dépasse pas 100%
    Étant donné le copropriétaire "Jean Dupont" possède 50% du lot
    Et le copropriétaire "Marie Martin" possède 30% du lot
    Quand je modifie la quote-part de "Jean Dupont" à 70%
    Alors la modification est acceptée
    Et le total des quotes-parts est 100%

  Scénario: Le frontend affiche le pourcentage disponible
    Étant donné le copropriétaire "Jean Dupont" possède 60% du lot
    Quand j'ouvre le formulaire d'ajout de copropriétaire
    Alors le formulaire indique "Disponible: 40.00%"
    Et le champ pourcentage a un maximum de 40

  Scénario: Le frontend bloque la soumission si dépassement
    Étant donné le copropriétaire "Jean Dupont" possède 60% du lot
    Quand j'ouvre le formulaire d'ajout de copropriétaire
    Et je saisis 50% comme quote-part
    Alors le bouton "Ajouter" est désactivé
    Et un message d'erreur indique que le total dépasserait 100%
