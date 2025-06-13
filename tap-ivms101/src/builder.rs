//! Builder patterns for IVMS 101 data structures
//!
//! This module provides builder patterns to simplify the construction
//! of IVMS 101 data structures with proper validation.

use crate::error::{Error, Result};
use crate::message::*;
use crate::person::*;
use crate::types::*;
use crate::validation::*;

/// Builder for natural person names
pub struct NaturalPersonNameBuilder {
    name_identifiers: Vec<NameIdentifier>,
    local_name_identifiers: Option<Vec<LocalNameIdentifier>>,
    phonetic_name_identifiers: Option<Vec<PhoneticNameIdentifier>>,
}

impl Default for NaturalPersonNameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NaturalPersonNameBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name_identifiers: Vec::new(),
            local_name_identifiers: None,
            phonetic_name_identifiers: None,
        }
    }

    /// Add a name identifier
    pub fn add_name_identifier(
        mut self,
        primary: impl Into<String>,
        secondary: impl Into<String>,
        name_type: NameIdentifierType,
    ) -> Self {
        self.name_identifiers
            .push(NameIdentifier::new(primary, secondary, name_type));
        self
    }

    /// Add a legal name
    pub fn legal_name(self, family_name: impl Into<String>, given_name: impl Into<String>) -> Self {
        self.add_name_identifier(family_name, given_name, NameIdentifierType::LegalName)
    }

    /// Build the natural person name
    pub fn build(self) -> Result<NaturalPersonName> {
        let name = NaturalPersonName {
            name_identifiers: self.name_identifiers,
            local_name_identifiers: self.local_name_identifiers,
            phonetic_name_identifiers: self.phonetic_name_identifiers,
        };
        name.validate()?;
        Ok(name)
    }
}

/// Builder for natural persons
pub struct NaturalPersonBuilder {
    name: Option<NaturalPersonName>,
    geographic_addresses: Option<Vec<GeographicAddress>>,
    national_identification: Option<NationalIdentification>,
    customer_identification: Option<CustomerIdentification>,
    date_and_place_of_birth: Option<DateAndPlaceOfBirth>,
    country_of_residence: Option<CountryCode>,
}

impl Default for NaturalPersonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NaturalPersonBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            geographic_addresses: None,
            national_identification: None,
            customer_identification: None,
            date_and_place_of_birth: None,
            country_of_residence: None,
        }
    }

    /// Set the name
    pub fn name(mut self, name: NaturalPersonName) -> Self {
        self.name = Some(name);
        self
    }

    /// Add a geographic address
    pub fn add_address(mut self, address: GeographicAddress) -> Self {
        self.geographic_addresses
            .get_or_insert_with(Vec::new)
            .push(address);
        self
    }

    /// Set national identification
    pub fn national_id(
        mut self,
        identifier: impl Into<String>,
        id_type: NationalIdentifierType,
        country_of_issue: impl Into<String>,
    ) -> Self {
        self.national_identification = Some(NationalIdentification {
            national_identifier: identifier.into(),
            national_identifier_type: id_type,
            country_of_issue: country_of_issue.into(),
            registration_authority: None,
        });
        self
    }

    /// Set customer identification
    pub fn customer_id(
        mut self,
        identifier: impl Into<String>,
        id_type: CustomerIdentificationType,
    ) -> Self {
        self.customer_identification = Some(CustomerIdentification {
            customer_identifier: identifier.into(),
            customer_identification_type: id_type,
        });
        self
    }

    /// Set date and place of birth
    pub fn birth_info(
        mut self,
        date_of_birth: impl Into<String>,
        city_of_birth: impl Into<String>,
        country_of_birth: impl Into<String>,
    ) -> Self {
        self.date_and_place_of_birth = Some(DateAndPlaceOfBirth {
            date_of_birth: date_of_birth.into(),
            city_of_birth: city_of_birth.into(),
            country_of_birth: country_of_birth.into(),
        });
        self
    }

    /// Set country of residence
    pub fn country_of_residence(mut self, country: impl Into<String>) -> Self {
        self.country_of_residence = Some(country.into());
        self
    }

    /// Build the natural person
    pub fn build(self) -> Result<NaturalPerson> {
        let name = self.name.ok_or_else(|| {
            Error::MissingRequiredField("Natural person name is required".to_string())
        })?;

        let person = NaturalPerson {
            name,
            geographic_addresses: self.geographic_addresses,
            national_identification: self.national_identification,
            customer_identification: self.customer_identification,
            date_and_place_of_birth: self.date_and_place_of_birth,
            country_of_residence: self.country_of_residence,
        };

        person.validate()?;
        Ok(person)
    }
}

/// Builder for legal person names
pub struct LegalPersonNameBuilder {
    name_identifiers: Vec<LegalPersonNameIdentifier>,
    local_name_identifiers: Option<Vec<LegalPersonNameIdentifier>>,
    phonetic_name_identifiers: Option<Vec<LegalPersonNameIdentifier>>,
}

