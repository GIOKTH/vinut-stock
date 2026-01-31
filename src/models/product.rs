use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub image: Option<String>,
    pub sale_price: Decimal,
    pub cost_price: Option<Decimal>,
    pub commission_price: Option<Decimal>,
    pub promotion_price: Option<Decimal>,
    pub quantity: i32,
    pub is_active: Option<bool>,
    pub low_stock_threshold: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateProductSchema {
    pub code: String,
    pub name: String,
    pub image: Option<String>,
    pub sale_price: Decimal,
    pub commission_price: Option<Decimal>,
    pub promotion_price: Option<Decimal>,
    pub quantity: i32,
    pub is_active: Option<bool>,
    pub low_stock_threshold: Option<i32>,
}
