# Agent activity — 2026-07-26 — Audit de réconciliation Issues / Projects / WBS ↔ réalité `feature/dev`

**Persona :** audit read-only (Tier 2 — diagnostic + proposition uniquement. Aucune fermeture/réouverture d'issue, aucune mutation de Project, aucun fix de code, aucune réécriture du WBS. Toute action listée en fin de rapport reste Tier-1, à valider par l'humain — CRITICAL.md #11).

**Branche :** analyse effectuée depuis `fix/frontend-typescript-downgrade`, sur les données live `origin/main`/`origin/feature/dev` + GitHub (issues, Projects, Actions).

**Scope :** demande explicite — faire le point du WBS, puis étendre à un audit de synchronisation Issues ↔ Projects ↔ réalité de `feature/dev`, puis creuser spécifiquement le code mergé sans issue associée (git log).

**Décisions de périmètre validées par l'utilisateur avant exécution** : audit ponctuel + rapport (pas de fermeture auto, pas d'outillage récurrent) ; CI rouge diagnostiquée (root cause) mais pas corrigée dans ce chantier.

---

## 0. Résumé exécutif

- **282 issues** au total (99 ouvertes / 183 fermées). **72% des issues ouvertes (72/99) n'ont pas été mises à jour depuis ≥60 jours.**
- Sur ces 72 "stale", **23 sont en réalité déjà implémentées** (commit correspondant trouvé sur `feature/dev`) — pas abandonnées, juste jamais fermées. Les 16 issues du pipeline Maury `#556` sans aucun commit sont cohérentes avec le report v0.2.0 déjà décidé dans le WBS pour les slices 4/5 — pas une anomalie.
- **2 GitHub Projects** existent (#2, #3) mais couvrent uniquement l'ère "Jalon 0" (issues #28-#99, automne 2025) — **aucun** board ne suit le travail WBS actuel. Là où ils s'appliquent, ils sont fiables (0 mismatch sur 42 items liés à une issue réelle).
- Sur 34 références réelles à des issues dans `docs/WBS_GO_LIVE_v0.1.0.md`, **30 (88%) sont encore ouvertes** — la déclaration "Track H #618 COMPLET" (commit `af29b542`) est largement en avance sur l'état GitHub réel.
- `feature/dev` (275 commits d'avance sur `main`) a sa CI rouge sur 4 gates/6. **Root cause identifiée et déjà documentée par #658/#634/#636** — pas de nouveau problème, mais **découverte importante** : le fix déjà préparé cette session sur `fix/frontend-typescript-downgrade` corrige le volet frontend proprement, mais son volet backend (downgrade `printpdf`) **réintroduit une vulnérabilité high (RUSTSEC-2026-0187) que #636 cherchait justement à corriger** — détail en §4.
- Pipeline d'export `make docs-export-github` **cassé** (`jq` absent de l'hôte, pas de sudo) — chiffres recalculés à la main pour ce rapport via `gh` direct.

---

## 1. Issues — inventaire et staleness

| Mesure | Valeur |
|---|---|
| Total | 282 (99 ouvertes / 183 fermées) |
| Sans label | 9 |
| Sans milestone | 144 |
| Milestones existants | 8 (aucun nommé "v0.1.0" — choix assumé, cf. WBS ligne 5, pour éviter le gate Tier-1 sur un doc de suivi) |
| Labels | 78 |
| Ouvertes sans update ≥60j | **72 / 99 (73%)** |

Le doc `docs/github-export/stats.rst` (dernière régénération réussie : mai) affichait encore `139 issues / 74 open / 65 closed / 60 labels` — export non rafraîchi depuis ~3 mois, sous-évaluait le volume réel de moitié.

**Nuance importante sur les 72 stale** : une bonne partie ne sont pas "abandonnées" mais "en cours sans que GitHub le voie" — voir §2, qui recoupe précisément cette liste avec les commits `feature/dev`.

Les plus anciennes (>120j, sans lien avec le travail courant) sont des tickets roadmap long-terme (#48 itsme/eID, #94 IA, #98 mobile natif, #109 IoT, #111 API v2, #266-268 K3s, #295-299 Tauri) : cohérent avec un statut "pas encore priorisé", pas un signe de dérive.

---

## 2. Code sans issue associée (git log `feature/dev`)

Sur les 274 commits où `feature/dev` dépasse `main`, **24 touchent `backend/src/` ou `frontend/src/` sans référence `#issue`** dans le message complet (bumps Dependabot exclus, qui ont leur PR# propre). En élargissant la recherche à toute mention `Story X.Y` dans l'ensemble des 274 commits (pas seulement ceux sans `#issue`) et en la recoupant avec les 40 issues ouvertes labellisées `maury` (pipeline épique **#556**, 39 stories / 7 slices), le tableau se précise :

### 2.1 — 23 issues déjà implémentées mais encore ouvertes (candidats fermeture, confiance haute : SHA connu)

| Slice | Issues | Commit(s) d'implémentation (extrait) |
|---|---|---|
| 0 (caractérisation) | #557 | `e68c9cd0`, `6f0535bf` |
| 1 (ACP + migration + conformité) | #558, #559, #560, #561 | `5d89ef4d`, `594c809d`, `701da509`, `3ede5099`… |
| 2 (sélecteur + bannière + Portfolio) | #562, #563, #564, #565, #566 | `69c159f2`, `7d9aab08`, `d01bd8fa`, `18bf529b`, `576f63eb`… |
| 3 (sous-rôles + magic link + PWA + mandats + ticketing) | #567, #568, #569, #570, #571, #572, #573, #574, #575 | `455490a4`(B1), `b9ab206d`(B2), `709f649c`, `edf171f6`, `ff3c2ae3`, `62570fbb`, `d820c39b`, `c53a7e14`… |
| 4 (partiel) | #580 (Story 4.5), #584 (Story 4.9 méga cluster-coord) | `3ede5099`, `ad8c524f` |
| Tx (transversal) | #593 (Tx.1 gate CI), #594 (Tx.2 helpers shared) | `6f0535bf`, `0ac81a74`, `8af9feaf` |

**Proposition** : fermer ces 23 issues en citant le(s) SHA ci-dessus comme preuve — **Tier-1, à valider par l'utilisateur**, idéalement en confirmant côté GitHub (état des specs Playwright/BDD liées) avant de cliquer, la détection ici étant du text-matching sur les messages de commit, pas une preuve d'exécution verte.

Sous-cas spécifique repéré à part cette session : **#617** ("Phase C — Stabilisation Documentation Vivante e2e") documente déjà que les specs Playwright de la Phase B FE (Stories B1-B8bis) sont instables — donc même si le code des 23 est fusionné, ne pas fermer aveuglément sans vérifier que la CI caractérisation associée est verte.

### 2.2 — Cluster "Phase B FE" (Story B0 à B8bis, ~10 commits) : aucune issue trouvée

Recherche `"Story B" in:title` et `"Phase B" in:title` sur l'ensemble du repo → **0 résultat** (seul #617 existe, et il parle de *stabiliser* des tests déjà écrits pour cette phase, pas de la tracker en amont). Commits concernés : `8cab49f3`(B0), `8ac5a83e`(B0bis), `455490a4`(B1), `b9ab206d`(B2), `6f4c1a4c`(B3), `f1c871db`(B5), `89e9afd3`(B6), `cf2219f1`(B7), `caa6315a`(B8), `c2fd56bf`(fix B8).

**C'est le vrai gap de traçabilité** (règle #6 "tout dans GitHub") : du travail réel (~10 commits, plusieurs fichiers chacun) sans trace GitHub en amont. **Proposition, à trancher par l'utilisateur, pas exécutée** :
- Option A : 1 issue rétroactive par story (cohérent avec le pattern `[Story X.Y]` déjà en place pour les slices 1-3, 5-8 items).
- Option B : 1 issue groupée "Phase B FE (B0-B8bis) — rétro-documentation" liée à l'épique #556 et à #617.

### 2.3 — 6 commits sans aucun repère (ni Story, ni issue)

`fdd908fc` (prettier auto-fix), `2e63dbf6` (wip salvage), `cf41ef47` + `ff3c2ae3` (fixes clippy), `fed175d9` (fix build statique PWA), `7c2d6644` (fix sérialisation dashboard). Impact mineur (formatage, fixes ponctuels) — **priorité basse, pas d'action proposée par défaut**.

---

## 3. GitHub Projects (v2) — lecture live

`gh auth refresh -s read:project` accordé en cours de session. Deux projects, tous deux `owner=gilmry` (repo de type `User`, pas d'org) :

| Project | Items | Liés à une issue réelle | `DraftIssue` (jamais convertis) |
|---|---|---|---|
| #2 "KoproGo Software Roadmap" | 56 | 37 | 19 |
| #3 "KoproGo Infrastructure Roadmap" | 38 | 5 | 33 |

**Croisement statut board ↔ état réel de l'issue** (open/closed) sur les 42 items liés : **0 mismatch**. Les Projects ne mentent pas sur ce qu'ils couvrent.

**Mais ils ne couvrent que #28-#99** (ère "Jalon 0 : Fondations Techniques ✅", automne 2025). Aucune des issues du WBS actuel (#433, #525, #553, #554, #556-#618, #634-#663) n'apparaît sur un board. **Ce ne sont pas des trackers vivants du travail en cours** — la désynchronisation à corriger n'est pas "board vs issues" (déjà cohérent), c'est "aucun board ne reflète l'activité actuelle".

---

## 4. WBS ↔ GitHub — divergences

`grep -oE '#[0-9]+' docs/WBS_GO_LIVE_v0.1.0.md` → 48 références uniques, dont 14 non-résolues (probablement des exemples génériques du template, ex. `#1`, `#2`, `#999`, à ne pas lire comme de vraies issues). Sur les **34 références réelles** :

| État | Nombre | % |
|---|---|---|
| OUVERTES | 30 | 88% |
| FERMÉES | 4 | 12% |

Les 30 ouvertes couvrent tout le spectre : bloqueurs go-live explicitement cités par le WBS lui-même (#634, #636, #603, #604, #617), l'umbrella Decimal (#433, #443), les EPICs (#555, #618), et une bonne partie des bugs `bug:majeur` (#521, #525, #534, #540, #548, #550, #553, #554).

**Proposition de correction pour `docs/WBS_GO_LIVE_v0.1.0.md` §618** (rédigée ici, non appliquée) : nuancer "Track H #618 COMPLET pour v0.1.0" (ligne 196) en distinguant explicitement (a) le code des slices 0-3 réellement mergé mais dont les issues ne sont pas closes (§2.1 ci-dessus — cohérent avec "complet côté code"), de (b) l'epic #618 lui-même et ses 30 issues satellites qui restent ouvertes administrativement. Une déclaration "COMPLET" devrait normalement s'accompagner de la fermeture des issues qu'elle couvre.

---

## 5. CI rouge sur `feature/dev` — diagnostic (root cause, pas de fix)

Dernier run complet (`1a436832`, 2026-07-25 13:13 UTC) : **4 gates/6 en échec** sur push + PR.

| Gate | Trigger | Statut | Root cause |
|---|---|---|---|
| CI Pipeline (Lint&Format/Clippy, Unit, BDD, E2E) | `push` sur `feature/dev` | ❌ | `lopdf 0.44.0` incompatible avec `time 0.3.47` : `error[E0599]: no variant... StringLiteral... for enum BorrowedFormatItem` dans `lopdf/src/datetime.rs`. Root cause déjà documentée dans **#658**, tirée par le bump Dependabot `printpdf 0.7→0.11.1` (#642+). |
| CI Pipeline — Contract Types Check / Frontend Check & Build / Frontend Unit Tests | idem | ❌ | `npm error ERESOLVE` : `typescript@^7.0.2` (root) vs `@astrojs/check@0.9.9` qui exige `peer typescript@"^5.0.0 \|\| ^6.0.0"`. Couvert par **#634**. |
| Security Audit (`cargo audit`) | `pull_request` (feature/dev → main/dev) | ❌ | 2 vulns **high** : `lopdf` RUSTSEC-2026-0187 (stack overflow PDF imbriqué, fix : upgrade `>=0.42.0`) + `quinn-proto` RUSTSEC-2026-0185 (memory exhaustion, fix : `>=0.11.15`). Documenté dans **#636**. |
| Docker Build and Push to GHCR | `pull_request` | ❌ | Conséquence directe des deux causes ci-dessus (build frontend échoue sur le même ERESOLVE, build backend sur le même `lopdf`). Pas de cause distincte. |
| Characterization E2E Gate | `pull_request` (feature/dev → main/dev) | ❌ (mais **pas une anomalie**) | Ce gate est documenté comme ne se déclenchant QUE sur PR vers `main`/`dev`, jamais sur push direct à `feature/dev` (mémoire `gitflow-feature-dev-buffer`). Vérifié : tous les runs "Characterization E2E Gate" sur `feature/dev` ont `event=pull_request`, aucun en `event=push`. **Comportement conforme au design, pas de nouvelle issue à ouvrir.** |
| CI Infra | — | ✅ | vert |

### ⚠️ Découverte importante : tension entre le fix préparé cette session et #636

La branche `fix/frontend-typescript-downgrade` (préparée avant cet audit, cf. `docs/cowork/` et le travail plus tôt dans la session) downgrade `typescript` 7.0.2→5.9.3 **et** `printpdf` 0.11→0.7. Vérification :

- Le volet **TypeScript** résout exactement l'ERESOLVE ci-dessus — safe, pas de compromis identifié.
- Le volet **printpdf → lopdf 0.31** (confirmé dans `backend/Cargo.lock` de la branche) fait effectivement disparaître l'erreur de compilation `lopdf`/`time` — mais c'est littéralement l'**Option 1** listée dans #658 lui-même, qui prévient : *« réintroduit la version visée par le bump »*. Cette version (`lopdf 0.31`) est précisément celle que **#636** signale comme vulnérable (`RUSTSEC-2026-0187`, high, 7.5). Downgrader résout la compilation mais **rouvre une vulnérabilité de sécurité déjà identifiée**.

**Recommandation (proposition, pas exécutée)** : merger le volet frontend (typescript/astro) tel quel — safe. Pour le backend, suivre plutôt l'**Option 2 de #658** (bumper `time` vers une version compatible avec `lopdf 0.44`, ce qui règle *à la fois* #658 et #636 sans compromis), plutôt que le downgrade `printpdf`. Ceci n'a pas été exécuté dans ce chantier (hors périmètre : diagnostic seulement).

---

## 6. Outillage — export GitHub cassé

`make docs-export-github` échoue : `jq: command not found` sur l'hôte, pas de `sudo` sans mot de passe pour l'installer, aucun container docker compose ne l'a non plus. Les chiffres de ce rapport (§1, §3) ont été recalculés directement via `gh issue list/api` en contournement. **Proposition** : soit installer `jq` dans l'image de setup (`make install-deps`?), soit conteneuriser `scripts/export-github-to-rst.sh` pour respecter CRITICAL.md #12 (tooling via docker compose).

---

## 7. Actions proposées (récapitulatif, aucune exécutée)

| # | Action | Tier | Détail |
|---|---|---|---|
| 1 | Fermer 23 issues (§2.1) avec SHA en justification | **Tier-1** | Vérifier CI caractérisation Phase B FE avant de fermer les stories B1-B8bis liées (#567-#575) |
| 2 | Créer les issues manquantes pour le cluster Phase B FE (§2.2) | Tier-2 (création) mais **volume → proposé, pas exécuté** | Trancher option A (10 issues) vs B (1 groupée) |
| 3 | Corriger `docs/WBS_GO_LIVE_v0.1.0.md` §618 (nuancer "COMPLET") | Décision produit (règle CRITICAL.md #5) | Texte de correction proposé en §4 |
| 4 | Fix CI backend via #658 Option 2 (bump `time`) plutôt que downgrade `printpdf` | Tier-1 (code + merge) | Évite de rouvrir #636 |
| 5 | Merger le volet frontend de `fix/frontend-typescript-downgrade` sur `feature/dev` | Tier-1 (merge) | Résout la partie npm ERESOLVE de #634 |
| 6 | Réparer `make docs-export-github` (installer `jq` ou conteneuriser) | Tier-2 | Évite un futur export silencieusement périmé |
| 7 | Ouvrir les 60 issues stale restantes (hors §2) au triage habituel | Tier-1 | Hors scope détaillé de ce rapport — juste signalé en §1 |

---

## Annexe — commandes utilisées (reproductibilité)

```bash
gh issue list --repo gilmry/koprogo --state all --limit 300 \
  --json number,state,title,labels,milestone,updatedAt,createdAt
gh api repos/gilmry/koprogo/milestones --paginate
gh label list --repo gilmry/koprogo --limit 200

gh auth refresh -s read:project
gh project item-list 2 --owner gilmry --format json --limit 200
gh project item-list 3 --owner gilmry --format json --limit 200

git log origin/main..origin/feature/dev --pretty=format:'%H|%s'
git log -1 --pretty=format:%B <sha>   # pour chercher #issue / Story X.Y dans le corps complet
git show --name-only --pretty=format: <sha>   # fichiers touchés, pour filtrer backend/src|frontend/src

grep -oE '#[0-9]+' docs/WBS_GO_LIVE_v0.1.0.md | sort -u

gh run list --repo gilmry/koprogo --branch feature/dev --limit 15 \
  --json databaseId,name,conclusion,status,createdAt,headSha,event
gh run view <run-id> --repo gilmry/koprogo
gh api repos/gilmry/koprogo/actions/jobs/<job-id>/logs

gh issue view 556|617|636|658 --repo gilmry/koprogo --json title,body
```
