use crate::domain::entities::{User, UserRole};
use bcrypt::{hash, DEFAULT_COST};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DatabaseSeeder {
    pool: PgPool,
}

impl DatabaseSeeder {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create the default superadmin user if it doesn't exist
    pub async fn seed_superadmin(&self) -> Result<User, String> {
        let superadmin_email = "admin@koprogo.com";
        let superadmin_password = "admin123"; // Change in production!

        // Check if superadmin already exists
        let existing = sqlx::query!("SELECT id FROM users WHERE email = $1", superadmin_email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to check existing superadmin: {}", e))?;

        if existing.is_some() {
            log::info!("Superadmin already exists");
            return Err("Superadmin already exists".to_string());
        }

        // Create superadmin
        let password_hash = hash(superadmin_password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let superadmin_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .map_err(|e| format!("Failed to parse UUID: {}", e))?;

        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            superadmin_id,
            superadmin_email,
            password_hash,
            "Super",
            "Admin",
            "superadmin",
            None::<Uuid>,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create superadmin: {}", e))?;

        log::info!("✅ Superadmin created: {}", superadmin_email);

        Ok(User {
            id: superadmin_id,
            email: superadmin_email.to_string(),
            password_hash,
            first_name: "Super".to_string(),
            last_name: "Admin".to_string(),
            role: UserRole::SuperAdmin,
            organization_id: None,
            is_active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Seed demo data for production demonstration
    pub async fn seed_demo_data(&self) -> Result<String, String> {
        log::info!("🌱 Starting demo data seeding...");

        // Check if demo data already exists
        let existing_orgs = sqlx::query!("SELECT COUNT(*) as count FROM organizations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count organizations: {}", e))?;

        if existing_orgs.count.unwrap_or(0) > 0 {
            return Err("Demo data already exists. Please clean the database first.".to_string());
        }

        // Create demo organization
        let org_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            org_id,
            "Copropriété Démo SAS",
            "copro-demo",
            "contact@copro-demo.fr",
            "+33 1 23 45 67 89",
            "professional",
            20,
            50,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create demo organization: {}", e))?;

        log::info!("✅ Organization created: Copropriété Démo SAS");

        // Create demo users
        let _syndic_id = self
            .create_demo_user(
                "syndic@copro-demo.fr",
                "syndic123",
                "Jean",
                "Dupont",
                "syndic",
                Some(org_id),
            )
            .await?;

        let _accountant_id = self
            .create_demo_user(
                "comptable@copro-demo.fr",
                "comptable123",
                "Marie",
                "Martin",
                "accountant",
                Some(org_id),
            )
            .await?;

        let _owner1_id = self
            .create_demo_user(
                "proprietaire1@copro-demo.fr",
                "owner123",
                "Pierre",
                "Durand",
                "owner",
                Some(org_id),
            )
            .await?;

        let _owner2_id = self
            .create_demo_user(
                "proprietaire2@copro-demo.fr",
                "owner123",
                "Sophie",
                "Bernard",
                "owner",
                Some(org_id),
            )
            .await?;

        log::info!("✅ Demo users created");

        // Create demo buildings
        let building1_id = self
            .create_demo_building(
                org_id,
                "Résidence Les Champs",
                "12 Avenue des Champs-Élysées",
                "Paris",
                "75008",
                "France",
                15,
                1995,
            )
            .await?;

        let building2_id = self
            .create_demo_building(
                org_id,
                "Le Jardin Fleuri",
                "45 Rue du Jardin",
                "Lyon",
                "69003",
                "France",
                8,
                2010,
            )
            .await?;

        log::info!("✅ Demo buildings created");

        // Create demo owners
        let owner1_db_id = self
            .create_demo_owner(
                "Pierre",
                "Durand",
                "pierre.durand@email.fr",
                "+33 6 12 34 56 78",
                "15 rue Victor Hugo",
                "Paris",
                "75015",
                "France",
            )
            .await?;

        let owner2_db_id = self
            .create_demo_owner(
                "Sophie",
                "Bernard",
                "sophie.bernard@email.fr",
                "+33 6 98 76 54 32",
                "28 avenue des Champs",
                "Paris",
                "75008",
                "France",
            )
            .await?;

        let owner3_db_id = self
            .create_demo_owner(
                "Michel",
                "Lefebvre",
                "michel.lefebvre@email.fr",
                "+33 6 11 22 33 44",
                "42 boulevard Saint-Germain",
                "Lyon",
                "69003",
                "France",
            )
            .await?;

        log::info!("✅ Demo owners created");

        // Create demo units
        let _unit1_id = self
            .create_demo_unit(
                building1_id,
                Some(owner1_db_id),
                "101",
                "apartment",
                Some(1),
                75.5,
                250.0,
            )
            .await?;

        let _unit2_id = self
            .create_demo_unit(
                building1_id,
                Some(owner2_db_id),
                "102",
                "apartment",
                Some(1),
                62.0,
                200.0,
            )
            .await?;

        let _unit3_id = self
            .create_demo_unit(building1_id, None, "103", "apartment", Some(1), 85.0, 300.0)
            .await?;

        let _unit4_id = self
            .create_demo_unit(
                building2_id,
                Some(owner3_db_id),
                "201",
                "apartment",
                Some(2),
                95.0,
                350.0,
            )
            .await?;

        log::info!("✅ Demo units created");

        // Create demo expenses
        self.create_demo_expense(
            building1_id,
            "Charges de copropriété Q1 2025 - Charges trimestrielles incluant eau, chauffage, entretien",
            5000.0,
            "2025-01-15",
            "administration",
            "pending",
            Some("Syndic Services"),
            Some("INV-2025-001"),
        )
        .await?;

        self.create_demo_expense(
            building1_id,
            "Réparation ascenseur - Maintenance et réparation de l'ascenseur principal",
            2500.0,
            "2025-02-10",
            "maintenance",
            "paid",
            Some("Ascenseurs Plus"),
            Some("ASC-2025-023"),
        )
        .await?;

        self.create_demo_expense(
            building2_id,
            "Charges de copropriété Q1 2025 - Charges trimestrielles",
            3000.0,
            "2025-01-15",
            "administration",
            "pending",
            Some("Syndic Services"),
            Some("INV-2025-002"),
        )
        .await?;

        self.create_demo_expense(
            building2_id,
            "Nettoyage des parties communes - Contrat annuel de nettoyage",
            1200.0,
            "2025-01-01",
            "cleaning",
            "paid",
            Some("CleanPro"),
            Some("CLN-2025-156"),
        )
        .await?;

        log::info!("✅ Demo expenses created");

        Ok("✅ Demo data seeded successfully!\n\n\
            📊 Summary:\n\
            - 1 Organization: Copropriété Démo SAS\n\
            - 4 Users: 1 Syndic, 1 Accountant, 2 Owners\n\
            - 2 Buildings: Résidence Les Champs, Le Jardin Fleuri\n\
            - 3 Owners\n\
            - 4 Units\n\
            - 4 Expenses\n\n\
            🔐 Demo credentials:\n\
            - Syndic: syndic@copro-demo.fr / syndic123\n\
            - Comptable: comptable@copro-demo.fr / comptable123\n\
            - Propriétaire 1: proprietaire1@copro-demo.fr / owner123\n\
            - Propriétaire 2: proprietaire2@copro-demo.fr / owner123\n\
            - SuperAdmin: admin@koprogo.com / admin123"
            .to_string())
    }

    async fn create_demo_user(
        &self,
        email: &str,
        password: &str,
        first_name: &str,
        last_name: &str,
        role: &str,
        organization_id: Option<Uuid>,
    ) -> Result<Uuid, String> {
        let password_hash =
            hash(password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            user_id,
            email,
            password_hash,
            first_name,
            last_name,
            role,
            organization_id,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user {}: {}", email, e))?;

        Ok(user_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_building(
        &self,
        org_id: Uuid,
        name: &str,
        address: &str,
        city: &str,
        postal_code: &str,
        country: &str,
        total_units: i32,
        construction_year: i32,
    ) -> Result<Uuid, String> {
        let building_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO buildings (id, organization_id, name, address, city, postal_code, country, total_units, construction_year, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            building_id,
            org_id,
            name,
            address,
            city,
            postal_code,
            country,
            total_units,
            construction_year,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create building {}: {}", name, e))?;

        Ok(building_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_owner(
        &self,
        first_name: &str,
        last_name: &str,
        email: &str,
        phone: &str,
        address: &str,
        city: &str,
        postal_code: &str,
        country: &str,
    ) -> Result<Uuid, String> {
        let owner_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO owners (id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            owner_id,
            first_name,
            last_name,
            email,
            phone,
            address,
            city,
            postal_code,
            country,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create owner {} {}: {}", first_name, last_name, e))?;

        Ok(owner_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_unit(
        &self,
        building_id: Uuid,
        owner_id: Option<Uuid>,
        unit_number: &str,
        unit_type: &str,
        floor: Option<i32>,
        surface_area: f64,
        quota: f64,
    ) -> Result<Uuid, String> {
        let unit_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO units (id, building_id, owner_id, unit_number, unit_type, floor, surface_area, quota, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::unit_type, $6, $7, $8, $9, $10)
            "#
        )
        .bind(unit_id)
        .bind(building_id)
        .bind(owner_id)
        .bind(unit_number)
        .bind(unit_type)
        .bind(floor)
        .bind(surface_area)
        .bind(quota)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create unit {}: {}", unit_number, e))?;

        Ok(unit_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_expense(
        &self,
        building_id: Uuid,
        description: &str,
        amount: f64,
        expense_date: &str,
        category: &str,
        payment_status: &str,
        supplier: Option<&str>,
        invoice_number: Option<&str>,
    ) -> Result<Uuid, String> {
        let expense_id = Uuid::new_v4();
        let now = Utc::now();
        let expense_date_parsed =
            chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", expense_date))
                .map_err(|e| format!("Failed to parse date: {}", e))?
                .with_timezone(&Utc);

        sqlx::query(
            r#"
            INSERT INTO expenses (id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, created_at, updated_at)
            VALUES ($1, $2, $3::expense_category, $4, $5, $6, $7::payment_status, $8, $9, $10, $11)
            "#
        )
        .bind(expense_id)
        .bind(building_id)
        .bind(category)
        .bind(description)
        .bind(amount)
        .bind(expense_date_parsed)
        .bind(payment_status)
        .bind(supplier)
        .bind(invoice_number)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create expense: {}", e))?;

        Ok(expense_id)
    }

    /// Clear all data (DANGEROUS - use with caution!)
    pub async fn clear_demo_data(&self) -> Result<String, String> {
        log::warn!("⚠️  Clearing all demo data...");

        // Delete in correct order due to foreign key constraints
        sqlx::query!("DELETE FROM expenses")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete expenses: {}", e))?;

        sqlx::query!("DELETE FROM units")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete units: {}", e))?;

        sqlx::query!("DELETE FROM owners")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete owners: {}", e))?;

        sqlx::query!("DELETE FROM buildings")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete buildings: {}", e))?;

        sqlx::query!("DELETE FROM users WHERE role != 'superadmin'")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete users: {}", e))?;

        sqlx::query!("DELETE FROM organizations")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete organizations: {}", e))?;

        log::info!("✅ Demo data cleared (superadmin preserved)");

        Ok("✅ Demo data cleared successfully!".to_string())
    }
}
