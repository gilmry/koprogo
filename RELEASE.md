<!--
  Registre d'état Foyer — SOURCE DE VÉRITÉ PARTAGÉE (PO ↔ dev).
  Tout agent LE LIT avant d'agir et LE MET À JOUR à chaque étape conclusive.
  Markdown volontairement lisible : le PO doit le comprendre sans outil.

  Méthode : submodule `.foyer` (épinglé). Commandes : /foyer-status, /foyer-next,
  /foyer-bascule. Si `.foyer/pilote/` est vide : `git submodule update --init --recursive`.
-->
# État Foyer — KoproGo

- **Porte active** : `release`
- **Parcours** : `.foyer/pilote/journeys/planification-release.md`
- **Archétype** : full-stack *(Rust hexagonal + Astro/Svelte 5 en îlots, PostgreSQL)*
- **Substrat d'exécution** : conteneur — `~/bin/kcargo` pour Rust, jamais `cargo` sur l'hôte
- **Démarré le** : 2026-09-12
- **Dernière mise à jour** : 2026-09-12 (par : Claude, sur demande de Gilles Maury)

## Répartition des rôles

| Rôle | Qui | Depuis |
|---|---|---|
| **PO** (tranche les modalités, valide sur preuve, signe) | Gilles Maury | 2026-09-12 |
| **Dev** (exécute, assemble la preuve, ne tranche jamais) | Gilles Maury + agent | 2026-09-12 |

**Les deux chapeaux sont portés par la même personne, et c'est temporaire.** Un
partenaire agent IPI en exercice reprendra le rôle de PO une fois le pilotage
stabilisé. D'ici là, le dogfooding ne vaut que si la séparation reste **écrite** :
un arbitrage se pose en 🔴 dans ce registre avec sa preuve **avant** d'être
tranché, même quand celui qui pose et celui qui tranche sont la même personne.
Se l'épargner reviendrait à valider ce qu'on vient de produire, ce que la méthode
appelle une signature et non une validation.

## Position courante

- **Phase / étape** : Phase A · conception BMAD ciblée sur la release · étape 4
  (Validateur → backlog « Agent IA Ready »)
- **Prochaine action attendue** : faire valider le découpage de la release. Le
  backlog est structuré et chiffré, il n'est **pas validé** — c'est un arbitrage
  de PO, inscrit plus bas.
- **Rôle à jouer** : `.foyer/pilote/roles/conception-bmad.md`

### Ce qui est déjà produit de la phase A

- **Analyste / cadrage** : `docs/WBS_v0_1_0.md` — le périmètre, sa provenance.
- **Product Manager / epics-stories** : `docs/BACKLOG_STRUCTURE_v0_1_0.md` —
  10 épopées, 32 capacités, 85 issues, classement **exhaustif et exclusif**
  (`scripts/backlog-structure.py --verifier` échoue sur toute issue non classée).
- **Architecte** : les quatre contextes bornés sont déclarés ET gardés en CI
  (`backend/tests/architecture.rs`), les dépendances croisées sont interdites.
- **Chiffrage** : 73,25 j de wall-clock superviseur · 293 tours. **Bornes hautes
  de première passe**, à resserrer sur le réel par le CSI.
- **Validateur** : ⚠️ **non fait**. 7 issues sur 85 portent les huit éléments
  d'une story prête (`scripts/backlog-pret.py`). Le livrable porte `NON SIGNÉ`.

## Gates (dernier statut — mesuré le 2026-09-12)

| Gate | Statut | Commande | Note |
|---|---|---|---|
| `plancher` secrets | 🟢 | `.claude/hooks/stop-leak-scan.sh` | bloquant, 3 hooks sur 8 bloquent vraiment |
| `plancher` migrations | ⚪ | — | réversibilité `down.sql` jamais vérifiée par un gate |
| `verify` structurel | 🟢 | `kcargo test --test architecture` + 15 gardes | 16 suites vertes |
| `contrat` anti-drift | 🟢 | gate OpenAPI + `oasdiff` en CI | #765 fermée |
| `unit` domaine | 🟢 | `kcargo test --lib` | 1989 tests |
| `integration` | 🟢 | suites `e2e_*.rs` (testcontainers) | |
| `bdd` | 🟢 | suites `bdd_*.rs` | |
| `e2e` parcours | 🔴 | `make test-e2e` | **vise `api.koprogo.com` sur le VPS** — #872 |
| `visuel` | ⚪ | — | pas de goldens |
| `doc-vivante` | 🔴 | `make docs-with-videos` | bloqué par #872, non bloquant par nature |
| front typecheck | 🟢 | `npx svelte-check --threshold error` | 0 erreur |
| front tests | 🟢 | `npx vitest run` | 652 tests, 119 fichiers |

**Le socle n'est pas vert.** La phase B du parcours exige de partir d'un vert
connu ; `e2e` est rouge et sa cause est nommée. Empiler la release avant de la
corriger serait la déviation que le parcours interdit.

## Périmètres / backlog

