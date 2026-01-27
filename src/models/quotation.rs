use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Quotation {
    pub id: Uuid,
    pub partner_name: Option<String>,
    pub user_id: Option<Uuid>,
    pub total_amount: Decimal,
    pub tax_rate: Decimal,
    pub discount_amount: Decimal,
    pub currency_code: String,
    pub exchange_rate: Decimal,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QuotationItem {
    pub id: Uuid,
    pub quotation_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotationItemSchema {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateQuotationSchema {
    pub partner_name: Option<String>,
    pub items: Vec<CreateQuotationItemSchema>,
    pub tax_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub currency_code: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuotationStatusSchema {
    pub status: String,
}