impl Default for LegalPersonNameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalPersonNameBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name_identifiers: Vec::new(),
            local_name_identifiers: None,
            phonetic_name_identifiers: None,
        }
    }

    /// Add a name identifier
    pub fn add_name_identifier(
        mut self,
        name: impl Into<String>,
        name_type: LegalPersonNameIdentifierType,
    ) -> Self {
        self.name_identifiers
            .push(LegalPersonNameIdentifier::new(name, name_type));
        self
    }

    /// Add a legal name
    pub fn legal_name(self, name: impl Into<String>) -> Self {
        self.add_name_identifier(name, LegalPersonNameIdentifierType::LegalName)
    }

    /// Add a trading name
    pub fn trading_name(self, name: impl Into<String>) -> Self {
        self.add_name_identifier(name, LegalPersonNameIdentifierType::TradingName)
    }

    /// Build the legal person name
    pub fn build(self) -> Result<LegalPersonName> {
        let name = LegalPersonName {
            name_identifiers: self.name_identifiers,
            local_name_identifiers: self.local_name_identifiers,
            phonetic_name_identifiers: self.phonetic_name_identifiers,
        };
        name.validate()?;
        Ok(name)
    }
}

/// Builder for legal persons
pub struct LegalPersonBuilder {
    name: Option<LegalPersonName>,
    geographic_addresses: Option<Vec<GeographicAddress>>,
    national_identification: Option<LegalPersonNationalIdentification>,
    customer_identification: Option<CustomerIdentification>,
    country_of_registration: Option<CountryCode>,
}

impl Default for LegalPersonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LegalPersonBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            geographic_addresses: None,
            national_identification: None,
            customer_identification: None,
            country_of_registration: None,
        }
    }

    /// Set the name
    pub fn name(mut self, name: LegalPersonName) -> Self {
        self.name = Some(name);
        self
    }

    /// Add a geographic address
    pub fn add_address(mut self, address: GeographicAddress) -> Self {
        self.geographic_addresses
            .get_or_insert_with(Vec::new)
            .push(address);
        self
    }

    /// Set national identification with LEI
    pub fn lei(mut self, lei_code: impl Into<String>) -> Result<Self> {
        let lei = lei_code.into();
        validate_lei(&lei)?;
        self.national_identification = Some(LegalPersonNationalIdentification {
            national_identifier: lei.clone(),
            national_identifier_type: Some("LEI".to_string()),
            country_of_issue: None,
            registration_authority: None,
            lei_code: Some(lei),
        });
        Ok(self)
    }

    /// Set customer identification
    pub fn customer_id(
        mut self,
        identifier: impl Into<String>,
        id_type: CustomerIdentificationType,
    ) -> Self {
        self.customer_identification = Some(CustomerIdentification {
            customer_identifier: identifier.into(),
            customer_identification_type: id_type,
        });
        self
    }

    /// Set country of registration
    pub fn country_of_registration(mut self, country: impl Into<String>) -> Self {
        self.country_of_registration = Some(country.into());
        self
    }

    /// Build the legal person
    pub fn build(self) -> Result<LegalPerson> {
        let name = self.name.ok_or_else(|| {
            Error::MissingRequiredField("Legal person name is required".to_string())
        })?;

        let person = LegalPerson {
            name,
            geographic_addresses: self.geographic_addresses,
            national_identification: self.national_identification,
            customer_identification: self.customer_identification,
            country_of_registration: self.country_of_registration,
        };

        person.validate()?;
        Ok(person)
    }
}

/// Builder for geographic addresses
pub struct GeographicAddressBuilder {
    address_type: Option<AddressType>,
    department: Option<String>,
    sub_department: Option<String>,
    street_name: Option<String>,
    building_number: Option<String>,
    building_name: Option<String>,
    floor: Option<String>,
    post_box: Option<String>,
    room: Option<String>,
    post_code: Option<String>,
    town_name: Option<String>,
    town_location_name: Option<String>,
    district_name: Option<String>,
    country_sub_division: Option<String>,
    address_line: Option<Vec<String>>,
    country: Option<CountryCode>,
}

