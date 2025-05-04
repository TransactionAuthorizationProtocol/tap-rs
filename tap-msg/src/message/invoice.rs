//! Invoice message types and structures according to TAIP-16.
//!
//! This module defines the structured Invoice object that can be embedded
//! in a TAIP-14 Payment Request message.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tax category for a line item or tax subtotal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxCategory {
    /// Tax category code (e.g., "S" for standard rate, "Z" for zero-rated)
    pub id: String,

    /// Tax rate percentage
    pub percent: f64,

    /// Tax scheme (e.g., "VAT", "GST")
    #[serde(rename = "taxScheme")]
    pub tax_scheme: String,
}

/// Line item in an invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    /// Unique identifier for the line item
    pub id: String,

    /// Description of the item or service
    pub description: String,

    /// Quantity of the item
    pub quantity: f64,

    /// Optional unit of measure (e.g., "KGM" for kilogram)
    #[serde(rename = "unitCode", skip_serializing_if = "Option::is_none")]
    pub unit_code: Option<String>,

    /// Price per unit
    #[serde(rename = "unitPrice")]
    pub unit_price: f64,

    /// Total amount for this line item
    #[serde(rename = "lineTotal")]
    pub line_total: f64,

    /// Optional tax category for the line item
    #[serde(rename = "taxCategory", skip_serializing_if = "Option::is_none")]
    pub tax_category: Option<TaxCategory>,
}

/// Tax subtotal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSubtotal {
    /// Amount subject to this tax
    #[serde(rename = "taxableAmount")]
    pub taxable_amount: f64,

    /// Tax amount for this category
    #[serde(rename = "taxAmount")]
    pub tax_amount: f64,

    /// Tax category information
    #[serde(rename = "taxCategory")]
    pub tax_category: TaxCategory,
}

/// Aggregate tax information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxTotal {
    /// Total tax amount for the invoice
    #[serde(rename = "taxAmount")]
    pub tax_amount: f64,

    /// Optional breakdown of taxes by category
    #[serde(rename = "taxSubtotal", skip_serializing_if = "Option::is_none")]
    pub tax_subtotal: Option<Vec<TaxSubtotal>>,
}

/// Order reference information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderReference {
    /// Order identifier
    pub id: String,

    /// Optional issue date of the order
    #[serde(rename = "issueDate", skip_serializing_if = "Option::is_none")]
    pub issue_date: Option<String>,
}

