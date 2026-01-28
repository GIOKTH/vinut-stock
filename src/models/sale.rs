use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Sale {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub total_amount: Decimal,
    pub discount_amount: Option<Decimal>,
    pub promotion_code: Option<String>,
    pub payment_method: Option<String>,
    pub currency_code: Option<String>,
    pub exchange_rate: Option<Decimal>,
    pub status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct SaleItem {
    pub id: Uuid,
    pub sale_id: Uuid,
    pub product_id: Option<Uuid>,
    pub quantity: i32,
    pub unit_price: Decimal,
    pub subtotal: Decimal,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateSaleItemSchema {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct CreateSaleSchema {
    pub items: Vec<CreateSaleItemSchema>,
    pub promotion_code: Option<String>,
    pub payment_method: Option<String>,
    pub currency_code: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateSaleStatusSchema {
    pub status: String,
}
