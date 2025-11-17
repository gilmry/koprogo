use crate::domain::entities::{Account, AccountType, User, UserRole};
use bcrypt::{hash, DEFAULT_COST};
use chrono::{NaiveDate, Utc};
use fake::faker::address::en::*;
use fake::faker::name::en::*;
use fake::Fake;
use rand::Rng;
use sqlx::{PgPool, Row};
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

        // Upsert superadmin role (preserve if exists, create if missing)
        sqlx::query(
            r#"
            INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
            VALUES (gen_random_uuid(), $1, 'superadmin', NULL, true, NOW(), NOW())
            ON CONFLICT (user_id, role, organization_id)
            DO UPDATE SET
                is_primary = true,
                updated_at = NOW()
            "#,
        )
        .bind(superadmin_id)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to upsert superadmin role: {}", e))?;

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
            processing_restricted: false,
            processing_restricted_at: None,
            marketing_opt_out: false,
            marketing_opt_out_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Seed Belgian PCMN (Plan Comptable Minimum Normalisé) for all organizations
    /// This ensures every organization has the base chart of accounts
    pub async fn seed_belgian_pcmn_for_all_organizations(&self) -> Result<String, String> {
        log::info!("🌱 Seeding Belgian PCMN for all organizations...");

        // Get all organizations
        let organizations = sqlx::query!("SELECT id FROM organizations")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch organizations: {}", e))?;

        let mut total_created = 0;
        let mut orgs_seeded = 0;

        for org in organizations {
            let org_id = org.id;

            // Check if this organization already has accounts
            let existing_count = sqlx::query!(
                "SELECT COUNT(*) as count FROM accounts WHERE organization_id = $1",
                org_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count accounts: {}", e))?;

            if existing_count.count.unwrap_or(0) > 0 {
                log::debug!(
                    "Organization {} already has {} accounts, skipping",
                    org_id,
                    existing_count.count.unwrap_or(0)
                );
                continue;
            }

            // Seed PCMN for this organization
            let created = self.seed_belgian_pcmn_for_org(org_id).await?;
            total_created += created;
            orgs_seeded += 1;
        }

        let message = format!(
            "✅ Seeded {} accounts across {} organizations",
            total_created, orgs_seeded
        );
        log::info!("{}", message);
        Ok(message)
    }

    /// Seed Belgian PCMN for a specific organization (idempotent)
    async fn seed_belgian_pcmn_for_org(&self, organization_id: Uuid) -> Result<i64, String> {
        // Base PCMN accounts based on Belgian accounting standards
        let base_accounts = vec![
            // Class 6: Charges (Expenses)
            (
                "6100",
                "Charges courantes",
                None,
                AccountType::Expense,
                true,
            ),
            (
                "6110",
                "Entretien et réparations",
                None,
                AccountType::Expense,
                true,
            ),
            ("6120", "Personnel", None, AccountType::Expense, true),
            (
                "6130",
                "Services extérieurs",
                None,
                AccountType::Expense,
                true,
            ),
            (
                "6140",
                "Honoraires et commissions",
                None,
                AccountType::Expense,
                true,
            ),
            ("6150", "Assurances", None, AccountType::Expense, true),
            (
                "6200",
                "Travaux extraordinaires",
                None,
                AccountType::Expense,
                true,
            ),
            // Class 7: Produits (Revenue)
            (
                "7000",
                "Produits de gestion",
                None,
                AccountType::Revenue,
                true,
            ),
            (
                "7100",
                "Appels de fonds",
                Some("7000"),
                AccountType::Revenue,
                true,
            ),
            (
                "7200",
                "Autres produits",
                Some("7000"),
                AccountType::Revenue,
                true,
            ),
            // Class 4: Tiers (Third parties)
            (
                "4000",
                "Comptes de tiers",
                None,
                AccountType::Liability,
                false,
            ),
            (
                "4100",
                "TVA à récupérer",
                Some("4000"),
                AccountType::Asset,
                true,
            ),
            (
                "4110",
                "TVA récupérable",
                Some("4100"),
                AccountType::Asset,
                true,
            ),
            (
                "4400",
                "Fournisseurs",
                Some("4000"),
                AccountType::Liability,
                true,
            ),
            (
                "4500",
                "Copropriétaires",
                Some("4000"),
                AccountType::Asset,
                true,
            ),
            // Class 5: Trésorerie (Cash/Bank)
            ("5500", "Banque", None, AccountType::Asset, true),
            ("5700", "Caisse", None, AccountType::Asset, true),
        ];

        let mut created_count = 0;

        for (code, label, parent_code, account_type, direct_use) in base_accounts {
            // Create account using domain entity
            let account = Account::new(
                code.to_string(),
                label.to_string(),
                parent_code.map(|s| s.to_string()),
                account_type,
                direct_use,
                organization_id,
            )?;

            // Insert into database (idempotent - skip if exists)
            let result = sqlx::query!(
                r#"
                INSERT INTO accounts (id, code, label, parent_code, account_type, direct_use, organization_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (code, organization_id) DO NOTHING
                "#,
                account.id,
                account.code,
                account.label,
                account.parent_code,
                account.account_type as AccountType,
                account.direct_use,
                account.organization_id,
                account.created_at,
                account.updated_at
            )
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to insert account {}: {}", code, e))?;

            if result.rows_affected() > 0 {
                created_count += 1;
            }
        }

        log::info!(
            "Created {} PCMN accounts for organization {}",
            created_count,
            organization_id
        );
        Ok(created_count)
    }

    /// Seed demo data for production demonstration
    pub async fn seed_demo_data(&self) -> Result<String, String> {
        log::info!("🌱 Starting demo data seeding...");

        // Check if seed data already exists (only check for seed organizations, not all)
        let existing_seed_orgs =
            sqlx::query!("SELECT COUNT(*) as count FROM organizations WHERE is_seed_data = true")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| format!("Failed to count seed organizations: {}", e))?;

        if existing_seed_orgs.count.unwrap_or(0) > 0 {
            return Err(
                "Seed data already exists. Please use 'Clear Seed Data' first.".to_string(),
            );
        }

        // ORGANIZATION 1
        let org1_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, is_seed_data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(org1_id)
        .bind("Résidence Grand Place SPRL")
        .bind("residence-grand-place")
        .bind("contact@grandplace.be")
        .bind("+32 2 501 23 45")
        .bind("professional")
        .bind(20)
        .bind(50)
        .bind(true) // is_active
        .bind(true) // is_seed_data
        .bind(now)
        .bind(now)
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

        let owner1_user_id = self
            .create_demo_user(
                "proprietaire1@grandplace.be",
                "owner123",
                "Pierre",
                "Durand",
                "owner",
                Some(org1_id),
            )
            .await?;

        let owner2_user_id = self
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

        // Link users to owners (for portal access)
        sqlx::query("UPDATE owners SET user_id = $1 WHERE id = $2")
            .bind(owner1_user_id)
            .bind(owner1_db_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to link owner1 to user: {}", e))?;

        sqlx::query("UPDATE owners SET user_id = $1 WHERE id = $2")
            .bind(owner2_user_id)
            .bind(owner2_db_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to link owner2 to user: {}", e))?;

        log::info!("✅ Users linked to owners");

        // Create demo units (owner_id is now deprecated, set to None)
        let unit1_id = self
            .create_demo_unit(
                org1_id,
                building1_id,
                None, // owner_id deprecated
                "101",
                "apartment",
                Some(1),
                75.5,
                250.0,
            )
            .await?;

        let unit2_id = self
            .create_demo_unit(
                org1_id,
                building1_id,
                None, // owner_id deprecated
                "102",
                "apartment",
                Some(1),
                62.0,
                200.0,
            )
            .await?;

        let unit3_id = self
            .create_demo_unit(
                org1_id,
                building1_id,
                None, // owner_id deprecated
                "103",
                "apartment",
                Some(1),
                85.0,
                300.0,
            )
            .await?;

        let unit4_id = self
            .create_demo_unit(
                org1_id,
                building2_id,
                None, // owner_id deprecated
                "201",
                "apartment",
                Some(2),
                95.0,
                350.0,
            )
            .await?;

        log::info!("✅ Demo units created");

        // Create unit_owners relationships
        // Scenario 1: Unit 101 - Single owner (Pierre Durand 100%)
        self.create_demo_unit_owner(
            unit1_id,
            owner1_db_id,
            1.0,  // 100%
            true, // primary contact
            None, // no end_date (active)
        )
        .await?;

        // Scenario 2: Unit 102 - Co-ownership (Sophie Bernard 60%, Michel Lefebvre 40%)
        self.create_demo_unit_owner(
            unit2_id,
            owner2_db_id,
            0.6,  // 60%
            true, // primary contact
            None,
        )
        .await?;

        self.create_demo_unit_owner(
            unit2_id,
            owner3_db_id,
            0.4,   // 40%
            false, // not primary contact
            None,
        )
        .await?;

        // Scenario 3: Unit 103 - Co-ownership with 3 owners (50%, 30%, 20%)
        self.create_demo_unit_owner(
            unit3_id,
            owner1_db_id,
            0.5,  // 50%
            true, // primary contact
            None,
        )
        .await?;

        self.create_demo_unit_owner(
            unit3_id,
            owner2_db_id,
            0.3, // 30%
            false,
            None,
        )
        .await?;

        self.create_demo_unit_owner(
            unit3_id,
            owner3_db_id,
            0.2, // 20%
            false,
            None,
        )
        .await?;

        // Scenario 4: Unit 201 - Michel Lefebvre owns multiple units (100% of this one)
        self.create_demo_unit_owner(
            unit4_id,
            owner3_db_id,
            1.0,  // 100%
            true, // primary contact
            None,
        )
        .await?;

        log::info!("✅ Demo unit_owners relationships created");

        // Seed Belgian PCMN accounts for this organization
        self.seed_pcmn_accounts(org1_id).await?;
        log::info!("✅ Belgian PCMN accounts seeded");

        // Create demo expenses with realistic Belgian VAT rates and accounting links
        // Expense 1: Quarterly condo fees (paid) - 21% VAT
        let expense1_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                "Charges copropriété T1 2025",
                4132.23, // HT
                21.0,    // VAT 21%
                "2025-01-15",
                "2025-02-15", // due date
                "administration",
                "paid",
                Some("Syndic Services SPRL"),
                Some("SYN-2025-001"),
                Some("6100"), // PCMN: Charges courantes
            )
            .await?;

        // Expense 2: Elevator repair (paid) - 21% VAT
        let expense2_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                "Réparation ascenseur - Remplacement moteur",
                2066.12, // HT
                21.0,    // VAT 21%
                "2025-02-10",
                "2025-03-10",
                "maintenance",
                "paid",
                Some("Ascenseurs Plus SA"),
                Some("ASC-2025-023"),
                Some("6110"), // PCMN: Entretien et réparations
            )
            .await?;

        // Expense 3: Quarterly condo fees building 2 (pending, will become overdue) - 21% VAT
        let expense3_id = self
            .create_demo_expense_with_vat(
                building2_id,
                org1_id,
                "Charges copropriété T1 2025",
                2479.34, // HT
                21.0,    // VAT 21%
                "2025-01-15",
                "2025-02-15", // OVERDUE (due 2 months ago)
                "administration",
                "overdue",
                Some("Syndic Services SPRL"),
                Some("SYN-2025-002"),
                Some("6100"), // PCMN: Charges courantes
            )
            .await?;

        // Expense 4: Cleaning (paid) - 6% VAT (reduced rate for certain services)
        let expense4_id = self
            .create_demo_expense_with_vat(
                building2_id,
                org1_id,
                "Nettoyage parties communes - Forfait annuel",
                1132.08, // HT
                6.0,     // VAT 6% (reduced rate)
                "2025-01-01",
                "2025-01-31",
                "cleaning",
                "paid",
                Some("CleanPro Belgium SPRL"),
                Some("CLN-2025-156"),
                Some("6130"), // PCMN: Services extérieurs
            )
            .await?;

        // Expense 5: Insurance (pending) - 0% VAT (insurance exempt)
        let expense5_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                "Assurance incendie immeuble 2025",
                1850.00, // HT (no VAT)
                0.0,     // VAT 0% (exempt)
                "2025-01-05",
                "2025-02-05",
                "insurance",
                "pending",
                Some("AXA Belgium"),
                Some("AXA-2025-8472"),
                Some("6150"), // PCMN: Assurances
            )
            .await?;

        // Expense 6: Facade works (pending approval) - 21% VAT
        let expense6_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                "Rénovation façade - Devis Entreprise Martin",
                12396.69, // HT
                21.0,     // VAT 21%
                "2025-03-01",
                "2025-04-30",
                "works",
                "pending",
                Some("Entreprise Martin & Fils SPRL"),
                Some("MART-2025-042"),
                Some("6200"), // PCMN: Travaux extraordinaires
            )
            .await?;

        // ===== CURRENT MONTH EXPENSES =====
        // Use relative dates based on today's date
        let now = Utc::now();
        let current_month = now.format("%B %Y").to_string();
        let month_start = format!("{}", now.format("%Y-%m-01"));
        let day_3 = format!("{}", now.format("%Y-%m-03"));
        let day_5 = format!("{}", now.format("%Y-%m-05"));
        let day_8 = format!("{}", now.format("%Y-%m-08"));
        let day_10 = format!("{}", now.format("%Y-%m-10"));
        let month_end = format!("{}", (now + chrono::Duration::days(30)).format("%Y-%m-%d"));

        // Expense 7: Elevator maintenance (current month) - paid - 21% VAT
        let expense7_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                &format!("Maintenance ascenseur {}", current_month),
                826.45, // HT
                21.0,   // VAT 21%
                &day_5,
                &month_end,
                "maintenance",
                "paid",
                Some("Ascenseurs Plus SA"),
                Some(&format!("ASC-{}-001", now.format("%Y-%m"))),
                Some("6110"), // PCMN: Entretien et réparations
            )
            .await?;

        // Expense 8: Electricity bill (current month) - paid - 21% VAT
        let expense8_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                &format!("Électricité communs {}", current_month),
                387.60, // HT
                21.0,   // VAT 21%
                &day_3,
                &format!("{}", (now + chrono::Duration::days(25)).format("%Y-%m-%d")),
                "utilities",
                "paid",
                Some("Engie Electrabel"),
                Some(&format!("ENGIE-{}-3847", now.format("%Y-%m"))),
                Some("6100"), // PCMN: Charges courantes (Électricité)
            )
            .await?;

        // Expense 9: Cleaning service (current month) - paid - 6% VAT
        let expense9_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                &format!("Nettoyage communs {}", current_month),
                471.70, // HT
                6.0,    // VAT 6% (labor-intensive services)
                &month_start,
                &format!("{}", (now + chrono::Duration::days(20)).format("%Y-%m-%d")),
                "cleaning",
                "paid",
                Some("NetClean Services SPRL"),
                Some(&format!("CLEAN-{}-074", now.format("%Y-%m"))),
                Some("6130"), // PCMN: Services extérieurs (Nettoyage)
            )
            .await?;

        // Expense 10: Water bill (current month) - pending - 6% VAT
        let expense10_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                &format!("Eau communs {}", current_month),
                156.60, // HT
                6.0,    // VAT 6%
                &day_8,
                &format!("{}", (now + chrono::Duration::days(30)).format("%Y-%m-%d")),
                "utilities",
                "pending",
                Some("Vivaqua"),
                Some(&format!("VIVA-{}-9284", now.format("%Y-%m"))),
                Some("6100"), // PCMN: Charges courantes (Eau)
            )
            .await?;

        // Expense 11: Heating gas (current month) - paid - 21% VAT
        let expense11_id = self
            .create_demo_expense_with_vat(
                building1_id,
                org1_id,
                &format!("Chauffage gaz {}", current_month),
                1240.00, // HT
                21.0,    // VAT 21%
                &day_10,
                &format!("{}", (now + chrono::Duration::days(30)).format("%Y-%m-%d")),
                "utilities",
                "paid",
                Some("Sibelga"),
                Some(&format!("SIBEL-{}-7453", now.format("%Y-%m"))),
                Some("6100"), // PCMN: Charges courantes (Chauffage)
            )
            .await?;

        log::info!("✅ Demo expenses with VAT created (including current month)");

        // Calculate and save charge distributions
        self.create_demo_distributions(expense1_id, org1_id).await?;
        self.create_demo_distributions(expense2_id, org1_id).await?;
        self.create_demo_distributions(expense3_id, org1_id).await?;
        self.create_demo_distributions(expense4_id, org1_id).await?;
        self.create_demo_distributions(expense5_id, org1_id).await?;
        self.create_demo_distributions(expense6_id, org1_id).await?;
        self.create_demo_distributions(expense7_id, org1_id).await?;
        self.create_demo_distributions(expense8_id, org1_id).await?;
        self.create_demo_distributions(expense9_id, org1_id).await?;
        self.create_demo_distributions(expense10_id, org1_id)
            .await?;
        self.create_demo_distributions(expense11_id, org1_id)
            .await?;
        log::info!("✅ Charge distributions calculated");

        // Create payment reminders for overdue expense
        self.create_demo_payment_reminder(
            expense3_id,
            owner2_db_id, // Sophie Bernard
            org1_id,
            "FirstReminder",
            20, // 20 days overdue
        )
        .await?;

        self.create_demo_payment_reminder(
            expense3_id,
            owner3_db_id, // Michel Lefebvre
            org1_id,
            "SecondReminder",
            35, // 35 days overdue
        )
        .await?;

        log::info!("✅ Payment reminders created");

        // Create owner contributions (revenue) for current month
        log::info!("Creating owner contributions...");

        // Get quarter number for current month (using Datelike trait)
        use chrono::Datelike;
        let quarter = ((now.month() - 1) / 3) + 1;
        let year = now.year();

        // Regular contributions (appels de fonds) for current month
        // Each owner pays quarterly fees
        self.create_demo_owner_contribution(
            org1_id,
            owner1_db_id, // Jean Dupont
            Some(unit1_id),
            &format!("Appel de fonds T{} {} - Charges courantes", quarter, year),
            650.0,
            "regular",
            &month_start,
            "paid",
            Some(&day_5),
            Some("7000"), // PCMN: Regular contributions
        )
        .await?;

        self.create_demo_owner_contribution(
            org1_id,
            owner2_db_id, // Sophie Bernard
            Some(unit2_id),
            &format!("Appel de fonds T{} {} - Charges courantes", quarter, year),
            750.0,
            "regular",
            &month_start,
            "paid",
            Some(&day_8),
            Some("7000"),
        )
        .await?;

        self.create_demo_owner_contribution(
            org1_id,
            owner3_db_id, // Michel Lefebvre
            Some(unit3_id),
            &format!("Appel de fonds T{} {} - Charges courantes", quarter, year),
            600.0,
            "regular",
            &month_start,
            "pending",
            None, // Not paid yet
            Some("7000"),
        )
        .await?;

        // Note: Only 3 owners in seed data, so we skip owner4

        // Extraordinary contribution for roof repairs (previous month)
        let prev_month = (now - chrono::Duration::days(20))
            .format("%Y-%m-05")
            .to_string();
        let prev_month_payment = (now - chrono::Duration::days(15))
            .format("%Y-%m-10")
            .to_string();

        self.create_demo_owner_contribution(
            org1_id,
            owner1_db_id, // Jean Dupont
            Some(unit1_id),
            "Appel de fonds extraordinaire - Réfection toiture",
            1200.0,
            "extraordinary",
            &prev_month,
            "paid",
            Some(&prev_month_payment),
            Some("7100"), // PCMN: Extraordinary contributions
        )
        .await?;

        self.create_demo_owner_contribution(
            org1_id,
            owner2_db_id, // Sophie Bernard
            Some(unit2_id),
            "Appel de fonds extraordinaire - Réfection toiture",
            1400.0,
            "extraordinary",
            &prev_month,
            "pending",
            None, // Not paid yet
            Some("7100"),
        )
        .await?;

        log::info!("✅ Owner contributions created");

        // Create meetings ORG 1 (in the past, 3-6 months ago)
        let meeting1_date = (now - chrono::Duration::days(90))
            .format("%Y-%m-%d")
            .to_string();
        let meeting2_date = (now - chrono::Duration::days(60))
            .format("%Y-%m-%d")
            .to_string();

        let meeting1_id = self
            .create_demo_meeting(
                building1_id,
                org1_id,
                &format!("Assemblée Générale Ordinaire {}", year),
                "ordinary",
                &meeting1_date,
                "completed",
            )
            .await?;

        let meeting2_id = self
            .create_demo_meeting(
                building2_id,
                org1_id,
                "Assemblée Générale Extraordinaire - Travaux",
                "extraordinary",
                &meeting2_date,
                "completed",
            )
            .await?;

        log::info!("✅ Demo meetings created");

        // Create board members ORG 1
        // Board mandates are for 1 year from meeting1_date
        let mandate_start = meeting1_date.clone();
        let mandate_end = (now + chrono::Duration::days(275))
            .format("%Y-%m-%d")
            .to_string(); // ~9 months from now

        // Elect owner1 as president for building1 (mandate: ~1 year as per Belgian law)
        self.create_demo_board_member(
            owner1_db_id,
            building1_id,
            org1_id,
            meeting1_id,
            "president",
            &mandate_start,
            &mandate_end,
        )
        .await?;

        // Elect owner2 as treasurer for building2 (mandate: ~1 year)
        self.create_demo_board_member(
            owner2_db_id,
            building2_id,
            org1_id,
            meeting2_id,
            "treasurer",
            &meeting2_date,
            &format!("{}", (now + chrono::Duration::days(305)).format("%Y-%m-%d")),
        )
        .await?;

        log::info!("✅ Demo board members elected");

        // Create board decisions ORG 1
        // Decision 1: Pending with deadline in 25 days (medium urgency)
        self.create_demo_board_decision(
            building1_id,
            org1_id,
            meeting1_id,
            "Rénovation de la façade",
            "Approuver les devis pour la rénovation de la façade principale",
            Some("2025-11-26"), // ~25 days from 2025-11-01
            "pending",
        )
        .await?;

        // Decision 2: In progress with deadline in 4 days (critical urgency)
        self.create_demo_board_decision(
            building1_id,
            org1_id,
            meeting1_id,
            "Contrat d'assurance",
            "Signer le nouveau contrat d'assurance avec AXA",
            Some("2025-11-05"), // 4 days from 2025-11-01
            "in_progress",
        )
        .await?;

        // Decision 3: Overdue (deadline passed)
        self.create_demo_board_decision(
            building1_id,
            org1_id,
            meeting1_id,
            "Nettoyage des gouttières",
            "Engager une entreprise pour le nettoyage annuel des gouttières",
            Some("2025-10-15"), // Past deadline
            "pending",
        )
        .await?;

        // Decision 4: Completed
        self.create_demo_board_decision(
            building1_id,
            org1_id,
            meeting1_id,
            "Installation caméras",
            "Installation du système de vidéosurveillance dans le hall",
            Some("2025-10-01"),
            "completed",
        )
        .await?;

        // Decision 5: Pending with deadline in 10 days (high urgency)
        self.create_demo_board_decision(
            building2_id,
            org1_id,
            meeting2_id,
            "Remplacement chaudière",
            "Valider le choix du fournisseur pour la nouvelle chaudière",
            Some("2025-11-11"), // 10 days from 2025-11-01
            "pending",
        )
        .await?;

        // Decision 6: In progress with deadline in 20 days (medium urgency)
        self.create_demo_board_decision(
            building2_id,
            org1_id,
            meeting2_id,
            "Aménagement parking vélos",
            "Organiser l'aménagement du parking à vélos au rez-de-chaussée",
            Some("2025-11-21"), // 20 days from 2025-11-01
            "in_progress",
        )
        .await?;

        log::info!("✅ Demo board decisions created");

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
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, is_seed_data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(org2_id)
        .bind("Copropriété Bruxelles SPRL")
        .bind("copro-bruxelles")
        .bind("info@copro-bruxelles.be")
        .bind("+32 2 123 45 67")
        .bind("starter")
        .bind(5)
        .bind(10)
        .bind(true) // is_active
        .bind(true) // is_seed_data
        .bind(now)
        .bind(now)
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
        sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug, contact_email, contact_phone, subscription_plan, max_buildings, max_users, is_active, is_seed_data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(org3_id)
        .bind("Syndic Liège SA")
        .bind("syndic-liege")
        .bind("contact@syndic-liege.be")
        .bind("+32 4 222 33 44")
        .bind("enterprise")
        .bind(50)
        .bind(100)
        .bind(true) // is_active
        .bind(true) // is_seed_data
        .bind(now)
        .bind(now)
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

        sqlx::query(
            r#"
            INSERT INTO user_roles (id, user_id, role, organization_id, is_primary, created_at, updated_at)
            VALUES (gen_random_uuid(), $1, $2, $3, true, $4, $4)
            ON CONFLICT (user_id, role, organization_id)
            DO UPDATE SET is_primary = true, updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(user_id)
        .bind(role)
        .bind(organization_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to assign role {} to user {}: {}", role, email, e))?;

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
        organization_id: Uuid,
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
            INSERT INTO units (id, organization_id, building_id, owner_id, unit_number, unit_type, floor, surface_area, quota, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6::unit_type, $7, $8, $9, $10, $11)
            "#
        )
        .bind(unit_id)
        .bind(organization_id)
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

    async fn create_demo_unit_owner(
        &self,
        unit_id: Uuid,
        owner_id: Uuid,
        ownership_percentage: f64,
        is_primary_contact: bool,
        end_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<Uuid, String> {
        let unit_owner_id = Uuid::new_v4();
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO unit_owners (id, unit_id, owner_id, ownership_percentage, start_date, end_date, is_primary_contact, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            unit_owner_id,
            unit_id,
            owner_id,
            ownership_percentage,
            now, // start_date
            end_date,
            is_primary_contact,
            now, // created_at
            now  // updated_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create unit_owner relationship: {}", e))?;

        Ok(unit_owner_id)
    }

    #[allow(dead_code)]
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

        // Set approval_status based on payment_status
        let approval_status = if payment_status == "paid" {
            "approved" // If already paid, it must be approved
        } else {
            "draft" // Otherwise, start as draft
        };

        // Set paid_date if already paid
        let paid_date = if payment_status == "paid" {
            Some(expense_date_parsed)
        } else {
            None
        };

        // Check if expense already exists (idempotency)
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM expenses WHERE description = $1 AND building_id = $2")
                .bind(description)
                .bind(building_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| format!("Failed to check existing expense: {}", e))?;

        let final_expense_id = if let Some((existing_id,)) = existing {
            // Update existing expense
            sqlx::query(
                r#"
                UPDATE expenses SET
                    category = $1::expense_category,
                    amount = $2,
                    expense_date = $3,
                    payment_status = $4::payment_status,
                    approval_status = $5::approval_status,
                    paid_date = $6,
                    supplier = $7,
                    invoice_number = $8,
                    updated_at = $9
                WHERE id = $10
                "#,
            )
            .bind(category)
            .bind(amount)
            .bind(expense_date_parsed)
            .bind(payment_status)
            .bind(approval_status)
            .bind(paid_date)
            .bind(supplier)
            .bind(invoice_number)
            .bind(now)
            .bind(existing_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update expense: {}", e))?;
            existing_id
        } else {
            // Insert new expense
            sqlx::query(
                r#"
                INSERT INTO expenses (id, organization_id, building_id, category, description, amount, expense_date, payment_status, approval_status, paid_date, supplier, invoice_number, created_at, updated_at)
                VALUES ($1, $2, $3, $4::expense_category, $5, $6, $7, $8::payment_status, $9::approval_status, $10, $11, $12, $13, $14)
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
            .bind(approval_status)
            .bind(paid_date)
            .bind(supplier)
            .bind(invoice_number)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create expense: {}", e))?;
            expense_id
        };

        Ok(final_expense_id)
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

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_board_member(
        &self,
        owner_id: Uuid,
        building_id: Uuid,
        org_id: Uuid,
        meeting_id: Uuid,
        position: &str,
        mandate_start: &str,
        mandate_end: &str,
    ) -> Result<Uuid, String> {
        let board_member_id = Uuid::new_v4();
        let now = Utc::now();

        let mandate_start_parsed = NaiveDate::parse_from_str(mandate_start, "%Y-%m-%d")
            .map_err(|e| format!("Failed to parse mandate_start date: {}", e))?
            .and_hms_opt(0, 0, 0)
            .ok_or("Failed to create datetime")?;

        let mandate_end_parsed = NaiveDate::parse_from_str(mandate_end, "%Y-%m-%d")
            .map_err(|e| format!("Failed to parse mandate_end date: {}", e))?
            .and_hms_opt(0, 0, 0)
            .ok_or("Failed to create datetime")?;

        sqlx::query(
            r#"
            INSERT INTO board_members (id, owner_id, building_id, organization_id, position, mandate_start, mandate_end, elected_by_meeting_id, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5::board_position, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(board_member_id)
        .bind(owner_id)
        .bind(building_id)
        .bind(org_id)
        .bind(position)
        .bind(mandate_start_parsed)
        .bind(mandate_end_parsed)
        .bind(meeting_id)
        .bind(true)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create board member: {}", e))?;

        Ok(board_member_id)
    }

    #[allow(clippy::too_many_arguments)]
    async fn create_demo_board_decision(
        &self,
        building_id: Uuid,
        org_id: Uuid,
        meeting_id: Uuid,
        subject: &str,
        decision_text: &str,
        deadline: Option<&str>,
        status: &str,
    ) -> Result<Uuid, String> {
        let decision_id = Uuid::new_v4();
        let now = Utc::now();

        let deadline_parsed = if let Some(deadline_str) = deadline {
            Some(
                NaiveDate::parse_from_str(deadline_str, "%Y-%m-%d")
                    .map_err(|e| format!("Failed to parse deadline date: {}", e))?
                    .and_hms_opt(0, 0, 0)
                    .ok_or("Failed to create datetime")?,
            )
        } else {
            None
        };

        sqlx::query(
            r#"
            INSERT INTO board_decisions (id, building_id, organization_id, meeting_id, subject, decision_text, deadline, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::decision_status, $9, $10)
            "#
        )
        .bind(decision_id)
        .bind(building_id)
        .bind(org_id)
        .bind(meeting_id)
        .bind(subject)
        .bind(decision_text)
        .bind(deadline_parsed)
        .bind(status)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create board decision: {}", e))?;

        Ok(decision_id)
    }

    /// Seed realistic data for load testing (optimized for 1 vCPU / 2GB RAM)
    /// Generates: 3 orgs, ~23 buildings, ~190 units, ~127 owners, ~60 expenses
    pub async fn seed_realistic_data(&self) -> Result<String, String> {
        log::info!("🌱 Starting realistic data seeding...");

        // Check if data already exists
        let existing_orgs = sqlx::query("SELECT COUNT(*) as count FROM organizations")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| format!("Failed to count organizations: {}", e))?;

        let count: i64 = existing_orgs
            .try_get("count")
            .map_err(|e| format!("Failed to get count: {}", e))?;
        if count > 0 {
            return Err("Data already exists. Please clear the database first.".to_string());
        }

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

        let mut total_buildings = 0;
        let mut total_units = 0;
        let mut total_owners = 0;
        let mut total_expenses = 0;

        for (idx, (org_name, size, num_buildings, target_units)) in org_configs.iter().enumerate() {
            let org_id = Uuid::new_v4();
            let now = Utc::now();

            log::info!(
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
            .bind(if *size == "large" { "enterprise" } else if *size == "medium" { "professional" } else { "starter" })
            .bind(*num_buildings)
            .bind(if *size == "large" { 50 } else if *size == "medium" { 20 } else { 10 })
            .bind(true)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create organization: {}", e))?;

            // Create admin user for this org
            let user_id = Uuid::new_v4();
            let password_hash = hash("admin123", DEFAULT_COST)
                .map_err(|e| format!("Failed to hash password: {}", e))?;

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
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

            // Create owners pool for this org
            let num_owners = (target_units * 2 / 3) as usize; // ~66% occupancy
            let mut owner_ids = Vec::new();

            for o in 0..num_owners {
                let owner_id = Uuid::new_v4();

                // Use faker for realistic Belgian data
                let first_name: String = FirstName().fake();
                let last_name: String = LastName().fake();
                let street: String = StreetName().fake();
                let city_idx = rng.random_range(0..cities.len());
                let owner_city = cities[city_idx];

                sqlx::query(
                    "INSERT INTO owners (id, organization_id, first_name, last_name, email, phone, address, city, postal_code, country, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
                )
                .bind(owner_id)
                .bind(org_id)
                .bind(&first_name)
                .bind(&last_name)
                .bind(format!("{}. {}{}@{}.be", first_name.chars().next().unwrap_or('x'), last_name.to_lowercase(), o + 1, size))
                .bind(format!("+32 {} {} {} {}",
                    if rng.random_bool(0.5) { "2" } else { "4" },
                    rng.random_range(100..999),
                    rng.random_range(10..99),
                    rng.random_range(10..99)
                ))
                .bind(format!("{} {}", street, rng.random_range(1..200)))
                .bind(owner_city)
                .bind(format!("{}", rng.random_range(1000..9999)))
                .bind("Belgium")
                .bind(now)
                .bind(now)
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to create owner: {}", e))?;

                owner_ids.push(owner_id);
            }

            total_owners += num_owners;

            // Create buildings for this org
            let units_per_building = target_units / num_buildings;
            let mut org_units = 0;

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
                .execute(&self.pool)
                .await
                .map_err(|e| format!("Failed to create building: {}", e))?;

                // Create units for this building
                let units_this_building = if b == num_buildings - 1 {
                    // Last building gets remainder
                    target_units - org_units
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
                    .execute(&self.pool)
                    .await
                    .map_err(|e| format!("Failed to create unit: {}", e))?;
                }

                org_units += units_this_building;

                // Check if expenses already exist for this building (idempotency)
                let existing_expenses: (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM expenses WHERE building_id = $1")
                        .bind(building_id)
                        .fetch_one(&self.pool)
                        .await
                        .map_err(|e| format!("Failed to check existing expenses: {}", e))?;

                // Only create random expenses if none exist yet
                if existing_expenses.0 == 0 {
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

                        // Set approval_status based on payment_status
                        let approval_status = if payment_status == "paid" {
                            "approved" // If already paid, it must be approved
                        } else {
                            "draft" // Otherwise, start as draft
                        };

                        // Set paid_date if already paid
                        let paid_date = if payment_status == "paid" {
                            Some(expense_date)
                        } else {
                            None
                        };

                        sqlx::query(
                        "INSERT INTO expenses (id, organization_id, building_id, category, description, amount, expense_date, payment_status, approval_status, paid_date, created_at, updated_at)
                         VALUES ($1, $2, $3, $4::expense_category, $5, $6, $7, $8::payment_status, $9::approval_status, $10, $11, $12)"
                    )
                    .bind(Uuid::new_v4())
                    .bind(org_id)
                    .bind(building_id)
                    .bind(category)
                    .bind(desc)
                    .bind(amount)
                    .bind(expense_date)
                    .bind(payment_status)
                    .bind(approval_status)
                    .bind(paid_date)
                    .bind(now)
                    .bind(now)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| format!("Failed to create expense: {}", e))?;

                        total_expenses += 1;
                    }
                } // End if existing_expenses.0 == 0
            }

            total_buildings += num_buildings;
            total_units += org_units as usize;

            log::info!(
                "  ✅ Created {} buildings, {} units, {} owners",
                num_buildings,
                org_units,
                num_owners
            );
        }

        Ok(format!(
            "✅ Realistic seed data created successfully!\n\
             Total: {} orgs, {} buildings, {} units, {} owners, {} expenses\n\
             \nTest credentials:\n\
             - Small org:  admin@small.be / admin123\n\
             - Medium org: admin@medium.be / admin123\n\
             - Large org:  admin@large.be / admin123",
            org_configs.len(),
            total_buildings,
            total_units,
            total_owners,
            total_expenses
        ))
    }

    /// Seed Belgian PCMN accounts for an organization
    async fn seed_pcmn_accounts(&self, organization_id: Uuid) -> Result<(), String> {
        // Call the existing account seeding endpoint logic
        // We'll seed the essential accounts for demo purposes
        let accounts = vec![
            // Class 6: Charges (Expenses)
            ("6100", "Charges courantes", "EXPENSE"),
            ("6110", "Entretien et réparations", "EXPENSE"),
            ("6120", "Personnel", "EXPENSE"),
            ("6130", "Services extérieurs", "EXPENSE"),
            ("6140", "Honoraires et commissions", "EXPENSE"),
            ("6150", "Assurances", "EXPENSE"),
            ("6200", "Travaux extraordinaires", "EXPENSE"),
            // Class 7: Produits (Revenue)
            ("7000", "Appels de fonds ordinaires", "REVENUE"),
            ("7100", "Appels de fonds extraordinaires", "REVENUE"),
            ("7200", "Autres produits", "REVENUE"),
            // Class 4: Créances et dettes (Assets/Liabilities)
            ("4000", "Copropriétaires débiteurs", "ASSET"),
            ("4110", "TVA à récupérer", "ASSET"),
            ("4400", "Fournisseurs", "LIABILITY"),
            ("4500", "TVA à payer", "LIABILITY"),
            // Class 5: Trésorerie (Assets)
            ("5500", "Banque compte courant", "ASSET"),
            ("5700", "Caisse", "ASSET"),
        ];

        let now = Utc::now();

        for (code, label, account_type_str) in accounts {
            sqlx::query(
                r#"
                INSERT INTO accounts (id, code, label, parent_code, account_type, direct_use, organization_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5::account_type, $6, $7, $8, $9)
                ON CONFLICT (code, organization_id) DO NOTHING
                "#
            )
            .bind(Uuid::new_v4())
            .bind(code)
            .bind(label)
            .bind(None::<String>) // parent_code
            .bind(account_type_str)
            .bind(true) // direct_use
            .bind(organization_id)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to seed account {}: {}", code, e))?;
        }

        Ok(())
    }

    /// Create a demo expense with VAT calculation
    #[allow(clippy::too_many_arguments)]
    async fn create_demo_expense_with_vat(
        &self,
        building_id: Uuid,
        organization_id: Uuid,
        description: &str,
        amount_excl_vat: f64,
        vat_rate: f64,
        expense_date: &str,
        due_date: &str,
        category: &str,
        payment_status: &str,
        supplier: Option<&str>,
        invoice_number: Option<&str>,
        account_code: Option<&str>,
    ) -> Result<Uuid, String> {
        let expense_id = Uuid::new_v4();
        let now = Utc::now();

        // Calculate VAT and total
        let vat_amount = (amount_excl_vat * vat_rate / 100.0 * 100.0).round() / 100.0;
        let amount = amount_excl_vat + vat_amount;

        let expense_date_parsed =
            chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", expense_date))
                .map_err(|e| format!("Failed to parse expense_date: {}", e))?
                .with_timezone(&Utc);

        let due_date_parsed =
            chrono::DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", due_date))
                .map_err(|e| format!("Failed to parse due_date: {}", e))?
                .with_timezone(&Utc);

        // Set paid_date if payment_status is "paid"
        let paid_date = if payment_status == "paid" {
            Some(expense_date_parsed) // Use expense_date as paid_date
        } else {
            None
        };

        // Check if expense already exists (idempotency)
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM expenses WHERE description = $1 AND building_id = $2")
                .bind(description)
                .bind(building_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| format!("Failed to check existing expense: {}", e))?;

        let expense_id = if let Some((existing_id,)) = existing {
            // Update existing expense
            sqlx::query(
                r#"
                UPDATE expenses SET
                    category = $1::expense_category,
                    amount = $2,
                    amount_excl_vat = $3,
                    vat_rate = $4,
                    expense_date = $5,
                    due_date = $6,
                    payment_status = $7::payment_status,
                    paid_date = $8,
                    approval_status = $9::approval_status,
                    supplier = $10,
                    invoice_number = $11,
                    account_code = $12,
                    updated_at = $13
                WHERE id = $14
                "#,
            )
            .bind(category)
            .bind(amount)
            .bind(amount_excl_vat)
            .bind(vat_rate)
            .bind(expense_date_parsed)
            .bind(due_date_parsed)
            .bind(payment_status)
            .bind(paid_date)
            .bind("approved")
            .bind(supplier)
            .bind(invoice_number)
            .bind(account_code)
            .bind(now)
            .bind(existing_id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to update expense: {}", e))?;
            existing_id
        } else {
            // Insert new expense
            sqlx::query(
                r#"
                INSERT INTO expenses (
                    id, organization_id, building_id, category, description,
                    amount, amount_excl_vat, vat_rate, expense_date, due_date,
                    payment_status, paid_date, approval_status, supplier, invoice_number,
                    account_code, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4::expense_category, $5, $6, $7, $8, $9, $10, $11::payment_status, $12, $13::approval_status, $14, $15, $16, $17, $18)
                "#
            )
            .bind(expense_id)
            .bind(organization_id)
            .bind(building_id)
            .bind(category)
            .bind(description)
            .bind(amount)
            .bind(amount_excl_vat)
            .bind(vat_rate)
            .bind(expense_date_parsed)
            .bind(due_date_parsed)
            .bind(payment_status)
            .bind(paid_date)
            .bind("approved")
            .bind(supplier)
            .bind(invoice_number)
            .bind(account_code)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create expense with VAT: {}", e))?;
            expense_id
        };

        // Generate journal entry for this expense (double-entry bookkeeping)
        if let Some(acc_code) = account_code {
            self.generate_journal_entry_for_expense(
                expense_id,
                organization_id,
                building_id,
                description,
                amount_excl_vat,
                vat_rate,
                amount,
                expense_date_parsed,
                acc_code,
                supplier,
                invoice_number,
            )
            .await?;
        }

        Ok(expense_id)
    }

    /// Generate journal entry for an expense (double-entry bookkeeping)
    ///
    /// This creates the accounting entries following Belgian PCMN:
    /// - Debit: Expense account (class 6)
    /// - Debit: VAT recoverable (4110)
    /// - Credit: Supplier account (4400)
    #[allow(clippy::too_many_arguments)]
    async fn generate_journal_entry_for_expense(
        &self,
        expense_id: Uuid,
        organization_id: Uuid,
        _building_id: Uuid,
        description: &str,
        amount_excl_vat: f64,
        vat_rate: f64,
        total_amount: f64,
        expense_date: chrono::DateTime<Utc>,
        account_code: &str,
        supplier: Option<&str>,
        invoice_number: Option<&str>,
    ) -> Result<(), String> {
        let journal_entry_id = Uuid::new_v4();
        let now = Utc::now();

        // Calculate VAT amount
        let vat_amount = total_amount - amount_excl_vat;

        // Start a transaction - the deferred trigger will only check at COMMIT
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Insert journal entry header
        sqlx::query!(
            r#"
            INSERT INTO journal_entries (
                id, organization_id, entry_date, description,
                document_ref, expense_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            journal_entry_id,
            organization_id,
            expense_date,
            format!("{} - {}", description, supplier.unwrap_or("Fournisseur")),
            invoice_number,
            expense_id,
            now,
            now
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create journal entry: {}", e))?;

        // Line 1: Debit expense account (class 6)
        sqlx::query!(
            r#"
            INSERT INTO journal_entry_lines (
                journal_entry_id, organization_id, account_code,
                debit, credit, description
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            journal_entry_id,
            organization_id,
            account_code,
            rust_decimal::Decimal::from_f64_retain(amount_excl_vat).unwrap_or_default(),
            rust_decimal::Decimal::from_f64_retain(0.0).unwrap_or_default(),
            format!("Dépense: {}", description)
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create expense debit line: {}", e))?;

        // Line 2: Debit VAT recoverable (4110) if VAT > 0
        if vat_amount > 0.01 {
            sqlx::query!(
                r#"
                INSERT INTO journal_entry_lines (
                    journal_entry_id, organization_id, account_code,
                    debit, credit, description
                )
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
                journal_entry_id,
                organization_id,
                "4110", // VAT Recoverable account
                rust_decimal::Decimal::from_f64_retain(vat_amount).unwrap_or_default(),
                rust_decimal::Decimal::from_f64_retain(0.0).unwrap_or_default(),
                format!("TVA récupérable {}%", vat_rate)
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to create VAT debit line: {}", e))?;
        }

        // Line 3: Credit supplier account (4400)
        sqlx::query!(
            r#"
            INSERT INTO journal_entry_lines (
                journal_entry_id, organization_id, account_code,
                debit, credit, description
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            journal_entry_id,
            organization_id,
            "4400", // Suppliers account
            rust_decimal::Decimal::from_f64_retain(0.0).unwrap_or_default(),
            rust_decimal::Decimal::from_f64_retain(total_amount).unwrap_or_default(),
            supplier.map(|s| format!("Fournisseur: {}", s))
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create supplier credit line: {}", e))?;

        // Commit transaction - trigger will validate balance here
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit journal entry transaction: {}", e))?;

        Ok(())
    }

    /// Create demo charge distributions for an expense
    async fn create_demo_distributions(
        &self,
        expense_id: Uuid,
        _organization_id: Uuid,
    ) -> Result<(), String> {
        // Get all units for the expense's building
        let expense_row = sqlx::query!(
            "SELECT building_id, amount FROM expenses WHERE id = $1",
            expense_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch expense: {}", e))?;

        let building_id = expense_row.building_id;
        let total_amount = expense_row.amount;

        // Get all units with their quotas (NOT unit_owners - one record per unit)
        let units = sqlx::query!(
            r#"
            SELECT u.id as unit_id, u.quota
            FROM units u
            WHERE u.building_id = $1
            "#,
            building_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch units: {}", e))?;

        if units.is_empty() {
            return Ok(()); // No units to distribute to
        }

        // Calculate total quotas for the building
        let total_quota: f64 = units.iter().map(|u| u.quota).sum();

        let now = Utc::now();

        // Create ONE distribution per unit (not per owner)
        // The primary owner will be responsible for collecting from co-owners
        for unit in units {
            // Get the primary contact owner for this unit
            let primary_owner = sqlx::query!(
                r#"
                SELECT owner_id
                FROM unit_owners
                WHERE unit_id = $1 AND end_date IS NULL AND is_primary_contact = true
                ORDER BY created_at ASC
                LIMIT 1
                "#,
                unit.unit_id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("Failed to fetch primary owner: {}", e))?;

            // Skip if no owner found for this unit
            let owner_id = match primary_owner {
                Some(owner) => owner.owner_id,
                None => continue, // Skip this unit if no owner
            };

            let quota_percentage = if total_quota > 0.0 {
                unit.quota / total_quota
            } else {
                0.0
            };

            let amount_due = if total_quota > 0.0 {
                (quota_percentage * total_amount * 100.0).round() / 100.0
            } else {
                0.0
            };

            sqlx::query(
                r#"
                INSERT INTO charge_distributions (
                    id, expense_id, unit_id, owner_id,
                    quota_percentage, amount_due, created_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(expense_id)
            .bind(unit.unit_id)
            .bind(owner_id)
            .bind(quota_percentage)
            .bind(amount_due)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to create charge distribution: {}", e))?;
        }

        Ok(())
    }

    /// Create a demo owner contribution (revenue)
    #[allow(clippy::too_many_arguments)]
    async fn create_demo_owner_contribution(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        unit_id: Option<Uuid>,
        description: &str,
        amount: f64,
        contribution_type: &str,
        contribution_date: &str,
        payment_status: &str,
        payment_date: Option<&str>,
        account_code: Option<&str>,
    ) -> Result<Uuid, String> {
        let contribution_id = Uuid::new_v4();
        let contribution_date = NaiveDate::parse_from_str(contribution_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid contribution date: {}", e))?
            .and_hms_opt(10, 0, 0)
            .ok_or("Invalid contribution time")?
            .and_local_timezone(Utc)
            .unwrap();

        let payment_date_tz = payment_date
            .map(|date_str| {
                NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid payment date: {}", e))
                    .and_then(|date| {
                        date.and_hms_opt(10, 0, 0)
                            .ok_or("Invalid payment time".to_string())
                    })
                    .map(|dt| dt.and_local_timezone(Utc).unwrap())
            })
            .transpose()?;

        let payment_method = if payment_status == "paid" {
            Some("bank_transfer")
        } else {
            None
        };

        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO owner_contributions (
                id, organization_id, owner_id, unit_id,
                description, amount, account_code,
                contribution_type, contribution_date, payment_date,
                payment_method, payment_status,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(contribution_id)
        .bind(organization_id)
        .bind(owner_id)
        .bind(unit_id)
        .bind(description)
        .bind(amount)
        .bind(account_code)
        .bind(contribution_type)
        .bind(contribution_date)
        .bind(payment_date_tz)
        .bind(payment_method)
        .bind(payment_status)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create owner contribution: {}", e))?;

        // Generate journal entry for this contribution (double-entry bookkeeping)
        if let Some(acc_code) = account_code {
            self.generate_journal_entry_for_contribution(
                contribution_id,
                organization_id,
                description,
                amount,
                contribution_date,
                acc_code,
            )
            .await?;
        }

        Ok(contribution_id)
    }

    /// Generate journal entry for an owner contribution (double-entry bookkeeping)
    ///
    /// This creates the accounting entries following Belgian PCMN:
    /// - Debit: Owner receivables (4000) - Money owed by owner
    /// - Credit: Revenue account (class 7) - Income for ACP
    async fn generate_journal_entry_for_contribution(
        &self,
        contribution_id: Uuid,
        organization_id: Uuid,
        description: &str,
        amount: f64,
        contribution_date: chrono::DateTime<Utc>,
        account_code: &str,
    ) -> Result<(), String> {
        let journal_entry_id = Uuid::new_v4();
        let now = Utc::now();

        // Start a transaction with deferred constraints
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Failed to begin transaction: {}", e))?;

        // Set constraints to deferred for this transaction
        sqlx::query("SET CONSTRAINTS ALL DEFERRED")
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to defer constraints: {}", e))?;

        // Create journal entry header
        sqlx::query(
            r#"
            INSERT INTO journal_entries (
                id, organization_id, entry_date, description,
                contribution_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(journal_entry_id)
        .bind(organization_id)
        .bind(contribution_date)
        .bind(description)
        .bind(contribution_id)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create journal entry: {}", e))?;

        // Line 1: DEBIT - Owner receivables (4000 = Copropriétaires débiteurs)
        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (
                id, journal_entry_id, organization_id, account_code,
                description, debit, credit, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(journal_entry_id)
        .bind(organization_id)
        .bind("4000") // Owner receivables
        .bind(format!("Créance - {}", description))
        .bind(amount) // Debit
        .bind(0.0) // Credit
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create debit line (4000): {}", e))?;

        // Line 2: CREDIT - Revenue account (class 7)
        sqlx::query(
            r#"
            INSERT INTO journal_entry_lines (
                id, journal_entry_id, organization_id, account_code,
                description, debit, credit, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(journal_entry_id)
        .bind(organization_id)
        .bind(account_code) // Revenue account (e.g., 7000)
        .bind(format!("Produit - {}", description))
        .bind(0.0) // Debit
        .bind(amount) // Credit
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| format!("Failed to create credit line ({}): {}", account_code, e))?;

        // Commit transaction (constraints will be checked here)
        tx.commit()
            .await
            .map_err(|e| format!("Failed to commit transaction: {}", e))?;

        Ok(())
    }

    /// Create a demo payment reminder
    #[allow(clippy::too_many_arguments)]
    async fn create_demo_payment_reminder(
        &self,
        expense_id: Uuid,
        owner_id: Uuid,
        organization_id: Uuid,
        reminder_level: &str,
        days_overdue: i64,
    ) -> Result<Uuid, String> {
        let reminder_id = Uuid::new_v4();
        let now = Utc::now();

        // Get expense amount and due date
        let expense = sqlx::query!(
            "SELECT amount, due_date FROM expenses WHERE id = $1",
            expense_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch expense: {}", e))?;

        let amount_owed = expense.amount;
        let due_date = expense
            .due_date
            .expect("Due date required for payment reminder");

        // Calculate penalty (8% annual rate)
        let penalty_amount = if days_overdue > 0 {
            let yearly_penalty = amount_owed * 0.08;
            let daily_penalty = yearly_penalty / 365.0;
            ((daily_penalty * days_overdue as f64) * 100.0).round() / 100.0
        } else {
            0.0
        };

        let total_amount = amount_owed + penalty_amount;
        let sent_date = now - chrono::Duration::days(5); // Sent 5 days ago

        sqlx::query(
            r#"
            INSERT INTO payment_reminders (
                id, organization_id, expense_id, owner_id,
                level, status, amount_owed, penalty_amount, total_amount,
                due_date, days_overdue, delivery_method, sent_date,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5::reminder_level, $6::reminder_status, $7, $8, $9, $10, $11, $12::delivery_method, $13, $14, $15)
            "#
        )
        .bind(reminder_id)
        .bind(organization_id)
        .bind(expense_id)
        .bind(owner_id)
        .bind(reminder_level) // FirstReminder, SecondReminder, etc.
        .bind("Sent") // status
        .bind(amount_owed)
        .bind(penalty_amount)
        .bind(total_amount)
        .bind(due_date)
        .bind(days_overdue as i32)
        .bind("Email") // delivery_method
        .bind(sent_date)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create payment reminder: {}", e))?;

        Ok(reminder_id)
    }

    /// Clear all data (DANGEROUS - use with caution!)
    pub async fn clear_demo_data(&self) -> Result<String, String> {
        log::warn!("⚠️  Clearing seed data only (preserving production data)...");

        // Get seed organization IDs
        let seed_org_ids: Vec<Uuid> =
            sqlx::query_scalar!("SELECT id FROM organizations WHERE is_seed_data = true")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| format!("Failed to fetch seed organizations: {}", e))?;

        if seed_org_ids.is_empty() {
            return Ok("ℹ️  No seed data found to clear.".to_string());
        }

        log::info!("Found {} seed organizations to clean", seed_org_ids.len());

        // Delete in correct order due to foreign key constraints
        // 1. Board decisions (reference board_members and meetings)
        sqlx::query!(
            "DELETE FROM board_decisions WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete board_decisions: {}", e))?;

        // 2. Board members (reference meetings)
        sqlx::query!(
            "DELETE FROM board_members WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete board_members: {}", e))?;

        // 3. Payment reminders (reference expenses and owners)
        sqlx::query!(
            "DELETE FROM payment_reminders WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete payment_reminders: {}", e))?;

        // 3b. Owner contributions (revenue)
        sqlx::query!(
            "DELETE FROM owner_contributions WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete owner_contributions: {}", e))?;

        // 4. Charge distributions (reference expenses)
        sqlx::query(
            "DELETE FROM charge_distributions WHERE expense_id IN (SELECT id FROM expenses WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1)))"
        )
        .bind(&seed_org_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete charge_distributions: {}", e))?;

        // 5. Invoice line items (reference expenses)
        sqlx::query(
            "DELETE FROM invoice_line_items WHERE expense_id IN (SELECT id FROM expenses WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1)))"
        )
        .bind(&seed_org_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete invoice_line_items: {}", e))?;

        // 6. Documents linked to buildings or expenses
        sqlx::query!(
            "DELETE FROM documents WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete documents: {}", e))?;

        // 7. Meetings (now safe to delete after board members)
        sqlx::query!(
            "DELETE FROM meetings WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete meetings: {}", e))?;

        // 8. Journal entry lines (reference accounts) - MUST be deleted before accounts
        sqlx::query!(
            "DELETE FROM journal_entry_lines WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete journal_entry_lines: {}", e))?;

        // 9. Journal entries (now safe after lines are deleted)
        sqlx::query!(
            "DELETE FROM journal_entries WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete journal_entries: {}", e))?;

        // 10. Expenses (now safe to delete after distributions, line items, and journal entries)
        sqlx::query!(
            "DELETE FROM expenses WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete expenses: {}", e))?;

        // Unit owners (junction table)
        sqlx::query(
            "DELETE FROM unit_owners WHERE unit_id IN (SELECT u.id FROM units u INNER JOIN buildings b ON u.building_id = b.id WHERE b.organization_id = ANY($1))"
        )
        .bind(&seed_org_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete unit_owners: {}", e))?;

        // Units
        sqlx::query!(
            "DELETE FROM units WHERE building_id IN (SELECT id FROM buildings WHERE organization_id = ANY($1))",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete units: {}", e))?;

        // Owners (only those linked to seed organizations through unit_owners)
        sqlx::query!(
            "DELETE FROM owners WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete owners: {}", e))?;

        // Buildings
        sqlx::query!(
            "DELETE FROM buildings WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete buildings: {}", e))?;

        // PCMN Accounts
        sqlx::query!(
            "DELETE FROM accounts WHERE organization_id = ANY($1)",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete accounts: {}", e))?;

        // User roles (before deleting users)
        sqlx::query(
            "DELETE FROM user_roles WHERE user_id IN (SELECT id FROM users WHERE organization_id = ANY($1) AND role != 'superadmin')"
        )
        .bind(&seed_org_ids)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete user_roles: {}", e))?;

        // Users (except superadmin)
        sqlx::query!(
            "DELETE FROM users WHERE organization_id = ANY($1) AND role != 'superadmin'",
            &seed_org_ids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete users: {}", e))?;

        // Finally, delete seed organizations
        sqlx::query!("DELETE FROM organizations WHERE is_seed_data = true")
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Failed to delete organizations: {}", e))?;

        log::info!("✅ Seed data cleared (production data and superadmin preserved)");

        Ok(format!(
            "✅ Seed data cleared successfully! ({} organizations removed)",
            seed_org_ids.len()
        ))
    }
}
