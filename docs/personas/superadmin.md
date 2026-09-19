---
persona: "Superadmin"
role_code: "superadmin"
statut: brouillon
population_recette: 1
eprouve: true
date: "2026-09-13"
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

# Superadmin

Crée les organisations et les ACP. N'appartient à aucune organisation —
c'est le seul rôle qui opère au-dessus du multi-tenant plutôt qu'à
l'intérieur. Un seul compte réel en base au 2026-09-13 : un rôle éprouvé,
mais à peine, et jamais en situation de charge multi-opérateur.

Le contrat `data-testid` gelé (#802/#803) ne distingue pas d'ancres
`superadmin-*` propres : ce rôle partage les ancres `admin-*` et
`organization-*` avec le rôle `admin` (cf. `admin.md`) — la distinction
entre les deux se joue côté permissions (`ADMIN_ROLES` dans
`frontend/src/lib/auth/permissions.ts`), pas côté interface.

## Parcours nominal

1. Se connecter → `admin-dashboard`.
2. Créer une organisation → `create-organization-button`,
   `organization-form`.
3. Créer l'ACP qui sera ensuite retranscrite par un syndic →
   `acp-create-toggle`, `acp-create-form`.
4. Générer un jeu de données de recette pour une démonstration →
   `admin-seed-tile`, `seed-generate-button`.

## Ce que ce rôle ne peut pas faire, et pourquoi

- Ne peut pas **retranscrire l'acte de base ni convoquer une assemblée** —
  une fois l'ACP créée, sa gestion courante appartient au syndic mandaté
  (cf. `syndic.md`), pas au superadmin qui l'a provisionnée.
- N'**appartient à aucune organisation** — c'est une contrainte de
  conception, pas une omission : un compte qui gère la plateforme ne doit
  pas être également partie prenante d'une des organisations qu'il
  administre.

## Références légales

La création d'une ACP est un acte de provisionnement technique, pas un
acte régi par le Code civil (l'acte de base lui-même l'est, mais c'est le
syndic qui le retranscrit — cf. `syndic.md`). Aucun article n'est cité ici
pour cette raison ; ne pas en inventer un.

## Ce qui ne marche pas encore

Aucune étape rouge tracée au 2026-09-13.

## Comptes de recette

- « Admin KoproGo », administrateur technique, cf.
  `docs/specs/00-personas-et-seed.rst` (« Admin plateforme »).
