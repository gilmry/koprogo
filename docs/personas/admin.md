---
persona: "Admin"
role_code: "admin"
statut: brouillon
population_recette: 0
eprouve: false
date: 2026-09-13
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Admin

Administrateur d'organisation. **Non éprouvé** : aucun compte réel n'a ce
rôle en base au 2026-09-13, contrairement à `syndic`, `owner`, `accountant`
et `superadmin`.

Ce n'est pas un rôle absent du code : `frontend/src/lib/auth/permissions.ts`
le distingue de `superadmin` (`ADMIN_ROLES = {"superadmin", "admin"}`,
visibilité identique en mode plateforme) et le contrat `data-testid` gelé
lui connaît des ancres propres (`admin-dashboard`,
`admin-organizations-tile`, `admin-gdpr-panel`, entre autres). L'interface
existe ; ce qui manque, c'est l'usage réel qui la mettrait à l'épreuve.

## Parcours nominal

Non éprouvé : aucun parcours numéroté n'est affirmé ici. Écrire un
enchaînement d'étapes sans un seul compte réel l'ayant suivi serait
inventer un parcours théorique — exactement ce que #805 (@edge) interdit.
Les ancres suivantes existent dans le contrat `data-testid` gelé et
délimitent ce que l'interface *permet*, sans garantir qu'un utilisateur
réel l'ait emprunté dans cet ordre : `admin-dashboard`,
`admin-organizations-tile`, `admin-gdpr-panel`, `admin-users-back-link`.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **créer une ACP ni une organisation** — réservé au
  superadmin (cf. `superadmin.md`), qui seul opère au-dessus du
  multi-tenant.
- Le périmètre exact de ce qu'un admin d'organisation peut faire
  au-delà de ce que `superadmin.md` couvre n'est pas établi ici : ce
  serait, là aussi, l'affirmer sans l'avoir vu en usage réel.

## Références légales

Aucune : un rôle d'administration d'organisation est une notion produit,
pas une qualité reconnue par le Code civil de la copropriété.

## Ce qui ne marche pas encore

Rôle non éprouvé dans son ensemble — pas une étape isolée, tout le
parcours. Aucune étape rouge individuelle n'est tracée au 2026-09-13 : le
constat porte sur l'absence totale d'usage réel, pas sur un défaut
identifié.

## Comptes de recette

Aucun persona nommé dans `docs/specs/00-personas-et-seed.rst` ne porte ce
rôle. Un compte de recette dédié reste à créer par la story qui détaillera
ce parcours (#806-#809, #815 ou #816, association à confirmer par le PO)
avant toute vidéo ou capture publiée.
