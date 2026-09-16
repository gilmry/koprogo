# Story 5.5 — Comptable (encodeur ET émetteur) 403 sur /community/* (#589)
#
# Contrepartie serveur de l'ADR 0052 (#556) : le comptable est un prestataire,
# pas un copropriétaire — il n'a pas accès aux échanges communautaires (SEL,
# sondages, annonces, objets partagés, réservations), sauf s'il cumule aussi
# un rôle de copropriétaire (auquel cas il y accède via CE rôle-là).
Feature: Comptable exclu des routes communautaires (FR28, FR30, INV-6)

  Background:
    Given un middleware de garde d'accès communautaire

  @happy
  Scenario: Owner Marie accède à une route communautaire
    Given Marie a uniquement le rôle "owner"
    When Marie appelle une route communautaire
    Then l'accès est autorisé

  @happy
  Scenario: Conseillère Catherine accède via son rôle de copropriétaire sous-jacent
    Given Catherine a le rôle "owner" et le rôle "board_member"
    When Catherine appelle une route communautaire
    Then l'accès est autorisé

  @edge
  Scenario: Comptable encodeur Paul cumule le rôle owner
    Given Paul a le rôle "accountant.encodeur" et le rôle "owner"
    When Paul appelle une route communautaire
    Then l'accès est autorisé

  @security
  Scenario: Comptable émetteur Pierre pur sans rôle owner est refusé
    Given Pierre a uniquement le rôle "accountant.emetteur"
    When Pierre appelle une route communautaire
    Then l'accès est refusé avec INV-6

  @security
  Scenario: Comptable encodeur Pierre pur sans rôle owner est aussi refusé
    Given Pierre a uniquement le rôle "accountant.encodeur"
    When Pierre appelle une route communautaire
    Then l'accès est refusé avec INV-6

  @negative
  Scenario: Accès direct par URL contourne l'UI mais pas le middleware
    Given Pierre a uniquement le rôle "accountant.emetteur"
    When Pierre appelle directement l'URL "/api/v1/buildings/11111111-1111-1111-1111-111111111111/exchanges" en contournant l'UI
    Then l'accès est refusé avec INV-6

  @negative
  Scenario: Une route non communautaire n'est jamais filtrée par ce garde
    Given Pierre a uniquement le rôle "accountant.emetteur"
    When Pierre appelle l'URL "/api/v1/buildings/11111111-1111-1111-1111-111111111111/expenses"
    Then le garde ne s'applique pas à cette route
