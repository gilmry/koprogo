# Issue #029: Import Relevés ISTA (Historique Consommations)

**Priorité**: Moyenne
**Effort estimé**: 6-8h
**Phase**: Phase 2 (K3s - Automation & Community)
**Dépendances**: Issue #030 (optionnel - intégration IoT pour validation croisée)
**Labels**: `feature`, `automation`, `energy`, `belgium`

---

## 📋 Contexte

**ISTA** est le leader européen du sous-comptage et de la répartition des frais de chauffage et d'eau en immeubles collectifs. En Belgique, ISTA équipe une grande partie des copropriétés avec des compteurs individuels pour:
- Eau froide et chaude
- Chauffage (répartiteurs de frais de chauffage - RFC)
- Calories consommées

Les relevés ISTA sont fournis annuellement (ou semestriellement) sous forme de:
- **Fichiers CSV** (export standard)
- **Fichiers XML** (export détaillé avec métadonnées)
- **PDF** (rapports visuels non-structurés)

**Objectif**: Permettre l'import automatique des relevés ISTA dans KoproGo pour:
1. **Historiser** les consommations par unité et par période
2. **Analyser** les tendances de consommation
3. **Détecter** les anomalies (surconsommation, fuites)
4. **Comparer** avec les factures fournisseurs (eau, gaz, électricité)
5. **Préparer** les décomptes individuels de charges

---

## 🎯 Objectifs

### Fonctionnels
- ✅ Importer fichiers CSV/XML ISTA avec validation de format
- ✅ Mapper automatiquement les compteurs ISTA aux unités (Unit) via référence cadastrale ou numéro d'appartement
- ✅ Stocker l'historique des relevés avec horodatage et métadonnées
- ✅ Calculer les consommations par période (delta entre deux relevés)
- ✅ Générer des rapports d'analyse de consommation (par unité, par bâtiment, par type)
- ✅ Détecter les anomalies (variation > 30% entre périodes, valeurs aberrantes)
- ✅ Exporter les données vers Excel pour comptabilité

### Techniques
- ✅ Support multi-formats (CSV, XML) avec parsers extensibles
- ✅ Transaction atomique pour import batch (tout ou rien)
- ✅ Logs d'audit détaillés (fichier importé, utilisateur, timestamp, nombre de lignes)
- ✅ Validation stricte des données (dates, valeurs numériques, unités de mesure)
- ✅ Déduplication automatique (même fichier importé 2 fois = rejet)

---

## 🏗️ Architecture Technique

### 1. Nouvelles Entités Domain

