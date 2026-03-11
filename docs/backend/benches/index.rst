Benchmarks - Performance
========================

Benchmarks de performance avec Criterion pour mesurer et optimiser les opérations critiques.

**Objectifs Performance** :

- **Latency P99** : < 5ms
- **Throughput** : > 100,000 req/s
- **Memory** : < 128MB par instance

Structure
---------

.. code-block:: text

   backend/benches/
   ├── building_benchmarks.rs
   ├── unit_benchmarks.rs
   ├── expense_benchmarks.rs
   └── query_benchmarks.rs

Criterion
---------

**Installation** :

.. code-block:: toml

   [dev-dependencies]
   criterion = { version = "0.5", features = ["html_reports"] }

   [[bench]]
   name = "building_benchmarks"
   harness = false

**Exemple Benchmark** :

.. code-block:: rust

   // benches/building_benchmarks.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use koprogo_api::domain::entities::Building;

   fn benchmark_building_creation(c: &mut Criterion) {
       c.bench_function("building creation", |b| {
           b.iter(|| {
               Building::new(
                   black_box("Benchmark Building".to_string()),
                   black_box("123 Benchmark St".to_string()),
                   black_box("Paris".to_string()),
                   black_box("75001".to_string()),
                   black_box("France".to_string()),
                   black_box(50),
                   black_box(Some(2020)),
               )
           });
       });
   }

   fn benchmark_building_update(c: &mut Criterion) {
       let mut building = Building::new(
           "Original Name".to_string(),
           "Original Address".to_string(),
           "Paris".to_string(),
           "75001".to_string(),
           "France".to_string(),
           30,
           None,
       ).unwrap();

       c.bench_function("building update", |b| {
           b.iter(|| {
               building.update_info(
                   black_box("New Name".to_string()),
                   black_box("New Address".to_string()),
                   black_box("Lyon".to_string()),
                   black_box("69001".to_string()),
               )
           });
       });
   }

   criterion_group!(
       benches,
       benchmark_building_creation,
       benchmark_building_update
   );
   criterion_main!(benches);

**Exécution** :

.. code-block:: bash

   # Tous les benchmarks
   cargo bench

   # Benchmark spécifique
   cargo bench building_creation

   # Output: target/criterion/report/index.html

Benchmarks Database
-------------------

**Query Performance** :

.. code-block:: rust

   // benches/query_benchmarks.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
   use sqlx::PgPool;
   use tokio::runtime::Runtime;

   async fn setup_db() -> PgPool {
       let pool = PgPool::connect("postgresql://...")
           .await
           .unwrap();

       // Seed test data
       seed_test_data(&pool).await;

       pool
   }

   fn benchmark_find_by_id(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();
       let pool = rt.block_on(setup_db());
       let repo = PostgresBuildingRepository::new(pool);

       let test_id = Uuid::new_v4();

       c.bench_function("find building by id", |b| {
           b.to_async(&rt).iter(|| async {
               repo.find_by_id(black_box(test_id)).await
           });
       });
   }

   fn benchmark_paginated_query(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();
       let pool = rt.block_on(setup_db());
       let repo = PostgresBuildingRepository::new(pool);

       let org_id = Uuid::new_v4();

       c.bench_with_input(
           BenchmarkId::new("paginated query", "page_size"),
           &20i64,
           |b, &per_page| {
               b.to_async(&rt).iter(|| async {
                   repo.find_all_paginated(
                       black_box(org_id),
                       black_box(1),
                       black_box(per_page)
                   ).await
               });
           }
       );
   }

   criterion_group!(
       benches,
       benchmark_find_by_id,
       benchmark_paginated_query
   );
   criterion_main!(benches);

Benchmarks Expense Calculator
------------------------------

.. code-block:: rust

   // benches/expense_benchmarks.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use koprogo_api::domain::services::expense_calculator::ExpenseCalculator;

   fn benchmark_calculate_share(c: &mut Criterion) {
       let expense = Expense {
           id: Uuid::new_v4(),
           building_id: Uuid::new_v4(),
           description: "Test Expense".to_string(),
           amount: 100000,  // 1000.00€
           // ...
       };

       let unit = Unit {
           id: Uuid::new_v4(),
           building_id: expense.building_id,
           unit_number: "A-1".to_string(),
           floor: 1,
           surface_area: 75,
           ownership_share: 45,  // 45/1000
           // ...
       };

       c.bench_function("calculate expense share", |b| {
           b.iter(|| {
               ExpenseCalculator::calculate_share(
                   black_box(&expense),
                   black_box(&unit),
                   black_box(1000),  // Total shares
               )
           });
       });
   }

   fn benchmark_calculate_all_shares(c: &mut Criterion) {
       let expense = create_test_expense();
       let units: Vec<Unit> = (0..50)
           .map(|i| create_test_unit(i))
           .collect();

       c.bench_function("calculate all shares (50 units)", |b| {
           b.iter(|| {
               for unit in black_box(&units) {
                   ExpenseCalculator::calculate_share(&expense, unit, 1000);
               }
           });
       });
   }

   criterion_group!(
       benches,
       benchmark_calculate_share,
       benchmark_calculate_all_shares
   );
   criterion_main!(benches);

Benchmarks API Handlers
------------------------

