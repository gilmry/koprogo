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
- **Dernière mise à jour** : 2026-09-12 (par : Claude — rangs 2 à 4 portés à « Agent IA Ready »)

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
- **Prochaine action attendue** : poursuivre l'étape 4 — rang 5, le **noyau
  légal** (C1.1 et C1.3, 9 issues), puis rang 6 (les `Should`, 23 issues) et
  rang 7 (les `Could`, 39). Les rangs 1 à 4 sont prêts ; restent 67 issues.
- **La fabrication n'ouvre pas encore.** Le PO a choisi une signature unique
  **après** la préparation complète ; ni #872 (harnais, ADR 0050) ni #855 ni
  #802 n'entrent en fabrication avant. L'ADR 0050 attend, elle n'est pas
  perdue.
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
- **Validateur** : ⏳ **en cours**. **17 issues sur 84** portent les huit
  éléments d'une story prête, et 35 portent les quatre classes de tests
  (`scripts/backlog-pret.py`, mesuré le 2026-09-12 après le rang 4). Le
  livrable porte `NON SIGNÉ` — la signature vient après la préparation.

> **Rangs 1 à 4 prêts.** C7.1 l'était (4/4) ; puis les sept issues de C4.1, C4.2
> et C4.3, #856 (C10.1), et #803 / #834 (C5.2 / C5.1 — #802 et #797 l'étaient
> déjà). Le contrôle est **de forme** : le script cherche des marqueurs, pas du
> sens. Il l'écrit lui-même — « une borne haute de la préparation, jamais un
> verdict ».

> **84, et non 85.** Le backlog structuré a été généré à 85 issues ouvertes ;
> #840 a été fermée le même jour, après la génération. Le document n'est pas
> faux, il est daté — il se régénère par `scripts/backlog-structure.py`.

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
| 1 | C7.1 — la recette peut se connecter et s'exécuter | débloquée — ADR 0050 à exécuter |
| 2 | C4.1 / C4.2 / C4.3 — identité, périmètre, RGPD | en cours (#845 à 1 route) |
| 3 | C10.1 — arbitrage du groupe « Communauté » | ✅ tranché (ADR 0052) · story prête |
| 4 | C5.2 puis C5.1 — contrat de tests, socle visuel | ✅ 4/4 stories prêtes |
| 5 | C1.1 / C1.3 — le noyau légal | #840 fermée |
| 6 | les `Should`, parallélisables | |
| 7 | les `Could` | |
| 8 | G1 puis G2 — revue humaine signée, puis le tag | hors périmètre agent |

## Arbitrages

### 🔴 En attente (le PO doit trancher une MODALITÉ)

- **Point** : #694 — son rang contredit ce qu'elle dit d'elle-même
  - **Preuve jointe** : l'issue écrit « **non bloquant pour v0.1.0** (bêta
    fermée) », et elle est classée `cap:C4.2` — **Must**, **rang 2**. Les deux
    ne peuvent pas être vrais ensemble. Relevé en rédigeant sa story.
  - **Question de modalité** : l'ordre de release est une décision de PO. Soit
    l'issue est périmée sur ce point, soit son rang l'est.
  - **Options** : A) elle reste au rang 2, la mention « non bloquant » est
    retirée · B) elle descend au rang 6 avec les `Should` · C) elle reste au
    rang 2 et sa mention est justifiée dans l'issue.
  - **Borne** : [ADR 0049](docs/adr/0049-perimetre-v0-1-0-integral.md) a fixé
    qu'elle ne sort **pas** du périmètre. Seul son rang est en question.

- **Point** : #694 — une question de **destination**, pas de modalité
  - **Preuve jointe** : la story le pose explicitement et bloque dessus.
  - **Question** : un syndic a-t-il accès à l'organisation entière par défaut,
    avec restriction optionnelle — ou refus par défaut et accès ACP explicite ?
    Le refus par défaut est plus sûr et plus coûteux à déployer sur l'existant.
  - **Borne** : ne se tranche pas au moment du code. La story n'entre pas en
    fabrication avant.

### ✅ Tranchés