Le backlog vit dans **`docs/BACKLOG_STRUCTURE_v0_1_0.md`**, régénéré par
`scripts/backlog-structure.py`. Ne pas le recopier ici : deux backlogs sont déjà
un défaut de structure. Seul l'ordre des capacités est repris.

| Rang | Capacité | État |
|---|---|---|
| 1 | C7.1 — la recette peut se connecter et s'exécuter | 🔴 bloquée par #872 |
| 2 | C4.1 / C4.2 / C4.3 — identité, périmètre, RGPD | en cours (#845 à 1 route) |
| 3 | C10.1 — arbitrage du groupe « Communauté » | 🔴 arbitrage |
| 4 | C5.2 puis C5.1 — contrat de tests, socle visuel | prêtes (#797, #802) |
| 5 | C1.1 / C1.3 — le noyau légal | #840 fermée |
| 6 | les `Should`, parallélisables | |
| 7 | les `Could` | |
| 8 | G1 puis G2 — revue humaine signée, puis le tag | hors périmètre agent |

## Arbitrages

### 🔴 En attente (le PO doit trancher une MODALITÉ)

- **Point** : découpage de la release v0.1.0
  - **Preuve jointe** : 85 issues classées en 32 capacités, exhaustif et exclusif,
    chiffrées à 73,25 j / 293 tours (`docs/BACKLOG_STRUCTURE_v0_1_0.md`).
  - **Question de modalité** : dans quel ordre et jusqu'où va la 0.1.0 ? La
    décision du 2026-09-06 a mis les 85 au périmètre du tag ; rien ne l'a
    réexaminée depuis que le chiffrage existe.
  - **Options** : A) tenir les 85 · B) sortir les `Could` (T4 doc vivante, C5.3
    maquettes, C9.2 IaC) vers 0.2.0 · C) ne garder que les `Must`.

- **Point** : #872 — pile de recette jetable sur le VPS
  - **Preuve jointe** : la pile de dev revendiquait les conteneurs et le **volume
    de données** de la démo ; fermé par `name:` explicite (commit `d35332de`),
    garde `garde-piles-compose-distinctes`. Mais `make test-e2e` vise toujours
    `api.koprogo.com`, et le port 80 de l'hôte est tenu par la démo.
  - **Question de modalité** : monte-t-on la pile de recette sur cet hôte (quels
    ports, quel réseau, quelle fenêtre) ou ailleurs ?
  - **Options** : A) ports décalés sur le VPS · B) hôte séparé · C) en CI seulement.

- **Point** : #855 — accès du notaire à un état daté
  - **Destination déjà tranchée** le 2026-09-12 : lien signé à durée limitée.
  - **Preuve jointe** : c'est la **dernière** route sans identité du produit
    (`garde_identite_absente` : 30 → 1) ; la référence ne porte que 32 bits
    d'aléa, sans expiration ni journal d'accès.
  - **Questions de modalité restantes** : (1) usage unique ou relecture pendant
    l'instruction de la vente ? (2) quel `subject_user_id` pour un notaire sans
    compte, le champ étant aujourd'hui obligatoire ?

- **Point** : #856 — le comptable voit-il un groupe « Communauté » réduit ?
  - **Preuve jointe** : la remise de design le propose, un test `@security` de
    `Navigation.test.ts` l'interdit. La revue tranche elle-même en faveur du test.
  - **Question de modalité** : change-t-on la règle dans `permissions.ts` — avec
    test mis à jour, commentaire et référence d'issue — ou la maquette ?
  - **Borne** : tant que ce n'est pas tranché, la story #802 livre tout sauf ce point.

- **Point** : signature du livrable BMAD
  - **Preuve jointe** : `docs/BACKLOG_STRUCTURE_v0_1_0.md`, frontmatter
    `etat: NON SIGNÉ`.
  - **Question de modalité** : le PO le relit-il maintenant, ou après l'arbitrage
    de découpage ci-dessus ?

### ✅ Tranchés

| Point | Décision (modalité) | Par | Le | ADR |
|---|---|---|---|---|
| Accès notaire à l'état daté | lien signé à durée limitée, émis par le syndic, révocable, journalisé | Gilles Maury | 2026-09-12 | à écrire — #855 |
| Mot de passe superadmin de la démo | `admin123` posé dans l'environnement, pour survivre à l'upsert du seed ; coût assumé : il est publié dans le dépôt | Gilles Maury | 2026-09-12 | #870 |
| Porte du pilote Foyer | `release` — le produit tourne, on cadre un gros incrément | Gilles Maury | 2026-09-12 | ce registre |

## Journal (chronologie courte)

- 2026-09-12 — pilote Foyer installé : submodule `.foyer` épinglé à `f6fe69c`,
  quatre commandes dans `.claude/commands/`, ce registre créé.
- 2026-09-12 — backlog restructuré par capacité (`1a14a8e4`), chiffré, et
  matérialisé sur GitHub en étiquettes `epic:*` / `cap:*`.
- 2026-09-12 — #872 ouverte puis son risque de destruction fermé (`d35332de`).
- 2026-09-12 — #840 fermée (Art. 3.87 §2, décision hors ordre du jour nulle).
- 2026-09-12 — huit commits poussés, `ca7de310..0adb9b8e`.
