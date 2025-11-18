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
