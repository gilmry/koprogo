# Issue draft — Inventaire consolidé des échecs BDD pré-existants révélés par #524

**Status** : draft pour review humain avant `gh issue create` (Tier 1). Suite explicite de la recette #524 (« ouvrir une issue séparée par groupe de scénarios cassés mis au jour »).

---

## Titre proposé

`bug(test-infra): inventaire consolidé des ~27 scénarios BDD pré-existants rouges (révélés post-#524) — débruiter le gate avant go-live`

## Labels

`bug`, `bug:majeur`, `test`, `priority:high`

## Constat

Depuis le fix #524 (harness BDD propage enfin tous les `.run()` + `--no-fail-fast` CI), le job **BDD Tests est rouge sur toute PR** à cause d'échecs **pré-existants non liés** à la PR courante. Mesuré sur CI run [25982956110](https://github.com/gilmry/koprogo/actions/runs/25982956110) (commit `fd5cdec`, C1) — **~27 scénarios rouges sur 8 groupes**, répartis sur 5 binaires `bdd*.rs` :

| Feature (groupe) | scénarios | échecs | symptôme racine | piste |
|---|---|---|---|---|
| Meeting Resolutions and Voting System | 14 | **14** | `world.operation_success` faux — résolution créée avant validation quorum (Art. 3.87 §5 CC) | ordre workflow test OU use-case exige quorum validé avant create |
| Board of Directors Management (CdC) | 23 | 3 | DB check `mandate_duration_one_year` vs attente "≈1 an" | contrainte vs spec mandat (Art. 3.89 CC) désalignées |
| Board of Directors (CdC) | 13 | 2 | `Art. 3.89 CC mandate >3y` vs message attendu "≈1 an" | idem — assertion message obsolète OU règle changée |
| Energy Buying Groups | 14 | 2 | `assertion left == right` | seed/agrégat énergie |
| Community Notice Board | 15 | 2 | `Expected 2 notices, got 0` | état publish/seed notices |
| Stats syndic urgent tasks | 7 | 2 | `expenses_amount_check` amount=0 (`Tiny`/`Zero` @edge) | **= #526 (déjà tracé)** — décision domaine amount>0 |
| Call for Funds | 10 | 1 | `Due date must be after call date` | logique dates seed du step |
| Gamification & Achievements | 13 | 1 | `Challenge should be created` | seed/precondition challenge |

(Le groupe Stats = #526, déjà ouvert ; les 7 autres ne sont **tracés nulle part** — uniquement logs CI + commentaires de session.)

## Cause

#524 a rendu le harness honnête : ces échecs **existaient déjà** mais étaient masqués (exit-code sur le seul dernier `run_and_exit`). Ce ne sont **pas des régressions** des PR récentes (C1 #535, A2 #539 etc.) — vérifié : mêmes erreurs présentes dans le CI run pré-C1 `25843439317`. Causes hétérogènes par groupe : steps test écrits sans vérifier les contraintes/ordre métier (cf. pattern récurrent constaté sur le step @bug525 C1 — `updated_at`/`status`/`resolution_type` devinés), assertions de messages obsolètes vs règles légales (Art. 3.89), seeds incohérents.

## Recette

1. **Issue de tracking (celle-ci)** = inventaire + ownership. Pas de fix « comme ça » (CRITICAL.md) — comprendre la cause par groupe.
2. **Un sous-fix RED-first 4-catégories par groupe** (ou regroupé par binaire) : reproduire le rouge, comprendre (test faux OU prod faux OU spec obsolète), corriger la cause, 4-cat verte.
3. **Priorisation** : (a) « Meeting Resolutions » 14/14 = plus gros + légalement sensible (quorum Art. 3.87 §5) → P1 ; (b) Board CdC mandat (Art. 3.89) → P1 (légal) ; (c) Energy/Notice/CallForFunds/Gamification = seeds/asserts → P2 ; (d) Stats = #526 (décision domaine séparée).
4. **Lien gate** : tant que non résolu, le critère GO WBS « `make ci` VERT ; BDD jugé par-scénario, zéro régression » n'est atteignable que par jugement par-scénario manuel. Cette issue rend le backlog explicite et auditable pour G1.

## Critères d'acceptation

- [ ] Chaque groupe (hors #526) a son sous-fix mergé OU un statut explicite « accepté-différé » justifié et tracé
- [ ] CI BDD : 0 rouge non-tracé ; tout rouge restant pointe une issue/sous-tâche
- [ ] « Meeting Resolutions » 14/14 : cause tranchée (test vs use-case) + 4-cat verte
- [ ] Board CdC mandat : assertion alignée sur la règle légale en vigueur (Art. 3.89) + 4-cat
- [ ] Gate WBS G1 peut juger BDD sans bruit (référence cette issue)

## Hors scope

- #526 (amount=0 décision domaine) — déjà tracé, juste référencé ici.
- #443 (cascade Decimal tests) — slice distincte, déjà ouverte.
- Le fix harness lui-même (#524, CLOSED).

## Refs

- #524 (harness — origine du dévoilement), #443 (Decimal cascade), #526 (amount=0), #534 (ADR-0008), #521 (umbrella Decimal)
- CI run référence : 25982956110 (job 76374997445)
- WBS : nouveau **WP-B3** (cf. `docs/WBS_GO_LIVE_v0.1.0.md`)
