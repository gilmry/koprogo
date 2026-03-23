================================================================================
R&D: Stratégies de test avancées — Property-based, Contract, Load Testing
================================================================================

:Issue: #233
:Date: 2026-03-23
:Status: R&D Documentation
:Phase: Post-MVP (Jalon 1+)

**Objectif**: Documenter les stratégies de test avancées pour KoproGo afin de
garantir la robustesse du système dans un contexte de conformité légale belge
et de fiabilité critique (propriété, finances).

Table des matières
==================

1. État actuel de la pyramide de test
2. Property-based testing avec proptest-rs
3. Contract testing avec Pact
4. Load testing avec k6 + Criterion
5. Mutation testing avec cargo-mutants
6. Snapshot testing pour les API responses
7. Recommandations d'implémentation

État actuel de la pyramide de test
==================================

KoproGo suit actuellement une **pyramide de test classique**:

.. code-block:: text

    E2E Tests (Playwright)
        ↑
    BDD Tests (Cucumber/Gherkin)
        ↑
    Integration Tests (testcontainers PostgreSQL)
        ↑
    Unit Tests (Domain logic)

**Couche 1: Unit Tests (100% des entités domain)**

- Location: ``backend/tests/lib.rs`` + ``#[cfg(test)]`` modules inline
- Cibles: Domain entities (Building, Expense, UnitOwner, Meeting, etc.)
- Framework: Rust ``#[test]`` ou ``cargo test --lib``
- Couverture: ~95% (domain invariants, state machines)
- Vitesse: < 100ms pour 1000 tests
- Exemple:

.. code-block:: rust

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_unit_owner_validation_exceeds_100_percent() {
            // Test métier belge: quotes-parts <= 100%
            let unit = Unit::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                "101",
                Some(2),
                50.0,
            ).unwrap();

            let owner1 = UnitOwner::new(unit.id, Uuid::new_v4(), 0.60);
            let owner2 = UnitOwner::new(unit.id, Uuid::new_v4(), 0.50);

            // La base de données devrait rejeter la somme > 100%
            assert!(owner2.is_err());
        }
    }

**Couche 2: Integration Tests (Avec testcontainers PostgreSQL)**

- Location: ``backend/tests/integration/`` (29 fichiers)
- Cibles: Repositories, Use Cases, workflows complets
- Framework: ``testcontainers`` (spin up PostgreSQL pour chaque test)
- Couverture: ~80% (business logic + database constraints)
- Vitesse: ~5-10s par test (I/O PostgreSQL)
- Exemple:

.. code-block:: rust

    #[tokio::test]
    async fn test_building_creation_with_multitenancy() {
        let container = PostgresContainer::new();
        let pool = container.connect().await.unwrap();

        let building = BuildingRepository::create(
            &pool,
            Building::new("Test Building", "Rue Test 1", 50, 2000).unwrap(),
        ).await.unwrap();

        let fetched = BuildingRepository::find_by_id(&pool, building.id)
            .await
            .unwrap();

        assert_eq!(fetched.name, "Test Building");
    }

**Couche 3: BDD Tests (Cucumber/Gherkin)**

