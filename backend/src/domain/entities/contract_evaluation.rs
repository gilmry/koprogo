use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Contract Evaluation (Review of contractor work)
/// Issue #276: Marketplace corps de métier + ContractEvaluation
/// Art. 3.89 §5 12° Code Civil Belge: Évaluations contracteurs (L13 annual report)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractEvaluation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub service_provider_id: Uuid,
    pub quote_id: Option<Uuid>,
    pub ticket_id: Option<Uuid>,
    pub evaluator_id: Uuid,
    pub building_id: Uuid,
    /// Criteria scores: qualite, delai, prix, communication, proprete, conformite_devis — each 0-5
    pub criteria: HashMap<String, u8>,
    pub global_score: f64, // weighted average 0-5
    pub comments: Option<String>,
    pub would_recommend: bool,
    pub is_legal_evaluation: bool, // true = rapport L13 (Art. 3.89 §5 12° CC)
    pub is_anonymous: bool,
    pub created_at: DateTime<Utc>,
}

impl ContractEvaluation {
    pub fn new(
        organization_id: Uuid,
        service_provider_id: Uuid,
        evaluator_id: Uuid,
        building_id: Uuid,
        criteria: HashMap<String, u8>,
        would_recommend: bool,
    ) -> Result<Self, String> {
        // Validate all criteria values are 0-5
        for (key, &val) in &criteria {
            if val > 5 {
                return Err(format!("Criteria '{}' must be 0-5, got {}", key, val));
            }
        }

        let global = if criteria.is_empty() {
            0.0
        } else {
            criteria.values().map(|&v| v as f64).sum::<f64>() / criteria.len() as f64
        };

        Ok(Self {
            id: Uuid::new_v4(),
            organization_id,
            service_provider_id,
            quote_id: None,
            ticket_id: None,
            evaluator_id,
            building_id,
            criteria,
            global_score: global,
            comments: None,
            would_recommend,
            is_legal_evaluation: false,
            is_anonymous: false,
            created_at: Utc::now(),
        })
    }

    /// Link evaluation to quote (optional)
    pub fn link_quote(&mut self, quote_id: Uuid) -> Result<(), String> {
        self.quote_id = Some(quote_id);
        Ok(())
    }

    /// Link evaluation to ticket (optional)
    pub fn link_ticket(&mut self, ticket_id: Uuid) -> Result<(), String> {
        self.ticket_id = Some(ticket_id);
        Ok(())
    }

    /// Mark as legal evaluation (L13 annual report)
    pub fn mark_as_legal_evaluation(&mut self) -> Result<(), String> {
        self.is_legal_evaluation = true;
        Ok(())
    }

    /// Mark as anonymous (GDPR compliant)
    pub fn mark_as_anonymous(&mut self) -> Result<(), String> {
        self.is_anonymous = true;
        Ok(())
    }

    /// Recalculate global score based on updated criteria
    pub fn recalculate_global_score(&mut self) -> Result<(), String> {
        self.global_score = if self.criteria.is_empty() {
            0.0
        } else {
            self.criteria.values().map(|&v| v as f64).sum::<f64>() / self.criteria.len() as f64
        };
        Ok(())
    }

    /// Set comments on evaluation
    pub fn set_comments(&mut self, comments: String) -> Result<(), String> {
        if comments.is_empty() {
            return Err("Comments cannot be empty".to_string());
        }
        self.comments = Some(comments);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_evaluation_new_success() {
        let org_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let evaluator_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut criteria = HashMap::new();
        criteria.insert("qualite".to_string(), 5);
        criteria.insert("delai".to_string(), 4);
        criteria.insert("prix".to_string(), 3);

        let eval = ContractEvaluation::new(
            org_id,
            provider_id,
            evaluator_id,
            building_id,
            criteria,
            true,
        );

        assert!(eval.is_ok());
        let e = eval.unwrap();
        assert_eq!(e.criteria.len(), 3);
        assert!(e.global_score > 0.0);
        assert!(e.would_recommend);
    }

    #[test]
    fn test_contract_evaluation_invalid_criteria() {
        let org_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let evaluator_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut criteria = HashMap::new();
        criteria.insert("qualite".to_string(), 6); // Invalid: > 5

        let result = ContractEvaluation::new(
            org_id,
            provider_id,
            evaluator_id,
            building_id,
            criteria,
            true,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_global_score_calculation() {
        let org_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let evaluator_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let mut criteria = HashMap::new();
        criteria.insert("qualite".to_string(), 5);
        criteria.insert("delai".to_string(), 3);

        let eval = ContractEvaluation::new(
            org_id,
            provider_id,
            evaluator_id,
            building_id,
            criteria,
            true,
        )
        .unwrap();

        // (5 + 3) / 2 = 4.0
        assert_eq!(eval.global_score, 4.0);
    }

    #[test]
    fn test_mark_legal_evaluation() {
        let org_id = Uuid::new_v4();
        let provider_id = Uuid::new_v4();
        let evaluator_id = Uuid::new_v4();
        let building_id = Uuid::new_v4();

        let criteria = HashMap::new();
        let mut eval =
            ContractEvaluation::new(org_id, provider_id, evaluator_id, building_id, criteria, true)
                .unwrap();

        assert!(!eval.is_legal_evaluation);
        let _ = eval.mark_as_legal_evaluation();
        assert!(eval.is_legal_evaluation);
    }
}
