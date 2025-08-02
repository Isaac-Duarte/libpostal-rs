//! Common types and enums for libpostal-rs.

use std::fmt;

/// Language codes for address parsing and normalization.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Language {
    /// English
    English,
    /// Spanish
    Spanish,
    /// French
    French,
    /// German
    German,
    /// Italian
    Italian,
    /// Portuguese
    Portuguese,
    /// Russian
    Russian,
    /// Chinese (Simplified)
    ChineseSimplified,
    /// Chinese (Traditional)
    ChineseTraditional,
    /// Japanese
    Japanese,
    /// Korean
    Korean,
    /// Arabic
    Arabic,
    /// Hindi
    Hindi,
    /// Dutch
    Dutch,
    /// Polish
    Polish,
    /// Swedish
    Swedish,
    /// Norwegian
    Norwegian,
    /// Danish
    Danish,
    /// Finnish
    Finnish,
    /// Czech
    Czech,
    /// Hungarian
    Hungarian,
    /// Romanian
    Romanian,
    /// Turkish
    Turkish,
    /// Greek
    Greek,
    /// Hebrew
    Hebrew,
    /// Thai
    Thai,
    /// Vietnamese
    Vietnamese,
    /// Indonesian
    Indonesian,
    /// Malay
    Malay,
    /// Custom language code
    Custom(String),
}

impl Language {
    /// Convert to ISO 639-1 language code string.
    pub fn to_string(&self) -> String {
        match self {
            Language::English => "en".to_string(),
            Language::Spanish => "es".to_string(),
            Language::French => "fr".to_string(),
            Language::German => "de".to_string(),
            Language::Italian => "it".to_string(),
            Language::Portuguese => "pt".to_string(),
            Language::Russian => "ru".to_string(),
            Language::ChineseSimplified => "zh".to_string(),
            Language::ChineseTraditional => "zh-TW".to_string(),
            Language::Japanese => "ja".to_string(),
            Language::Korean => "ko".to_string(),
            Language::Arabic => "ar".to_string(),
            Language::Hindi => "hi".to_string(),
            Language::Dutch => "nl".to_string(),
            Language::Polish => "pl".to_string(),
            Language::Swedish => "sv".to_string(),
            Language::Norwegian => "no".to_string(),
            Language::Danish => "da".to_string(),
            Language::Finnish => "fi".to_string(),
            Language::Czech => "cs".to_string(),
            Language::Hungarian => "hu".to_string(),
            Language::Romanian => "ro".to_string(),
            Language::Turkish => "tr".to_string(),
            Language::Greek => "el".to_string(),
            Language::Hebrew => "he".to_string(),
            Language::Thai => "th".to_string(),
            Language::Vietnamese => "vi".to_string(),
            Language::Indonesian => "id".to_string(),
            Language::Malay => "ms".to_string(),
            Language::Custom(code) => code.clone(),
        }
    }

