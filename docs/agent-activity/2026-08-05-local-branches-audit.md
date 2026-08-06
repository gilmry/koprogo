# Agent activity — 2026-08-05 — Audit branches locales

**Persona :** diagnostic (Tier 2 lecture, aucune mutation).

**Contexte :** demande utilisateur « regardes toutes les branches du dépôt local », après le push de `docs/resync-github-export-audit` (commits `c2337287`, `ffdcb286`, PR non encore ouverte).

---

## Méthode

`git fetch --all --prune`, puis `git merge-base --is-ancestor <sha> origin/feature/dev|origin/main` pour chaque branche locale, `git worktree list` pour repérer les worktrees actifs, `gh pr list --state open` pour croiser avec les PRs GitHub existantes.

## Inventaire (15 branches locales)

### Déjà mergées / branches de déploiement (rien à faire, hors scope)

`main`, `infra-dev`, `infra-integration`, `infra-prod`, `infra-staging`, `integration`, `staging`, `production`, `dev` — toutes ancêtres de `origin/main` et/ou `origin/feature/dev`, ou branches de déploiement GitOps hors scope applicatif.

### Branche courante

`docs/resync-github-export-audit` (`ffdcb286`) — pushée sur `origin/docs/resync-github-export-audit`, PR pas encore ouverte.

### `feature/dev` local en retard

`feature/dev` local = `1a436832`, `origin/feature/dev` a 12 commits d'avance (inclut le resync docs + les 2 bumps deps du 2026-08-05 déjà mergés). Pas d'action : c'est juste le local qui n'a pas fetché/pull, aucun commit local orphelin.

### Travail non mergé trouvé (worktrees `prunable`, job précédent)

Trois branches portent des commits absents de `origin/feature/dev` et `origin/main`, dans des worktrees dont le répertoire de travail a disparu (`prunable`, job `9f9b3673-...` — session antérieure à celle-ci) :

| Branche                                     | Commit(s)               | Contenu                                                                                                                                                                 | PR existante ?                                                                                                                                                                                                                     |
| ------------------------------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `chore/forward-port-rustsec-2026-0145-0141` | `f37bac29`              | bump `astral-tokio-tar` 0.6.1→0.6.2 + `lettre` 0.11.21→0.11.22 (RUSTSEC-2026-0145/0141), déjà mergé sur `main` via PR #600 mais jamais forward-porté vers `feature/dev` | Aucune                                                                                                                                                                                                                             |
| `chore/dependabot-openssl`                  | `f37bac29` + `fd909cc2` | le forward-port rustsec ci-dessus, empilé avec un bump `openssl`                                                                                                        | Aucune (PR #601 existe mais cible `main`, branche différente `dependabot/cargo/backend/cargo-b5bfc02d2b`)                                                                                                                          |
| `chore/dependabot-actions-cache-6`          | `b2f378ec`              | bump `actions/cache` 5→6                                                                                                                                                | PR #627 ouverte, mais depuis la branche dependabot native `dependabot/github_actions/feature/dev/actions/cache-6`, pas depuis cette branche locale — probablement une tentative de rebase manuel sur la base sécurité, non poussée |

Ce constat recoupe exactement `docs/agent-activity/2026-07-31-branch-audit-merges.md` (« 3 branches Dependabot ouvertes, stackées sur l'ancienne base sécurité »/« écart sécurité réel »). Le travail de forward-port du 2026-07-31 semble avoir été commencé (ces 3 branches) mais pas terminé/poussé.

### Branche orpheline signalée précédemment, toujours présente

`fix/organization-creation-svelte5-buttons` (`31092c4f`) — « boutons `<Button on:click>` inopérants (migration Svelte 5 runes incomplète) ». Aucun upstream, jamais mergée nulle part, déjà signalée comme travail potentiellement oublié dans l'audit du 2026-07-31. Toujours à vérifier par @gilmry.

### Worktree du jour

`worktree-wbs-audit-2026-08-05` (`.claude/worktrees/wbs-audit-2026-08-05`, locked) — au même SHA que `main`, aucun commit propre encore.

## Risque identifié

Les 3 branches `chore/dependabot-*` / `chore/forward-port-rustsec-*` contiennent du vrai travail (forward-port sécurité RUSTSEC + 2 bumps deps) qui n'existe QUE dans ces branches locales — leurs worktrees ont disparu (répertoire `/tmp/claude-1000/.../9f9b3673-.../scratchpad/*` nettoyé) mais git a gardé les branches et leurs commits. Rien n'est perdu tant que les branches ne sont pas supprimées, mais rien n'est non plus sauvegardé côté remote.

## Aucune action prise

Diagnostic seul (Tier 2). Aucun push, merge, rebase ou suppression de branche.
