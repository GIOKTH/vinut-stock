use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Purchase {
    pub id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub total_amount: Decimal,
    pub currency_code: String,
    pub exchange_rate: Decimal,
    pub shipping_cost: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct PurchaseItem {
    pub id: Uuid,
    pub purchase_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub buy_price: Decimal,
    pub subtotal: Decimal,
    pub new_sale_price: Option<Decimal>,
    pub new_commission_price: Option<Decimal>,
    pub new_promotion_price: Option<Decimal>,
    pub landed_cost_base: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePurchaseItemSchema {
    pub product_id: Uuid,
    pub quantity: i32,
    pub buy_price: Decimal,
    pub new_sale_price: Option<Decimal>,
    pub new_commission_price: Option<Decimal>,
    pub new_promotion_price: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreatePurchaseSchema {
    pub supplier_id: Option<Uuid>,
    pub items: Vec<CreatePurchaseItemSchema>,
    pub shipping_cost: Option<Decimal>,
    pub tax_rate: Option<Decimal>,
    pub currency_code: Option<String>,
    pub margin_price: Option<Decimal>,
}