#### `ISTAReading` (Relevé ISTA)
```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Représente un relevé de compteur ISTA importé.
/// Chaque relevé correspond à une ligne dans le fichier CSV/XML.
pub struct ISTAReading {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Option<Uuid>, // None si mapping pas encore effectué
    pub import_batch_id: Uuid, // Référence au batch d'import

    // Identification du compteur
    pub meter_number: String, // Numéro du compteur ISTA (ex: "12345678")
    pub meter_type: MeterType, // ColdWater, HotWater, Heating, Calories
    pub meter_location: Option<String>, // "Cuisine", "Salle de bain", etc.

    // Données de relevé
    pub reading_date: DateTime<Utc>, // Date du relevé
    pub reading_value: f64, // Valeur du compteur (ex: 1234.5 m³)
    pub previous_reading_value: Option<f64>, // Valeur précédente (si fournie par ISTA)
    pub consumption: Option<f64>, // Consommation calculée (reading_value - previous)
    pub unit_of_measure: String, // "m³", "kWh", "unités RFC", etc.

    // Métadonnées ISTA
    pub ista_contract_number: Option<String>, // Numéro de contrat ISTA
    pub billing_period_start: Option<DateTime<Utc>>,
    pub billing_period_end: Option<DateTime<Utc>>,
    pub estimated: bool, // true si relevé estimé (non réel)

    // Statut
    pub mapped_to_unit: bool, // true si unit_id a été mappé avec succès
    pub anomaly_detected: bool, // true si anomalie détectée
    pub anomaly_reason: Option<String>, // Description de l'anomalie

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeterType {
    ColdWater,    // Eau froide
    HotWater,     // Eau chaude sanitaire
    Heating,      // Chauffage (RFC - Répartiteur de Frais de Chauffage)
    Calories,     // Calories (compteur thermique)
}

impl ISTAReading {
    pub fn new(
        organization_id: Uuid,
        building_id: Uuid,
        import_batch_id: Uuid,
        meter_number: String,
        meter_type: MeterType,
        reading_date: DateTime<Utc>,
        reading_value: f64,
        unit_of_measure: String,
        estimated: bool,
    ) -> Result<Self, String> {
        // Validations
        if meter_number.trim().is_empty() {
            return Err("Meter number cannot be empty".to_string());
        }
        if reading_value < 0.0 {
            return Err("Reading value cannot be negative".to_string());
        }

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            unit_id: None,
            import_batch_id,
            meter_number,
            meter_type,
            meter_location: None,
            reading_date,
            reading_value,
            previous_reading_value: None,
            consumption: None,
            unit_of_measure,
            ista_contract_number: None,
            billing_period_start: None,
            billing_period_end: None,
            estimated,
            mapped_to_unit: false,
            anomaly_detected: false,
            anomaly_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Calcule la consommation si previous_reading_value est disponible
    pub fn calculate_consumption(&mut self) {
        if let Some(prev) = self.previous_reading_value {
            self.consumption = Some((self.reading_value - prev).max(0.0));
        }
    }

    /// Détecte une anomalie de consommation (> 30% de variation)
    pub fn detect_anomaly(&mut self, avg_consumption: f64) {
        if let Some(consumption) = self.consumption {
            if consumption > avg_consumption * 1.3 || consumption < avg_consumption * 0.3 {
                self.anomaly_detected = true;
                self.anomaly_reason = Some(format!(
                    "Variation anormale: {} {} vs moyenne {} {}",
                    consumption, self.unit_of_measure, avg_consumption, self.unit_of_measure
                ));
            }
        }
    }
}
```

#### `ISTAImportBatch` (Batch d'import)
```rust
/// Représente un batch d'import de fichier ISTA.
pub struct ISTAImportBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub imported_by: Uuid, // User ID

    // Fichier importé
    pub file_name: String,
    pub file_size: i64, // Taille en bytes
    pub file_format: FileFormat, // CSV, XML
    pub file_hash: String, // SHA-256 pour déduplication

    // Résultats de l'import
    pub total_rows: i32,
    pub successful_rows: i32,
    pub failed_rows: i32,
    pub duplicate_rows: i32,
    pub warnings: Vec<String>, // Avertissements non-bloquants
    pub errors: Vec<String>, // Erreurs de parsing

    // Statut
    pub status: ImportStatus, // Pending, Processing, Completed, Failed

    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    CSV,
    XML,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}
```

#### `MeterUnitMapping` (Mapping compteur → unité)
```rust
/// Configuration du mapping entre compteurs ISTA et unités.
/// Permet l'auto-mapping lors des futurs imports.
pub struct MeterUnitMapping {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub building_id: Uuid,
    pub unit_id: Uuid,

    pub meter_number: String, // Numéro du compteur ISTA
    pub meter_type: MeterType,
    pub meter_location: Option<String>,

    // Mapping alternatif (si numéro compteur change)
    pub fallback_reference: Option<String>, // Ex: cadastre, numéro d'appartement

    pub active: bool, // false si compteur remplacé
    pub created_at: DateTime<Utc>,
    pub deactivated_at: Option<DateTime<Utc>>,
}
```

---

### 2. Parsers (Infrastructure)

