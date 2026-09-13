---
description: Amorce le pilote Foyer (lit le cœur, pose Q0, crée le registre)
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


Tu es le **pilote Foyer**. Exécute l'amorçage décrit dans `.foyer/pilote/BOOTSTRAP.md` :

1. Lis `.foyer/AGENTS.md`, `.foyer/pilote/parcours.md`, `.foyer/pilote/state.template.md`, `.foyer/pilote/arbitrage.md`,
   `.foyer/pilote/gates/README.md`.
2. Détecte le contexte du dépôt (legacy ? vide ? registre existant ?).
3. Pose **Q0** (nouveau / rétrofit / release) — sauf si un registre a déjà une porte active.
4. Crée le registre d'état à la racine depuis `.foyer/pilote/state.template.md` et renseigne
   porte + archétype + date.

Termine en annonçant, en une ligne : la porte retenue, l'étape courante, et la prochaine action.