/// Reference to an additional document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReference {
    /// Document identifier
    pub id: String,

    /// Optional document type
    #[serde(rename = "documentType", skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,

    /// Optional URL where the document can be accessed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Invoice structure according to TAIP-16
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique identifier for the invoice
    pub id: String,

    /// Date when the invoice was issued (ISO 8601 format)
    #[serde(rename = "issueDate")]
    pub issue_date: String,

    /// ISO 4217 currency code
    #[serde(rename = "currencyCode")]
    pub currency_code: String,

    /// Line items in the invoice
    #[serde(rename = "lineItems")]
    pub line_items: Vec<LineItem>,

    /// Optional tax total information
    #[serde(rename = "taxTotal", skip_serializing_if = "Option::is_none")]
    pub tax_total: Option<TaxTotal>,

    /// Total amount of the invoice, including taxes
    pub total: f64,

    /// Optional sum of line totals before taxes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_total: Option<f64>,

    /// Optional due date for payment (ISO 8601 format)
    #[serde(rename = "dueDate", skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,

    /// Optional additional notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Optional payment terms
    #[serde(rename = "paymentTerms", skip_serializing_if = "Option::is_none")]
    pub payment_terms: Option<String>,

    /// Optional accounting cost code
    #[serde(rename = "accountingCost", skip_serializing_if = "Option::is_none")]
    pub accounting_cost: Option<String>,

    /// Optional order reference
    #[serde(rename = "orderReference", skip_serializing_if = "Option::is_none")]
    pub order_reference: Option<OrderReference>,

    /// Optional references to additional documents
    #[serde(
        rename = "additionalDocumentReference",
        skip_serializing_if = "Option::is_none"
    )]
    pub additional_document_reference: Option<Vec<DocumentReference>>,

    /// Additional metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Invoice {
    /// Creates a new basic Invoice
    pub fn new(
        id: String,
        issue_date: String,
        currency_code: String,
        line_items: Vec<LineItem>,
        total: f64,
    ) -> Self {
        Self {
            id,
            issue_date,
            currency_code,
            line_items,
            tax_total: None,
            total,
            sub_total: None,
            due_date: None,
            note: None,
            payment_terms: None,
            accounting_cost: None,
            order_reference: None,
            additional_document_reference: None,
            metadata: HashMap::new(),
        }
    }

    /// Validate the Invoice according to TAIP-16 rules
    pub fn validate(&self) -> crate::error::Result<()> {
        use crate::error::Error;

        // Required fields validation
        if self.id.is_empty() {
            return Err(Error::Validation("Invoice ID is required".to_string()));
        }

        if self.issue_date.is_empty() {
            return Err(Error::Validation("Issue date is required".to_string()));
        }

        if self.currency_code.is_empty() {
            return Err(Error::Validation("Currency code is required".to_string()));
        }

        if self.line_items.is_empty() {
            return Err(Error::Validation(
                "At least one line item is required".to_string(),
            ));
        }

        // Validate line items
        for (i, item) in self.line_items.iter().enumerate() {
            if item.id.is_empty() {
                return Err(Error::Validation(format!(
                    "Line item {} is missing an ID",
                    i
                )));
            }

            if item.description.is_empty() {
                return Err(Error::Validation(format!(
                    "Line item {} is missing a description",
                    i
                )));
            }

            // Validate that line total is approximately equal to quantity * unit price
            // Allow for some floating point imprecision
            let calculated_total = item.quantity * item.unit_price;
            let difference = (calculated_total - item.line_total).abs();
            if difference > 0.01 {
                // Allow a small tolerance for floating point calculations
                return Err(Error::Validation(format!(
                    "Line item {}: Line total ({}) does not match quantity ({}) * unit price ({})",
                    i, item.line_total, item.quantity, item.unit_price
                )));
            }
        }

        // Validate sub_total if present
        if let Some(sub_total) = self.sub_total {
            let calculated_sub_total: f64 =
                self.line_items.iter().map(|item| item.line_total).sum();
            let difference = (calculated_sub_total - sub_total).abs();
            if difference > 0.01 {
                // Allow a small tolerance for floating point calculations
                return Err(Error::Validation(format!(
                    "Sub-total ({}) does not match the sum of line totals ({})",
                    sub_total, calculated_sub_total
                )));
            }
        }

        // Validate tax_total if present
        if let Some(tax_total) = &self.tax_total {
            if let Some(tax_subtotals) = &tax_total.tax_subtotal {
                let sum_of_subtotals: f64 = tax_subtotals.iter().map(|st| st.tax_amount).sum();
                let difference = (sum_of_subtotals - tax_total.tax_amount).abs();
                if difference > 0.01 {
                    // Allow a small tolerance for floating point calculations
                    return Err(Error::Validation(format!(
                        "Tax total amount ({}) does not match the sum of tax subtotal amounts ({})",
                        tax_total.tax_amount, sum_of_subtotals
                    )));
                }
            }
        }

        // Validate total
        let sub_total = self
            .sub_total
            .unwrap_or_else(|| self.line_items.iter().map(|item| item.line_total).sum());
        let tax_amount = self.tax_total.as_ref().map_or(0.0, |tt| tt.tax_amount);
        let calculated_total = sub_total + tax_amount;
        let difference = (calculated_total - self.total).abs();
        if difference > 0.01 {
            // Allow a small tolerance for floating point calculations
            return Err(Error::Validation(format!(
                "Total ({}) does not match sub-total ({}) + tax amount ({})",
                self.total, sub_total, tax_amount
            )));
        }

        // Validate date formats
        if self.issue_date.len() != 10 {
            return Err(Error::SerializationError(
                "issue_date must be in YYYY-MM-DD format".to_string(),
            ));
        }
        if chrono::NaiveDate::parse_from_str(&self.issue_date, "%Y-%m-%d").is_err() {
            return Err(Error::SerializationError(
                "Invalid issue_date format or value".to_string(),
            ));
        }

        if let Some(due_date) = &self.due_date {
            if due_date.len() != 10 {
                return Err(Error::SerializationError(
                    "due_date must be in YYYY-MM-DD format".to_string(),
                ));
            }
            if chrono::NaiveDate::parse_from_str(due_date, "%Y-%m-%d").is_err() {
                return Err(Error::SerializationError(
                    "Invalid due_date format or value".to_string(),
                ));
            }
        }

        Ok(())
    }
}