#### `ISTACSVParser`
```rust
// backend/src/infrastructure/ista/parsers/csv_parser.rs

use csv::ReaderBuilder;
use chrono::NaiveDate;

pub struct ISTACSVParser;

impl ISTACSVParser {
    /// Parse un fichier CSV ISTA standard.
    /// Format attendu (colonnes):
    /// - meter_number: Numéro compteur
    /// - meter_type: Type (water_cold, water_hot, heating, calories)
    /// - location: Emplacement (optionnel)
    /// - reading_date: Date relevé (DD/MM/YYYY)
    /// - reading_value: Valeur
    /// - previous_value: Valeur précédente (optionnel)
    /// - unit: Unité (m³, kWh, etc.)
    /// - estimated: Oui/Non
    pub async fn parse(
        &self,
        file_content: &[u8],
        building_id: Uuid,
        organization_id: Uuid,
    ) -> Result<Vec<ISTAReading>, String> {
        let mut reader = ReaderBuilder::new()
            .delimiter(b';') // ISTA utilise souvent ';' en Europe
            .from_reader(file_content);

        let mut readings = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let record = result.map_err(|e| format!("CSV parsing error at line {}: {}", idx + 1, e))?;

            // Validation et parsing
            let meter_number = record.get(0)
                .ok_or_else(|| format!("Missing meter_number at line {}", idx + 1))?
                .to_string();

            let meter_type = Self::parse_meter_type(
                record.get(1).unwrap_or("unknown")
            )?;

            let reading_date = Self::parse_date(
                record.get(3).unwrap_or("")
            )?;

            let reading_value: f64 = record.get(4)
                .ok_or_else(|| format!("Missing reading_value at line {}", idx + 1))?
                .parse()
                .map_err(|_| format!("Invalid reading_value at line {}", idx + 1))?;

            let previous_value: Option<f64> = record.get(5)
                .and_then(|s| s.parse().ok());

            let unit = record.get(6).unwrap_or("m³").to_string();

            let estimated = record.get(7).unwrap_or("Non") == "Oui";

            let mut reading = ISTAReading::new(
                organization_id,
                building_id,
                Uuid::new_v4(), // batch_id sera assigné plus tard
                meter_number,
                meter_type,
                reading_date.and_hms_opt(0, 0, 0).unwrap().and_utc(),
                reading_value,
                unit,
                estimated,
            )?;

            reading.previous_reading_value = previous_value;
            reading.meter_location = record.get(2).map(|s| s.to_string());
            reading.calculate_consumption();

            readings.push(reading);
        }

        Ok(readings)
    }

    fn parse_meter_type(s: &str) -> Result<MeterType, String> {
        match s.to_lowercase().as_str() {
            "water_cold" | "eau_froide" | "cold_water" => Ok(MeterType::ColdWater),
            "water_hot" | "eau_chaude" | "hot_water" => Ok(MeterType::HotWater),
            "heating" | "chauffage" | "rfc" => Ok(MeterType::Heating),
            "calories" | "thermique" => Ok(MeterType::Calories),
            _ => Err(format!("Unknown meter type: {}", s)),
        }
    }

    fn parse_date(s: &str) -> Result<NaiveDate, String> {
        // Support formats: DD/MM/YYYY, YYYY-MM-DD
        NaiveDate::parse_from_str(s, "%d/%m/%Y")
            .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
            .map_err(|_| format!("Invalid date format: {}", s))
    }
}
```

#### `ISTAXMLParser` (similaire pour XML)
```rust
// backend/src/infrastructure/ista/parsers/xml_parser.rs
// Utilise quick-xml pour parser le format XML ISTA
// Structure similaire au CSV mais avec balises XML
```

---

### 3. Use Cases

