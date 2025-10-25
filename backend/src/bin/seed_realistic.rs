use chrono::Utc;
use rand::Rng;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://koprogo:koprogo123@localhost:5432/koprogo_db".to_string()
    });

    println!("🌱 Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    println!("🧹 Clearing existing demo data...");
    clear_demo_data(&pool).await?;

    println!("📊 Generating realistic seed data for 1 vCPU / 2GB RAM server...");
    println!("Target: 3 orgs, ~20 buildings, ~150 units, ~100 owners, ~50 expenses\n");

    let mut rng = rand::rng();

    // Belgian cities for variety
    let cities = [
        "Bruxelles",
        "Anvers",
        "Gand",
        "Charleroi",
        "Liège",
        "Bruges",
        "Namur",
        "Louvain",
    ];
    let street_types = ["Rue", "Avenue", "Boulevard", "Place", "Chaussée"];
    let street_names = [
        "des Fleurs",
        "du Parc",
        "de la Gare",
        "Royale",
        "de l'Église",
        "du Commerce",
        "de la Liberté",
        "des Arts",
        "Victor Hugo",
        "Louise",
    ];

    // Create 3 organizations with different sizes
    let org_configs = [
        ("Petite Copropriété SPRL", "small", 5, 30), // 5 buildings, ~30 units
        ("Copropriété Moyenne SA", "medium", 8, 60), // 8 buildings, ~60 units
        ("Grande Résidence NV", "large", 10, 100),   // 10 buildings, ~100 units
    ];

    for (idx, (org_name, size, num_buildings, target_units)) in org_configs.iter().enumerate() {
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        println!(
            "📍 Organization {}: {} ({} buildings, ~{} units)",
            idx + 1,
            org_name,
            num_buildings,
            target_units
        );

        // Create organization
        sqlx::query(
            "INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(org_id)
        .bind(*org_name)
        .bind(format!("{}-{}", size, idx))
        .bind(format!("contact@{}.be", size))
        .bind(format!("+32 2 {} {} {}", rng.random_range(100..999), rng.random_range(10..99), rng.random_range(10..99)))
        .bind(if *size == "large" { "enterprise" } else if *size == "medium" { "professional" } else { "basic" })
        .bind(*num_buildings)
        .bind(if *size == "large" { 50 } else if *size == "medium" { 20 } else { 10 })
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await?;

        // Create admin user for this org
        let user_id = Uuid::new_v4();
        let password_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST)?;

        sqlx::query(
            "INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(user_id)
        .bind(format!("admin@{}.be", size))
        .bind(&password_hash)
        .bind("Admin")
        .bind(org_name.split_whitespace().next().unwrap_or("User"))
        .bind("syndic")
        .bind(Some(org_id))
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await?;

        // Create owners pool for this org
        let num_owners = (target_units * 2 / 3) as usize; // ~66% occupancy
        let mut owner_ids = Vec::new();

        for o in 0..num_owners {
            let owner_id = Uuid::new_v4();
            let first_names = [
                "Pierre",
                "Marie",
                "Jean",
                "Sophie",
                "Luc",
                "Anne",
                "François",
                "Julie",
                "Thomas",
                "Emma",
            ];
            let last_names = [
                "Dupont", "Martin", "Bernard", "Dubois", "Laurent", "Simon", "Michel", "Lefebvre",
                "Moreau", "Garcia",
            ];

            sqlx::query(
                "INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(owner_id)
            .bind(org_id)
            .bind(first_names[rng.random_range(0..first_names.len())])
            .bind(last_names[rng.random_range(0..last_names.len())])
            .bind(format!("owner{}@{}.be", o + 1, size))
            .bind(format!("+32 {} {} {} {}",
                if rng.random_bool(0.5) { "2" } else { "4" },
                rng.random_range(100..999),
                rng.random_range(10..99),
                rng.random_range(10..99)
            ))
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await?;

            owner_ids.push(owner_id);
        }

        // Create buildings for this org
        let units_per_building = target_units / num_buildings;
        let mut total_units = 0;

        for b in 0..*num_buildings {
            let building_id = Uuid::new_v4();
            let city = cities[rng.random_range(0..cities.len())];
            let street_type = street_types[rng.random_range(0..street_types.len())];
            let street_name = street_names[rng.random_range(0..street_names.len())];
            let building_name = format!("Résidence {}", street_name);

            sqlx::query(
                "INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
            )
            .bind(building_id)
            .bind(org_id)
            .bind(&building_name)
            .bind(format!("{} {} {}", street_type, street_name, rng.random_range(1..200)))
            .bind(city)
            .bind(format!("{}", rng.random_range(1000..9999)))
            .bind("Belgium")
            .bind(units_per_building)
            .bind(rng.random_range(1960..2024))
            .bind(now)
            .bind(now)
            .execute(&pool)
            .await?;

            // Create units for this building
            let units_this_building = if b == num_buildings - 1 {
                // Last building gets remainder
                target_units - total_units
            } else {
                units_per_building
            };

            for u in 0..units_this_building {
                let floor = u / 4; // 4 units per floor
                let unit_number = format!("{}.{}", floor, (u % 4) + 1);

                // 66% chance to have an owner
                let owner_id = if rng.random_bool(0.66) && !owner_ids.is_empty() {
                    Some(owner_ids[rng.random_range(0..owner_ids.len())])
                } else {
                    None
                };

                // Valid unit_type ENUM values: apartment, parking, cellar, commercial, other
                let unit_types = ["apartment", "apartment", "apartment", "parking", "cellar"];
                let unit_type = unit_types[rng.random_range(0..unit_types.len())];

                sqlx::query(
                    "INSERT INTO units (id, organization_id, building_id, unit_number, unit_type, floor, surface_area, quota, owner_id, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5::unit_type, $6, $7, $8, $9, $10, $11)"
                )
                .bind(Uuid::new_v4())
                .bind(org_id)
                .bind(building_id)
                .bind(&unit_number)
                .bind(unit_type)
                .bind(floor)
                .bind(rng.random_range(45.0..150.0))
                .bind(rng.random_range(50..200) as i32)
                .bind(owner_id)
                .bind(now)
                .bind(now)
                .execute(&pool)
                .await?;
            }

            total_units += units_this_building;

            // Create 2-3 expenses per building
            let num_expenses = rng.random_range(2..=3);
            let expense_types = [
                ("Entretien ascenseur", 450.0, 800.0),
                ("Nettoyage parties communes", 300.0, 600.0),
                ("Chauffage collectif", 1500.0, 3000.0),
                ("Assurance immeuble", 800.0, 1500.0),
                ("Travaux façade", 5000.0, 15000.0),
            ];

            for _ in 0..num_expenses {
                let (desc, min_amount, max_amount) =
                    expense_types[rng.random_range(0..expense_types.len())];
                let amount = rng.random_range(min_amount..max_amount);
                let days_ago = rng.random_range(0..90);
                let expense_date = Utc::now() - chrono::Duration::days(days_ago);

                // Valid expense_category ENUM: maintenance, repairs, insurance, utilities, cleaning, administration, works, other
                let categories = [
                    "maintenance",
                    "repairs",
                    "insurance",
                    "utilities",
                    "cleaning",
                    "administration",
                    "works",
                ];
                let category = categories[rng.random_range(0..categories.len())];

                // Valid payment_status ENUM: pending, paid, overdue, cancelled
                let payment_status = if rng.random_bool(0.7) {
                    "paid"
                } else {
                    "pending"
                };

                sqlx::query(
                    "INSERT INTO expenses (id, organization_id, building_id, category, description, amount, expense_date, payment_status, created_at, updated_at)
                     VALUES ($1, $2, $3, $4::expense_category, $5, $6, $7, $8::payment_status, $9, $10)"
                )
                .bind(Uuid::new_v4())
                .bind(org_id)
                .bind(building_id)
                .bind(category)
                .bind(desc)
                .bind(amount)
                .bind(expense_date)
                .bind(payment_status)
                .bind(now)
                .bind(now)
                .execute(&pool)
                .await?;
            }
        }

        println!(
            "  ✅ Created {} buildings, {} units, {} owners",
            num_buildings, total_units, num_owners
        );
    }

    println!("\n✅ Realistic seed data created successfully!");
    println!("\nTest credentials:");
    println!("  Small org:  admin@small.be / admin123");
    println!("  Medium org: admin@medium.be / admin123");
    println!("  Large org:  admin@large.be / admin123");

    Ok(())
}

async fn clear_demo_data(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DELETE FROM documents").execute(pool).await?;
    sqlx::query("DELETE FROM meetings").execute(pool).await?;
    sqlx::query("DELETE FROM expenses").execute(pool).await?;
    sqlx::query("DELETE FROM units").execute(pool).await?;
    sqlx::query("DELETE FROM owners").execute(pool).await?;
    sqlx::query("DELETE FROM buildings").execute(pool).await?;
    sqlx::query("DELETE FROM users WHERE role != 'superadmin'")
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM organizations")
        .execute(pool)
        .await?;

    Ok(())
}
