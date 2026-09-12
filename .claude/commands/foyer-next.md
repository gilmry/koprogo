---
description: Exécute la prochaine étape déterministe du parcours Foyer
---
---

**Avant toute chose.** Si `.foyer/pilote/` est vide ou absent, la méthode n'est pas
là et rien de ce qui suit ne peut fonctionner. Arrête-toi et dis-le :

```bash
git submodule update --init --recursive
```

Foyer est un **submodule** de ce dépôt (`.foyer`, épinglé à un commit). Un clone
sans `--recurse-submodules` le laisse vide, et toutes les commandes échoueraient
alors sur des fichiers introuvables, sans que le message parle du submodule.

Tous les chemins `pilote/…`, `skills/…`, `bmad/…` et `AGENTS.md` ci-dessous sont
**relatifs à `.foyer/`**. Le registre d'état, lui, est à la racine de ce dépôt :
`RELEASE.md`.


Applique la **boucle « next »** de `.foyer/pilote/parcours.md` :

1. Lis le registre → porte active + phase/étape courante.
2. Ouvre `.foyer/pilote/journeys/<porte>.md` → localise l'étape → identifie le **rôle**
   (`.foyer/pilote/roles/<rôle>.md`) et sa **condition de sortie**.
3. Vérifie les pré-conditions (gates de l'étape précédente 🟢 — voir `.foyer/pilote/gates/README.md`).
   Rouge → **stop**, renvoie au rôle `gate-runner`, ne passe pas.
4. **Si l'étape est un point irréversible** (`.foyer/pilote/arbitrage.md`) → n'exécute PAS : bascule vers
   `/foyer-bascule`.
5. Sinon → joue le rôle, produis la sortie, fais passer les gates, **mets à jour le registre**,
   et **committe** `<porte>(<périmètre>): <étape>`.

Termine par : ce qui a été fait, l'état des gates, et la prochaine action.