#### `ISTAUseCases`
```rust
// backend/src/application/use_cases/ista_use_cases.rs

pub struct ISTAUseCases {
    reading_repo: Arc<dyn ISTAReadingRepository>,
    batch_repo: Arc<dyn ISTAImportBatchRepository>,
    mapping_repo: Arc<dyn MeterUnitMappingRepository>,
    unit_repo: Arc<dyn UnitRepository>,
    csv_parser: ISTACSVParser,
    xml_parser: ISTAXMLParser,
}

impl ISTAUseCases {
    /// Importe un fichier ISTA (CSV ou XML)
    pub async fn import_file(
        &self,
        organization_id: Uuid,
        building_id: Uuid,
        imported_by: Uuid,
        file_name: String,
        file_content: Vec<u8>,
    ) -> Result<ISTAImportBatch, String> {
        // 1. Calculer hash pour déduplication
        let file_hash = format!("{:x}", sha2::Sha256::digest(&file_content));

        // Vérifier si déjà importé
        if self.batch_repo.exists_by_hash(organization_id, &file_hash).await? {
            return Err("File already imported (duplicate detected)".to_string());
        }

        // 2. Créer batch
        let mut batch = ISTAImportBatch {
            id: Uuid::new_v4(),
            organization_id,
            building_id,
            imported_by,
            file_name: file_name.clone(),
            file_size: file_content.len() as i64,
            file_format: Self::detect_format(&file_name)?,
            file_hash,
            total_rows: 0,
            successful_rows: 0,
            failed_rows: 0,
            duplicate_rows: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            status: ImportStatus::Processing,
            started_at: Utc::now(),
            completed_at: None,
            created_at: Utc::now(),
        };

        // 3. Parser selon le format
        let readings = match batch.file_format {
            FileFormat::CSV => self.csv_parser.parse(&file_content, building_id, organization_id).await?,
            FileFormat::XML => self.xml_parser.parse(&file_content, building_id, organization_id).await?,
        };

        batch.total_rows = readings.len() as i32;

        // 4. Assigner batch_id aux readings
        let mut final_readings: Vec<ISTAReading> = readings.into_iter()
            .map(|mut r| { r.import_batch_id = batch.id; r })
            .collect();

        // 5. Auto-mapping via MeterUnitMapping
        self.apply_auto_mapping(&mut final_readings).await?;

        // 6. Détection d'anomalies
        self.detect_anomalies(&mut final_readings).await?;

        // 7. Persister les readings (transaction)
        for reading in &final_readings {
            match self.reading_repo.create(reading).await {
                Ok(_) => batch.successful_rows += 1,
                Err(e) => {
                    batch.failed_rows += 1;
                    batch.errors.push(format!("Meter {}: {}", reading.meter_number, e));
                }
            }
        }

        // 8. Finaliser batch
        batch.status = if batch.failed_rows == 0 {
            ImportStatus::Completed
        } else {
            ImportStatus::Failed
        };
        batch.completed_at = Some(Utc::now());

        self.batch_repo.create(&batch).await?;

        Ok(batch)
    }

    /// Applique les mappings compteur → unité automatiquement
    async fn apply_auto_mapping(&self, readings: &mut [ISTAReading]) -> Result<(), String> {
        let mappings = self.mapping_repo.find_all_active(readings[0].building_id).await?;

        for reading in readings.iter_mut() {
            if let Some(mapping) = mappings.iter().find(|m|
                m.meter_number == reading.meter_number && m.meter_type == reading.meter_type
            ) {
                reading.unit_id = Some(mapping.unit_id);
                reading.mapped_to_unit = true;
            }
        }

        Ok(())
    }

    /// Détecte les anomalies de consommation
    async fn detect_anomalies(&self, readings: &mut [ISTAReading]) -> Result<(), String> {
        // Calculer la moyenne de consommation par type de compteur
        let avg_by_type = self.calculate_average_consumption(readings).await?;

        for reading in readings.iter_mut() {
            if let Some(avg) = avg_by_type.get(&reading.meter_type) {
                reading.detect_anomaly(*avg);
            }
        }

        Ok(())
    }

    /// Génère un rapport d'analyse de consommation
    pub async fn generate_consumption_report(
        &self,
        building_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<ConsumptionReport, String> {
        let readings = self.reading_repo
            .find_by_building_and_period(building_id, period_start, period_end)
            .await?;

        // Grouper par unité et par type
        let mut report = ConsumptionReport::new(building_id, period_start, period_end);

        for reading in readings {
            if let Some(consumption) = reading.consumption {
                report.add_consumption(
                    reading.unit_id,
                    reading.meter_type,
                    consumption,
                    reading.unit_of_measure.clone(),
                );
            }
        }

        Ok(report)
    }

    fn detect_format(file_name: &str) -> Result<FileFormat, String> {
        if file_name.to_lowercase().ends_with(".csv") {
            Ok(FileFormat::CSV)
        } else if file_name.to_lowercase().ends_with(".xml") {
            Ok(FileFormat::XML)
        } else {
            Err(format!("Unsupported file format: {}", file_name))
        }
    }
}
```

---

### 4. API Endpoints

#### Routes
```rust
// backend/src/infrastructure/web/routes.rs

cfg.service(
    web::scope("/api/v1")
        // ISTA Import
        .service(upload_ista_file)
        .service(list_import_batches)
        .service(get_import_batch)
        .service(list_readings_by_building)
        .service(list_readings_by_unit)
        .service(generate_consumption_report)
        .service(export_consumption_excel)
        .service(create_meter_mapping)
        .service(list_meter_mappings)
        .service(update_meter_mapping)
        .service(delete_meter_mapping)
);
```

