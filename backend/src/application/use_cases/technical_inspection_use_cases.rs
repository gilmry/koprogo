use crate::application::dto::{
    AddCertificateDto, AddInspectionPhotoDto, AddReportDto, CreateTechnicalInspectionDto,
    InspectionStatusDto, PageRequest, TechnicalInspectionFilters,
    TechnicalInspectionListResponseDto, TechnicalInspectionResponseDto,
    UpdateTechnicalInspectionDto,
};
use crate::application::ports::TechnicalInspectionRepository;
use crate::domain::entities::{InspectionStatus, TechnicalInspection};
use chrono::DateTime;
use std::sync::Arc;
use uuid::Uuid;

pub struct TechnicalInspectionUseCases {
    repository: Arc<dyn TechnicalInspectionRepository>,
}

impl TechnicalInspectionUseCases {
    pub fn new(repository: Arc<dyn TechnicalInspectionRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_technical_inspection(
        &self,
        dto: CreateTechnicalInspectionDto,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building_id format".to_string())?;

        let inspection_date = DateTime::parse_from_rfc3339(&dto.inspection_date)
            .map_err(|_| "Invalid inspection_date format".to_string())?
            .with_timezone(&chrono::Utc);

        let compliance_valid_until = if let Some(ref date_str) = dto.compliance_valid_until {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|_| "Invalid compliance_valid_until format".to_string())?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        let inspection = TechnicalInspection::new(
            organization_id,
            building_id,
            dto.title,
            dto.description,
            dto.inspection_type.clone(),
            dto.inspector_name,
            inspection_date,
        );

        let mut inspection = inspection;
        inspection.inspector_company = dto.inspector_company;
        inspection.inspector_certification = dto.inspector_certification;
        inspection.result_summary = dto.result_summary;
        inspection.defects_found = dto.defects_found;
        inspection.recommendations = dto.recommendations;
        inspection.compliant = dto.compliant;
        inspection.compliance_certificate_number = dto.compliance_certificate_number;
        inspection.compliance_valid_until = compliance_valid_until;
        inspection.cost = dto.cost;
        inspection.invoice_number = dto.invoice_number;
        inspection.notes = dto.notes;

        let created = self.repository.create(&inspection).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_technical_inspection(
        &self,
        id: Uuid,
    ) -> Result<Option<TechnicalInspectionResponseDto>, String> {
        let inspection = self.repository.find_by_id(id).await?;
        Ok(inspection.map(|i| self.to_response_dto(&i)))
    }

    pub async fn list_technical_inspections_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<TechnicalInspectionResponseDto>, String> {
        let inspections = self.repository.find_by_building(building_id).await?;
        Ok(inspections
            .iter()
            .map(|i| self.to_response_dto(i))
            .collect())
    }

    pub async fn list_technical_inspections_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<TechnicalInspectionResponseDto>, String> {
        let inspections = self
            .repository
            .find_by_organization(organization_id)
            .await?;
        Ok(inspections
            .iter()
            .map(|i| self.to_response_dto(i))
            .collect())
    }

    pub async fn list_technical_inspections_paginated(
        &self,
        page_request: &PageRequest,
        filters: &TechnicalInspectionFilters,
    ) -> Result<TechnicalInspectionListResponseDto, String> {
        let (inspections, total) = self
            .repository
            .find_all_paginated(page_request, filters)
            .await?;

        let dtos = inspections
            .iter()
            .map(|i| self.to_response_dto(i))
            .collect();

        Ok(TechnicalInspectionListResponseDto {
            inspections: dtos,
            total,
            page: page_request.page,
            page_size: page_request.per_page,
        })
    }

    pub async fn get_overdue_inspections(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<InspectionStatusDto>, String> {
        let inspections = self.repository.find_overdue(building_id).await?;

        Ok(inspections
            .iter()
            .map(|i| InspectionStatusDto {
                inspection_id: i.id.to_string(),
                title: i.title.clone(),
                inspection_type: i.inspection_type.clone(),
                next_due_date: i.next_due_date.to_rfc3339(),
                status: i.status.clone(),
                is_overdue: i.is_overdue(),
                days_until_due: i.days_until_due(),
            })
            .collect())
    }

    pub async fn get_upcoming_inspections(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<InspectionStatusDto>, String> {
        let inspections = self.repository.find_upcoming(building_id, days).await?;

        Ok(inspections
            .iter()
            .map(|i| InspectionStatusDto {
                inspection_id: i.id.to_string(),
                title: i.title.clone(),
                inspection_type: i.inspection_type.clone(),
                next_due_date: i.next_due_date.to_rfc3339(),
                status: i.status.clone(),
                is_overdue: i.is_overdue(),
                days_until_due: i.days_until_due(),
            })
            .collect())
    }

    pub async fn get_inspections_by_type(
        &self,
        building_id: Uuid,
        inspection_type: &str,
    ) -> Result<Vec<TechnicalInspectionResponseDto>, String> {
        let inspections = self
            .repository
            .find_by_type(building_id, inspection_type)
            .await?;

        Ok(inspections
            .iter()
            .map(|i| self.to_response_dto(i))
            .collect())
    }

    pub async fn update_technical_inspection(
        &self,
        id: Uuid,
        dto: UpdateTechnicalInspectionDto,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let mut inspection = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Technical inspection not found".to_string())?;

        if let Some(title) = dto.title {
            inspection.title = title;
        }
        if let Some(description) = dto.description {
            inspection.description = Some(description);
        }
        if let Some(inspection_type) = dto.inspection_type {
            inspection.inspection_type = inspection_type;
            // Recalculate next_due_date when type changes
            inspection.next_due_date = inspection.calculate_next_due_date();
        }
        if let Some(inspector_name) = dto.inspector_name {
            inspection.inspector_name = inspector_name;
        }
        if let Some(inspector_company) = dto.inspector_company {
            inspection.inspector_company = Some(inspector_company);
        }
        if let Some(inspector_certification) = dto.inspector_certification {
            inspection.inspector_certification = Some(inspector_certification);
        }
        if let Some(inspection_date_str) = dto.inspection_date {
            let inspection_date = DateTime::parse_from_rfc3339(&inspection_date_str)
                .map_err(|_| "Invalid inspection_date format".to_string())?
                .with_timezone(&chrono::Utc);
            inspection.inspection_date = inspection_date;
            // Recalculate next_due_date when date changes
            inspection.next_due_date = inspection.calculate_next_due_date();
        }
        if let Some(status) = dto.status {
            inspection.status = status;
        }
        if let Some(result_summary) = dto.result_summary {
            inspection.result_summary = Some(result_summary);
        }
        if let Some(defects_found) = dto.defects_found {
            inspection.defects_found = Some(defects_found);
        }
        if let Some(recommendations) = dto.recommendations {
            inspection.recommendations = Some(recommendations);
        }
        if let Some(compliant) = dto.compliant {
            inspection.compliant = Some(compliant);
        }
        if let Some(compliance_certificate_number) = dto.compliance_certificate_number {
            inspection.compliance_certificate_number = Some(compliance_certificate_number);
        }
        if let Some(compliance_valid_until_str) = dto.compliance_valid_until {
            let compliance_valid_until = DateTime::parse_from_rfc3339(&compliance_valid_until_str)
                .map_err(|_| "Invalid compliance_valid_until format".to_string())?
                .with_timezone(&chrono::Utc);
            inspection.compliance_valid_until = Some(compliance_valid_until);
        }
        if let Some(cost) = dto.cost {
            inspection.cost = Some(cost);
        }
        if let Some(invoice_number) = dto.invoice_number {
            inspection.invoice_number = Some(invoice_number);
        }
        if let Some(notes) = dto.notes {
            inspection.notes = Some(notes);
        }

        inspection.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&inspection).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn mark_as_completed(
        &self,
        id: Uuid,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let mut inspection = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Technical inspection not found".to_string())?;

        inspection.status = InspectionStatus::Completed;
        inspection.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&inspection).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn add_report(
        &self,
        id: Uuid,
        dto: AddReportDto,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let mut inspection = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Technical inspection not found".to_string())?;

        inspection.add_report(dto.report_path);

        let updated = self.repository.update(&inspection).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn add_photo(
        &self,
        id: Uuid,
        dto: AddInspectionPhotoDto,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let mut inspection = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Technical inspection not found".to_string())?;

        inspection.add_photo(dto.photo_path);

        let updated = self.repository.update(&inspection).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn add_certificate(
        &self,
        id: Uuid,
        dto: AddCertificateDto,
    ) -> Result<TechnicalInspectionResponseDto, String> {
        let mut inspection = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Technical inspection not found".to_string())?;

        inspection.add_certificate(dto.certificate_path);

        let updated = self.repository.update(&inspection).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn delete_technical_inspection(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    fn to_response_dto(&self, inspection: &TechnicalInspection) -> TechnicalInspectionResponseDto {
        TechnicalInspectionResponseDto {
            id: inspection.id.to_string(),
            organization_id: inspection.organization_id.to_string(),
            building_id: inspection.building_id.to_string(),
            title: inspection.title.clone(),
            description: inspection.description.clone(),
            inspection_type: inspection.inspection_type.clone(),
            inspector_name: inspection.inspector_name.clone(),
            inspector_company: inspection.inspector_company.clone(),
            inspector_certification: inspection.inspector_certification.clone(),
            inspection_date: inspection.inspection_date.to_rfc3339(),
            next_due_date: inspection.next_due_date.to_rfc3339(),
            status: inspection.status.clone(),
            result_summary: inspection.result_summary.clone(),
            defects_found: inspection.defects_found.clone(),
            recommendations: inspection.recommendations.clone(),
            compliant: inspection.compliant,
            compliance_certificate_number: inspection.compliance_certificate_number.clone(),
            compliance_valid_until: inspection
                .compliance_valid_until
                .as_ref()
                .map(|d| d.to_rfc3339()),
            cost: inspection.cost,
            invoice_number: inspection.invoice_number.clone(),
            reports: inspection.reports.clone(),
            photos: inspection.photos.clone(),
            certificates: inspection.certificates.clone(),
            notes: inspection.notes.clone(),
            is_overdue: inspection.is_overdue(),
            days_until_due: inspection.days_until_due(),
            created_at: inspection.created_at.to_rfc3339(),
            updated_at: inspection.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{
        AddInspectionPhotoDto, AddReportDto, CreateTechnicalInspectionDto, PageRequest,
        TechnicalInspectionFilters,
    };
    use crate::application::ports::TechnicalInspectionRepository;
    use crate::domain::entities::{InspectionStatus, InspectionType, TechnicalInspection};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========== Mock Repository ==========

    struct MockTechnicalInspectionRepository {
        inspections: Mutex<HashMap<Uuid, TechnicalInspection>>,
    }

    impl MockTechnicalInspectionRepository {
        fn new() -> Self {
            Self {
                inspections: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl TechnicalInspectionRepository for MockTechnicalInspectionRepository {
        async fn create(
            &self,
            inspection: &TechnicalInspection,
        ) -> Result<TechnicalInspection, String> {
            let mut inspections = self.inspections.lock().unwrap();
            inspections.insert(inspection.id, inspection.clone());
            Ok(inspection.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections.get(&id).cloned())
        }

        async fn find_by_building(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections
                .values()
                .filter(|i| i.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections
                .values()
                .filter(|i| i.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &PageRequest,
            _filters: &TechnicalInspectionFilters,
        ) -> Result<(Vec<TechnicalInspection>, i64), String> {
            let inspections = self.inspections.lock().unwrap();
            let all: Vec<TechnicalInspection> = inspections.values().cloned().collect();
            let count = all.len() as i64;
            Ok((all, count))
        }

        async fn find_overdue(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections
                .values()
                .filter(|i| i.building_id == building_id && i.is_overdue())
                .cloned()
                .collect())
        }

        async fn find_upcoming(
            &self,
            building_id: Uuid,
            days: i32,
        ) -> Result<Vec<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections
                .values()
                .filter(|i| {
                    i.building_id == building_id
                        && i.days_until_due() >= 0
                        && i.days_until_due() <= days as i64
                })
                .cloned()
                .collect())
        }

        async fn find_by_type(
            &self,
            building_id: Uuid,
            inspection_type: &str,
        ) -> Result<Vec<TechnicalInspection>, String> {
            let inspections = self.inspections.lock().unwrap();
            Ok(inspections
                .values()
                .filter(|i| {
                    i.building_id == building_id
                        && format!("{:?}", i.inspection_type).to_lowercase()
                            == inspection_type.to_lowercase()
                })
                .cloned()
                .collect())
        }

        async fn update(
            &self,
            inspection: &TechnicalInspection,
        ) -> Result<TechnicalInspection, String> {
            let mut inspections = self.inspections.lock().unwrap();
            inspections.insert(inspection.id, inspection.clone());
            Ok(inspection.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut inspections = self.inspections.lock().unwrap();
            Ok(inspections.remove(&id).is_some())
        }
    }

    // ========== Helpers ==========

    fn make_use_cases(repo: MockTechnicalInspectionRepository) -> TechnicalInspectionUseCases {
        TechnicalInspectionUseCases::new(Arc::new(repo))
    }

    fn valid_create_dto(org_id: Uuid, building_id: Uuid) -> CreateTechnicalInspectionDto {
        CreateTechnicalInspectionDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: "Inspection annuelle ascenseur".to_string(),
            description: Some("Vérification complète de l'ascenseur".to_string()),
            inspection_type: InspectionType::Elevator,
            inspector_name: "Schindler Belgium".to_string(),
            inspector_company: Some("Schindler SA".to_string()),
            inspector_certification: Some("CERT-2026-001".to_string()),
            inspection_date: "2026-03-01T10:00:00Z".to_string(),
            result_summary: None,
            defects_found: None,
            recommendations: None,
            compliant: None,
            compliance_certificate_number: None,
            compliance_valid_until: None,
            cost: Some(450.0),
            invoice_number: Some("INV-2026-100".to_string()),
            notes: None,
        }
    }

    // ========== Tests ==========

    #[tokio::test]
    async fn test_create_technical_inspection_success() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let result = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.organization_id, org_id.to_string());
        assert_eq!(dto.building_id, building_id.to_string());
        assert_eq!(dto.title, "Inspection annuelle ascenseur");
        assert_eq!(dto.inspector_name, "Schindler Belgium");
        assert_eq!(dto.inspector_company, Some("Schindler SA".to_string()));
        assert_eq!(dto.cost, Some(450.0));
        assert_eq!(dto.status, InspectionStatus::Scheduled);
        assert!(dto.reports.is_empty());
        assert!(dto.photos.is_empty());
        assert!(dto.certificates.is_empty());
    }

    #[tokio::test]
    async fn test_create_technical_inspection_invalid_date_format() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut dto = valid_create_dto(org_id, building_id);
        dto.inspection_date = "not-a-date".to_string();

        let result = uc.create_technical_inspection(dto).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid inspection_date format"
        );
    }

    #[tokio::test]
    async fn test_create_technical_inspection_invalid_org_id() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let mut dto = valid_create_dto(Uuid::new_v4(), Uuid::new_v4());
        dto.organization_id = "bad-uuid".to_string();

        let result = uc.create_technical_inspection(dto).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid organization_id format");
    }

    #[tokio::test]
    async fn test_get_technical_inspection_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();

        let result = uc.get_technical_inspection(inspection_id).await;
        assert!(result.is_ok());
        let found = result.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.title, "Inspection annuelle ascenseur");
    }

    #[tokio::test]
    async fn test_get_technical_inspection_not_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let result = uc.get_technical_inspection(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_technical_inspections_by_building() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_a = Uuid::new_v4();
        let building_b = Uuid::new_v4();

        // Create 2 inspections for building A
        let mut dto_a1 = valid_create_dto(org_id, building_a);
        dto_a1.title = "Elevator inspection".to_string();
        uc.create_technical_inspection(dto_a1).await.unwrap();

        let mut dto_a2 = valid_create_dto(org_id, building_a);
        dto_a2.title = "Boiler inspection".to_string();
        dto_a2.inspection_type = InspectionType::Boiler;
        uc.create_technical_inspection(dto_a2).await.unwrap();

        // Create 1 inspection for building B
        let dto_b = valid_create_dto(org_id, building_b);
        uc.create_technical_inspection(dto_b).await.unwrap();

        let result = uc
            .list_technical_inspections_by_building(building_a)
            .await;
        assert!(result.is_ok());
        let inspections = result.unwrap();
        assert_eq!(inspections.len(), 2);
        assert!(inspections
            .iter()
            .all(|i| i.building_id == building_a.to_string()));
    }

    #[tokio::test]
    async fn test_mark_as_completed() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();
        assert_eq!(created.status, InspectionStatus::Scheduled);

        let result = uc.mark_as_completed(inspection_id).await;
        assert!(result.is_ok());
        let completed = result.unwrap();
        assert_eq!(completed.status, InspectionStatus::Completed);
    }

    #[tokio::test]
    async fn test_mark_as_completed_not_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let result = uc.mark_as_completed(Uuid::new_v4()).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Technical inspection not found");
    }

    #[tokio::test]
    async fn test_add_report() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();
        assert!(created.reports.is_empty());

        let result = uc
            .add_report(
                inspection_id,
                AddReportDto {
                    report_path: "/uploads/reports/elevator-2026-03.pdf".to_string(),
                },
            )
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.reports.len(), 1);
        assert_eq!(
            updated.reports[0],
            "/uploads/reports/elevator-2026-03.pdf"
        );
    }

    #[tokio::test]
    async fn test_add_report_not_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let result = uc
            .add_report(
                Uuid::new_v4(),
                AddReportDto {
                    report_path: "/uploads/reports/test.pdf".to_string(),
                },
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Technical inspection not found");
    }

    #[tokio::test]
    async fn test_add_photo() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();

        let result = uc
            .add_photo(
                inspection_id,
                AddInspectionPhotoDto {
                    photo_path: "/uploads/photos/elevator-panel.jpg".to_string(),
                },
            )
            .await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.photos.len(), 1);
        assert_eq!(updated.photos[0], "/uploads/photos/elevator-panel.jpg");
    }

    #[tokio::test]
    async fn test_add_photo_not_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let result = uc
            .add_photo(
                Uuid::new_v4(),
                AddInspectionPhotoDto {
                    photo_path: "/uploads/photos/test.jpg".to_string(),
                },
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Technical inspection not found");
    }

    #[tokio::test]
    async fn test_delete_technical_inspection() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();

        // Delete should succeed
        let result = uc.delete_technical_inspection(inspection_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Should no longer be found
        let get_result = uc.get_technical_inspection(inspection_id).await;
        assert!(get_result.is_ok());
        assert!(get_result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_technical_inspection_not_found() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);

        let result = uc.delete_technical_inspection(Uuid::new_v4()).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_add_multiple_reports_and_photos() {
        let repo = MockTechnicalInspectionRepository::new();
        let uc = make_use_cases(repo);
        let org_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let created = uc
            .create_technical_inspection(valid_create_dto(org_id, building_id))
            .await
            .unwrap();
        let inspection_id = Uuid::parse_str(&created.id).unwrap();

        // Add 2 reports
        uc.add_report(
            inspection_id,
            AddReportDto {
                report_path: "/reports/report1.pdf".to_string(),
            },
        )
        .await
        .unwrap();
        uc.add_report(
            inspection_id,
            AddReportDto {
                report_path: "/reports/report2.pdf".to_string(),
            },
        )
        .await
        .unwrap();

        // Add 1 photo
        uc.add_photo(
            inspection_id,
            AddInspectionPhotoDto {
                photo_path: "/photos/photo1.jpg".to_string(),
            },
        )
        .await
        .unwrap();

        let inspection = uc
            .get_technical_inspection(inspection_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(inspection.reports.len(), 2);
        assert_eq!(inspection.photos.len(), 1);
    }
}