- Location: ``backend/tests/features/*.feature`` (26 fichiers)
- Cibles: User journeys (auth, meetings, voting, expenses)
- Framework: ``cucumber`` crate (BDD scenarios)
- Couverture: ~70% (user-facing workflows)
- Format: Gherkin (français + english)
- Exemple:

.. code-block:: gherkin

    Feature: Meeting Voting System
        As a co-owner
        I want to vote on resolutions
        So that my voice is heard in decision-making

        Scenario: Simple majority voting
            Given a meeting with 100 total voting power
            When a resolution requires simple majority
            And 40 votes are cast (60 units available)
            And the resolution receives 25 votes (62.5% of 40)
            Then the resolution is Adopted

**Couche 4: E2E Tests (Playwright)**

- Location: ``frontend/tests/e2e/`` (à implémenter)
- Cibles: User flows (UI + backend)
- Framework: ``@playwright/test``
- Couverture: ~30% (critical paths)
- Vitesse: ~20-50s par test (browser automation)

Property-based Testing avec proptest-rs
========================================

**Concept**: Au lieu de tester des cas spécifiques, générer automatiquement
des centaines de cas d'entrée aléatoires et vérifier que l'invariant tient.

**Bénéfice pour KoproGo**:

- Trouver des bugs de coin (edge cases) que les tests manuels manquent
- Tester des combinaisons impossibles à écrire manuellement
- Validation des règles métier belges (quotes-parts, quorums, votes)

Ajout à ``Cargo.toml``:

.. code-block:: toml

    [dev-dependencies]
    proptest = "1.4"

**Exemple 1: UnitOwner quotes-parts validation**

.. code-block:: rust

    #[cfg(test)]
    mod property_tests {
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn prop_unit_owner_percentages_always_valid(percentages in prop::collection::vec(0.0..=1.0, 0..10)) {
                // Property: La somme de toutes les quotes-parts ne dépasse jamais 100%
                let total: f64 = percentages.iter().sum();

                if total > 1.0 {
                    // La base de données devrait rejeter
                    prop_assert!(total <= 1.0 + 0.01); // Tolérance ±0.01%
                }
            }
        }
    }

**Exemple 2: Meeting quorum calculation**

.. code-block:: rust

    proptest! {
        #[test]
        fn prop_meeting_quorum_always_achievable(
            total_voting_power in 1i32..=10000,
            present_voting_power in 0i32..=10000,
        ) {
            if present_voting_power > total_voting_power {
                // Property: quorum% ne peut pas être > 100%
                prop_assume!(present_voting_power <= total_voting_power);
            }

            let quorum_percent = (present_voting_power as f64 / total_voting_power as f64) * 100.0;
            prop_assert!(quorum_percent >= 0.0 && quorum_percent <= 100.0);
        }
    }

**Exemple 3: Payment amount validation**

.. code-block:: rust

    proptest! {
        #[test]
        fn prop_payment_refund_never_exceeds_original(
            original_cents in 100i64..=999999999,
            refund_cents in 0i64..=999999999,
        ) {
            // Property: Montant remboursé ne dépasse jamais l'original
            if refund_cents > original_cents {
                // Should fail validation
                prop_assert!(refund_cents <= original_cents);
            }
        }
    }

Contract Testing avec Pact
===========================

**Concept**: Vérifier que les contrats API (requête/réponse) sont respectés
entre services/versions sans tester l'intégration complète.

**Bénéfice pour KoproGo**:

- Tester que l'API v2 n'enfreint pas les clients existants
- Documenter les contrats API de manière exécutable
- Détecter les changements d'API incompatibles avant le déploiement

Ajout à ``Cargo.toml``:

.. code-block:: toml

    [dev-dependencies]
    pact = "1.0"

**Exemple 1: Building API contract**

.. code-block:: rust

    #[cfg(test)]
    mod contract_tests {
        use pact::consumer::prelude::*;

        #[tokio::test]
        async fn test_get_building_contract() {
            let mut pact_builder = PactBuilder::new("KoproGo Frontend", "KoproGo Backend")
                .interaction("a request for building details", |i| {
                    i.request
                        .get()
                        .path("/api/v1/buildings/uuid-123");
                    i.response
                        .status(200)
                        .header("Content-Type", "application/json")
                        .json_body(json!({
                            "id": "uuid-123",
                            "name": "Building A",
                            "address": "Rue Test 1",
                            "total_units": 50,
                            "construction_year": 2000,
                            "created_at": "2024-01-01T00:00:00Z",
                            "updated_at": "2024-01-01T00:00:00Z"
                        }));
                });

            // Verify consumer expectations against provider
            pact_builder.verify_with_provider(|| async {
                let client = reqwest::Client::new();
                let resp = client
                    .get("http://localhost:8080/api/v1/buildings/uuid-123")
                    .send()
                    .await
                    .unwrap();
                assert_eq!(resp.status(), 200);
            })
            .await;
        }
    }

**Exemple 2: Expense approval workflow contract**

.. code-block:: rust

    #[tokio::test]
    async fn test_expense_approval_contract() {
        let mut pact_builder = PactBuilder::new("Syndic App", "KoproGo Backend")
            .interaction("approve an expense", |i| {
                i.request
                    .post()
                    .path("/api/v1/expenses/uuid-456/approve")
                    .json_body(json!({
                        "approved_by": "accountant-123",
                        "approval_date": "2024-03-23T12:00:00Z"
                    }));
                i.response
                    .status(200)
                    .json_body(json!({
                        "id": "uuid-456",
                        "status": "Approved",
                        "amount_cents": 50000,
                        "approved_at": "2024-03-23T12:00:00Z"
                    }));
            });

        pact_builder.verify_with_provider(|| async {
            // Provider verification
        })
        .await;
    }

Load Testing avec k6 + Criterion
================================

**k6 (LoadImpact)**: Outils de load testing pour vérifier P99 < 5ms

Fichier d'exemple: ``load-tests/spike-test.js``

.. code-block:: javascript

    import http from 'k6/http';
    import { check, sleep } from 'k6';

    export let options = {
        stages: [
            { duration: '2m', target: 100 },   // Ramp-up à 100 users
            { duration: '5m', target: 100 },   // Stay at 100 users
            { duration: '2m', target: 200 },   // Spike à 200 users
            { duration: '5m', target: 200 },   // Hold the spike
            { duration: '2m', target: 0 },     // Ramp-down to 0 users
        ],
        thresholds: {
            http_req_duration: ['p(99)<5000'],    // P99 latency < 5 seconds
            http_req_failed: ['rate<0.1'],        // Error rate < 0.1%
        },
    };

    export default function () {
        // Test buildings list endpoint
        let res = http.get(`${__ENV.BASE_URL}/api/v1/buildings?page=1&per_page=50`);
        check(res, {
            'status is 200': (r) => r.status === 200,
            'response time < 100ms': (r) => r.timings.duration < 100,
        });

        // Test expense creation (write-heavy)
        res = http.post(`${__ENV.BASE_URL}/api/v1/expenses`, JSON.stringify({
            building_id: '550e8400-e29b-41d4-a716-446655440000',
            title: 'Test Expense',
            amount_cents: 50000,
            category: 'Maintenance',
        }), {
            headers: { 'Content-Type': 'application/json' },
        });
        check(res, {
            'expense created': (r) => r.status === 201,
        });

        sleep(1);
    }

**Criterion (Rust benchmarking)**:

.. code-block:: rust

    // file: backend/benches/domain_benchmark.rs
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use koprogo_domain::entities::Building;

    fn bench_building_creation(c: &mut Criterion) {
        c.bench_function("building_creation", |b| {
            b.iter(|| {
                Building::new(
                    black_box("Test Building"),
                    black_box("Rue Test 1"),
                    black_box(50),
                    black_box(2000),
                )
            });
        });
    }

    criterion_group!(benches, bench_building_creation);
    criterion_main!(benches);

Exécuter:

.. code-block:: bash

    cargo bench --bench domain_benchmark

Mutation Testing avec cargo-mutants
====================================

**Concept**: Modifier intentionnellement le code source (injecter des bugs)
et vérifier que les tests détectent les mutations. Cela teste « la qualité
des tests ».

Ajout à ``Cargo.toml``:

.. code-block:: toml

    [dev-dependencies]
    cargo-mutants = "0.1"

**Exemple**: Vérifier que le test de validation des quotes-parts est bon

Code original:

.. code-block:: rust

    impl UnitOwner {
        pub fn validate_total_percentage(owners: &[UnitOwner]) -> Result<(), String> {
            let total: f64 = owners.iter().map(|o| o.percentage).sum();
            if total > 1.0 + 0.0001 {  // Tolérance ±0.01%
                return Err("Total percentage exceeds 100%".to_string());
            }
            Ok(())
        }
    }

Mutations que cargo-mutants testent:

1. ``if total > 1.0`` → ``if total >= 1.0``  (le test doit échouer)
2. ``+ 0.0001`` → ``+ 0.0`` (le test doit échouer)
3. ``> 1.0`` → ``< 1.0`` (le test doit échouer)

Exécuter:

.. code-block:: bash

    cargo mutants --test unit_owner_tests

Rapport généré:

.. code-block:: text

    Total mutations: 87
    Killed (détectées): 85
    Survived (manquées): 2
    Timeout: 0
    Coverage: 97.7%

**Les 2 mutations survivantes** = vous devez ajouter des tests pour les couvrir.

Snapshot Testing pour les API responses
===============================================

**Concept**: Enregistrer la réponse API attendue une fois, puis vérifier que
toutes les futures réponses correspondent (détecte les changements inattendus).

Dépendance:

.. code-block:: toml

    [dev-dependencies]
    insta = "1.34"

**Exemple 1: Building API response snapshot**

.. code-block:: rust

    #[tokio::test]
    async fn test_building_list_response_shape() {
        let client = TestClient::new().await;
        let response = client.get("/api/v1/buildings?page=1&per_page=10").await;

        insta::assert_json_snapshot!(response.json(), @r#"
        {
            "data": [
                {
                    "id": "550e8400-e29b-41d4-a716-446655440000",
                    "name": "Building A",
                    "address": "Rue Test 1",
                    "total_units": 50,
                    "construction_year": 2000,
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ],
            "pagination": {
                "page": 1,
                "per_page": 10,
                "total": 1
            }
        }
        "#);
    }

**Exemple 2: Expense approval response snapshot**

.. code-block:: rust

    #[tokio::test]
    async fn test_expense_approval_response_shape() {
        let mut client = TestClient::new().await;
        let expense_id = client.create_expense().await.id;

        let response = client
            .post(&format!("/api/v1/expenses/{}/approve", expense_id))
            .json(&json!({"approved_by": "accountant-123"}))
            .await;

        insta::assert_json_snapshot!(response.json());
    }

Exécuter les snapshot tests:

.. code-block:: bash

    # Créer les snapshots initiaux
    cargo test --test api_snapshots -- --nocapture

    # Vérifier que les réponses correspondent
    cargo insta test --check

    # Mettre à jour les snapshots après une modification volontaire
    cargo insta test --review

Recommandations d'implémentation
=================================

**Phase 1 (Court terme - Jalon 1)**

1. **Property-based testing** (1-2 semaines)
   - Ajouter ``proptest`` à ``Cargo.toml``
   - Tester 5 domaines critiques (UnitOwner %, Meetings quorum, Payments, Votes, Expenses)
   - Objectif: Trouver 3-5 bugs de coin

2. **Snapshot testing** (1 semaine)
   - Ajouter ``insta`` à ``Cargo.toml``
   - Créer snapshots pour 10 endpoints critiques (buildings, expenses, meetings, payments)
   - Protéger contre les changements de schéma API involontaires

**Phase 2 (Moyen terme - Jalon 2)**

3. **Contract testing** (2 semaines)
   - Ajouter ``pact`` à ``Cargo.toml``
   - Tester contrats entre Frontend (Astro) et Backend
   - Tester contrats entre API v2 et clients tiers (notaires, PropTech)

4. **Load testing** (1 semaine)
   - Mettre à jour ``load-tests/*.js``
   - Exécuter tests avant chaque release (P99 < 5ms)
   - Benchmarker les endpoints critiques avec Criterion

**Phase 3 (Long terme - Jalon 3+)**

5. **Mutation testing** (1 semaine)
   - Ajouter ``cargo-mutants`` à ``Cargo.toml``
   - Mesurer la qualité des tests (mutation score >= 95%)
   - Refactor les tests faibles

6. **E2E testing** (2 semaines)
   - Implémenter ``@playwright/test`` pour 10 user journeys critiques
   - Tester sur Firefox, Chrome, Safari
   - Automatiser dans CI/CD

**Métriques de succès**

.. code-block:: text

    Couverture de code          : >= 95% (domain), >= 80% (app), >= 70% (infra)
    Property-based test cases   : >= 500 aléatoires / scenario
    Mutation score              : >= 95% (tests tuent les mutations)
    P99 latency                 : < 5ms (tous les endpoints)
    Load test capacity          : >= 1,000 req/s simultanés
    Snapshot test coverage      : >= 80% des endpoints
    Contract test coverage      : 100% des APIs publiques

**Outils recommandés**

.. code-block:: text

    Test framework       | Rust           | Frontend
    ─────────────────────────────────────────────────
    Unit testing        | cargo test     | Vitest
    Integration testing | testcontainers | Playwright
    BDD/Gherkin         | cucumber       | (N/A)
    Property-based      | proptest       | fast-check
    Snapshot testing    | insta          | Vitest snapshots
    Contract testing    | pact           | pact-js
    Load testing        | criterion, k6  | k6
    Mutation testing    | cargo-mutants  | stryker
    Coverage reporting  | tarpaulin      | c8

Références
==========

- `proptest documentation <https://docs.rs/proptest/>`_
- `Pact testing <https://pact.foundation/>`_
- `k6 documentation <https://k6.io/docs/>`_
- `Criterion.rs <https://docs.rs/criterion/>`_
- `cargo-mutants <https://github.com/sourcefrog/cargo-mutants>`_
- `Insta snapshots <https://insta.rs/>`_
- `Belgian property law <https://www.juridique.be/>`_

Conclusion
==========

KoproGo bénéficiera grandement de ces stratégies de test avancées pour :

- **Robustesse**: Trouver les bugs avant qu'ils ne touchent les utilisateurs
- **Conformité**: Valider les règles métier complexes du droit belge
- **Performance**: Garantir P99 < 5ms même sous charge
- **Confidence**: Les tests deviennent de la documentation exécutable

Implémentation recommandée: **Property-based + Snapshot testing** en Jalon 1,
suivi de **Contract + Load + Mutation testing** en Jalon 2.
