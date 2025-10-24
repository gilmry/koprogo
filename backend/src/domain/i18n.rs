use serde::{Deserialize, Serialize};

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    NL, // Nederlands (Dutch) - 60% Belgium
    FR, // Français (French) - 40% Belgium
    DE, // Deutsch (German) - <1% Belgium
    EN, // English - International
}

impl Language {
    /// Parse language from Accept-Language header or language code
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "nl" => Some(Language::NL),
            "fr" => Some(Language::FR),
            "de" => Some(Language::DE),
            "en" => Some(Language::EN),
            _ => None,
        }
    }

    /// Get language code (e.g., "nl", "fr", "de", "en")
    pub fn code(&self) -> &'static str {
        match self {
            Language::NL => "nl",
            Language::FR => "fr",
            Language::DE => "de",
            Language::EN => "en",
        }
    }
}

/// Translation keys for error messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationKey {
    // Building errors
    BuildingNameEmpty,
    TotalUnitsMustBePositive,

    // Expense errors
    DescriptionEmpty,
    AmountMustBePositive,

    // Owner errors
    FirstNameEmpty,
    LastNameEmpty,
    InvalidEmailFormat,

    // General errors
    NotFound,
    Unauthorized,
    InternalError,
}

/// Translation service
pub struct I18n;

