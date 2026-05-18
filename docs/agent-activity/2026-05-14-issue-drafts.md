# 2026-05-14 — Issue drafts from CI investigation (#521 Story A side findings)

**Context** : while implementing BDD scaffold for Story A (#521 f64/NUMERIC panic), CI run 25843439317 revealed three pre-existing findings unrelated to #521 itself. Each warrants a distinct issue.

---

## Issue draft 1 — BDD test runner silently swallows scenario failures

### Title proposed
`bug(test-infra): BDD harness in bdd_financial.rs only exits non-zero on the LAST run_and_exit — all prior .run() failures invisible to CI`

### Labels
`bug`, `bug:majeur`, `test`, `priority:high`

### Constat

Dans `backend/tests/bdd_financial.rs:main()` (lignes ~7355-7395), 12 features sont exécutées via `FinancialWorld::cucumber().run("...")` puis une dernière via `.run_and_exit("...expenses.feature")`. Le binaire de test exit avec le code de retour du DERNIER `run_and_exit` uniquement — toutes les failures des `.run()` précédents sont invisibles côté process exit.

**Effets observés dans CI run 25843439317 (commit 021ae95)** :
- ≥ 10 scenarios panic dans les features intermédiaires (board_members, call_for_funds, resolutions, energy_campaigns, stats_urgent_tasks, etc.)
- `cargo test --test bdd_financial` retourne 0
- GitHub Actions report **"BDD Tests: success"** alors que ~10 scenarios échouent réellement

Le même pattern est probablement reproduit dans `bdd.rs`, `bdd_governance.rs`, `bdd_community.rs`, `bdd_operations.rs`, `bdd_iot.rs` (à vérifier).

### Cause

L'API cucumber-rs `.run()` n'exit pas, retourne juste après. Seul `.run_and_exit()` regarde `world.failures_count()` et appelle `process::exit(1)` si > 0. Le code actuel ne consolide pas les failures across features.

### Recette

Deux options :

**a) `.run_and_exit()` pour CHAQUE feature** — simple mais arrête au premier feature en échec, ne donne pas un rapport complet.

**b) Accumulator pattern** — collecter `cucumber::Cucumber::run().await` results et exit 1 si une feature a échoué :
```rust
let mut had_failures = false;
let summary = FinancialWorld::cucumber().run("tests/features/payments.feature").await;
if summary.failed_count() > 0 { had_failures = true; }
// ... répéter pour toutes ...
if had_failures { std::process::exit(1); }
```

Option **b** recommandée car elle préserve l'exécution de toutes les features pour rapport complet, tout en faisant fail le job CI si une seule scenario panic.

### Critères

- [ ] Pour les 5 fichiers `bdd*.rs`, le test process exit 1 dès qu'un scenario fail dans n'importe quelle feature
- [ ] CI Action GitHub "BDD Tests" devient RED pour les ~10 scenarios cassés actuels (intentionnel — révélateur)
- [ ] Suivi : ouvrir une issue séparée par groupe de scenarios cassés mis au jour

### Impact

Bloque toute observation fiable des tests BDD. **Critique pour la Story A #521** : sans ce fix, on ne peut pas démontrer le RED du panic f64/NUMERIC en CI.

---

## Issue draft 2 — Decimal vs FLOAT8 ColumnDecode sur colonne `quota` (vote_repository)

### Title proposed
`bug(backend): ColumnDecode Decimal vs FLOAT8 on quota column — inverse de #521, vote_repository`

### Labels
`bug`, `bug:majeur`, `backend`, `priority:high`

### Constat

Dans CI run 25843439317 BDD logs, plusieurs panics répétés :

```
called `Result::unwrap()` on an `Err` value: ColumnDecode { 
  index: "quota", 
  source: "mismatched types; Rust type `rust_decimal::decimal::Decimal` 
           (as SQL type `NUMERIC`) is not compatible with SQL type `FLOAT8`" 
}
```

Apparait ~13 fois sur des scenarios votes/resolution (bdd_governance.rs probablement).

### Cause

L'**inverse exact de #521** : ici la colonne `quota` est `FLOAT8` en DB mais le code Rust attend `Decimal`. Il y a deux interprétations possibles :
- Soit la colonne devrait être `NUMERIC` (cohérent avec `project_no-f64-in-money.md`) → migration SQL nécessaire
- Soit le code Rust devrait lire `f64` (mais alors viole `no-f64-in-money` si quota a une sémantique financière)

`quota` dans le contexte des résolutions AG = pourcentage de tantièmes pour quorum/majorité → c'est un ratio, pas un montant. Donc f64 acceptable, mais alors la migration aurait dû être en FLOAT8 et le code en f64. Le mismatch indique une demi-migration vers Decimal jamais terminée.

### Recette

1. Localiser le fichier (probablement `backend/src/infrastructure/database/repositories/resolution_repository_impl.rs` ou `vote_repository_impl.rs`)
2. Décider sémantiquement : `quota` est-il un montant (Decimal/NUMERIC) ou un ratio (f64/FLOAT8) ?
3. Aligner code + schéma DB conformément
4. Tests RED-first 4 catégories sur l'endpoint qui appelle ce code

### Critères

- [ ] `grep "quota" backend/src/infrastructure/database/repositories/` retourne 0 mismatch type SQL/Rust
- [ ] BDD scenarios resolution / vote ne panic plus avec ColumnDecode
- [ ] Décision sémantique tracée dans le code (commentaire) ou ADR

### Refs

- Inverse de #521 (f64-on-NUMERIC)
- `project_no-f64-in-money.md`

---

## Issue draft 3 — `expenses_amount_check` constraint : 0-amount expenses interdites ?

### Title proposed
`question(domain): expenses_amount_check rejects amount = 0 — intentionnel ou trop strict ?`

### Labels
`question`, `domain`, `backend`

### Constat

Lors de l'écriture des BDD @edge pour #521 Story A, le scenario test "Zero amount = 0.0000" déclenche :

```
violates check constraint "expenses_amount_check"
detail: "Failing row contains (..., maintenance, Zero, 0.00, ...)"
```

### Question domain

Est-ce qu'une charge à 0 € a un sens métier ? Cas réels possibles :
- Devis prévisionnel à 0 (placeholder) — probablement non, on n'inscrirait pas l'expense
- Régulation comptable avec ligne à 0 (rare)
- Charge offerte par un fournisseur (gratuité ponctuelle)

Si la contrainte est intentionnelle (amount > 0), alors le test @edge devrait utiliser une valeur minimale légale (e.g. 0.01) au lieu de 0. Si la contrainte est trop stricte (amount >= 0), il faut relaxer la check constraint.

### Recette

- Décision produit / domain expert
- Si garder `> 0` : ajuster le test #521 @edge Zero → utiliser 0.01
- Si relaxer à `>= 0` : migration SQL + test BDD passe tel quel

### Refs

- Découvert via BDD #521 Story A scenario @edge

### Hors scope #521

Cette question est tangente à #521 (qui concerne le decode f64/NUMERIC). Devrait être traitée séparément pour ne pas polluer le scope Story A.

---

## Recommandation d'ordre

1. **Issue 1 (BDD infra)** d'abord — bloque la validation CI de toutes les autres issues
2. **Issue 2 (quota Decimal/FLOAT8)** — bug réel impactant des features
3. **Issue 3 (expenses_amount_check)** — clarification domain, basse priorité

**Côté #521 Story A** : continuer la migration prod code (f64→Decimal sur les 3 endpoints) ; quand Issue 1 sera fixée, le RED apparaîtra naturellement en CI. En attendant, le RED est constaté manuellement dans les logs (cf. ce document).
