use chrono::Utc;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use koprogo_api::domain::entities::{Building, Expense, ExpenseCategory, Owner, Unit, UnitType};
use koprogo_api::domain::services::ExpenseCalculator;
use std::hint::black_box;
use uuid::Uuid;

fn benchmark_building_creation(c: &mut Criterion) {
    let org_id = Uuid::new_v4();
    c.bench_function("create_building", |b| {
        b.iter(|| {
            Building::new(
                black_box(org_id),
                black_box("Test Building".to_string()),
                black_box("123 Test St".to_string()),
                black_box("Paris".to_string()),
                black_box("75001".to_string()),
                black_box("France".to_string()),
                black_box(10),
                black_box(1000),
                black_box(Some(2000)),
            )
        });
    });
}

fn benchmark_unit_creation(c: &mut Criterion) {
    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();

    c.bench_function("create_unit", |b| {
        b.iter(|| {
            Unit::new(
                black_box(org_id),
                black_box(building_id),
                black_box("A101".to_string()),
                black_box(UnitType::Apartment),
                black_box(Some(1)),
                black_box(75.5),
                black_box(50.0),
            )
        });
    });
}

fn benchmark_owner_creation(c: &mut Criterion) {
    c.bench_function("create_owner", |b| {
        let org_id = Uuid::new_v4();
        b.iter(|| {
            Owner::new(
                black_box(org_id),
                black_box("Jean".to_string()),
                black_box("Dupont".to_string()),
                black_box("jean.dupont@example.com".to_string()),
                black_box(Some("+33612345678".to_string())),
                black_box("123 Rue de la Paix".to_string()),
                black_box("Paris".to_string()),
                black_box("75001".to_string()),
                black_box("France".to_string()),
            )
        });
    });
}

fn benchmark_expense_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("expense_calculation");

    for num_expenses in [10, 100, 1000].iter() {
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();
        let expenses: Vec<Expense> = (0..*num_expenses)
            .map(|i| {
                Expense::new(
                    org_id,
                    building_id,
                    ExpenseCategory::Maintenance,
                    format!("Expense {}", i),
                    100.0,
                    Utc::now(),
                    None,
                    None,
                )
                .unwrap()
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("total_expenses", num_expenses),
            &expenses,
            |b, expenses| {
                b.iter(|| ExpenseCalculator::calculate_total_expenses(black_box(expenses)));
            },
        );
    }

    group.finish();
}

fn benchmark_unit_share_calculation(c: &mut Criterion) {
    let org_id = Uuid::new_v4();
    let building_id = Uuid::new_v4();
    let expense = Expense::new(
        org_id,
        building_id,
        ExpenseCategory::Maintenance,
        "Test".to_string(),
        1000.0,
        Utc::now(),
        None,
        None,
    )
    .unwrap();

    let unit = Unit::new(
        org_id,
        building_id,
        "A101".to_string(),
        UnitType::Apartment,
        Some(1),
        75.0,
        50.0,
    )
    .unwrap();

    c.bench_function("calculate_unit_share", |b| {
        b.iter(|| ExpenseCalculator::calculate_unit_share(black_box(&expense), black_box(&unit)));
    });
}

criterion_group!(
    benches,
    benchmark_building_creation,
    benchmark_unit_creation,
    benchmark_owner_creation,
    benchmark_expense_calculation,
    benchmark_unit_share_calculation
);
criterion_main!(benches);
