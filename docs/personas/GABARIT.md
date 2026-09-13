---
persona: "<Nom lisible du rôle, ex. Syndic>"
role_code: "<code exact du rôle, ex. syndic — cf. frontend/src/lib/auth/permissions.ts>"
statut: brouillon
population_recette: <nombre d'utilisateurs réellement peuplés en base, cf. #805>
eprouve: <true si population_recette > 0, sinon false>
date: <AAAA-MM-JJ, date de dernière relecture>
version: "0.1"
superviseur: null
signature_humaine: null
issue_cadre: 805
issue_parcours: null
videos: []
---

> Ce fichier est le gabarit de la documentation vivante multi-persona
> (#805). Il n'est pas lui-même un document de rôle : il ne compte pas dans
> le cliquet de `backend/tests/garde_documentation_personas.rs`, qui ne lit
> que les six fichiers nommés dans `docs/personas/README.md`.
>
> Copier ce fichier vers `<role_code>.md`, remplir l'en-tête, écrire les
> cinq sections. Ne jamais remplir `superviseur` ni `signature_humaine` en
> tant qu'agent : ce sont des champs Tier 1 (cf. `.claude/rules/CRITICAL.md`
> §11), un humain les valide.

## Parcours nominal

Numéroté, de la première connexion à l'acte le plus courant du rôle. Chaque
étape nomme au moins une ancre `data-testid` du contrat gelé
(`frontend/src/lib/__tests__/data-testid.contrat.json`, #802/#803) — une
étape sans ancre est un parcours qu'on ne peut pas prouver atteignable.

1. `<verbe métier>` — ancre : `` `<data-testid>` ``. Référence légale : voir
   la section ci-dessous, pas recopiée ici.
2. …

Si une étape n'est pas atteignable aujourd'hui, ne pas l'omettre : la
signaler dans « Ce qui ne marche pas encore » avec la marque `🔴` et son
issue.

## Ce que ce rôle ne peut pas faire, et pourquoi

Les refus sont des décisions produit. Une ligne par refus, avec la raison
(légale, organisationnelle, ou de sécurité) — pas seulement le constat.

- Ne peut pas `<action>` — parce que `<raison>` (cf. `<article ou ADR>`).

## Références légales

Citer l'article et renvoyer au registre exécutable, jamais recopier le
texte ou le délai qu'il impose — un nombre recopié se périme sans que rien
ne le signale (cf. `backend/src/domain/copropriete/registre_legal.rs`).

- Art. `<X.YZ>` — voir `backend/src/domain/copropriete/registre_legal.rs`
  et `docs/legal/<role_code>/`.

## Ce qui ne marche pas encore

Nommer, pas lisser. Une capacité inatteignable décrite comme si elle
marchait ment ; c'est le motif dominant des défauts de ce produit (#805).

- 🔴 **Étape rouge** — `<description de ce qui casse>` (#`<issue>`)

Rien à signaler : écrire « Aucune étape rouge connue au `<date>`. »

## Comptes de recette

Uniquement les personas fictifs de
`docs/specs/00-personas-et-seed.rst` (« Résidence du Parc Royal »),
désignés par leur **nom**, jamais par une adresse email recopiée ici.
Aucune donnée réelle de copropriété — la galerie de vidéos qui accompagnera
ce document est un document public.

- `<Prénom Nom>`, cf. `docs/specs/00-personas-et-seed.rst`.
