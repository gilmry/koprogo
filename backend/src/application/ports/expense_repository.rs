use crate::application::dto::{ExpenseFilters, PageRequest};
use crate::domain::entities::Expense;
use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

#[async_trait]
pub trait ExpenseRepository: Send + Sync {
    async fn create(&self, expense: &Expense) -> Result<Expense, String>;

    /// Enregistre le détail d'une facture saisie ligne par ligne.
    ///
    /// Pas de méthode par défaut, et c'est délibéré : une implémentation par
    /// défaut qui ne ferait rien reproduirait exactement le défaut qu'on
    /// corrige — accepter la donnée et la perdre en silence. Chaque
    /// implémentation doit dire ce qu'elle en fait, quitte à dire qu'elle
    /// n'en fait rien parce qu'elle est un mock.
    async fn enregistrer_lignes_de_facture(
        &self,
        expense_id: Uuid,
        lignes: &[LigneDeFacture],
    ) -> Result<(), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Expense>, String>;
    async fn find_by_building(&self, building_id: Uuid) -> Result<Vec<Expense>, String>;

    /// Find all expenses with pagination and filters
    /// Returns tuple of (expenses, total_count)
    async fn find_all_paginated(
        &self,
        page_request: &PageRequest,
        filters: &ExpenseFilters,
    ) -> Result<(Vec<Expense>, i64), String>;

    async fn update(&self, expense: &Expense) -> Result<Expense, String>;
    async fn delete(&self, id: Uuid) -> Result<bool, String>;
}

/// Une ligne de facture, telle que le domaine la connaît.
///
/// Les montants dérivés — hors TVA, TVA, TTC — sont calculés à partir de la
/// quantité, du prix unitaire et du taux, jamais transmis par l'appelant :
/// un client qui enverrait un total incohérent avec ses composants ferait
/// mentir la facture.
#[derive(Debug, Clone, PartialEq)]
pub struct LigneDeFacture {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

impl LigneDeFacture {
    /// Le montant hors TVA : quantité × prix unitaire.
    pub fn montant_hors_tva(&self) -> Decimal {
        self.quantity * self.unit_price
    }

    /// La TVA due sur la ligne.
    ///
    /// Multiplication avant division, comme partout où un `Decimal` traverse
    /// une fraction : diviser d'abord laisse une traîne d'arrondi qui finit
    /// par déséquilibrer un total.
    pub fn montant_tva(&self) -> Decimal {
        self.montant_hors_tva() * self.vat_rate / Decimal::from(100)
    }

    /// Le montant TVA comprise.
    pub fn montant_tva_comprise(&self) -> Decimal {
        self.montant_hors_tva() + self.montant_tva()
    }
}
