//! Complete example demonstrating IVMS 101 functionality

use tap_ivms101::builder::*;
use tap_ivms101::message::*;
use tap_ivms101::types::*;
use tap_ivms101::{LegalPerson, NaturalPerson};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an originator (natural person sending funds)
    let originator = create_originator()?;

    // Create a beneficiary (natural person receiving funds)
    let beneficiary = create_beneficiary()?;

    // Create the originating VASP
    let originating_vasp = create_originating_vasp()?;

    // Create the beneficiary VASP
    let beneficiary_vasp = create_beneficiary_vasp()?;

    // Create the IVMS message
    let mut message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(originator)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary)])
        .originating_vasp(Person::LegalPerson(originating_vasp))
        .transaction(
            "50000.00",
            "USD",
            TransactionDirection::Outgoing,
            "TRX-2024-001234",
            "2024-01-15T14:30:00Z",
        )?
        .build()?;

    // Add optional beneficiary VASP
    message.beneficiary_vasp = Some(BeneficiaryVasp::new(Person::LegalPerson(beneficiary_vasp)));

    // Add optional transaction details
    message.transaction.payment_type = Some(PaymentType::InvestmentCapital);
    message.transaction.transaction_network = Some(TransactionNetworkType::Bitcoin);
    message.transaction.transaction_hash =
        Some("0000000000000000000123456789abcdef0123456789abcdef0123456789ab".to_string());

    // Validate the complete message
    message.validate()?;

    // Serialize to JSON
    let json = message.to_json_pretty()?;
    println!("IVMS 101 Message (JSON):");
    println!("{}", json);

    // Demonstrate deserialization
    let parsed = IvmsMessage::from_json(&json)?;
    println!("\nSuccessfully parsed IVMS message");
    println!(
        "Transaction amount: {} {}",
        parsed.transaction.amount, parsed.transaction.currency
    );
    println!(
        "Transaction ID: {}",
        parsed.transaction.transaction_identifier
    );

    Ok(())
}

fn create_originator() -> Result<NaturalPerson, Box<dyn std::error::Error>> {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Anderson", "Alice Marie")
        .add_name_identifier("Anderson", "Allie", NameIdentifierType::ShortName)
        .build()?;

    let address = GeographicAddressBuilder::new()
        .address_type(AddressType::Home)
        .street_name("Main Street")
        .building_number("123")
        .post_code("10001")
        .town_name("New York")
        .country("US")
        .build()?;

    let person = NaturalPersonBuilder::new()
        .name(name)
        .add_address(address)
        .national_id(
            "123-45-6789",
            NationalIdentifierType::SocialSecurityNumber,
            "US",
        )
        .customer_id(
            "CUST-2024-001",
            CustomerIdentificationType::CustomerIdentificationNumber,
        )
        .birth_info("1985-03-15", "New York", "US")
        .country_of_residence("US")
        .build()?;

    Ok(person)
}

fn create_beneficiary() -> Result<NaturalPerson, Box<dyn std::error::Error>> {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Brown", "Robert James")
        .build()?;

    let address = GeographicAddressBuilder::new()
        .address_type(AddressType::Home)
        .street_name("High Street")
        .building_number("456")
        .post_code("SW1A 1AA")
        .town_name("London")
        .country("GB")
        .build()?;

    let person = NaturalPersonBuilder::new()
        .name(name)
        .add_address(address)
        .national_id(
            "AB123456C",
            NationalIdentifierType::NationalIdentityNumber,
            "GB",
        )
        .country_of_residence("GB")
        .build()?;

    Ok(person)
}

fn create_originating_vasp() -> Result<LegalPerson, Box<dyn std::error::Error>> {
    let name = LegalPersonNameBuilder::new()
        .legal_name("Crypto Exchange USA Inc.")
        .trading_name("CryptoUSA")
        .add_name_identifier("CUSA", LegalPersonNameIdentifierType::ShortName)
        .build()?;

    let address = GeographicAddressBuilder::new()
        .address_type(AddressType::Business)
        .street_name("Wall Street")
        .building_number("100")
        .building_name("Financial Tower")
        .floor("42")
        .post_code("10005")
        .town_name("New York")
        .country_sub_division("NY")
        .country("US")
        .build()?;

    let vasp = LegalPersonBuilder::new()
        .name(name)
        .add_address(address)
        .lei("529900HNOAA1KXQJUQ27")?
        .customer_id(
            "VASP-US-001",
            CustomerIdentificationType::CustomerIdentificationNumber,
        )
        .country_of_registration("US")
        .build()?;

    Ok(vasp)
}

fn create_beneficiary_vasp() -> Result<LegalPerson, Box<dyn std::error::Error>> {
    let name = LegalPersonNameBuilder::new()
        .legal_name("European Digital Assets Ltd.")
        .trading_name("EuroAssets")
        .build()?;

    let address = GeographicAddressBuilder::new()
        .address_type(AddressType::Business)
        .street_name("Canary Wharf")
        .building_number("1")
        .post_code("E14 5AB")
        .town_name("London")
        .country("GB")
        .build()?;

    let vasp = LegalPersonBuilder::new()
        .name(name)
        .add_address(address)
        .lei("213800HNOAA1KXQJUQ37")?
        .country_of_registration("GB")
        .build()?;

    Ok(vasp)
}
