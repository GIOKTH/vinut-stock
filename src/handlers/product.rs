use crate::db::AppState;
use crate::models::product::{CreateProductSchema, Product};
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

pub async fn get_products(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as!(Product, "SELECT * FROM products")
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn create_product(
    body: web::Json<CreateProductSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = Uuid::new_v4();
    let result = sqlx::query_as!(
        Product,
        "INSERT INTO products (id, code, name, image, sale_price, commission_price, promotion_price, quantity, low_stock_threshold) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
        id, body.code, body.name, body.image, body.sale_price, body.commission_price, body.promotion_price, body.quantity, body.low_stock_threshold
    )
    .fetch_one(&data.db)
    .await;

    match result {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn get_product_by_id(path: web::Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(Product, "SELECT * FROM products WHERE id = $1", id)
        .fetch_optional(&data.db)
        .await;

    match result {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Product not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

pub async fn get_product_purchases(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let product_id = path.into_inner();

    // We need a struct to represent the result of the join
    #[derive(serde::Serialize, sqlx::FromRow)]
    struct ProductPurchaseHistory {
        id: Uuid, // Purchase Item ID or Purchase ID? Ideally Purchase ID for reference.
        // Let's return Purchase details primarily
        purchase_id: Uuid,
        quantity: i32,
        buy_price: Decimal,        // Foreign price from item
        supplier_id: Option<Uuid>, // From Header
        created_at: Option<chrono::DateTime<chrono::Utc>>, // From Header
        currency_code: String,     // From Header
        exchange_rate: Decimal,    // From Header
    }

    let result = sqlx::query_as!(
        ProductPurchaseHistory,
        r#"
        SELECT 
            pi.id as id,
            p.id as purchase_id,
            pi.quantity,
            pi.buy_price,
            p.supplier_id,
            p.created_at,
            p.currency_code,
            p.exchange_rate
        FROM purchase_items pi
        JOIN purchases p ON pi.purchase_id = p.id
        WHERE pi.product_id = $1
        ORDER BY p.created_at DESC
        "#,
        product_id
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
