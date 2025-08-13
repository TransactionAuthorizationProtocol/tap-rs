//! Tests for LineItem Product attributes per schema.org/Product

#[cfg(test)]
mod tests {
    use tap_msg::LineItem;

    #[test]
    fn test_line_item_with_product_name() {
        let line_item = LineItem {
            id: "item-001".to_string(),
            description: "Premium Coffee Beans".to_string(),
            quantity: 2.0,
            unit_code: Some("KGM".to_string()),
            unit_price: 25.99,
            line_total: 51.98,
            tax_category: None,
            name: Some("Colombian Arabica Premium Blend".to_string()),
            image: None,
            url: None,
        };

        assert_eq!(
            line_item.name,
            Some("Colombian Arabica Premium Blend".to_string())
        );

        // Test serialization includes name field
        let json = serde_json::to_value(&line_item).unwrap();
        assert_eq!(json["name"], "Colombian Arabica Premium Blend");
    }

    #[test]
    fn test_line_item_with_product_image() {
        let line_item = LineItem {
            id: "item-002".to_string(),
            description: "Organic Green Tea".to_string(),
            quantity: 3.0,
            unit_code: Some("BOX".to_string()),
            unit_price: 12.50,
            line_total: 37.50,
            tax_category: None,
            name: None,
            image: Some("https://example.com/products/green-tea.jpg".to_string()),
            url: None,
        };

        assert_eq!(
            line_item.image,
            Some("https://example.com/products/green-tea.jpg".to_string())
        );

        // Test serialization includes image field
        let json = serde_json::to_value(&line_item).unwrap();
        assert_eq!(json["image"], "https://example.com/products/green-tea.jpg");
    }

    #[test]
    fn test_line_item_with_product_url() {
        let line_item = LineItem {
            id: "item-003".to_string(),
            description: "Laptop Stand".to_string(),
            quantity: 1.0,
            unit_code: Some("EA".to_string()),
            unit_price: 89.99,
            line_total: 89.99,
            tax_category: None,
            name: None,
            image: None,
            url: Some("https://shop.example.com/products/laptop-stand-adjustable".to_string()),
        };

        assert_eq!(
            line_item.url,
            Some("https://shop.example.com/products/laptop-stand-adjustable".to_string())
        );

        // Test serialization includes url field
        let json = serde_json::to_value(&line_item).unwrap();
        assert_eq!(
            json["url"],
            "https://shop.example.com/products/laptop-stand-adjustable"
        );
    }

    #[test]
    fn test_line_item_with_all_product_fields() {
        let line_item = LineItem {
            id: "item-004".to_string(),
            description: "Wireless Mouse".to_string(),
            quantity: 2.0,
            unit_code: Some("EA".to_string()),
            unit_price: 45.00,
            line_total: 90.00,
            tax_category: None,
            name: Some("ErgoTech Pro Wireless Mouse".to_string()),
            image: Some("https://cdn.example.com/images/ergotech-mouse.png".to_string()),
            url: Some("https://shop.example.com/ergotech-pro-mouse".to_string()),
        };

        assert!(line_item.name.is_some());
        assert!(line_item.image.is_some());
        assert!(line_item.url.is_some());

        // Test serialization includes all product fields
        let json = serde_json::to_value(&line_item).unwrap();
        assert_eq!(json["name"], "ErgoTech Pro Wireless Mouse");
        assert_eq!(
            json["image"],
            "https://cdn.example.com/images/ergotech-mouse.png"
        );
        assert_eq!(json["url"], "https://shop.example.com/ergotech-pro-mouse");
        assert_eq!(json["description"], "Wireless Mouse");
    }

    #[test]
    fn test_line_item_without_product_fields() {
        let line_item = LineItem {
            id: "item-005".to_string(),
            description: "Consulting Services".to_string(),
            quantity: 8.0,
            unit_code: Some("HUR".to_string()),
            unit_price: 150.00,
            line_total: 1200.00,
            tax_category: None,
            name: None,
            image: None,
            url: None,
        };

        assert!(line_item.name.is_none());
        assert!(line_item.image.is_none());
        assert!(line_item.url.is_none());

        // Test that optional fields are not serialized when None
        let json = serde_json::to_value(&line_item).unwrap();
        assert!(!json.as_object().unwrap().contains_key("name"));
        assert!(!json.as_object().unwrap().contains_key("image"));
        assert!(!json.as_object().unwrap().contains_key("url"));
    }

    #[test]
    fn test_line_item_builder_with_product_fields() {
        let line_item = LineItem::builder()
            .id("item-006".to_string())
            .description("Office Chair".to_string())
            .quantity(1.0)
            .unit_code("EA".to_string())
            .unit_price(299.99)
            .line_total(299.99)
            .name("Executive Ergonomic Office Chair".to_string())
            .image("https://furniture.example.com/chair-exec-01.jpg".to_string())
            .url("https://furniture.example.com/products/exec-chair".to_string())
            .build();

        assert_eq!(
            line_item.name,
            Some("Executive Ergonomic Office Chair".to_string())
        );
        assert_eq!(
            line_item.image,
            Some("https://furniture.example.com/chair-exec-01.jpg".to_string())
        );
        assert_eq!(
            line_item.url,
            Some("https://furniture.example.com/products/exec-chair".to_string())
        );
    }

    #[test]
    fn test_line_item_deserialization_with_product_fields() {
        let json = serde_json::json!({
            "id": "item-007",
            "description": "Smart Watch",
            "quantity": 1,
            "unitCode": "EA",
            "unitPrice": 399.99,
            "lineTotal": 399.99,
            "name": "TechFit Pro 5",
            "image": "https://tech.example.com/images/techfit-pro5.webp",
            "url": "https://tech.example.com/smartwatch/techfit-pro5"
        });

        let line_item: LineItem = serde_json::from_value(json).unwrap();

        assert_eq!(line_item.name, Some("TechFit Pro 5".to_string()));
        assert_eq!(
            line_item.image,
            Some("https://tech.example.com/images/techfit-pro5.webp".to_string())
        );
        assert_eq!(
            line_item.url,
            Some("https://tech.example.com/smartwatch/techfit-pro5".to_string())
        );
    }
}