impl Default for GeographicAddressBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GeographicAddressBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            address_type: None,
            department: None,
            sub_department: None,
            street_name: None,
            building_number: None,
            building_name: None,
            floor: None,
            post_box: None,
            room: None,
            post_code: None,
            town_name: None,
            town_location_name: None,
            district_name: None,
            country_sub_division: None,
            address_line: None,
            country: None,
        }
    }

    /// Set the address type
    pub fn address_type(mut self, address_type: AddressType) -> Self {
        self.address_type = Some(address_type);
        self
    }

    /// Set the street name
    pub fn street_name(mut self, street: impl Into<String>) -> Self {
        self.street_name = Some(street.into());
        self
    }

    /// Set the building number
    pub fn building_number(mut self, number: impl Into<String>) -> Self {
        self.building_number = Some(number.into());
        self
    }

    /// Set the post code
    pub fn post_code(mut self, code: impl Into<String>) -> Self {
        self.post_code = Some(code.into());
        self
    }

    /// Set the town name
    pub fn town_name(mut self, town: impl Into<String>) -> Self {
        self.town_name = Some(town.into());
        self
    }

    /// Set the building name
    pub fn building_name(mut self, name: impl Into<String>) -> Self {
        self.building_name = Some(name.into());
        self
    }

    /// Set the floor
    pub fn floor(mut self, floor: impl Into<String>) -> Self {
        self.floor = Some(floor.into());
        self
    }

    /// Set the country subdivision
    pub fn country_sub_division(mut self, subdivision: impl Into<String>) -> Self {
        self.country_sub_division = Some(subdivision.into());
        self
    }

    /// Set the country
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Build the geographic address
    pub fn build(self) -> Result<GeographicAddress> {
        let street_name = self
            .street_name
            .ok_or_else(|| Error::MissingRequiredField("Street name is required".to_string()))?;

        let post_code = self
            .post_code
            .ok_or_else(|| Error::MissingRequiredField("Post code is required".to_string()))?;

        let town_name = self
            .town_name
            .ok_or_else(|| Error::MissingRequiredField("Town name is required".to_string()))?;

        let country = self
            .country
            .ok_or_else(|| Error::MissingRequiredField("Country is required".to_string()))?;

        validate_country_code(&country)?;

        Ok(GeographicAddress {
            address_type: self.address_type,
            department: self.department,
            sub_department: self.sub_department,
            street_name,
            building_number: self.building_number,
            building_name: self.building_name,
            floor: self.floor,
            post_box: self.post_box,
            room: self.room,
            post_code,
            town_name,
            town_location_name: self.town_location_name,
            district_name: self.district_name,
            country_sub_division: self.country_sub_division,
            address_line: self.address_line,
            country,
        })
    }
}

/// Builder for IVMS messages
pub struct IvmsMessageBuilder {
    originator: Option<Originator>,
    beneficiary: Option<Beneficiary>,
    originating_vasp: Option<OriginatingVasp>,
    beneficiary_vasp: Option<BeneficiaryVasp>,
    transaction: Option<TransactionData>,
}

impl Default for IvmsMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl IvmsMessageBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            originator: None,
            beneficiary: None,
            originating_vasp: None,
            beneficiary_vasp: None,
            transaction: None,
        }
    }

    /// Set the originator
    pub fn originator(mut self, persons: Vec<Person>) -> Self {
        self.originator = Some(Originator::new(persons));
        self
    }

    /// Set the beneficiary
    pub fn beneficiary(mut self, persons: Vec<Person>) -> Self {
        self.beneficiary = Some(Beneficiary::new(persons));
        self
    }

    /// Set the originating VASP
    pub fn originating_vasp(mut self, vasp: Person) -> Self {
        self.originating_vasp = Some(OriginatingVasp::new(vasp));
        self
    }

    /// Set the beneficiary VASP
    pub fn beneficiary_vasp(mut self, vasp: Person) -> Self {
        self.beneficiary_vasp = Some(BeneficiaryVasp::new(vasp));
        self
    }

    /// Set the transaction data
    pub fn transaction(
        mut self,
        amount: impl Into<String>,
        currency: impl Into<String>,
        direction: TransactionDirection,
        identifier: impl Into<String>,
        datetime: impl Into<String>,
    ) -> Result<Self> {
        let currency_code = currency.into();
        validate_currency_code(&currency_code)?;

        self.transaction = Some(TransactionData {
            amount: amount.into(),
            currency: currency_code,
            direction,
            payment_type: None,
            transaction_identifier: identifier.into(),
            transaction_datetime: datetime.into(),
            transaction_network: None,
            transaction_hash: None,
        });
        Ok(self)
    }

    /// Build the IVMS message
    pub fn build(self) -> Result<IvmsMessage> {
        let originator = self
            .originator
            .ok_or_else(|| Error::MissingRequiredField("Originator is required".to_string()))?;

        let beneficiary = self
            .beneficiary
            .ok_or_else(|| Error::MissingRequiredField("Beneficiary is required".to_string()))?;

        let originating_vasp = self.originating_vasp.ok_or_else(|| {
            Error::MissingRequiredField("Originating VASP is required".to_string())
        })?;

        let transaction = self.transaction.ok_or_else(|| {
            Error::MissingRequiredField("Transaction data is required".to_string())
        })?;

        let message = IvmsMessage {
            originator,
            beneficiary,
            originating_vasp,
            beneficiary_vasp: self.beneficiary_vasp,
            transaction,
        };

        message.validate()?;
        Ok(message)
    }
}
