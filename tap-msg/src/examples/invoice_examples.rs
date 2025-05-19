//! Examples for using the Invoice and Payment functionality.

use crate::didcomm::PlainMessage;
use crate::error::Result;
use crate::message::invoice::{Invoice, LineItem, TaxCategory, TaxSubtotal, TaxTotal};
use crate::message::payment::PaymentBuilder;
use crate::message::tap_message_trait::TapMessageBody;
use crate::message::{Participant, Payment};
use std::collections::HashMap;
use std::str::FromStr;
use tap_caip::AssetId;

/// Example of creating a basic invoice with line items
pub fn create_basic_invoice_example() -> Result<Invoice> {
    // Create line items
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

    // Calculate the total
    let total = line_items.iter().map(|item| item.line_total).sum();

    // Create a basic invoice
    let invoice = Invoice::new(
        "INV001".to_string(),
        "2023-09-01".to_string(),
        "USD".to_string(),
        line_items,
        total,
    );

    // Validate the invoice
    invoice.validate()?;

    Ok(invoice)
}

/// Example of creating an invoice with tax information
pub fn create_invoice_with_tax_example() -> Result<Invoice> {
    // Create line items
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

    // Calculate the subtotal
    let sub_total = line_items.iter().map(|item| item.line_total).sum();

    // Create tax information
    let tax_category = TaxCategory {
        id: "S".to_string(),
        percent: 15.0,
        tax_scheme: "VAT".to_string(),
    };

    let tax_amount = sub_total * (tax_category.percent / 100.0);
    let total = sub_total + tax_amount;

    let tax_subtotal = TaxSubtotal {
        taxable_amount: sub_total,
        tax_amount,
        tax_category,
    };

    let tax_total = TaxTotal {
        tax_amount,
        tax_subtotal: Some(vec![tax_subtotal]),
    };

    // Create the invoice with tax information
    let invoice = Invoice {
        id: "INV001".to_string(),
        issue_date: "2023-09-01".to_string(),
        currency_code: "USD".to_string(),
        line_items,
        tax_total: Some(tax_total),
        total,
        sub_total: Some(sub_total),
        due_date: Some("2023-10-01".to_string()),
        note: None,
        payment_terms: Some("NET30".to_string()),
        accounting_cost: None,
        order_reference: None,
        additional_document_reference: None,
        metadata: HashMap::new(),
    };

    // Validate the invoice
    invoice.validate()?;

    Ok(invoice)
}

/// Example of creating a Payment with an embedded invoice
pub fn create_payment_request_with_invoice_example(
    merchant_did: &str,
    customer_did: Option<&str>,
) -> Result<PlainMessage> {
    // Create merchant participant
    let merchant = Participant {
        id: merchant_did.to_string(),
        role: Some("merchant".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create a merchant agent (e.g., a payment processor)
    let agent = Participant {
        id: "did:example:payment_processor".to_string(),
        role: Some("agent".to_string()),
        policies: None,
        leiCode: None,
    };

    // Create an invoice with tax
    let invoice = create_invoice_with_tax_example()?;

    // Create a Payment using the new API
    let asset =
        AssetId::from_str("eip155:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f").unwrap();

    // Create transaction ID
    let transaction_id = uuid::Uuid::new_v4().to_string();

    // Create a customer participant if provided
    let customer = customer_did.map(|cust_did| Participant {
        id: cust_did.to_string(),
        role: Some("customer".to_string()),
        policies: None,
        leiCode: None,
    });

    // Use the builder pattern to create the payment
    let mut payment_request = PaymentBuilder::default()
        .transaction_id(transaction_id)
        .asset(asset)
        .amount(format!("{:.2}", invoice.total))
        .currency_code(invoice.currency_code.clone())
        .merchant(merchant.clone())
        .add_agent(agent)
        .build();

    // Set customer if provided
    payment_request.customer = customer;

    // Add the invoice directly to the payment
    payment_request.invoice = Some(invoice);

    // Add expiry (e.g., 30 days)
    payment_request.expiry = Some("2023-10-01T00:00:00Z".to_string());

    // Convert to a DIDComm message
    let recipients = if let Some(cust_did) = customer_did {
        vec![cust_did]
    } else {
        vec![]
    };

    let message =
        payment_request.to_didcomm_with_route(merchant_did, recipients.iter().copied())?;

    Ok(message)
}

/// Example of extracting and validating an invoice from a Payment message
pub fn process_payment_request_with_invoice_example(message: &PlainMessage) -> Result<()> {
    // Extract the Payment
    let payment_request = Payment::from_didcomm(message)?;

    // Validate the Payment
    payment_request.validate()?;

    // Print merchant information
    println!("Merchant: {}", payment_request.merchant.id);

    // Print customer information if present
    if let Some(customer) = &payment_request.customer {
        println!("Customer: {}", customer.id);
    } else {
        println!("Customer: Not specified");
    }

    println!("Amount: {}", payment_request.amount);

    if let Some(currency) = &payment_request.currency_code {
        println!("Currency: {}", currency);
    }

    if let Some(asset) = &payment_request.asset {
        println!("Asset: {}", asset);
    }

    // Check if it has an invoice directly in the payment
    if let Some(invoice) = &payment_request.invoice {
        println!("Invoice ID: {}", invoice.id);
        println!("Currency: {}", invoice.currency_code);
        println!("Total amount: {:.2}", invoice.total);

        // Print line items
        println!("Line items:");
        for (i, item) in invoice.line_items.iter().enumerate() {
            println!(
                "  {}: {} x {} @ {:.2} = {:.2}",
                i + 1,
                item.quantity,
                item.description,
                item.unit_price,
                item.line_total
            );
        }

        // Print tax information if present
        if let Some(tax_total) = &invoice.tax_total {
            println!("Tax amount: {:.2}", tax_total.tax_amount);

            if let Some(tax_subtotals) = &tax_total.tax_subtotal {
                for (i, subtotal) in tax_subtotals.iter().enumerate() {
                    println!(
                        "  Tax {}: {:.2}% {} on {:.2} = {:.2}",
                        i + 1,
                        subtotal.tax_category.percent,
                        subtotal.tax_category.tax_scheme,
                        subtotal.taxable_amount,
                        subtotal.tax_amount
                    );
                }
            }
        }

        println!(
            "Due date: {}",
            invoice.due_date.as_deref().unwrap_or("Not specified")
        );
    } else {
        println!("Payment request does not contain an invoice");
    }

    // Print expiry if present
    if let Some(expiry) = &payment_request.expiry {
        println!("Expires: {}", expiry);
    }

    Ok(())
}
