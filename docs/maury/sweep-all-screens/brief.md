---
feature: sweep-all-screens
phase: Brief court (audit systématique, pas une nouvelle capacité)
status: SIGNED v1.0 par @gilmry 2026-08-09
date: 2026-08-09
authors: [Claude Sonnet 5 (drafting)]
related_issues: []
parent_maury: suite de docs/maury/fix-admin-buttons-acp (#697/#698)
---

# Brief — Audit systématique de tous les écrans, tous les rôles

## 1. Vision

Après #697 (boutons `on:click` morts) et #698 (dérive `organization_id`/`acp_id`), demande explicite : passer en revue **tous les écrans, tous les boutons, des 4 rôles** (admin, syndic, accountant, owner), au clic réel, et pousser chaque parcours qui fonctionne jusqu'au bout. Objectif : trouver la prochaine classe de bug systémique avant qu'un utilisateur ne la trouve.

## 2. Méthode (révisée après incident)

**Tentative initiale** : script Playwright générique cliquant tous les boutons visibles d'une page, détection par diff DOM/réseau. Deux défauts découverts en le faisant tourner :
1. Faux positifs sur pages à forte volumétrie (diff de texte du body noyé dans 1000+ lignes).
2. **Incident réel** : le dédup par `(testid, label)` ne strippait pas les UUID embarqués dans le *label* (seulement dans le testid) — un bouton "Révoquer" listé une fois par ligne avec l'UUID cible dans son texte a été testé sur *toutes* les lignes au lieu de 2, supprimant réellement 514 role-assignments de test (dev only, sans conséquence, mais incident de méthode réel).

**Méthode retenue** : analyse statique en priorité (grep de patterns de bug connus — `on:click=` sur composant, comparaisons de statut PascalCase vs enum backend `#[serde(rename_all="snake_case")]`) + vérification API ciblée (curl) pour les actions d'état + clic Playwright **uniquement** sur les boutons non-destructifs (ouverture de dialog/formulaire), jamais en boucle sur une liste.

## 3. Portée

- Statique : grep systématique des 2 classes de bug déjà identifiées (event binding Svelte4→5, enum casing) sur l'ensemble du frontend.
- Dynamique : clic réel ciblé (1 occurrence par action, jamais en boucle sur liste) pour les boutons ouvrant un dialog/formulaire, par page, par rôle.
- Hors scope : actions destructives (delete/revoke/erase) testées par lecture de code uniquement, jamais cliquées en masse.

## 4. Hors-scope explicite

- Divergences de contrat déjà documentées comme dette connue dans le code lui-même (ex. `InspectionStatus`, STORY-P7-704) — signalées, pas re-fixées ici sauf demande explicite.
- Chantier Track H (conformité légale) — hors périmètre de cet audit.

## 5. Signature

```
Mary (Brief) : SIGNED v1.0 par @gilmry 2026-08-09
```
