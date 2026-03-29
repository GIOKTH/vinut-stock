use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
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
    pub payment_amount: Option<Decimal>,
    pub payment_currency: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct QuotationResponse {
    pub id: Uuid,
    pub partner_name: Option<String>,
    pub user_id: Option<Uuid>,
    pub total_amount: Decimal,
    pub tax_rate: Decimal,
    pub discount_amount: Decimal,
    pub currency_code: String,
    pub exchange_rate: Decimal,
    pub status: Option<String>,
    pub payment_amount: Option<Decimal>,
    pub payment_currency: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct QuotationItem {
    pub id: Uuid,
    pub quotation_id: Option<Uuid>,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateQuotationItemSchema {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateQuotationSchema {
    pub partner_name: Option<String>,
    pub items: Vec<CreateQuotationItemSchema>,
    pub tax_rate: Option<Decimal>,
    pub discount_amount: Option<Decimal>,
    pub currency_code: Option<String>,
    pub status: Option<String>,
    pub payment_amount: Option<Decimal>,
    pub payment_currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateQuotationStatusSchema {
    pub status: String,
}
