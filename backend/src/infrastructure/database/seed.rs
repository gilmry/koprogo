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

    /// Create or update the default superadmin user
    pub async fn seed_superadmin(&self) -> Result<User, String> {
        let superadmin_email = "admin@koprogo.com";
        let superadmin_password = "admin123"; // Change in production!

        // Hash password
        let password_hash = hash(superadmin_password, DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let superadmin_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
            .map_err(|e| format!("Failed to parse UUID: {}", e))?;

        let now = Utc::now();

        // Upsert superadmin (insert or update if exists)
        sqlx::query!(
            r#"
            INSERT INTO users (id, email, password_hash, first_name, last_name, role, organization_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (email)
            DO UPDATE SET
                password_hash = EXCLUDED.password_hash,
                updated_at = EXCLUDED.updated_at,
                is_active = true
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
        .map_err(|e| format!("Failed to upsert superadmin: {}", e))?;

        log::info!("✅ Superadmin ready: {}", superadmin_email);

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

        // ORGANIZATION 1
        let org1_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            org1_id,
            "Résidence Grand Place SPRL",
            "residence-grand-place",
            "contact@grandplace.be",
            "+32 2 501 23 45",
            "professional",
            20,
            50,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create demo organization 1: {}", e))?;

        log::info!("✅ Organization 1 created: Résidence Grand Place SPRL");

        // Create demo users ORG 1
        let syndic1_id = self
            .create_demo_user(
                "syndic@grandplace.be",
                "syndic123",
                "Jean",
                "Dupont",
                "syndic",
                Some(org1_id),
            )
            .await?;

        let _accountant_id = self
            .create_demo_user(
                "comptable@grandplace.be",
                "comptable123",
                "Marie",
                "Martin",
                "accountant",
                Some(org1_id),
            )
            .await?;

        let _owner1_id = self
            .create_demo_user(
                "proprietaire1@grandplace.be",
                "owner123",
                "Pierre",
                "Durand",
                "owner",
                Some(org1_id),
            )
            .await?;

        let _owner2_id = self
            .create_demo_user(
                "proprietaire2@grandplace.be",
                "owner123",
                "Sophie",
                "Bernard",
                "owner",
                Some(org1_id),
            )
            .await?;

        log::info!("✅ Demo users created");

        // Create demo buildings ORG 1
        let building1_id = self
            .create_demo_building(
                org1_id,
                "Résidence Grand Place",
                "Grand Place 15",
                "Bruxelles",
                "1000",
                "Belgique",
                15,
                1995,
            )
            .await?;

        let building2_id = self
            .create_demo_building(
                org1_id,
                "Les Jardins d'Ixelles",
                "Rue du Trône 85",
                "Bruxelles",
                "1050",
                "Belgique",
                8,
                2010,
            )
            .await?;

        log::info!("✅ Demo buildings created");

        // Create demo owners
        let owner1_db_id = self
            .create_demo_owner(
                org1_id,
                "Pierre",
                "Durand",
                "pierre.durand@email.be",
                "+32 476 12 34 56",
                "Avenue Louise 15",
                "Bruxelles",
                "1050",
                "Belgique",
            )
            .await?;

        let owner2_db_id = self
            .create_demo_owner(
                org1_id,
                "Sophie",
                "Bernard",
                "sophie.bernard@email.be",
                "+32 495 98 76 54",
                "Rue Royale 28",
                "Bruxelles",
                "1000",
                "Belgique",
            )
            .await?;

        let owner3_db_id = self
            .create_demo_owner(
                org1_id,
                "Michel",
                "Lefebvre",
                "michel.lefebvre@email.be",
                "+32 477 11 22 33",
                "Boulevard d'Avroy 42",
                "Liège",
                "4000",
                "Belgique",
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
            org1_id,
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
            org1_id,
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
            org1_id,
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
            org1_id,
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

        // Create meetings ORG 1
        self.create_demo_meeting(
            building1_id,
            org1_id,
            "Assemblée Générale Ordinaire 2025",
            "ordinary",
            "2025-03-15",
            "scheduled",
        )
        .await?;

        self.create_demo_meeting(
            building2_id,
            org1_id,
            "Assemblée Générale Extraordinaire - Travaux",
            "extraordinary",
            "2025-04-20",
            "scheduled",
        )
        .await?;

        log::info!("✅ Demo meetings created");

        // Create documents ORG 1
        self.create_demo_document(
            building1_id,
            org1_id,
            "Procès-Verbal AG 2024",
            "meeting_minutes",
            "/uploads/demo/pv-ag-2024.pdf",
            syndic1_id,
        )
        .await?;

        self.create_demo_document(
            building1_id,
            org1_id,
            "Règlement de copropriété",
            "regulation",
            "/uploads/demo/reglement.pdf",
            syndic1_id,
        )
        .await?;

        log::info!("✅ Demo documents created");

        // ORGANIZATION 2 - Bruxelles
        let org2_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            org2_id,
            "Copropriété Bruxelles SPRL",
            "copro-bruxelles",
            "info@copro-bruxelles.be",
            "+32 2 123 45 67",
            "starter",
            5,
            10,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create demo organization 2: {}", e))?;

        let _syndic2_id = self
            .create_demo_user(
                "syndic@copro-bruxelles.be",
                "syndic123",
                "Marc",
                "Dubois",
                "syndic",
                Some(org2_id),
            )
            .await?;

        let building3_id = self
            .create_demo_building(
                org2_id,
                "Résidence Européenne",
                "Avenue Louise 123",
                "Bruxelles",
                "1050",
                "Belgique",
                12,
                2005,
            )
            .await?;

        self.create_demo_meeting(
            building3_id,
            org2_id,
            "AG Annuelle 2025",
            "ordinary",
            "2025-05-10",
            "scheduled",
        )
        .await?;

        log::info!("✅ Organization 2 created");

        // ORGANIZATION 3 - Liège
        let org3_id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            org3_id,
            "Syndic Liège SA",
            "syndic-liege",
            "contact@syndic-liege.be",
            "+32 4 222 33 44",
            "enterprise",
            50,
            100,
            true,
            now,
            now
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create demo organization 3: {}", e))?;

        let _syndic3_id = self
            .create_demo_user(
                "syndic@syndic-liege.be",
                "syndic123",
                "Sophie",
                "Lambert",
                "syndic",
                Some(org3_id),
            )
            .await?;

        let _building4_id = self
            .create_demo_building(
                org3_id,
                "Les Terrasses de Liège",
                "Boulevard de la Sauvenière 45",
                "Liège",
                "4000",
                "Belgique",
                8,
                2018,
            )
            .await?;

        log::info!("✅ Organization 3 created");

        Ok("✅ Demo data seeded successfully!\n\n\
            📊 Summary:\n\
            - 3 Organizations: Grand Place (Bruxelles), Bruxelles Louise, Liège\n\
            - 6+ Users: 3 Syndics, 1 Accountant, 2+ Owners\n\
            - 4 Buildings across Belgium\n\
            - 3 Owners (database records)\n\
            - 4 Units\n\
            - 4 Expenses\n\
            - 3 Meetings\n\
            - 2 Documents\n\n\
            🇧🇪 Belgian Demo - Credentials:\n\
            - Org 1 (Grand Place): syndic@grandplace.be / syndic123\n\
            - Org 2 (Bruxelles): syndic@copro-bruxelles.be / syndic123\n\
            - Org 3 (Liège): syndic@syndic-liege.be / syndic123\n\
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
        organization_id: Uuid,
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
            INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            owner_id,
            organization_id,
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
        organization_id: Uuid,
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
            INSERT INTO expenses (id, organization_id, building_id, category, description, amount, expense_date, payment_status, supplier, invoice_number, created_at, updated_at)
            VALUES ($1, $2, $3, $4::expense_category, $5, $6, $7, $8::payment_status, $9, $10, $11, $12)
            "#
        )
        .bind(expense_id)
        .bind(organization_id)
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

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_meeting(
        &self,
        building_id: Uuid,
        org_id: Uuid,
        title: &str,
        meeting_type: &str,
        scheduled_date: &str,
        status: &str,
    ) -> Result<Uuid, String> {
        let meeting_id = Uuid::new_v4();
        let now = Utc::now();
        let scheduled_date_parsed =
            chrono::DateTime::parse_from_rfc3339(&format!("{}T10:00:00Z", scheduled_date))
                .map_err(|e| format!("Failed to parse date: {}", e))?
                .with_timezone(&Utc);

        let agenda_json = serde_json::json!([
            "Approbation des comptes",
            "Travaux à prévoir",
            "Questions diverses"
        ]);

        sqlx::query(
            r#"
            INSERT INTO meetings (id, building_id, organization_id, meeting_type, title, description, scheduled_date, location, status, agenda, created_at, updated_at)
            VALUES ($1, $2, $3, $4::meeting_type, $5, $6, $7, $8, $9::meeting_status, $10, $11, $12)
            "#
        )
        .bind(meeting_id)
        .bind(building_id)
        .bind(org_id)
        .bind(meeting_type)
        .bind(title)
        .bind(Some("Assemblée générale annuelle"))
        .bind(scheduled_date_parsed)
        .bind("Salle polyvalente")
        .bind(status)
        .bind(agenda_json)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create meeting: {}", e))?;

        Ok(meeting_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_document(
        &self,
        building_id: Uuid,
        org_id: Uuid,
        title: &str,
        document_type: &str,
        file_path: &str,
        uploaded_by: Uuid,
    ) -> Result<Uuid, String> {
        let document_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO documents (id, building_id, organization_id, document_type, title, description, file_path, file_size, mime_type, uploaded_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4::document_type, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(document_id)
        .bind(building_id)
        .bind(org_id)
        .bind(document_type)
        .bind(title)
        .bind(Some("Document de démonstration"))
        .bind(file_path)
        .bind(1024_i64)
        .bind("application/pdf")
        .bind(uploaded_by)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create document: {}", e))?;

        Ok(document_id)
    }

    /// Clear all data (DANGEROUS - use with caution!)
    pub async fn clear_demo_data(&self) -> Result<String, String> {
        log::warn!("⚠️  Clearing all demo data...");

        // Delete in correct order due to foreign key constraints
        sqlx::query("DELETE FROM documents")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete documents: {}", e))?;

        sqlx::query("DELETE FROM meetings")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete meetings: {}", e))?;

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
