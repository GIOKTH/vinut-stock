use crate::db::AppState;
use crate::models::product::{CreateProductSchema, Product};
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/api/products",
    responses(
        (status = 200, description = "List all products", body = [Product]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_products(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as!(Product, "SELECT * FROM products")
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    post,
    path = "/api/products",
    request_body = CreateProductSchema,
    responses(
        (status = 200, description = "Product created successfully", body = Product),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_product(
    body: web::Json<CreateProductSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = Uuid::new_v4();
    let is_active = body.is_active.unwrap_or(true);
    let result = sqlx::query_as!(
        Product,
        "INSERT INTO products (id, code, name, image, sale_price, commission_price, promotion_price, quantity, is_active, low_stock_threshold) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        id, body.code, body.name, body.image, body.sale_price, body.commission_price, body.promotion_price, body.quantity, is_active, body.low_stock_threshold
    )
    .fetch_one(&data.db)
    .await;

    match result {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct UpdateProductStatusSchema {
    pub is_active: bool,
}

#[utoipa::path(
    patch,
    path = "/api/products/{id}/status",
    request_body = UpdateProductStatusSchema,
    responses(
        (status = 200, description = "Product status updated successfully", body = Product),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_product_status(
    path: web::Path<Uuid>,
    body: web::Json<UpdateProductStatusSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(
        Product,
        "UPDATE products SET is_active = $1 WHERE id = $2 RETURNING *",
        body.is_active,
        id
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Product not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/products/{id}",
    params(
        ("id" = Uuid, Path, description = "Product Database ID")
    ),
    responses(
        (status = 200, description = "Product fetched successfully", body = Product),
        (status = 404, description = "Product not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products",
    security(
        ("bearer_auth" = [])
    )
)]
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

#[utoipa::path(
    get,
    path = "/api/products/{id}/purchases",
    params(
        ("id" = Uuid, Path, description = "Product Database ID")
    ),
    responses(
        (status = 200, description = "Product purchase history fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Products",
    security(
        ("bearer_auth" = [])
    )
)]
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
