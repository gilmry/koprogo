use crate::application::dto::{
    AddDocumentDto, AddPhotoDto, CreateWorkReportDto, PageRequest, UpdateWorkReportDto,
    WarrantyStatusDto, WorkReportFilters, WorkReportListResponseDto, WorkReportResponseDto,
};
use crate::application::ports::WorkReportRepository;
use crate::domain::entities::WorkReport;
use chrono::DateTime;
use std::sync::Arc;
use uuid::Uuid;

pub struct WorkReportUseCases {
    repository: Arc<dyn WorkReportRepository>,
}

impl WorkReportUseCases {
    pub fn new(repository: Arc<dyn WorkReportRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_work_report(
        &self,
        dto: CreateWorkReportDto,
    ) -> Result<WorkReportResponseDto, String> {
        let organization_id = Uuid::parse_str(&dto.organization_id)
            .map_err(|_| "Invalid organization_id format".to_string())?;
        let building_id = Uuid::parse_str(&dto.building_id)
            .map_err(|_| "Invalid building_id format".to_string())?;

        let work_date = DateTime::parse_from_rfc3339(&dto.work_date)
            .map_err(|_| "Invalid work_date format".to_string())?
            .with_timezone(&chrono::Utc);

        let completion_date = if let Some(ref date_str) = dto.completion_date {
            Some(
                DateTime::parse_from_rfc3339(date_str)
                    .map_err(|_| "Invalid completion_date format".to_string())?
                    .with_timezone(&chrono::Utc),
            )
        } else {
            None
        };

        let work_report = WorkReport::new(
            organization_id,
            building_id,
            dto.title,
            dto.description,
            dto.work_type,
            dto.contractor_name,
            work_date,
            dto.cost,
            dto.warranty_type.clone(),
        );

        let mut work_report = work_report;
        work_report.contractor_contact = dto.contractor_contact;
        work_report.completion_date = completion_date;
        work_report.invoice_number = dto.invoice_number;
        work_report.notes = dto.notes;

        let created = self.repository.create(&work_report).await?;
        Ok(self.to_response_dto(&created))
    }

    pub async fn get_work_report(&self, id: Uuid) -> Result<Option<WorkReportResponseDto>, String> {
        let work_report = self.repository.find_by_id(id).await?;
        Ok(work_report.map(|w| self.to_response_dto(&w)))
    }

    pub async fn list_work_reports_by_building(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<WorkReportResponseDto>, String> {
        let work_reports = self.repository.find_by_building(building_id).await?;
        Ok(work_reports
            .iter()
            .map(|w| self.to_response_dto(w))
            .collect())
    }

    pub async fn list_work_reports_by_organization(
        &self,
        organization_id: Uuid,
    ) -> Result<Vec<WorkReportResponseDto>, String> {
        let work_reports = self
            .repository
            .find_by_organization(organization_id)
            .await?;
        Ok(work_reports
            .iter()
            .map(|w| self.to_response_dto(w))
            .collect())
    }

    pub async fn list_work_reports_paginated(
        &self,
        page_request: &PageRequest,
        filters: &WorkReportFilters,
    ) -> Result<WorkReportListResponseDto, String> {
        let (work_reports, total) = self
            .repository
            .find_all_paginated(page_request, filters)
            .await?;

        let dtos = work_reports
            .iter()
            .map(|w| self.to_response_dto(w))
            .collect();

        Ok(WorkReportListResponseDto {
            work_reports: dtos,
            total,
            page: page_request.page,
            page_size: page_request.per_page,
        })
    }

    pub async fn get_active_warranties(
        &self,
        building_id: Uuid,
    ) -> Result<Vec<WarrantyStatusDto>, String> {
        let work_reports = self
            .repository
            .find_with_active_warranty(building_id)
            .await?;

        Ok(work_reports
            .iter()
            .map(|w| WarrantyStatusDto {
                work_report_id: w.id.to_string(),
                title: w.title.clone(),
                warranty_type: w.warranty_type.clone(),
                warranty_expiry: w.warranty_expiry.to_rfc3339(),
                is_valid: w.is_warranty_valid(),
                days_remaining: w.warranty_days_remaining(),
            })
            .collect())
    }

    pub async fn get_expiring_warranties(
        &self,
        building_id: Uuid,
        days: i32,
    ) -> Result<Vec<WarrantyStatusDto>, String> {
        let work_reports = self
            .repository
            .find_with_expiring_warranty(building_id, days)
            .await?;

        Ok(work_reports
            .iter()
            .map(|w| WarrantyStatusDto {
                work_report_id: w.id.to_string(),
                title: w.title.clone(),
                warranty_type: w.warranty_type.clone(),
                warranty_expiry: w.warranty_expiry.to_rfc3339(),
                is_valid: w.is_warranty_valid(),
                days_remaining: w.warranty_days_remaining(),
            })
            .collect())
    }

    pub async fn update_work_report(
        &self,
        id: Uuid,
        dto: UpdateWorkReportDto,
    ) -> Result<WorkReportResponseDto, String> {
        let mut work_report = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Work report not found".to_string())?;

        if let Some(title) = dto.title {
            work_report.title = title;
        }
        if let Some(description) = dto.description {
            work_report.description = description;
        }
        if let Some(work_type) = dto.work_type {
            work_report.work_type = work_type;
        }
        if let Some(contractor_name) = dto.contractor_name {
            work_report.contractor_name = contractor_name;
        }
        if let Some(contractor_contact) = dto.contractor_contact {
            work_report.contractor_contact = Some(contractor_contact);
        }
        if let Some(work_date_str) = dto.work_date {
            let work_date = DateTime::parse_from_rfc3339(&work_date_str)
                .map_err(|_| "Invalid work_date format".to_string())?
                .with_timezone(&chrono::Utc);
            work_report.work_date = work_date;
        }
        if let Some(completion_date_str) = dto.completion_date {
            let completion_date = DateTime::parse_from_rfc3339(&completion_date_str)
                .map_err(|_| "Invalid completion_date format".to_string())?
                .with_timezone(&chrono::Utc);
            work_report.completion_date = Some(completion_date);
        }
        if let Some(cost) = dto.cost {
            work_report.cost = cost;
        }
        if let Some(invoice_number) = dto.invoice_number {
            work_report.invoice_number = Some(invoice_number);
        }
        if let Some(notes) = dto.notes {
            work_report.notes = Some(notes);
        }
        if let Some(warranty_type) = dto.warranty_type {
            work_report.warranty_type = warranty_type;
            // Recalculate warranty expiry when type changes
            work_report.warranty_expiry = match work_report.warranty_type {
                crate::domain::entities::WarrantyType::None => chrono::Utc::now(),
                crate::domain::entities::WarrantyType::Standard => {
                    work_report.work_date + chrono::Duration::days(2 * 365)
                }
                crate::domain::entities::WarrantyType::Decennial => {
                    work_report.work_date + chrono::Duration::days(10 * 365)
                }
                crate::domain::entities::WarrantyType::Extended => {
                    work_report.work_date + chrono::Duration::days(3 * 365)
                }
                crate::domain::entities::WarrantyType::Custom { years } => {
                    work_report.work_date + chrono::Duration::days(years as i64 * 365)
                }
            };
        }

        work_report.updated_at = chrono::Utc::now();

        let updated = self.repository.update(&work_report).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn add_photo(
        &self,
        id: Uuid,
        dto: AddPhotoDto,
    ) -> Result<WorkReportResponseDto, String> {
        let mut work_report = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Work report not found".to_string())?;

        work_report.add_photo(dto.photo_path);

        let updated = self.repository.update(&work_report).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn add_document(
        &self,
        id: Uuid,
        dto: AddDocumentDto,
    ) -> Result<WorkReportResponseDto, String> {
        let mut work_report = self
            .repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| "Work report not found".to_string())?;

        work_report.add_document(dto.document_path);

        let updated = self.repository.update(&work_report).await?;
        Ok(self.to_response_dto(&updated))
    }

    pub async fn delete_work_report(&self, id: Uuid) -> Result<bool, String> {
        self.repository.delete(id).await
    }

    fn to_response_dto(&self, work_report: &WorkReport) -> WorkReportResponseDto {
        WorkReportResponseDto {
            id: work_report.id.to_string(),
            organization_id: work_report.organization_id.to_string(),
            building_id: work_report.building_id.to_string(),
            title: work_report.title.clone(),
            description: work_report.description.clone(),
            work_type: work_report.work_type.clone(),
            contractor_name: work_report.contractor_name.clone(),
            contractor_contact: work_report.contractor_contact.clone(),
            work_date: work_report.work_date.to_rfc3339(),
            completion_date: work_report.completion_date.as_ref().map(|d| d.to_rfc3339()),
            cost: work_report.cost,
            invoice_number: work_report.invoice_number.clone(),
            photos: work_report.photos.clone(),
            documents: work_report.documents.clone(),
            notes: work_report.notes.clone(),
            warranty_type: work_report.warranty_type.clone(),
            warranty_expiry: work_report.warranty_expiry.to_rfc3339(),
            is_warranty_valid: work_report.is_warranty_valid(),
            warranty_days_remaining: work_report.warranty_days_remaining(),
            created_at: work_report.created_at.to_rfc3339(),
            updated_at: work_report.updated_at.to_rfc3339(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::dto::{CreateWorkReportDto, UpdateWorkReportDto, WorkReportFilters};
    use crate::application::ports::WorkReportRepository;
    use crate::domain::entities::{WarrantyType, WorkReport, WorkType};
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::Uuid;

    /// In-memory mock for WorkReportRepository
    struct MockWorkReportRepository {
        reports: Mutex<Vec<WorkReport>>,
    }

    impl MockWorkReportRepository {
        fn new() -> Self {
            Self {
                reports: Mutex::new(Vec::new()),
            }
        }

        fn with_reports(reports: Vec<WorkReport>) -> Self {
            Self {
                reports: Mutex::new(reports),
            }
        }
    }

    #[async_trait]
    impl WorkReportRepository for MockWorkReportRepository {
        async fn create(&self, work_report: &WorkReport) -> Result<WorkReport, String> {
            let mut reports = self.reports.lock().await;
            reports.push(work_report.clone());
            Ok(work_report.clone())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkReport>, String> {
            let reports = self.reports.lock().await;
            Ok(reports.iter().find(|r| r.id == id).cloned())
        }

        async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<WorkReport>, String> {
            let reports = self.reports.lock().await;
            Ok(reports
                .iter()
                .filter(|r| r.building_id == building_id)
                .cloned()
                .collect())
        }

        async fn find_by_organization(
            &self,
            organization_id: Uuid,
        ) -> Result<Vec<WorkReport>, String> {
            let reports = self.reports.lock().await;
            Ok(reports
                .iter()
                .filter(|r| r.organization_id == organization_id)
                .cloned()
                .collect())
        }

        async fn find_all_paginated(
            &self,
            _page_request: &crate::application::dto::PageRequest,
            _filters: &WorkReportFilters,
        ) -> Result<(Vec<WorkReport>, i64), String> {
            let reports = self.reports.lock().await;
            let total = reports.len() as i64;
            Ok((reports.clone(), total))
        }

        async fn find_with_active_warranty(
            &self,
            building_id: Uuid,
        ) -> Result<Vec<WorkReport>, String> {
            let reports = self.reports.lock().await;
            Ok(reports
                .iter()
                .filter(|r| r.building_id == building_id && r.is_warranty_valid())
                .cloned()
                .collect())
        }

        async fn find_with_expiring_warranty(
            &self,
            building_id: Uuid,
            days: i32,
        ) -> Result<Vec<WorkReport>, String> {
            let reports = self.reports.lock().await;
            Ok(reports
                .iter()
                .filter(|r| {
                    r.building_id == building_id
                        && r.is_warranty_valid()
                        && r.warranty_days_remaining() <= days as i64
                })
                .cloned()
                .collect())
        }

        async fn update(&self, work_report: &WorkReport) -> Result<WorkReport, String> {
            let mut reports = self.reports.lock().await;
            if let Some(pos) = reports.iter().position(|r| r.id == work_report.id) {
                reports[pos] = work_report.clone();
            }
            Ok(work_report.clone())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, String> {
            let mut reports = self.reports.lock().await;
            let len_before = reports.len();
            reports.retain(|r| r.id != id);
            Ok(reports.len() < len_before)
        }
    }

    fn make_org_and_building() -> (Uuid, Uuid) {
        (Uuid::new_v4(), Uuid::new_v4())
    }

    #[tokio::test]
    async fn test_create_work_report() {
        let (org_id, building_id) = make_org_and_building();
        let repo = Arc::new(MockWorkReportRepository::new());
        let uc = WorkReportUseCases::new(repo);

        let work_date = Utc::now().to_rfc3339();
        let dto = CreateWorkReportDto {
            organization_id: org_id.to_string(),
            building_id: building_id.to_string(),
            title: "Elevator repair".to_string(),
            description: "Cable replacement".to_string(),
            work_type: WorkType::Repair,
            contractor_name: "Schindler".to_string(),
            contractor_contact: None,
            work_date,
            completion_date: None,
            cost: 2500.0,
            invoice_number: Some("INV-001".to_string()),
            notes: None,
            warranty_type: WarrantyType::Standard,
        };

        let result = uc.create_work_report(dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.title, "Elevator repair");
        assert_eq!(resp.cost, 2500.0);
        assert_eq!(resp.contractor_name, "Schindler");
        assert!(resp.is_warranty_valid);
    }

    #[tokio::test]
    async fn test_update_work_report() {
        let (org_id, building_id) = make_org_and_building();
        let report = WorkReport::new(
            org_id,
            building_id,
            "Old title".to_string(),
            "Old desc".to_string(),
            WorkType::Maintenance,
            "Contractor A".to_string(),
            Utc::now(),
            1000.0,
            WarrantyType::None,
        );
        let report_id = report.id;

        let repo = Arc::new(MockWorkReportRepository::with_reports(vec![report]));
        let uc = WorkReportUseCases::new(repo);

        let dto = UpdateWorkReportDto {
            title: Some("Updated title".to_string()),
            description: None,
            work_type: None,
            contractor_name: None,
            contractor_contact: None,
            work_date: None,
            completion_date: None,
            cost: Some(1500.0),
            invoice_number: None,
            notes: None,
            warranty_type: None,
        };

        let result = uc.update_work_report(report_id, dto).await;
        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.title, "Updated title");
        assert_eq!(resp.cost, 1500.0);
    }

    #[tokio::test]
    async fn test_list_work_reports_by_building() {
        let (org_id, building_id) = make_org_and_building();
        let other_building = Uuid::new_v4();

        let r1 = WorkReport::new(
            org_id,
            building_id,
            "Report 1".into(),
            "Desc".into(),
            WorkType::Repair,
            "C1".into(),
            Utc::now(),
            100.0,
            WarrantyType::None,
        );
        let r2 = WorkReport::new(
            org_id,
            building_id,
            "Report 2".into(),
            "Desc".into(),
            WorkType::Maintenance,
            "C2".into(),
            Utc::now(),
            200.0,
            WarrantyType::None,
        );
        let r3 = WorkReport::new(
            org_id,
            other_building,
            "Report 3".into(),
            "Desc".into(),
            WorkType::Emergency,
            "C3".into(),
            Utc::now(),
            300.0,
            WarrantyType::None,
        );

        let repo = Arc::new(MockWorkReportRepository::with_reports(vec![r1, r2, r3]));
        let uc = WorkReportUseCases::new(repo);

        let result = uc.list_work_reports_by_building(building_id).await;
        assert!(result.is_ok());
        let reports = result.unwrap();
        assert_eq!(reports.len(), 2);
        assert!(reports
            .iter()
            .all(|r| r.building_id == building_id.to_string()));
    }

    #[tokio::test]
    async fn test_get_active_warranties() {
        let (org_id, building_id) = make_org_and_building();

        // Active warranty (Standard = 2 years from now)
        let r1 = WorkReport::new(
            org_id,
            building_id,
            "Active warranty".into(),
            "Desc".into(),
            WorkType::Renovation,
            "C1".into(),
            Utc::now(),
            5000.0,
            WarrantyType::Standard,
        );
        // No warranty
        let r2 = WorkReport::new(
            org_id,
            building_id,
            "No warranty".into(),
            "Desc".into(),
            WorkType::Maintenance,
            "C2".into(),
            Utc::now(),
            100.0,
            WarrantyType::None,
        );

        let repo = Arc::new(MockWorkReportRepository::with_reports(vec![r1, r2]));
        let uc = WorkReportUseCases::new(repo);

        let result = uc.get_active_warranties(building_id).await;
        assert!(result.is_ok());
        let warranties = result.unwrap();
        assert_eq!(warranties.len(), 1);
        assert_eq!(warranties[0].title, "Active warranty");
        assert!(warranties[0].is_valid);
        assert!(warranties[0].days_remaining > 700);
    }

    #[tokio::test]
    async fn test_get_expiring_warranties() {
        let (org_id, building_id) = make_org_and_building();

        // Warranty expiring in ~30 days (custom 0-year warranty set to expire soon)
        let mut r1 = WorkReport::new(
            org_id,
            building_id,
            "Expiring soon".into(),
            "Desc".into(),
            WorkType::Repair,
            "C1".into(),
            Utc::now(),
            1000.0,
            WarrantyType::Standard,
        );
        // Override warranty_expiry to 20 days from now
        r1.warranty_expiry = Utc::now() + chrono::Duration::days(20);

        // Warranty valid for a long time (decennial)
        let r2 = WorkReport::new(
            org_id,
            building_id,
            "Long warranty".into(),
            "Desc".into(),
            WorkType::Renovation,
            "C2".into(),
            Utc::now(),
            50000.0,
            WarrantyType::Decennial,
        );

        let repo = Arc::new(MockWorkReportRepository::with_reports(vec![r1, r2]));
        let uc = WorkReportUseCases::new(repo);

        let result = uc.get_expiring_warranties(building_id, 30).await;
        assert!(result.is_ok());
        let expiring = result.unwrap();
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring[0].title, "Expiring soon");
        assert!(expiring[0].days_remaining <= 30);
    }
}