#### Handlers
```rust
// backend/src/infrastructure/web/handlers/ista_handlers.rs

/// Upload et import d'un fichier ISTA (CSV/XML)
#[post("/ista/import")]
pub async fn upload_ista_file(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    mut payload: Multipart,
) -> impl Responder {
    // Role: Syndic, Accountant, SuperAdmin
    if !matches!(user.role.as_str(), "syndic" | "accountant" | "superadmin") {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Insufficient permissions".to_string(),
        });
    }

    // Extract building_id from form data
    let building_id = /* parse from multipart */;

    // Extract file content
    while let Some(item) = payload.next().await {
        let mut field = item.unwrap();
        let content_disposition = field.content_disposition();
        let file_name = content_disposition.get_filename().unwrap_or("unknown.csv").to_string();

        let mut file_content = Vec::new();
        while let Some(chunk) = field.next().await {
            file_content.extend_from_slice(&chunk.unwrap());
        }

        // Import file
        match state.ista_use_cases.import_file(
            user.organization_id,
            building_id,
            user.user_id,
            file_name,
            file_content,
        ).await {
            Ok(batch) => {
                state.audit_logger.log(AuditLogEntry::new(
                    AuditEventType::ISTAFileImported,
                    user.user_id,
                    Some(user.organization_id),
                    format!("Imported ISTA file: {} ({} rows)", batch.file_name, batch.total_rows),
                )).await;

                return HttpResponse::Ok().json(batch);
            }
            Err(e) => return HttpResponse::BadRequest().json(ErrorResponse { error: e }),
        }
    }

    HttpResponse::BadRequest().json(ErrorResponse {
        error: "No file provided".to_string(),
    })
}

/// Liste des batches d'import pour un bâtiment
#[get("/buildings/{building_id}/ista/imports")]
pub async fn list_import_batches(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
) -> impl Responder {
    let building_id = Uuid::parse_str(&path.into_inner()).unwrap();

    match state.ista_use_cases.list_batches(building_id).await {
        Ok(batches) => HttpResponse::Ok().json(batches),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

/// Rapport de consommation pour un bâtiment
#[get("/buildings/{building_id}/ista/consumption-report")]
pub async fn generate_consumption_report(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ConsumptionReportQuery>,
) -> impl Responder {
    let building_id = Uuid::parse_str(&path.into_inner()).unwrap();

    match state.ista_use_cases.generate_consumption_report(
        building_id,
        query.period_start,
        query.period_end,
    ).await {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse { error: e }),
    }
}

/// Export Excel du rapport de consommation
#[get("/buildings/{building_id}/ista/consumption-report/excel")]
pub async fn export_consumption_excel(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    path: web::Path<String>,
    query: web::Query<ConsumptionReportQuery>,
) -> impl Responder {
    // Générer rapport puis exporter vers Excel (rust_xlsxwriter)
    // Retourner fichier .xlsx en téléchargement
}

/// Créer un mapping compteur → unité
#[post("/ista/meter-mappings")]
pub async fn create_meter_mapping(
    state: web::Data<AppState>,
    user: AuthenticatedUser,
    dto: web::Json<CreateMeterMappingDto>,
) -> impl Responder {
    // Créer mapping pour auto-mapping lors des futurs imports
}
```

---

### 5. Frontend

#### Pages Svelte
- **`ImportISTAPage.svelte`**: Upload de fichiers ISTA
- **`ISTAImportHistoryPage.svelte`**: Liste des imports avec statuts
- **`ConsumptionReportPage.svelte`**: Rapports d'analyse de consommation
- **`MeterMappingConfigPage.svelte`**: Configuration des mappings compteur → unité

#### Composants
- **`ISTAFileUploader.svelte`**: Drag & drop pour upload CSV/XML
- **`ImportBatchCard.svelte`**: Carte affichant statut d'un batch (success/failed/warnings)
- **`ConsumptionChart.svelte`**: Graphiques de consommation (Chart.js)
- **`AnomalyAlert.svelte`**: Alertes pour anomalies détectées
- **`MeterMappingTable.svelte`**: Table éditable pour mappings

---

## 🧪 Tests

