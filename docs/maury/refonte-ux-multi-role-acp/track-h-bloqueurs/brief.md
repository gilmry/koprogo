---
feature: refonte-ux-multi-role-acp/track-h-bloqueurs
phase: A (Vision TOGAF)
status: SIGNED v1.0 par @gilmry 2026-06-15
date: 2026-06-15
authors: [Claude Opus 4.7 (drafting), @gilmry (signature 2026-06-15)]
related_issues: [553, 554]
parent_maury: refonte-ux-multi-role-acp (brief signé 2026-05-20)
---

# Brief Track H — Bloqueurs légaux v0.1.0

## 1. Vision

**KoproGo v0.1.0 ne peut pas être mise en bêta sans garantir que tout calcul opérationnel (charges, appels de fonds, répartition tantièmes, AG clôturable) repose sur une copropriété juridiquement conforme.**

Trois invariants légaux belges (Code civil livre 3, titre 8 « De la copropriété ») doivent être inviolables au runtime :

1. **Admin garant conformité** (`Building.is_conformant()`) — un building dont la fiche ment (count units ≠ `total_units` OR Σ quotas ≠ `total_tantiemes`) est **invisible** aux syndics et **inutilisable** pour tout calcul. **NB acte de base** : `total_tantiemes` est lu sur le building lui-même (1000 millièmes typique mais 10000 fréquent pour immeubles à lots fractionnés finement) — PAS constant.
2. **Validate-before-compute** — tout use-case produisant un chiffre opérationnel commence par `building.assert_conformant()?`. Si KO, erreur typée 422 + audit + banner FE non-modifiable.
3. **AG `assert_can_complete()`** (Art. 3.87 §3-5 CC) — Meeting ne passe pas à `Completed` sans : convocations envoyées (Art. 3.87 §3), votes clôturés (Art. 3.87 §4), présents/représentés enregistrés (Art. 3.87 §5), PV draft.

Ces invariants sont **non-négociables**. Une bêta avec ces gaps = risque juridique pour @gilmry (syndic responsable) et données opérationnelles corrompues.

## 2. Personas concernés

### 2.1. Admin garant conformité (interne — @gilmry)

- **Rôle** : Saisit ou édite la fiche immeuble (total_units, quotas par lot).
- **Responsabilité légale** : Garantit qu'au moment où un syndic prend possession de l'immeuble, `Σ quotas = total_tantiemes` (acte de base — typique 1000 millièmes, mais 10000 sur immeubles à lots fractionnés finement comme celui de @gilmry à 182 lots) ET `count(units) = total_units`.
- **Frustration actuelle** : Aucun garde-fou. Drift silencieux. **Bug critique courant** : `building.rs:219` `is_conformant` compare à constante `CONFORMANT_QUOTA_TOTAL = dec!(1000)` au lieu de `self.total_tantiemes` — TOUT immeuble avec acte de base ≠ 1000 (10000 par ex.) est aujourd'hui classifié non-conforme à tort. Cf. mémoire `project_admin-publishes-conform-buildings`.
- **Besoin** : Voir clairement quel building est conforme/non-conforme + bloquer publication si KO.

### 2.2. Syndic exploitant (cible v0.1.0)

- **Rôle** : Génère charges, appels de fonds, AG, états de date sur les immeubles dont il a la gestion.
- **Responsabilité légale** : Art. 3.89 CC — chiffres communiqués aux copropriétaires doivent être exacts.
- **Frustration actuelle** : Peut générer un appel de fonds sur un building dont la somme quotas dérive (= répartition fausse, contestation possible en justice).
- **Besoin** : Erreur explicite (pas calcul erroné silencieux) si building non-conforme. Banner FE qui indique « immeuble non utilisable — contacter l'admin ».

### 2.3. Owner constatant (lecture)

- **Rôle** : Consulte ses charges, vote en AG, télécharge PV.
- **Responsabilité** : Aucune sur conformité — passif, contrôle a posteriori.
- **Frustration potentielle** : Recevrait un appel de fonds basé sur quotas erronés sans le savoir.
- **Besoin v0.1.0** : Protection passive — la chaîne de validate-before-compute empêche que ça arrive.

## 3. Capacités business (CB)