| Point | Décision (modalité) | Par | Le | ADR |
|---|---|---|---|---|
| Accès notaire à l'état daté *(destination)* | lien signé à durée limitée, émis par le syndic, révocable, journalisé | Gilles Maury | 2026-09-12 | [ADR 0051](docs/adr/0051-lien-notaire-sept-jours-renouvelable.md) |
| Mot de passe superadmin de la démo | `admin123` posé dans l'environnement, pour survivre à l'upsert du seed ; coût assumé : il est publié dans le dépôt | Gilles Maury | 2026-09-12 | #870 |
| Porte du pilote Foyer | `release` — le produit tourne, on cadre un gros incrément | Gilles Maury | 2026-09-12 | ce registre |
| **Découpage de la release v0.1.0** | **option A — périmètre intégral** : les 84 issues ouvertes restent au tag, aucun report en 0.2.0. `Must/Should/Could` ordonne l'exécution, ne retire rien | Gilles Maury | 2026-09-12 | [ADR 0049](docs/adr/0049-perimetre-v0-1-0-integral.md) |
| **#872 — pile de recette** | **sur le VPS, ports décalés** : 8090 HTTP, 8091 dashboard, 15432 PG, 19000/19001 MinIO. La séparation `koprogo-dev` ≠ `koprogo` étant déjà commitée, il ne restait que des ports | Gilles Maury | 2026-09-12 | [ADR 0050](docs/adr/0050-pile-de-recette-sur-le-vps-ports-decales.md) |
| **#855 — lien du notaire** | **sept jours, plusieurs lectures, renouvelable** par le syndic ; chaque consultation journalisée | Gilles Maury | 2026-09-12 | [ADR 0051](docs/adr/0051-lien-notaire-sept-jours-renouvelable.md) |
| **#856 — « Communauté » et le comptable** | **la maquette est corrigée** ; `permissions.ts` et le test `@security` ne bougent pas. Débloque #802 intégralement | Gilles Maury | 2026-09-12 | [ADR 0052](docs/adr/0052-le-comptable-ne-voit-pas-communaute.md) |
| **Signature du livrable BMAD** | **une seule signature, après** que les stories portent les huit éléments. Elle attestera que la fabrication peut commencer, pas seulement que le classement tient | Gilles Maury | 2026-09-12 | ce registre |

## Journal (chronologie courte)

- 2026-09-12 — **rang 4 porté à « Agent IA Ready »** : #803 (C5.2) et #834
  (C5.1) ; #802 et #797 l'étaient déjà. Les deux stories partagent une même
  règle de séquencement — **celui qui touche un écran l'ancre et le traduit dans
  le même commit**, plutôt qu'une correction de masse invérifiable. 17/84.
- 2026-09-12 — **rang 3 porté à « Agent IA Ready »** : #856 (C10.1). Sa story
  exécute l'ADR 0052 et nomme la lecture qui concilie tout — l'accès
  communautaire d'un comptable viendrait d'une **désignation** de community
  manager (ADR 0046), jamais de son rôle. 15/84.
- 2026-09-12 — **rang 2 porté à « Agent IA Ready »** : sept stories écrites
  sur #845, #864 (C4.1), #868, #841, #798, #694 (C4.2) et #842 (C4.3). La
  préparation passe de 7/84 à **14/84**, mesuré. #772 étant fermée, la
  précondition de #798 est levée.
- 2026-09-12 — **les quatre arbitrages restants tranchés** : #872 recette sur
  le VPS à ports décalés (ADR 0050), #855 lien notaire sept jours renouvelable
  (ADR 0051), #856 la maquette cède au test `@security` (ADR 0052), signature
  BMAD unique et différée. Plus aucun 🔴 ouvert.
- 2026-09-12 — **une mesure a corrigé le registre** : `make test-e2e` ne « vise »
  pas `api.koprogo.com` par configuration — il vise `http://localhost`, que
  Traefik route vers la démo sur cet hôte. Et la raison décisive de ne pas
  l'y laisser n'est pas la pollution : c'est que `make seed-reset` et
  `make reset-db`, préconditions de toute recette reproductible, effaceraient
  la démo.
- 2026-09-12 — **découpage de la release tranché : option A**, périmètre
  intégral confirmé face à son chiffrage (ADR 0049). La décision du 2026-09-06
  n'est plus seulement héritée, elle est opposée à son coût.
- 2026-09-12 — pilote Foyer installé : submodule `.foyer` épinglé à `f6fe69c`,
  quatre commandes dans `.claude/commands/`, ce registre créé.
- 2026-09-12 — backlog restructuré par capacité (`1a14a8e4`), chiffré, et
  matérialisé sur GitHub en étiquettes `epic:*` / `cap:*`.
- 2026-09-12 — #872 ouverte puis son risque de destruction fermé (`d35332de`).
- 2026-09-12 — #840 fermée (Art. 3.87 §2, décision hors ordre du jour nulle).
- 2026-09-12 — huit commits poussés, `ca7de310..0adb9b8e`.
