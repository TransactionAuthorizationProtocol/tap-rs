//! Core IVMS 101.2023 data types
//!
//! This module implements the data types defined in the IVMS 101.2023 specification
//! for the interVASP Messaging Standard.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ISO 3166-1 alpha-2 country code
pub type CountryCode = String;

/// ISO 4217 currency code
pub type CurrencyCode = String;

/// Legal Entity Identifier (LEI) - 20 character alphanumeric code
pub type LeiCode = String;

/// Business Identifier Code (BIC) - 8 or 11 character code
pub type BicCode = String;

/// Name identifier type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NameIdentifierType {
    /// Legal name
    LegalName,
    /// Short name
    ShortName,
    /// Trading name
    TradingName,
    /// Other name type
    OtherName,
}

impl fmt::Display for NameIdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegalName => write!(f, "LEGAL_NAME"),
            Self::ShortName => write!(f, "SHORT_NAME"),
            Self::TradingName => write!(f, "TRADING_NAME"),
            Self::OtherName => write!(f, "OTHER_NAME"),
        }
    }
}

/// Legal person name identifier type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LegalPersonNameIdentifierType {
    /// Legal name
    LegalName,
    /// Short name
    ShortName,
    /// Trading name
    TradingName,
}

impl fmt::Display for LegalPersonNameIdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LegalName => write!(f, "LEGAL_NAME"),
            Self::ShortName => write!(f, "SHORT_NAME"),
            Self::TradingName => write!(f, "TRADING_NAME"),
        }
    }
}

/// Address type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AddressType {
    /// Home address
    Home,
    /// Business address
    Business,
    /// Geographic address
    Geographic,
}

impl fmt::Display for AddressType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Home => write!(f, "HOME"),
            Self::Business => write!(f, "BUSINESS"),
            Self::Geographic => write!(f, "GEOGRAPHIC"),
        }
    }
}

/// National identifier type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NationalIdentifierType {
    /// National identity number
    NationalIdentityNumber,
    /// Social security number
    SocialSecurityNumber,
    /// Tax identification number
    TaxIdentificationNumber,
    /// Alien registration number
    AlienRegistrationNumber,
    /// Passport number
    PassportNumber,
    /// Driver license number
    DriverLicenseNumber,
    /// Other identifier type
    OtherIdentifierType,
}

impl fmt::Display for NationalIdentifierType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NationalIdentityNumber => write!(f, "NATIONAL_IDENTITY_NUMBER"),
            Self::SocialSecurityNumber => write!(f, "SOCIAL_SECURITY_NUMBER"),
            Self::TaxIdentificationNumber => write!(f, "TAX_IDENTIFICATION_NUMBER"),
            Self::AlienRegistrationNumber => write!(f, "ALIEN_REGISTRATION_NUMBER"),
            Self::PassportNumber => write!(f, "PASSPORT_NUMBER"),
            Self::DriverLicenseNumber => write!(f, "DRIVER_LICENSE_NUMBER"),
            Self::OtherIdentifierType => write!(f, "OTHER_IDENTIFIER_TYPE"),
        }
    }
}

/// Customer identification type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerIdentificationType {
    /// Customer identification number
    CustomerIdentificationNumber,
    /// Unique transaction reference
    UniqueTransactionReference,
    /// Other customer ID type
    OtherCustomerIdType,
}

impl fmt::Display for CustomerIdentificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CustomerIdentificationNumber => write!(f, "CUSTOMER_IDENTIFICATION_NUMBER"),
            Self::UniqueTransactionReference => write!(f, "UNIQUE_TRANSACTION_REFERENCE"),
            Self::OtherCustomerIdType => write!(f, "OTHER_CUSTOMER_ID_TYPE"),
        }
    }
}

/// Registration authority type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegistrationAuthorityType {
    /// Registration authority entity identifier
    RaEntityId,
    /// Registration authority name
    RaName,
    /// Other registration authority
    OtherRegistrationAuthority,
}

impl fmt::Display for RegistrationAuthorityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RaEntityId => write!(f, "RA_ENTITY_ID"),
            Self::RaName => write!(f, "RA_NAME"),
            Self::OtherRegistrationAuthority => write!(f, "OTHER_REGISTRATION_AUTHORITY"),
        }
    }
}