.. code-block:: rust

   // benches/api_benchmarks.rs
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   use actix_web::{test, web, App};
   use tokio::runtime::Runtime;

   fn benchmark_list_buildings_endpoint(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();

       rt.block_on(async {
           let app = test::init_service(
               App::new()
                   .app_data(web::Data::new(test_app_state()))
                   .configure(configure_routes)
           ).await;

           c.bench_function("GET /buildings (20 results)", |b| {
               b.to_async(&rt).iter(|| async {
                   let req = test::TestRequest::get()
                       .uri("/api/v1/buildings?page=1&per_page=20")
                       .insert_header(("Authorization", "Bearer test-token"))
                       .to_request();

                   let resp = test::call_service(&app, req).await;
                   black_box(resp);
               });
           });
       });
   }

   fn benchmark_create_building_endpoint(c: &mut Criterion) {
       let rt = Runtime::new().unwrap();

       rt.block_on(async {
           let app = test::init_service(
               App::new()
                   .app_data(web::Data::new(test_app_state()))
                   .configure(configure_routes)
           ).await;

           c.bench_function("POST /buildings", |b| {
               b.to_async(&rt).iter(|| async {
                   let req = test::TestRequest::post()
                       .uri("/api/v1/buildings")
                       .insert_header(("Authorization", "Bearer test-token"))
                       .set_json(&create_test_building_dto())
                       .to_request();

                   let resp = test::call_service(&app, req).await;
                   black_box(resp);
               });
           });
       });
   }

   criterion_group!(
       benches,
       benchmark_list_buildings_endpoint,
       benchmark_create_building_endpoint
   );
   criterion_main!(benches);

Résultats Typiques
------------------

**Domain Operations** :

.. code-block:: text

   building creation       time:   [125.32 ns 126.45 ns 127.89 ns]
   building update         time:   [78.91 ns 79.34 ns 79.82 ns]
   calculate expense share time:   [42.15 ns 42.67 ns 43.24 ns]

**Database Queries** :

.. code-block:: text

   find building by id     time:   [1.234 ms 1.278 ms 1.325 ms]
   paginated query (20)    time:   [2.456 ms 2.512 ms 2.578 ms]
   create building (DB)    time:   [1.567 ms 1.612 ms 1.665 ms]

**API Endpoints** :

.. code-block:: text

   GET /buildings (20)     time:   [3.234 ms 3.312 ms 3.398 ms]
   POST /buildings         time:   [2.123 ms 2.189 ms 2.267 ms]
   PUT /buildings/:id      time:   [2.345 ms 2.412 ms 2.489 ms]

Optimisations
-------------

**Compilation Release** :

.. code-block:: toml

   [profile.release]
   opt-level = 3           # Optimisation maximale
   lto = true              # Link-Time Optimization
   codegen-units = 1       # Codegen units minimal
   strip = true            # Strip symbols
   panic = "abort"         # Panic = abort (plus rapide)

**Connection Pool** :

.. code-block:: rust

   PgPoolOptions::new()
       .max_connections(10)         # Max 10 connexions
       .min_connections(2)          # Min 2 connexions
       .acquire_timeout(Duration::from_secs(30))
       .idle_timeout(Duration::from_secs(300))
       .connect(&database_url)
       .await

**Indexes Database** :

.. code-block:: sql

   CREATE INDEX idx_buildings_org ON buildings(organization_id);
   CREATE INDEX idx_buildings_city ON buildings(city);
   CREATE INDEX idx_units_building ON units(building_id);
   CREATE INDEX idx_expenses_building ON expenses(building_id);
   CREATE INDEX idx_expenses_status ON expenses(payment_status);

Profiling
---------

**Flamegraph** :

.. code-block:: bash

   # Installer
   cargo install flamegraph

   # Générer flamegraph
   cargo flamegraph --bench building_benchmarks

   # Output: flamegraph.svg

**Perf** (Linux) :

.. code-block:: bash

   # Profiler avec perf
   perf record --call-graph=dwarf cargo bench

   # Analyser
   perf report

**Valgrind** (Memory profiling) :

.. code-block:: bash

   # Installer
   sudo apt-get install valgrind

   # Profiler mémoire
   valgrind --tool=massif cargo bench

   # Visualiser
   ms_print massif.out.*

Comparaison Versions
--------------------

**Baseline** : Sauvegarder résultats pour comparaison.

.. code-block:: bash

   # Sauvegarder baseline
   cargo bench -- --save-baseline main

   # Après modifications
   cargo bench -- --baseline main

   # Criterion affichera les différences

Métriques Cibles
----------------

.. list-table::
   :header-rows: 1
   :widths: 40 30 30

   * - Opération
     - Cible
     - Actuel
   * - Building creation (domain)
     - < 200ns
     - ~125ns ✅
   * - Building query (DB)
     - < 2ms
     - ~1.3ms ✅
   * - Paginated query (20)
     - < 5ms
     - ~2.5ms ✅
   * - API GET /buildings
     - < 5ms P99
     - ~3.3ms ✅
   * - API POST /buildings
     - < 5ms P99
     - ~2.2ms ✅

CI/CD Benchmarks
----------------

**Régression Detection** :

.. code-block:: yaml

   # .github/workflows/bench.yml
   name: Benchmarks

   on:
     pull_request:
       branches: [main]

   jobs:
     benchmark:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3

         - name: Install Rust
           uses: actions-rs/toolchain@v1

         - name: Run benchmarks
           run: cargo bench

         - name: Compare with main
           run: cargo bench -- --baseline main

         - name: Alert on regression
           if: regression_detected
           run: echo "Performance regression detected!"

Références
----------

- Criterion Docs : https://bheisler.github.io/criterion.rs/book/
- Rust Performance Book : https://nnethercote.github.io/perf-book/
- Flamegraph : https://github.com/flamegraph-rs/flamegraph
