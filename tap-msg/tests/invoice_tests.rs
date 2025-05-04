use std::collections::HashMap;
use tap_msg::message::invoice::{Invoice, LineItem, TaxCategory, TaxSubtotal, TaxTotal};
use tap_msg::message::tap_message_trait::TapMessageBody;
use tap_msg::message::types::PaymentRequest;
use tap_msg::Participant;

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
    // Create a merchant participant
    let merchant = Participant {
        id: "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create an agent participant
    let agent = Participant {
        id: "did:key:z6MkmRsjkKHNrBiVz5mhiqhJVYf9E9mxg3MVGqgqMkRwCJd6".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
    };

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

    // Create a PaymentRequest with currency and invoice
    let mut payment_request = PaymentRequest::with_currency(
        "USD".to_string(),
        "100.0".to_string(),
        merchant.clone(),
        vec![agent.clone()],
    );

    payment_request.invoice = Some(invoice.clone());

    // This should validate correctly
    assert!(payment_request.validate().is_ok());

    // Test validation failure when amount doesn't match invoice total
    let mut mismatched_amount = payment_request.clone();
    mismatched_amount.amount = "200.0".to_string();
    assert!(mismatched_amount.validate().is_err());

    // Test validation failure when currency doesn't match invoice currency
    let mut mismatched_currency = payment_request.clone();
    mismatched_currency.currency = Some("EUR".to_string());
    assert!(mismatched_currency.validate().is_err());

    // Convert to DIDComm
    let didcomm_message = payment_request
        .to_didcomm(None)
        .expect("Failed to convert PaymentRequest to DIDComm");

    // Verify DIDComm message type
    assert_eq!(
        didcomm_message.type_,
        "https://tap.rsvp/schema/1.0#paymentrequest"
    );

    // Verify that we can extract the message body including the invoice
    let extracted = PaymentRequest::from_didcomm(&didcomm_message)
        .expect("Failed to extract PaymentRequest from DIDComm");

    assert_eq!(extracted.amount, "100.0");
    assert_eq!(extracted.currency, Some("USD".to_string()));
    assert!(extracted.invoice.is_some());

    let extracted_invoice = extracted.invoice.unwrap();
    assert_eq!(extracted_invoice.id, "INV001");
    assert_eq!(extracted_invoice.currency_code, "USD");
    assert_eq!(extracted_invoice.total, 100.0);
}