/// Transaction direction enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionDirection {
    /// Outgoing transaction (from originator)
    Outgoing,
    /// Incoming transaction (to beneficiary)
    Incoming,
}

impl fmt::Display for TransactionDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outgoing => write!(f, "outgoing"),
            Self::Incoming => write!(f, "incoming"),
        }
    }
}

/// Payment type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentType {
    /// Annuity payment
    Annuity,
    /// Business expenses
    BusinessExpenses,
    /// Charity donation
    CharityDonation,
    /// Goods
    Goods,
    /// Hotel accommodation
    HotelAccommodation,
    /// Investment income
    InvestmentIncome,
    /// Investment capital
    InvestmentCapital,
    /// Lottery payout
    LotteryPayout,
    /// Other payment type
    Other,
    /// Pension payment
    Pension,
    /// Rental income
    RentalIncome,
    /// Royalties and fees
    RoyaltiesAndFees,
    /// Salary and wages
    SalaryAndWages,
    /// Services
    Services,
    /// Study costs
    StudyCosts,
    /// Travel and tourism
    TravelAndTourism,
}

impl fmt::Display for PaymentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Annuity => "ANNUITY",
            Self::BusinessExpenses => "BUSINESS_EXPENSES",
            Self::CharityDonation => "CHARITY_DONATION",
            Self::Goods => "GOODS",
            Self::HotelAccommodation => "HOTEL_ACCOMMODATION",
            Self::InvestmentIncome => "INVESTMENT_INCOME",
            Self::InvestmentCapital => "INVESTMENT_CAPITAL",
            Self::LotteryPayout => "LOTTERY_PAYOUT",
            Self::Other => "OTHER",
            Self::Pension => "PENSION",
            Self::RentalIncome => "RENTAL_INCOME",
            Self::RoyaltiesAndFees => "ROYALTIES_AND_FEES",
            Self::SalaryAndWages => "SALARY_AND_WAGES",
            Self::Services => "SERVICES",
            Self::StudyCosts => "STUDY_COSTS",
            Self::TravelAndTourism => "TRAVEL_AND_TOURISM",
        };
        write!(f, "{}", s)
    }
}

/// Transaction network type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionNetworkType {
    /// Bitcoin network
    Bitcoin,
    /// Ethereum network
    Ethereum,
    /// Litecoin network
    Litecoin,
    /// XRP Ledger
    XrpLedger,
    /// Stellar network
    Stellar,
    /// Other network
    Other,
}

impl fmt::Display for TransactionNetworkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Bitcoin => "BITCOIN",
            Self::Ethereum => "ETHEREUM",
            Self::Litecoin => "LITECOIN",
            Self::XrpLedger => "XRP_LEDGER",
            Self::Stellar => "STELLAR",
            Self::Other => "OTHER",
        };
        write!(f, "{}", s)
    }
}

/// Date and place of birth
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateAndPlaceOfBirth {
    /// Date of birth in YYYY-MM-DD format
    pub date_of_birth: String,
    /// City of birth
    pub city_of_birth: String,
    /// Country of birth (ISO 3166-1 alpha-2)
    pub country_of_birth: CountryCode,
}

/// Geographic address
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeographicAddress {
    /// Address type (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_type: Option<AddressType>,
    /// Department (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    /// Sub department (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_department: Option<String>,
    /// Street name
    pub street_name: String,
    /// Building number (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_number: Option<String>,
    /// Building name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub building_name: Option<String>,
    /// Floor (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor: Option<String>,
    /// Post box (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_box: Option<String>,
    /// Room (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
    /// Post code
    pub post_code: String,
    /// Town name
    pub town_name: String,
    /// Town location name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub town_location_name: Option<String>,
    /// District name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district_name: Option<String>,
    /// Country sub division (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_sub_division: Option<String>,
    /// Address line (optional, max 7 lines)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_line: Option<Vec<String>>,
    /// Country (ISO 3166-1 alpha-2)
    pub country: CountryCode,
}
