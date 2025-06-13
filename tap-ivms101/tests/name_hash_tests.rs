//! Tests for TAIP-12 name hashing with IVMS101 data structures

use tap_ivms101::{
    builder::{
        LegalPersonBuilder, LegalPersonNameBuilder, NaturalPersonBuilder, NaturalPersonNameBuilder,
    },
    message::Person,
};
use tap_msg::utils::NameHashable;

#[test]
fn test_natural_person_name_hash() {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Lee", "Alice")
        .build()
        .unwrap();

    let person = NaturalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::NaturalPerson(person);

    // Get the full name and hash it
    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "Alice Lee");

    let hash = Person::hash_name(&full_name);
    assert_eq!(
        hash,
        "b117f44426c9670da91b563db728cd0bc8bafa7d1a6bb5e764d1aad2ca25032e"
    );
}

#[test]
fn test_legal_person_name_hash() {
    let name = LegalPersonNameBuilder::new()
        .legal_name("Example VASP Ltd.")
        .build()
        .unwrap();

    let person = LegalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::LegalPerson(person);

    // Get the full name and hash it
    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "Example VASP Ltd.");

    let hash = Person::hash_name(&full_name);
    assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
}

#[test]
fn test_natural_person_with_multiple_names() {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("García-López", "María José")
        .build()
        .unwrap();

    let person = NaturalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::NaturalPerson(person);

    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "María José García-López");

    // Hash should be consistent regardless of spacing
    let hash1 = Person::hash_name(&full_name);
    let hash2 = Person::hash_name("María   José   García-López");
    assert_eq!(hash1, hash2);
}

#[test]
fn test_natural_person_name_normalization() {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("SMITH", "BOB")
        .build()
        .unwrap();

    let person = NaturalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::NaturalPerson(person);

    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "BOB SMITH");

    let hash = Person::hash_name(&full_name);
    assert_eq!(
        hash,
        "5432e86b4d4a3a2b4be57b713b12c5c576c88459fe1cfdd760fd6c99a0e06686"
    );
}

#[test]
fn test_empty_secondary_identifier() {
    let name = NaturalPersonNameBuilder::new()
        .legal_name("Lee", "")
        .build()
        .unwrap();

    let person = NaturalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::NaturalPerson(person);

    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "Lee"); // Should handle empty secondary identifier gracefully
}

#[test]
fn test_legal_person_with_special_characters() {
    let name = LegalPersonNameBuilder::new()
        .legal_name("O'Brien & Associates, LLC")
        .build()
        .unwrap();

    let person = LegalPersonBuilder::new().name(name).build().unwrap();

    let ivms_person = Person::LegalPerson(person);

    let full_name = ivms_person.get_full_name().unwrap();
    assert_eq!(full_name, "O'Brien & Associates, LLC");

    let hash = Person::hash_name(&full_name);
    assert_eq!(hash.len(), 64);
}