impl I18n {
    /// Get translated message for a key and language
    pub fn translate(key: TranslationKey, lang: Language) -> String {
        use Language::*;
        use TranslationKey::*;

        match (key, lang) {
            // Building errors
            (BuildingNameEmpty, NL) => "Gebouwnaam mag niet leeg zijn".to_string(),
            (BuildingNameEmpty, FR) => "Le nom du bâtiment ne peut pas être vide".to_string(),
            (BuildingNameEmpty, DE) => "Gebäudename darf nicht leer sein".to_string(),
            (BuildingNameEmpty, EN) => "Building name cannot be empty".to_string(),

            (TotalUnitsMustBePositive, NL) => {
                "Totaal aantal eenheden moet groter zijn dan 0".to_string()
            }
            (TotalUnitsMustBePositive, FR) => {
                "Le nombre total d'unités doit être supérieur à 0".to_string()
            }
            (TotalUnitsMustBePositive, DE) => {
                "Gesamtzahl der Einheiten muss größer als 0 sein".to_string()
            }
            (TotalUnitsMustBePositive, EN) => "Total units must be greater than 0".to_string(),

            // Expense errors
            (DescriptionEmpty, NL) => "Beschrijving mag niet leeg zijn".to_string(),
            (DescriptionEmpty, FR) => "La description ne peut pas être vide".to_string(),
            (DescriptionEmpty, DE) => "Beschreibung darf nicht leer sein".to_string(),
            (DescriptionEmpty, EN) => "Description cannot be empty".to_string(),

            (AmountMustBePositive, NL) => "Bedrag moet groter zijn dan 0".to_string(),
            (AmountMustBePositive, FR) => "Le montant doit être supérieur à 0".to_string(),
            (AmountMustBePositive, DE) => "Betrag muss größer als 0 sein".to_string(),
            (AmountMustBePositive, EN) => "Amount must be greater than 0".to_string(),

            // Owner errors
            (FirstNameEmpty, NL) => "Voornaam mag niet leeg zijn".to_string(),
            (FirstNameEmpty, FR) => "Le prénom ne peut pas être vide".to_string(),
            (FirstNameEmpty, DE) => "Vorname darf nicht leer sein".to_string(),
            (FirstNameEmpty, EN) => "First name cannot be empty".to_string(),

            (LastNameEmpty, NL) => "Achternaam mag niet leeg zijn".to_string(),
            (LastNameEmpty, FR) => "Le nom de famille ne peut pas être vide".to_string(),
            (LastNameEmpty, DE) => "Nachname darf nicht leer sein".to_string(),
            (LastNameEmpty, EN) => "Last name cannot be empty".to_string(),

            (InvalidEmailFormat, NL) => "Ongeldig e-mailformaat".to_string(),
            (InvalidEmailFormat, FR) => "Format d'e-mail invalide".to_string(),
            (InvalidEmailFormat, DE) => "Ungültiges E-Mail-Format".to_string(),
            (InvalidEmailFormat, EN) => "Invalid email format".to_string(),

            // General errors
            (NotFound, NL) => "Niet gevonden".to_string(),
            (NotFound, FR) => "Non trouvé".to_string(),
            (NotFound, DE) => "Nicht gefunden".to_string(),
            (NotFound, EN) => "Not found".to_string(),

            (Unauthorized, NL) => "Niet geautoriseerd".to_string(),
            (Unauthorized, FR) => "Non autorisé".to_string(),
            (Unauthorized, DE) => "Nicht autorisiert".to_string(),
            (Unauthorized, EN) => "Unauthorized".to_string(),

            (InternalError, NL) => "Interne serverfout".to_string(),
            (InternalError, FR) => "Erreur interne du serveur".to_string(),
            (InternalError, DE) => "Interner Serverfehler".to_string(),
            (InternalError, EN) => "Internal server error".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Language Code Tests =====

    #[test]
    fn test_language_from_code_nl() {
        assert_eq!(Language::from_code("nl"), Some(Language::NL));
        assert_eq!(Language::from_code("NL"), Some(Language::NL));
    }

    #[test]
    fn test_language_from_code_fr() {
        assert_eq!(Language::from_code("fr"), Some(Language::FR));
        assert_eq!(Language::from_code("FR"), Some(Language::FR));
    }

    #[test]
    fn test_language_from_code_de() {
        assert_eq!(Language::from_code("de"), Some(Language::DE));
        assert_eq!(Language::from_code("DE"), Some(Language::DE));
    }

    #[test]
    fn test_language_from_code_en() {
        assert_eq!(Language::from_code("en"), Some(Language::EN));
        assert_eq!(Language::from_code("EN"), Some(Language::EN));
    }

    #[test]
    fn test_language_from_code_invalid() {
        assert_eq!(Language::from_code("es"), None);
        assert_eq!(Language::from_code("invalid"), None);
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::NL.code(), "nl");
        assert_eq!(Language::FR.code(), "fr");
        assert_eq!(Language::DE.code(), "de");
        assert_eq!(Language::EN.code(), "en");
    }

    #[test]
    fn test_language_default_is_dutch() {
        assert_eq!(Language::default(), Language::NL);
    }

    // ===== Translation Tests =====

    #[test]
    fn test_translate_building_name_empty_nl() {
        let result = I18n::translate(TranslationKey::BuildingNameEmpty, Language::NL);
        assert_eq!(result, "Gebouwnaam mag niet leeg zijn");
    }

    #[test]
    fn test_translate_building_name_empty_fr() {
        let result = I18n::translate(TranslationKey::BuildingNameEmpty, Language::FR);
        assert_eq!(result, "Le nom du bâtiment ne peut pas être vide");
    }

    #[test]
    fn test_translate_building_name_empty_de() {
        let result = I18n::translate(TranslationKey::BuildingNameEmpty, Language::DE);
        assert_eq!(result, "Gebäudename darf nicht leer sein");
    }

    #[test]
    fn test_translate_building_name_empty_en() {
        let result = I18n::translate(TranslationKey::BuildingNameEmpty, Language::EN);
        assert_eq!(result, "Building name cannot be empty");
    }

    #[test]
    fn test_translate_total_units_positive_nl() {
        let result = I18n::translate(TranslationKey::TotalUnitsMustBePositive, Language::NL);
        assert_eq!(result, "Totaal aantal eenheden moet groter zijn dan 0");
    }

    #[test]
    fn test_translate_total_units_positive_fr() {
        let result = I18n::translate(TranslationKey::TotalUnitsMustBePositive, Language::FR);
        assert_eq!(result, "Le nombre total d'unités doit être supérieur à 0");
    }

    #[test]
    fn test_translate_description_empty_nl() {
        let result = I18n::translate(TranslationKey::DescriptionEmpty, Language::NL);
        assert_eq!(result, "Beschrijving mag niet leeg zijn");
    }

    #[test]
    fn test_translate_description_empty_fr() {
        let result = I18n::translate(TranslationKey::DescriptionEmpty, Language::FR);
        assert_eq!(result, "La description ne peut pas être vide");
    }

    #[test]
    fn test_translate_amount_positive_nl() {
        let result = I18n::translate(TranslationKey::AmountMustBePositive, Language::NL);
        assert_eq!(result, "Bedrag moet groter zijn dan 0");
    }

    #[test]
    fn test_translate_amount_positive_fr() {
        let result = I18n::translate(TranslationKey::AmountMustBePositive, Language::FR);
        assert_eq!(result, "Le montant doit être supérieur à 0");
    }

    #[test]
    fn test_translate_first_name_empty_nl() {
        let result = I18n::translate(TranslationKey::FirstNameEmpty, Language::NL);
        assert_eq!(result, "Voornaam mag niet leeg zijn");
    }

    #[test]
    fn test_translate_first_name_empty_fr() {
        let result = I18n::translate(TranslationKey::FirstNameEmpty, Language::FR);
        assert_eq!(result, "Le prénom ne peut pas être vide");
    }

    #[test]
    fn test_translate_last_name_empty_de() {
        let result = I18n::translate(TranslationKey::LastNameEmpty, Language::DE);
        assert_eq!(result, "Nachname darf nicht leer sein");
    }

    #[test]
    fn test_translate_last_name_empty_en() {
        let result = I18n::translate(TranslationKey::LastNameEmpty, Language::EN);
        assert_eq!(result, "Last name cannot be empty");
    }

    #[test]
    fn test_translate_invalid_email_nl() {
        let result = I18n::translate(TranslationKey::InvalidEmailFormat, Language::NL);
        assert_eq!(result, "Ongeldig e-mailformaat");
    }

    #[test]
    fn test_translate_invalid_email_fr() {
        let result = I18n::translate(TranslationKey::InvalidEmailFormat, Language::FR);
        assert_eq!(result, "Format d'e-mail invalide");
    }

    #[test]
    fn test_translate_invalid_email_de() {
        let result = I18n::translate(TranslationKey::InvalidEmailFormat, Language::DE);
        assert_eq!(result, "Ungültiges E-Mail-Format");
    }

    #[test]
    fn test_translate_invalid_email_en() {
        let result = I18n::translate(TranslationKey::InvalidEmailFormat, Language::EN);
        assert_eq!(result, "Invalid email format");
    }
}