### Tests Unitaires
```rust
// backend/tests/unit/ista_reading_test.rs

#[test]
fn test_calculate_consumption() {
    let mut reading = ISTAReading::new(/* ... */).unwrap();
    reading.previous_reading_value = Some(100.0);
    reading.reading_value = 150.0;

    reading.calculate_consumption();

    assert_eq!(reading.consumption, Some(50.0));
}

#[test]
fn test_detect_anomaly_high_consumption() {
    let mut reading = ISTAReading::new(/* ... */).unwrap();
    reading.consumption = Some(200.0);

    reading.detect_anomaly(100.0); // Moyenne = 100

    assert!(reading.anomaly_detected);
    assert!(reading.anomaly_reason.is_some());
}
```

### Tests d'Intégration
```rust
// backend/tests/integration/ista_import_test.rs

#[tokio::test]
async fn test_import_csv_file() {
    let pool = setup_test_db().await;
    let use_cases = setup_ista_use_cases(pool.clone()).await;

    let csv_content = r#"meter_number;meter_type;location;reading_date;reading_value;previous_value;unit;estimated
12345678;water_cold;Cuisine;15/01/2025;1234.5;1200.0;m³;Non
87654321;heating;Salon;15/01/2025;567;550;unités;Non"#;

    let batch = use_cases.import_file(
        org_id,
        building_id,
        user_id,
        "test.csv".to_string(),
        csv_content.as_bytes().to_vec(),
    ).await.unwrap();

    assert_eq!(batch.total_rows, 2);
    assert_eq!(batch.successful_rows, 2);
    assert_eq!(batch.status, ImportStatus::Completed);
}

#[tokio::test]
async fn test_duplicate_file_rejection() {
    // Importer 2 fois le même fichier
    // Vérifier que le 2ème import est rejeté
}
```

### Tests E2E (BDD)
```gherkin
# backend/tests/features/ista_import.feature

Feature: Import ISTA Meter Readings
  As a Syndic
  I want to import ISTA meter readings
  So that I can track consumption history

  Scenario: Successfully import CSV file
    Given I am authenticated as a Syndic
    And I have a building with ID "building-123"
    When I upload an ISTA CSV file "readings_jan_2025.csv"
    Then the import batch should be created
    And the batch status should be "Completed"
    And 25 readings should be imported
    And meter mappings should be applied automatically

  Scenario: Detect consumption anomaly
    Given I have imported ISTA readings for the past year
    When a reading shows 200% increase compared to average
    Then an anomaly should be flagged
    And the syndic should receive an email alert
```

---

## 📚 Documentation Utilisateur

### Guide d'Import ISTA

