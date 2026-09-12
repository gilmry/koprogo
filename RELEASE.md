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
- **Dernière mise à jour** : 2026-09-12 (par : Claude — les cinq arbitrages 🔴 tranchés par le PO)

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
- **Prochaine action attendue** : exécuter [ADR 0050](docs/adr/0050-pile-de-recette-sur-le-vps-ports-decales.md)
  — livrables 2, 3 et 5 de #872 : décaler les quatre ports, faire viser
  `http://localhost:8090` à `make test-e2e` et `make docs-with-videos`,
  corriger `docs/E2E_TESTING_GUIDE.rst`. C'est du **harnais**, que le parcours
  place avant le métier ; ça rend `e2e` vert, donc `cap:C7.1` (rang 1) tenable,
  donc la phase B franchissable.
- **Après**, et pas avant : porter les stories à « Agent IA Ready ». Le PO a
  choisi une signature unique **après** cette préparation — la fabrication des
  fonctionnalités (#855, #802) n'ouvre donc pas tant que les stories ne sont
  pas prêtes.
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
- **Validateur** : ⚠️ **non fait**. **7 issues sur 84** portent les huit
  éléments d'une story prête, et 25 portent les quatre classes de tests
  (`scripts/backlog-pret.py`, mesuré le 2026-09-12). Le livrable porte
  `NON SIGNÉ`.

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
| 3 | C10.1 — arbitrage du groupe « Communauté » | ✅ tranché (ADR 0052) |
| 4 | C5.2 puis C5.1 — contrat de tests, socle visuel | prêtes (#797, #802 débloquée) |
| 5 | C1.1 / C1.3 — le noyau légal | #840 fermée |
| 6 | les `Should`, parallélisables | |
| 7 | les `Could` | |
| 8 | G1 puis G2 — revue humaine signée, puis le tag | hors périmètre agent |

## Arbitrages

### 🔴 En attente (le PO doit trancher une MODALITÉ)

**Aucun.** Les cinq arbitrages ouverts ont été tranchés le 2026-09-12. Ce qui
reste est du travail, pas une décision — et se lit dans « Position courante ».

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