| CB | Description | WP WBS |
|---|---|---|
| **CB-H1** | Admin voit le statut de conformité (banner + delta quotas) sur la fiche d'un building. | H1 (partiel fait) |
| **CB-H2** | Syndic ne voit dans sa liste QUE les buildings conformes (filtrage role-based). | H1 (fait) |
| **CB-H3** | Erreur 422 typée + payload détaillé (`deltas: { units: -1, quota: -2.5 }`) si syndic force un calcul sur un building non-conforme via API. | H1 (gap erreur) |
| **CB-H4** | Pre-check `assert_conformant()?` dans `expense_use_cases::create_expense`, `call_for_funds_use_cases::send_call_for_funds`, `charge_distribution_use_case::compute_distribution`, `etat_date_use_cases::generate_etat_date`. | H2 |
| **CB-H5** | Pre-check `Meeting::assert_can_complete()?` dans `meeting_use_cases::complete_meeting`. | H3 |
| **CB-H6** | Audit log (`security_incident`) chaque tentative de calcul/clôture sur entité non-conforme. | H1+H2+H3 |
| **CB-H7** | FE banner non-conforme persistant + désactivation boutons calcul (charges, appel de fonds, clôturer AG). | H1+H2+H3 |
| **CB-H8** | Toast 422 narratif (« 2 lots manquants — Σ quotas = 997.5 / {quota_basis} » où `quota_basis` vient du payload, p.ex. 1000 ou 10000) sur action bloquée. | H2+H3 |
| **CB-H9** | Bouton « Modifier immeuble » admin fonctionnel + regression spec Playwright. | B4 |

## 4. Invariants techniques (INV-H)

| INV | Énoncé | Origine |
|---|---|---|
| **INV-H1** | `Building::is_conformant(metrics) == (metrics.units_count == self.total_units && metrics.quota_sum == Decimal::from(self.total_tantiemes))`. Decimal strict, pas de tolérance arrondi. **Total = acte de base du building (1000/10000/autre)**, PAS constante. | Existant building.rs:213 (à fixer — bug constante `CONFORMANT_QUOTA_TOTAL`). |
| **INV-H2** | `Building::assert_conformant(metrics) -> Result<(), BuildingNotConformantError>` retourne `BuildingNotConformantError { building_id, units_delta, quota_delta, quota_basis }` typé (pas `String`). `quota_basis = self.total_tantiemes` capture l'acte de base. | NOUVEAU. Règle CRITICAL.md #4. |
| **INV-H3** | `BuildingNotConformantError` → `AppError::Validation` → HTTP 422 + body JSON avec `details: { building_id, units_delta, quota_delta, quota_basis }`. FE peut afficher "997.5 / 10000" et pas "997.5 / 1000". | NOUVEAU. |
| **INV-H4** | Tout use-case dans la liste CB-H4 appelle `assert_conformant(metrics)?` **avant** toute mutation/calcul. Test 4-cat `@security` qui prouve : impossible de bypass. | NOUVEAU. |
| **INV-H5** | `Meeting::assert_can_complete(checklist) -> Result<(), MeetingNotCompletableError>` vec MissingInvariant : [`ConvocationsNotSent`, `VotesNotClosed`, `AttendanceNotRecorded`, `QuorumNotReached`, `MinutesDraftMissing`]. | NOUVEAU. Art. 3.87 §3-5 CC. |
| **INV-H6** | Toute tentative de bypass (calcul, clôture, mutation) sur entité non-conforme génère 1 `security_incident` (type `BUILDING_NOT_CONFORMANT` ou `MEETING_NOT_COMPLETABLE`) audité. | NOUVEAU. |
| **INV-H7** | FE banner conformity persistant sur toute page liée au building non-conforme (BuildingDetail, ExpenseList, CallForFundsList, MeetingList, EtatDateList). ARIA `role="alert"` + texte + icône + couleur (a11y daltonien). | NOUVEAU. Mémoire a11y-wcag-aa-baseline. |
| **INV-H8** | FE boutons calcul (« Nouvelle dépense », « Générer appel de fonds », « Clôturer AG », « Générer état de date ») désactivés (`disabled` + `aria-disabled`) si banner conformity actif. | NOUVEAU. |
| **INV-H9** | Tests 4-cat (`@happy` + `@edge` + `@security` + `@negative`) RED-first par invariant ET par use-case impacté. Pas de fix « pour que le test passe » — comprendre la cause (CRITICAL.md rouge). | Mémoire tdd-bdd-four-categories. |

## 5. Critères de succès (SCB)

