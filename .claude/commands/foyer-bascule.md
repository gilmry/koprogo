---
description: Prépare un point irréversible et demande l'arbitrage de modalité au PO
argument-hint: "[nom du point, ex. bascule-n2-medecins]"
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


Un **point irréversible** est atteint. Applique le protocole de `.foyer/pilote/arbitrage.md` — **sans
exécuter la bascule** :

1. Identifie le point ($ARGUMENTS) dans le tableau de `.foyer/pilote/arbitrage.md`.
2. **Assemble la preuve** exigée (tests 4 couches + caractérisation + régression visuelle selon
   le point) et vérifie qu'elle est verte via `gate-runner`.
3. **Écris une entrée 🔴 « arbitrage en attente »** dans le registre : preuve jointe + question de
   **modalité** (timing / séquençage / rollback) + 2–3 options.
4. **Présente au PO** l'état, la preuve et la question. **Arrête-toi.** Ne committe rien
   d'irréversible, ne supprime aucun legacy.

Quand le PO tranche : écris l'**ADR** (décision + pourquoi + alternatives écartées), déplace
l'entrée en ✅ dans le registre, puis exécute uniquement la modalité choisie.
