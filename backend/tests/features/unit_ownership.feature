# language: fr
Fonctionnalit\u00e9: Gestion des copropri\u00e9taires multiples par lot

  Contexte:
    \u00c9tant donn\u00e9 une organisation "Syndic Test"
    Et un immeuble "R\u00e9sidence Exemple" dans cette organisation
    Et un lot "A101" de type Appartement dans cet immeuble

  Sc\u00e9nario: Ajouter un propri\u00e9taire unique \u00e0 un lot
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Jean Dupont" avec l'email "jean.dupont@example.com"
    Quand j'ajoute "Jean Dupont" au lot "A101" avec 100% de quote-part
    Et je le d\u00e9signe comme contact principal
    Alors le lot "A101" devrait avoir 1 propri\u00e9taire actif
    Et "Jean Dupont" devrait d\u00e9tenir 100% du lot "A101"
    Et "Jean Dupont" devrait \u00eatre le contact principal du lot "A101"

  Sc\u00e9nario: Ajouter plusieurs copropri\u00e9taires en indivision
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Marie Martin" avec l'email "marie.martin@example.com"
    Et un copropri\u00e9taire "Pierre Durand" avec l'email "pierre.durand@example.com"
    Et un copropri\u00e9taire "Sophie Leclerc" avec l'email "sophie.leclerc@example.com"
    Quand j'ajoute "Marie Martin" au lot "A101" avec 50% de quote-part comme contact principal
    Et j'ajoute "Pierre Durand" au lot "A101" avec 30% de quote-part
    Et j'ajoute "Sophie Leclerc" au lot "A101" avec 20% de quote-part
    Alors le lot "A101" devrait avoir 3 propri\u00e9taires actifs
    Et la quote-part totale du lot "A101" devrait \u00eatre 100%
    Et "Marie Martin" devrait \u00eatre le contact principal du lot "A101"

  Sc\u00e9nario: Emp\u00eacher l'ajout d'un propri\u00e9taire qui d\u00e9passe 100%
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Marie Martin" avec l'email "marie.martin@example.com"
    Et un copropri\u00e9taire "Pierre Durand" avec l'email "pierre.durand@example.com"
    Et "Marie Martin" poss\u00e8de d\u00e9j\u00e0 70% du lot "A101"
    Quand j'essaie d'ajouter "Pierre Durand" au lot "A101" avec 50% de quote-part
    Alors l'op\u00e9ration devrait \u00e9chouer avec l'erreur "exceed 100%"
    Et le lot "A101" devrait avoir 1 propri\u00e9taire actif

  Sc\u00e9nario: Retirer un copropri\u00e9taire d'un lot
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Jean Dupont" avec l'email "jean.dupont@example.com"
    Et "Jean Dupont" poss\u00e8de d\u00e9j\u00e0 100% du lot "A101"
    Quand je retire "Jean Dupont" du lot "A101"
    Alors le lot "A101" devrait avoir 0 propri\u00e9taire actif
    Et "Jean Dupont" devrait apparaitre dans l'historique du lot "A101"
    Et la relation devrait avoir une date de fin

  Sc\u00e9nario: Transf\u00e9rer la propri\u00e9t\u00e9 d'un lot
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Vendeur Ancien" avec l'email "vendeur@example.com"
    Et un copropri\u00e9taire "Acheteur Nouveau" avec l'email "acheteur@example.com"
    Et "Vendeur Ancien" poss\u00e8de d\u00e9j\u00e0 100% du lot "A101"
    Quand je tranf\u00e8re la propri\u00e9t\u00e9 de "Vendeur Ancien" \u00e0 "Acheteur Nouveau" pour le lot "A101"
    Alors le lot "A101" devrait avoir 1 propri\u00e9taire actif
    Et "Acheteur Nouveau" devrait d\u00e9tenir 100% du lot "A101"
    Et "Vendeur Ancien" devrait apparaitre dans l'historique du lot "A101"
    Et "Vendeur Ancien" ne devrait plus \u00eatre propri\u00e9taire actif du lot "A101"

  Sc\u00e9nario: Modifier la quote-part d'un copropri\u00e9taire
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Marie Martin" avec l'email "marie.martin@example.com"
    Et "Marie Martin" poss\u00e8de d\u00e9j\u00e0 50% du lot "A101"
    Quand je modifie la quote-part de "Marie Martin" \u00e0 75% pour le lot "A101"
    Alors "Marie Martin" devrait d\u00e9tenir 75% du lot "A101"

  Sc\u00e9nario: Changer le contact principal
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Marie Martin" avec l'email "marie.martin@example.com"
    Et un copropri\u00e9taire "Pierre Durand" avec l'email "pierre.durand@example.com"
    Et "Marie Martin" poss\u00e8de d\u00e9j\u00e0 60% du lot "A101" en tant que contact principal
    Et "Pierre Durand" poss\u00e8de d\u00e9j\u00e0 40% du lot "A101"
    Quand je d\u00e9signe "Pierre Durand" comme nouveau contact principal du lot "A101"
    Alors "Pierre Durand" devrait \u00eatre le contact principal du lot "A101"
    Et "Marie Martin" ne devrait plus \u00eatre le contact principal du lot "A101"

  Sc\u00e9nario: Un copropri\u00e9taire poss\u00e8de plusieurs lots
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Investisseur Multi" avec l'email "investisseur@example.com"
    Et un lot "A102" de type Appartement dans cet immeuble
    Et un lot "B201" de type Appartement dans cet immeuble
    Quand j'ajoute "Investisseur Multi" au lot "A101" avec 100% de quote-part
    Et j'ajoute "Investisseur Multi" au lot "A102" avec 100% de quote-part
    Et j'ajoute "Investisseur Multi" au lot "B201" avec 100% de quote-part
    Alors "Investisseur Multi" devrait poss\u00e9der 3 lots
    Et les lots de "Investisseur Multi" devraient inclure "A101", "A102" et "B201"

  Sc\u00e9nario: Consulter l'historique de propri\u00e9t\u00e9 d'un lot
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Premier Proprio" avec l'email "premier@example.com"
    Et un copropri\u00e9taire "Deuxi\u00e8me Proprio" avec l'email "deuxieme@example.com"
    Et un copropri\u00e9taire "Troisi\u00e8me Proprio" avec l'email "troisieme@example.com"
    Et "Premier Proprio" a poss\u00e9d\u00e9 100% du lot "A101" de 2020 \u00e0 2021
    Et "Deuxi\u00e8me Proprio" a poss\u00e9d\u00e9 100% du lot "A101" de 2021 \u00e0 2023
    Et "Troisi\u00e8me Proprio" poss\u00e8de d\u00e9j\u00e0 100% du lot "A101"
    Quand je consulte l'historique du lot "A101"
    Alors l'historique devrait contenir 3 entr\u00e9es
    Et il devrait y avoir 1 propri\u00e9taire actif
    Et il devrait y avoir 2 anciens propri\u00e9taires

  Sc\u00e9nario: V\u00e9rifier la quote-part totale d'un lot
    \u00c9tant donn\u00e9 un copropri\u00e9taire "Marie Martin" avec l'email "marie.martin@example.com"
    Et un copropri\u00e9taire "Pierre Durand" avec l'email "pierre.durand@example.com"
    Et "Marie Martin" poss\u00e8de d\u00e9j\u00e0 65% du lot "A101"
    Et "Pierre Durand" poss\u00e8de d\u00e9j\u00e0 35% du lot "A101"
    Quand je v\u00e9rifie la quote-part totale du lot "A101"
    Alors la quote-part totale devrait \u00eatre 100%