**Étape 1: Obtenir le fichier ISTA**
- Se connecter au portail ISTA (https://www.ista.be)
- Télécharger l'export annuel au format CSV ou XML
- Vérifier que le fichier contient les colonnes requises

**Étape 2: Configurer les mappings (première fois)**
- Aller dans `Paramètres > Compteurs ISTA`
- Créer les mappings entre numéros de compteur et unités
- Exemple: Compteur `12345678` → Appartement 101

**Étape 3: Importer le fichier**
- Aller dans `Bâtiment > Import ISTA`
- Glisser-déposer le fichier CSV/XML
- Vérifier le statut de l'import (succès/erreurs)

**Étape 4: Analyser les consommations**
- Aller dans `Rapports > Consommations ISTA`
- Sélectionner la période d'analyse
- Consulter les graphiques et anomalies

---

## 🔒 Sécurité & Validation

- **Upload limité à 10 MB** par fichier
- **Formats autorisés**: CSV, XML uniquement
- **Validation stricte** des colonnes et types de données
- **Transaction atomique**: Si 1 ligne échoue, tout le batch échoue (configurable)
- **Audit logging**: Tous les imports sont tracés (fichier, utilisateur, timestamp)
- **Déduplication**: Hash SHA-256 du fichier pour éviter les doublons

---

## 🚀 Migration & Déploiement

### Migration SQL
```sql
-- backend/migrations/20250XXX_create_ista_tables.sql

CREATE TYPE meter_type AS ENUM ('cold_water', 'hot_water', 'heating', 'calories');
CREATE TYPE file_format AS ENUM ('csv', 'xml');
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');

CREATE TABLE ista_import_batches (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    building_id UUID NOT NULL REFERENCES buildings(id),
    imported_by UUID NOT NULL REFERENCES users(id),

    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    file_format file_format NOT NULL,
    file_hash VARCHAR(64) NOT NULL, -- SHA-256

    total_rows INTEGER NOT NULL DEFAULT 0,
    successful_rows INTEGER NOT NULL DEFAULT 0,
    failed_rows INTEGER NOT NULL DEFAULT 0,
    duplicate_rows INTEGER NOT NULL DEFAULT 0,
    warnings TEXT[],
    errors TEXT[],

    status import_status NOT NULL DEFAULT 'pending',
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, file_hash) -- Déduplication
);

CREATE TABLE ista_readings (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    building_id UUID NOT NULL REFERENCES buildings(id),
    unit_id UUID REFERENCES units(id), -- NULL si pas mappé
    import_batch_id UUID NOT NULL REFERENCES ista_import_batches(id) ON DELETE CASCADE,

    meter_number VARCHAR(100) NOT NULL,
    meter_type meter_type NOT NULL,
    meter_location VARCHAR(255),

    reading_date TIMESTAMP WITH TIME ZONE NOT NULL,
    reading_value DOUBLE PRECISION NOT NULL,
    previous_reading_value DOUBLE PRECISION,
    consumption DOUBLE PRECISION,
    unit_of_measure VARCHAR(20) NOT NULL,

    ista_contract_number VARCHAR(100),
    billing_period_start TIMESTAMP WITH TIME ZONE,
    billing_period_end TIMESTAMP WITH TIME ZONE,
    estimated BOOLEAN NOT NULL DEFAULT false,

    mapped_to_unit BOOLEAN NOT NULL DEFAULT false,
    anomaly_detected BOOLEAN NOT NULL DEFAULT false,
    anomaly_reason TEXT,

    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE meter_unit_mappings (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id),
    building_id UUID NOT NULL REFERENCES buildings(id),
    unit_id UUID NOT NULL REFERENCES units(id),

    meter_number VARCHAR(100) NOT NULL,
    meter_type meter_type NOT NULL,
    meter_location VARCHAR(255),

    fallback_reference VARCHAR(100),

    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deactivated_at TIMESTAMP WITH TIME ZONE,

    UNIQUE(organization_id, meter_number, meter_type) -- Un compteur = une unité
);

-- Index pour performance
CREATE INDEX idx_ista_readings_building ON ista_readings(building_id, reading_date);
CREATE INDEX idx_ista_readings_unit ON ista_readings(unit_id, reading_date);
CREATE INDEX idx_ista_readings_meter ON ista_readings(meter_number, meter_type);
CREATE INDEX idx_ista_batches_org ON ista_import_batches(organization_id, created_at DESC);
CREATE INDEX idx_meter_mappings_active ON meter_unit_mappings(building_id, active);
```

---

## 📊 Évolutions Futures

### Phase 3 (K8s - Real-time)
- **Alertes temps réel** pour anomalies détectées
- **API ISTA automatisée** (si ISTA fournit API)
- **Prévisions de consommation** (ML) basées sur l'historique
- **Intégration IoT** (Issue #030) pour validation croisée relevés ISTA vs capteurs temps réel

### Intégration avec Issue #028 (Commande Groupée Énergie)
- Utiliser l'historique ISTA pour **estimer la consommation future** lors des appels d'offres
- Comparer les **économies réalisées** après changement de fournisseur

---

## ✅ Checklist de Complétion

- [ ] Entités Domain créées et testées
- [ ] Parsers CSV et XML implémentés
- [ ] Use Cases avec logique métier complète
- [ ] Repositories PostgreSQL avec migrations
- [ ] API endpoints avec authentification
- [ ] Frontend: Pages et composants Svelte
- [ ] Tests unitaires (100% couverture entities)
- [ ] Tests d'intégration (import, mapping, rapports)
- [ ] Tests BDD (scénarios utilisateur)
- [ ] Documentation utilisateur (guide d'import)
- [ ] Migration SQL validée
- [ ] Logs d'audit intégrés
- [ ] Déploiement en staging et validation QA

---

**Responsable**: À assigner
**Milestone**: Phase 2 - K3s Automation
**Estimation**: 6-8h
**Dépendances**: Aucune (standalone), synergie avec Issue #030 (IoT)
