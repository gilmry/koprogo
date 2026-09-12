---
description: Restitue l'état Foyer (où on en est, gates, arbitrages en attente)
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


Lis le **registre d'état** du projet (`RELEASE.md`) et rends une
synthèse courte, lisible par un PO non-technique :

- **Porte active** + phase/étape courante + prochaine action attendue.
- **Gates** : statut du dernier passage (🟢/🔴), en signalant tout rouge et sa cause.
- **Périmètres / backlog** : migrés / en cours / à faire.
- **Arbitrages 🔴 en attente** : pour chacun, la question de **modalité** à trancher et les options.

N'exécute rien. Ne modifie pas le registre. C'est une lecture.
