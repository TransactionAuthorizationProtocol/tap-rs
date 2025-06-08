//! Tests for IVMS 101 JSON serialization format compliance

use serde_json::json;
use tap_ivms101::builder::*;
use tap_ivms101::message::*;
use tap_ivms101::types::*;

#[test]
fn test_ivms_message_json_structure() {
    // Create a complete IVMS message with all fields
    let originator_name = NaturalPersonNameBuilder::new()
        .legal_name("Smith", "John")
        .build()
        .unwrap();

    let originator_address = GeographicAddressBuilder::new()
        .address_type(AddressType::Home)
        .street_name("123 Main Street")
        .building_number("123")
        .post_code("10001")
        .town_name("New York")
        .country("US")
        .build()
        .unwrap();

    let originator = NaturalPersonBuilder::new()
        .name(originator_name)
        .add_address(originator_address)
        .national_id(
            "123456789",
            NationalIdentifierType::NationalIdentityNumber,
            "US",
        )
        .customer_id(
            "CUST-001",
            CustomerIdentificationType::CustomerIdentificationNumber,
        )
        .birth_info("1980-01-15", "New York", "US")
        .country_of_residence("US")
        .build()
        .unwrap();

    let beneficiary_name = NaturalPersonNameBuilder::new()
        .legal_name("Jones", "Alice")
        .build()
        .unwrap();

    let beneficiary = NaturalPersonBuilder::new()
        .name(beneficiary_name)
        .country_of_residence("GB")
        .build()
        .unwrap();

    let vasp_name = LegalPersonNameBuilder::new()
        .legal_name("Example Exchange Ltd.")
        .trading_name("Example Exchange")
        .build()
        .unwrap();

    let vasp_address = GeographicAddressBuilder::new()
        .address_type(AddressType::Business)
        .street_name("100 Financial District")
        .post_code("EC2N 4AY")
        .town_name("London")
        .country("GB")
        .build()
        .unwrap();

    let originating_vasp = LegalPersonBuilder::new()
        .name(vasp_name)
        .add_address(vasp_address)
        .lei("529900HNOAA1KXQJUQ27")
        .unwrap()
        .country_of_registration("GB")
        .build()
        .unwrap();

    let beneficiary_vasp_name = LegalPersonNameBuilder::new()
        .legal_name("Another Exchange Inc.")
        .build()
        .unwrap();

    let beneficiary_vasp = LegalPersonBuilder::new()
        .name(beneficiary_vasp_name)
        .country_of_registration("US")
        .build()
        .unwrap();

    let mut message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(originator)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary)])
        .originating_vasp(Person::LegalPerson(originating_vasp))
        .transaction(
            "10000.00",
            "USD",
            TransactionDirection::Outgoing,
            "TX-123456789",
            "2024-01-15T10:30:00Z",
        )
        .unwrap()
        .build()
        .unwrap();

    message.beneficiary_vasp = Some(BeneficiaryVasp::new(Person::LegalPerson(beneficiary_vasp)));
    message.transaction.payment_type = Some(PaymentType::InvestmentCapital);
    message.transaction.transaction_network = Some(TransactionNetworkType::Bitcoin);
    message.transaction.transaction_hash = Some("0x1234567890abcdef".to_string());

    // Convert to JSON
    let json = serde_json::to_value(&message).unwrap();

    // Verify the JSON structure matches IVMS 101 specification
    assert!(json.is_object());

    // Check originator structure
    assert!(json["originator"].is_object());
    assert!(json["originator"]["originatorPersons"].is_array());
    assert_eq!(
        json["originator"]["originatorPersons"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let originator_person = &json["originator"]["originatorPersons"][0];
    assert!(originator_person["naturalPerson"].is_object());
    assert!(originator_person["naturalPerson"]["name"]["nameIdentifiers"].is_array());
    assert_eq!(
        originator_person["naturalPerson"]["name"]["nameIdentifiers"][0]["primaryIdentifier"],
        "Smith"
    );
    assert_eq!(
        originator_person["naturalPerson"]["name"]["nameIdentifiers"][0]["secondaryIdentifier"],
        "John"
    );
    assert_eq!(
        originator_person["naturalPerson"]["name"]["nameIdentifiers"][0]["nameIdentifierType"],
        "LEGAL_NAME"
    );

    // Check beneficiary structure
    assert!(json["beneficiary"].is_object());
    assert!(json["beneficiary"]["beneficiaryPersons"].is_array());

    // Check originating VASP structure
    assert!(json["originatingVasp"].is_object());
    assert!(json["originatingVasp"]["originatingVasp"]["legalPerson"].is_object());

    // Check beneficiary VASP structure
    assert!(json["beneficiaryVasp"].is_object());
    assert!(json["beneficiaryVasp"]["beneficiaryVasp"]["legalPerson"].is_object());

    // Check transaction structure
    assert!(json["transaction"].is_object());
    assert_eq!(json["transaction"]["amount"], "10000.00");
    assert_eq!(json["transaction"]["currency"], "USD");
    assert_eq!(json["transaction"]["direction"], "outgoing");
    assert_eq!(json["transaction"]["paymentType"], "INVESTMENT_CAPITAL");
    assert_eq!(json["transaction"]["transactionIdentifier"], "TX-123456789");
    assert_eq!(
        json["transaction"]["transactionDatetime"],
        "2024-01-15T10:30:00Z"
    );
    assert_eq!(json["transaction"]["transactionNetwork"], "BITCOIN");
    assert_eq!(json["transaction"]["transactionHash"], "0x1234567890abcdef");
}

#[test]
fn test_minimal_ivms_message() {
    // Test minimal required fields
    let originator_name = NaturalPersonNameBuilder::new()
        .legal_name("Doe", "John")
        .build()
        .unwrap();

    let originator = NaturalPersonBuilder::new()
        .name(originator_name)
        .build()
        .unwrap();

    let beneficiary_name = NaturalPersonNameBuilder::new()
        .legal_name("Smith", "Jane")
        .build()
        .unwrap();

    let beneficiary = NaturalPersonBuilder::new()
        .name(beneficiary_name)
        .build()
        .unwrap();

    let vasp_name = LegalPersonNameBuilder::new()
        .legal_name("VASP Inc.")
        .build()
        .unwrap();

    let vasp = LegalPersonBuilder::new().name(vasp_name).build().unwrap();

    let message = IvmsMessageBuilder::new()
        .originator(vec![Person::NaturalPerson(originator)])
        .beneficiary(vec![Person::NaturalPerson(beneficiary)])
        .originating_vasp(Person::LegalPerson(vasp))
        .transaction(
            "100.00",
            "EUR",
            TransactionDirection::Incoming,
            "TRX-001",
            "2024-01-01T00:00:00Z",
        )
        .unwrap()
        .build()
        .unwrap();

    // Should serialize without errors
    let json_str = message.to_json().unwrap();
    assert!(json_str.contains("\"naturalPerson\""));
    assert!(json_str.contains("\"legalPerson\""));

    // Should deserialize back
    let parsed = IvmsMessage::from_json(&json_str).unwrap();
    assert_eq!(parsed.transaction.amount, "100.00");
    assert_eq!(parsed.transaction.currency, "EUR");
}

#[test]
fn test_address_serialization() {
    let address = GeographicAddress {
        address_type: Some(AddressType::Business),
        department: Some("Finance".to_string()),
        sub_department: Some("Compliance".to_string()),
        street_name: "Wall Street".to_string(),
        building_number: Some("100".to_string()),
        building_name: Some("Exchange Tower".to_string()),
        floor: Some("42".to_string()),
        post_box: Some("PO Box 123".to_string()),
        room: Some("Suite 4200".to_string()),
        post_code: "10005".to_string(),
        town_name: "New York".to_string(),
        town_location_name: Some("Financial District".to_string()),
        district_name: Some("Manhattan".to_string()),
        country_sub_division: Some("NY".to_string()),
        address_line: Some(vec![
            "Exchange Tower".to_string(),
            "100 Wall Street".to_string(),
        ]),
        country: "US".to_string(),
    };

    let json = serde_json::to_value(&address).unwrap();

    // Verify all fields are serialized correctly
    assert_eq!(json["addressType"], "BUSINESS");
    assert_eq!(json["department"], "Finance");
    assert_eq!(json["subDepartment"], "Compliance");
    assert_eq!(json["streetName"], "Wall Street");
    assert_eq!(json["buildingNumber"], "100");
    assert_eq!(json["buildingName"], "Exchange Tower");
    assert_eq!(json["floor"], "42");
    assert_eq!(json["postBox"], "PO Box 123");
    assert_eq!(json["room"], "Suite 4200");
    assert_eq!(json["postCode"], "10005");
    assert_eq!(json["townName"], "New York");
    assert_eq!(json["townLocationName"], "Financial District");
    assert_eq!(json["districtName"], "Manhattan");
    assert_eq!(json["countrySubDivision"], "NY");
    assert_eq!(json["addressLine"][0], "Exchange Tower");
    assert_eq!(json["addressLine"][1], "100 Wall Street");
    assert_eq!(json["country"], "US");
}

#[test]
fn test_enum_serialization() {
    // Test that enums serialize to correct string values
    assert_eq!(
        serde_json::to_value(&NameIdentifierType::LegalName).unwrap(),
        json!("LEGAL_NAME")
    );
    assert_eq!(
        serde_json::to_value(&NameIdentifierType::TradingName).unwrap(),
        json!("TRADING_NAME")
    );
    assert_eq!(
        serde_json::to_value(&AddressType::Home).unwrap(),
        json!("HOME")
    );
    assert_eq!(
        serde_json::to_value(&NationalIdentifierType::PassportNumber).unwrap(),
        json!("PASSPORT_NUMBER")
    );
    assert_eq!(
        serde_json::to_value(&PaymentType::SalaryAndWages).unwrap(),
        json!("SALARY_AND_WAGES")
    );
    assert_eq!(
        serde_json::to_value(&TransactionDirection::Outgoing).unwrap(),
        json!("outgoing")
    );
}