| SCB | Mesure |
|---|---|
| **SCB-H1** | `cargo test --lib building::tests::assert_conformant` → 4 scénarios (happy/edge/security/negative) GREEN. |
| **SCB-H2** | `cargo test --test bdd validate_before_compute` → 4 use-cases × 4-cat = 16 scénarios GREEN. |
| **SCB-H3** | `cargo test --lib meeting::tests::assert_can_complete` → 5 invariants × 4-cat ≥ 20 scénarios GREEN. |
| **SCB-H4** | Playwright `building-conformity-blocking.spec.ts` GREEN multi-rôle (admin crée non-conforme → syndic ne le voit pas → admin corrige → syndic le voit). |
| **SCB-H5** | Playwright `meeting-completion-blocked.spec.ts` GREEN (syndic tente clôture sans quorum → toast 422 + boutons disabled). |
| **SCB-H6** | Playwright `building-edit-modal.spec.ts` GREEN (regression WP-B4 : admin clique Modifier → modal → submit → flash success). |
| **SCB-H7** | `make ci` GREEN sur feature/dev avec Track H mergé. Pas de nouveau testIgnore. |
| **SCB-H8** | Aucun nouvel `unwrap()` / `expect()` introduit. `Result<E>` typé partout. Hook PostToolUse propre. |
| **SCB-H9** | Audit `security_incident_repository` retourne ≥ 1 row par tentative bypass (test BDD). |

## 6. Hors-scope explicite

- **Refacto SQL `is_conformant` via index ou view matérialisée** — performance perçue acceptable jusqu'à v0.2.0 (50 buildings max bêta fermée).
- **Migration data buildings existants** — la constante `CONFORMANT_QUOTA_TOTAL=1000` était en vigueur jusqu'à présent ; donc tous les buildings créés avec `total_tantiemes != 1000` étaient déjà mal classifiés. Pas de migration de données nécessaire (`is_conformant` est pure calcul, pas stocké) — juste le code change.
- **Self-healing data migration** — admin doit corriger manuellement les buildings existants drifted. Script de rapport read-only autorisé, pas de mutation auto.
- **Notifications email admin si drift introduit** — v0.2.0.
- **Workflow `request to publish` admin → syndic** — v0.2.0 (#553 mention extension mais pas v0.1.0).
- **Meeting cycle de vie complet** (scheduled → in_progress → completed → contestable) — v0.1.0 garde le triple state actuel (Scheduled/Completed/Cancelled), juste enrichi par `assert_can_complete()`.
- **Stories 4.1-4.4, 4.6-4.8** (Maury slice 4 governance hybride + eIDAS) — restent hors v0.1.0 (cf. WBS arbitrage line 173).

## 7. Risques et mitigations

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Cascade refactor `String` → `AppError` dans use-cases legacy (call_for_funds.rs) explose le scope | Moyenne | Élevé | Story H2 documente exhaustivement les 4 use-cases + mention « Result<_, String> hérité, garde-le tel quel hors gap conformity ». Pas de refacto global ; juste ajout pre-check + nouvelle erreur. |
| Tests BDD `validate_before_compute.feature` lents (testcontainers Postgres × 16 scénarios) | Moyenne | Moyen | Réutiliser WorldBuilder existant (cf. mémoire `world-model-seed`). Scénarios partagent setup buildings conformes/non-conformes. |
| FE banner conformity casse layouts existants (BuildingDetail, ExpenseList, etc.) | Faible | Moyen | Pattern atomique `<ConformityBanner>` réutilisable (analogue ConformityBadge déjà existant). Snapshot tests Vitest. |
| Audit `security_incident` insert lourd si beaucoup de tentatives bypass | Faible | Faible | Insert async + table dédiée déjà existante. Pas de prélog/postlog. |
| Story H2 `[cluster-coord]` 4 use-cases déraille parallélisme docker | Moyenne | Moyen | Mémoire `docker-parallelism-bottleneck` : 1 agent BE-H2 séquentiel sur les 4 use-cases (caches partagés sérialisent quand même). Pas 4 // agents. |
| Modifier immeuble (B4) déjà cassé en production de manière qu'on n'a pas vu | Faible | Faible | Cartographie 2026-06-15 montre code et binding OK. Regression spec confirmera. Si fail réel à l'exécution, ouvrir issue séparée. |

## 8. Budget tokens estimé

- Brief signé (ce fichier) : ~300 lignes ✓
- PRD : ~250 lignes (FR-H1, FR-H2, FR-H3, FR-B4)
- Architecture : ~400 lignes (error types + use-case pattern + FE banner)
- Stories : ~700 lignes (4 stories Maury-grade)
- **Total docs** : ~1650 lignes ≈ 80k tokens
- Exec agents (BE H1+H2+H3 + FE wave 5-6 composants + 6 specs) : ~150-200k tokens budget agents
- **Budget total Track H** : ~250k tokens (vs ~2.1M tokens Phase B FE — beaucoup plus petit, scope ciblé).

## 9. Signature

```
Mary (Brief)         : SIGNED v1.0 par @gilmry 2026-06-15
```

→ PRD débloqué (`prd.md`).
