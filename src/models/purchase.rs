use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Purchase {
    pub id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub total_amount: Decimal,
    pub currency_code: String,
    pub exchange_rate: Decimal,
    pub shipping_cost: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PurchaseItem {
    pub id: Uuid,
    pub purchase_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub buy_price: Decimal,
    pub subtotal: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseItemSchema {
    pub product_id: Uuid,
    pub quantity: i32,
    pub buy_price: Decimal,
    pub new_sale_price: Option<Decimal>,
    pub new_commission_price: Option<Decimal>,
    pub new_promotion_price: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseSchema {
    pub supplier_id: Option<Uuid>,
    pub items: Vec<CreatePurchaseItemSchema>,
    pub shipping_cost: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub currency_code: Option<String>,
    pub margin_price: Option<Decimal>,
}