    /// Parse from ISO 639-1 language code string.
    pub fn from_str(code: &str) -> Self {
        match code {
            "en" => Language::English,
            "es" => Language::Spanish,
            "fr" => Language::French,
            "de" => Language::German,
            "it" => Language::Italian,
            "pt" => Language::Portuguese,
            "ru" => Language::Russian,
            "zh" => Language::ChineseSimplified,
            "zh-TW" => Language::ChineseTraditional,
            "ja" => Language::Japanese,
            "ko" => Language::Korean,
            "ar" => Language::Arabic,
            "hi" => Language::Hindi,
            "nl" => Language::Dutch,
            "pl" => Language::Polish,
            "sv" => Language::Swedish,
            "no" => Language::Norwegian,
            "da" => Language::Danish,
            "fi" => Language::Finnish,
            "cs" => Language::Czech,
            "hu" => Language::Hungarian,
            "ro" => Language::Romanian,
            "tr" => Language::Turkish,
            "el" => Language::Greek,
            "he" => Language::Hebrew,
            "th" => Language::Thai,
            "vi" => Language::Vietnamese,
            "id" => Language::Indonesian,
            "ms" => Language::Malay,
            _ => Language::Custom(code.to_string()),
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Country codes for address parsing hints.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Country {
    /// United States
    UnitedStates,
    /// Canada
    Canada,
    /// United Kingdom
    UnitedKingdom,
    /// Germany
    Germany,
    /// France
    France,
    /// Italy
    Italy,
    /// Spain
    Spain,
    /// Portugal
    Portugal,
    /// Netherlands
    Netherlands,
    /// Belgium
    Belgium,
    /// Switzerland
    Switzerland,
    /// Austria
    Austria,
    /// Sweden
    Sweden,
    /// Norway
    Norway,
    /// Denmark
    Denmark,
    /// Finland
    Finland,
    /// Poland
    Poland,
    /// Czech Republic
    CzechRepublic,
    /// Hungary
    Hungary,
    /// Romania
    Romania,
    /// Greece
    Greece,
    /// Turkey
    Turkey,
    /// Russia
    Russia,
    /// China
    China,
    /// Japan
    Japan,
    /// South Korea
    SouthKorea,
    /// India
    India,
    /// Australia
    Australia,
    /// New Zealand
    NewZealand,
    /// Brazil
    Brazil,
    /// Mexico
    Mexico,
    /// Argentina
    Argentina,
    /// Chile
    Chile,
    /// South Africa
    SouthAfrica,
    /// Israel
    Israel,
    /// Saudi Arabia
    SaudiArabia,
    /// United Arab Emirates
    UnitedArabEmirates,
    /// Thailand
    Thailand,
    /// Vietnam
    Vietnam,
    /// Indonesia
    Indonesia,
    /// Malaysia
    Malaysia,
    /// Singapore
    Singapore,
    /// Philippines
    Philippines,
    /// Custom country code
    Custom(String),
}

impl Country {
    /// Convert to ISO 3166-1 alpha-2 country code string.
    pub fn to_string(&self) -> String {
        match self {
            Country::UnitedStates => "US".to_string(),
            Country::Canada => "CA".to_string(),
            Country::UnitedKingdom => "GB".to_string(),
            Country::Germany => "DE".to_string(),
            Country::France => "FR".to_string(),
            Country::Italy => "IT".to_string(),
            Country::Spain => "ES".to_string(),
            Country::Portugal => "PT".to_string(),
            Country::Netherlands => "NL".to_string(),
            Country::Belgium => "BE".to_string(),
            Country::Switzerland => "CH".to_string(),
            Country::Austria => "AT".to_string(),
            Country::Sweden => "SE".to_string(),
            Country::Norway => "NO".to_string(),
            Country::Denmark => "DK".to_string(),
            Country::Finland => "FI".to_string(),
            Country::Poland => "PL".to_string(),
            Country::CzechRepublic => "CZ".to_string(),
            Country::Hungary => "HU".to_string(),
            Country::Romania => "RO".to_string(),
            Country::Greece => "GR".to_string(),
            Country::Turkey => "TR".to_string(),
            Country::Russia => "RU".to_string(),
            Country::China => "CN".to_string(),
            Country::Japan => "JP".to_string(),
            Country::SouthKorea => "KR".to_string(),
            Country::India => "IN".to_string(),
            Country::Australia => "AU".to_string(),
            Country::NewZealand => "NZ".to_string(),
            Country::Brazil => "BR".to_string(),
            Country::Mexico => "MX".to_string(),
            Country::Argentina => "AR".to_string(),
            Country::Chile => "CL".to_string(),
            Country::SouthAfrica => "ZA".to_string(),
            Country::Israel => "IL".to_string(),
            Country::SaudiArabia => "SA".to_string(),
            Country::UnitedArabEmirates => "AE".to_string(),
            Country::Thailand => "TH".to_string(),
            Country::Vietnam => "VN".to_string(),
            Country::Indonesia => "ID".to_string(),
            Country::Malaysia => "MY".to_string(),
            Country::Singapore => "SG".to_string(),
            Country::Philippines => "PH".to_string(),
            Country::Custom(code) => code.clone(),
        }
    }

    /// Parse from ISO 3166-1 alpha-2 country code string.
    pub fn from_str(code: &str) -> Self {
        match code.to_uppercase().as_str() {
            "US" => Country::UnitedStates,
            "CA" => Country::Canada,
            "GB" => Country::UnitedKingdom,
            "DE" => Country::Germany,
            "FR" => Country::France,
            "IT" => Country::Italy,
            "ES" => Country::Spain,
            "PT" => Country::Portugal,
            "NL" => Country::Netherlands,
            "BE" => Country::Belgium,
            "CH" => Country::Switzerland,
            "AT" => Country::Austria,
            "SE" => Country::Sweden,
            "NO" => Country::Norway,
            "DK" => Country::Denmark,
            "FI" => Country::Finland,
            "PL" => Country::Poland,
            "CZ" => Country::CzechRepublic,
            "HU" => Country::Hungary,
            "RO" => Country::Romania,
            "GR" => Country::Greece,
            "TR" => Country::Turkey,
            "RU" => Country::Russia,
            "CN" => Country::China,
            "JP" => Country::Japan,
            "KR" => Country::SouthKorea,
            "IN" => Country::India,
            "AU" => Country::Australia,
            "NZ" => Country::NewZealand,
            "BR" => Country::Brazil,
            "MX" => Country::Mexico,
            "AR" => Country::Argentina,
            "CL" => Country::Chile,
            "ZA" => Country::SouthAfrica,
            "IL" => Country::Israel,
            "SA" => Country::SaudiArabia,
            "AE" => Country::UnitedArabEmirates,
            "TH" => Country::Thailand,
            "VN" => Country::Vietnam,
            "ID" => Country::Indonesia,
            "MY" => Country::Malaysia,
            "SG" => Country::Singapore,
            "PH" => Country::Philippines,
            _ => Country::Custom(code.to_string()),
        }
    }
}

impl fmt::Display for Country {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Hints for address parsing.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddressHint {
    /// Language hint
    pub language: Option<Language>,
    /// Country hint
    pub country: Option<Country>,
}

impl AddressHint {
    /// Create a new address hint.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set language hint.
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Set country hint.
    pub fn with_country(mut self, country: Country) -> Self {
        self.country = Some(country);
        self
    }
}

/// Normalization levels for address processing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub enum NormalizationLevel {
    /// Light normalization (basic cleanup)
    Light,
    /// Medium normalization (standard processing)
    #[default]
    Medium,
    /// Aggressive normalization (maximum processing)
    Aggressive,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_conversion() {
        assert_eq!(Language::English.to_string(), "en");
        assert_eq!(Language::from_str("en"), Language::English);
        assert_eq!(
            Language::from_str("unknown"),
            Language::Custom("unknown".to_string())
        );
    }

    #[test]
    fn test_country_conversion() {
        assert_eq!(Country::UnitedStates.to_string(), "US");
        assert_eq!(Country::from_str("US"), Country::UnitedStates);
        assert_eq!(Country::from_str("us"), Country::UnitedStates);
        assert_eq!(Country::from_str("XY"), Country::Custom("XY".to_string()));
    }

    #[test]
    fn test_address_hint() {
        let hint = AddressHint::new()
            .with_language(Language::English)
            .with_country(Country::UnitedStates);

        assert_eq!(hint.language, Some(Language::English));
        assert_eq!(hint.country, Some(Country::UnitedStates));
    }
}
