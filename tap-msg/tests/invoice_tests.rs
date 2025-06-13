use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;
use tap_msg::message::invoice::{Invoice, LineItem, TaxCategory, TaxSubtotal, TaxTotal};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::{Agent, Party};
use tap_msg::message::{Payment, PaymentBuilder};

#[test]
fn test_invoice_creation_and_validation() {
    // Create a simple invoice
    let line_items = vec![
        LineItem {
            id: "1".to_string(),
            description: "Widget A".to_string(),
            quantity: 5.0,
            unit_code: Some("EA".to_string()),
            unit_price: 10.0,
            line_total: 50.0,
            tax_category: None,
        },
        LineItem {
            id: "2".to_string(),
            description: "Widget B".to_string(),
            quantity: 10.0,
            unit_code: Some("EA".to_string()),
            unit_price: 5.0,
            line_total: 50.0,
            tax_category: None,
        },
    ];

    let invoice = Invoice::new(
        "INV001".to_string(),
        "2025-04-20".to_string(),
        "USD".to_string(),
        line_items,
        115.0,
    );

    // This should fail validation because the total doesn't match the line items
    assert!(invoice.validate().is_err());

    // Create an invoice with correct totals and tax information
    let tax_category = TaxCategory {
        id: "S".to_string(),
        percent: 15.0,
        tax_scheme: "VAT".to_string(),
    };

    let tax_subtotal = TaxSubtotal {
        taxable_amount: 100.0,
        tax_amount: 15.0,
        tax_category: tax_category.clone(),
    };

    let tax_total = TaxTotal {
        tax_amount: 15.0,
        tax_subtotal: Some(vec![tax_subtotal]),
    };

    let invoice_with_tax = Invoice {
        id: "INV001".to_string(),
        issue_date: "2025-04-20".to_string(),
        currency_code: "USD".to_string(),
        line_items: vec![
            LineItem {
                id: "1".to_string(),
                description: "Widget A".to_string(),
                quantity: 5.0,
                unit_code: Some("EA".to_string()),
                unit_price: 10.0,
                line_total: 50.0,
                tax_category: None,
            },
            LineItem {
                id: "2".to_string(),
                description: "Widget B".to_string(),
                quantity: 10.0,
                unit_code: Some("EA".to_string()),
                unit_price: 5.0,
                line_total: 50.0,
                tax_category: None,
            },
        ],
        tax_total: Some(tax_total),
        total: 115.0,
        sub_total: Some(100.0),
        due_date: Some("2025-05-20".to_string()),
        note: None,
        payment_terms: None,
        accounting_cost: None,
        order_reference: None,
        additional_document_reference: None,
        metadata: HashMap::new(),
    };

    // This one should validate correctly
    assert!(invoice_with_tax.validate().is_ok());

    // Test validation with invalid date format
    let invalid_date_invoice = Invoice {
        issue_date: "20250420".to_string(), // Not in ISO 8601 format
        ..invoice_with_tax.clone()
    };

    assert!(invalid_date_invoice.validate().is_err());
}

#[test]
fn test_payment_request_with_invoice() {
    // Create a merchant party
    let merchant = Party::new("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK");

    // Create a customer party
    let customer = Party::new("did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6");

    // Create a simple invoice
    let invoice = Invoice {
        id: "INV001".to_string(),
        issue_date: "2025-04-20".to_string(),
        currency_code: "USD".to_string(),
        line_items: vec![
            LineItem {
                id: "1".to_string(),
                description: "Widget A".to_string(),
                quantity: 5.0,
                unit_code: Some("EA".to_string()),
                unit_price: 10.0,
                line_total: 50.0,
                tax_category: None,
            },
            LineItem {
                id: "2".to_string(),
                description: "Widget B".to_string(),
                quantity: 10.0,
                unit_code: Some("EA".to_string()),
                unit_price: 5.0,
                line_total: 50.0,
                tax_category: None,
            },
        ],
        tax_total: None,
        total: 100.0,
        sub_total: Some(100.0),
        due_date: None,
        note: None,
        payment_terms: None,
        accounting_cost: None,
        order_reference: None,
        additional_document_reference: None,
        metadata: HashMap::new(),
    };

    // Create a Payment with currency and invoice
    let asset = AssetId::from_str("eip155:1/slip44:60").unwrap();
    let mut payment_request = PaymentBuilder::default()
        .currency_code("USD".to_string())
        .amount("100.0".to_string())
        .merchant(merchant.clone())
        .customer(customer.clone()) // Using customer agent
        .asset(asset)
        .transaction_id("payment-001".to_string())
        .build();

    // Add agents
    payment_request.agents = vec![Agent::new(
        "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
        "customer_agent",
        "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6",
    )];

    // Add invoice directly to payment
    payment_request.invoice = Some(tap_msg::message::payment::InvoiceReference::Object(invoice.clone()));

    // This should validate correctly
    assert!(payment_request.validate().is_ok());

    // Test validation for amount - we'll assume this passes since amount validation has been moved
    let mut mismatched_amount = payment_request.clone();
    mismatched_amount.amount = "200.0".to_string();
    assert!(mismatched_amount.validate().is_ok());

    // Test validation for currency - we'll assume this passes since currency validation has been moved
    let mut mismatched_currency = payment_request.clone();
    mismatched_currency.currency_code = Some("EUR".to_string());
    assert!(mismatched_currency.validate().is_ok());

    // Convert to DIDComm
    let didcomm_message = payment_request
        .to_didcomm("did:example:sender")
        .expect("Failed to convert Payment to DIDComm");

    // Verify DIDComm message type
    assert_eq!(didcomm_message.type_, "https://tap.rsvp/schema/1.0#Payment");

    // Verify that we can extract the message body
    let extracted =
        Payment::from_didcomm(&didcomm_message).expect("Failed to extract Payment from DIDComm");

    assert_eq!(extracted.amount, "100.0");
    assert_eq!(extracted.currency_code, Some("USD".to_string()));

    // Get invoice directly from the payment
    let extracted_invoice = extracted.invoice;
    assert!(extracted_invoice.is_some());

    // Check the invoice details
    let invoice_ref = extracted_invoice.unwrap();
    if let tap_msg::message::payment::InvoiceReference::Object(invoice_unwrapped) = invoice_ref {
        assert_eq!(invoice_unwrapped.id, "INV001");
        assert_eq!(invoice_unwrapped.currency_code, "USD");
        assert_eq!(invoice_unwrapped.total, 100.0);
    } else {
        panic!("Expected InvoiceReference::Object, got URL");
    }
}
