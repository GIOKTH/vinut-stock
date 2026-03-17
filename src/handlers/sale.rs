use crate::db::AppState;
use crate::models::product::Product;
use crate::models::sale::{
    CreateSaleSchema, Sale, SaleItem, SalesQuerySchema, UpdateSaleStatusSchema,
};
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/sales",
    request_body = CreateSaleSchema,
    responses(
        (status = 200, description = "Sale created successfully", body = Sale),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Sales",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_sale(
    body: web::Json<CreateSaleSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let sale_id = Uuid::new_v4();
    let mut total_amount = Decimal::new(0, 2);

    let currency = body
        .currency_code
        .clone()
        .unwrap_or_else(|| "BASE".to_string());
    let status = body.status.clone().unwrap_or_else(|| "PENDING".to_string());
    let exchange_rate = if currency == "BASE" {
        Decimal::new(1, 0)
    } else {
        match sqlx::query!(
            "SELECT rate_to_base FROM exchange_rates WHERE currency_code = $1",
            currency
        )
        .fetch_optional(&data.db)
        .await
        {
            Ok(Some(r)) => r.rate_to_base,
            Ok(None) => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": format!("Currency {} not supported (rate not found)", currency)}));
            }
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .json(json!({"error": format!("Failed to fetch exchange rate: {}", e)}));
            }
        }
    };

    // Start transaction
    let mut tx = data.db.begin().await.expect("Failed to start transaction");

    let mut sale_items = Vec::new();

    for item in &body.items {
        if item.quantity <= 0 {
            return HttpResponse::BadRequest()
                .json(json!({"error": format!("Invalid quantity for product {}. Must be greater than 0.", item.product_id)}));
        }

        // Get product price
        let product = sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE id = $1",
            item.product_id
        )
        .fetch_one(&mut *tx)
        .await;

        match product {
            Ok(p) => {
                // Check if product is active
                if !p.is_active.unwrap_or(true) {
                    return HttpResponse::BadRequest().json(json!({
                        "error": format!("Product {} is not active for sale", p.name)
                    }));
                }

                // Check stock availability
                if p.quantity < item.quantity {
                    return HttpResponse::BadRequest().json(json!({
                        "error": format!("Insufficient stock for product {}. Available: {}, Requested: {}", p.name, p.quantity, item.quantity)
                    }));
                }

                let subtotal = p.sale_price * Decimal::from(item.quantity) * exchange_rate;
                total_amount += subtotal;

                let sale_item = SaleItem {
                    id: Uuid::new_v4(),
                    sale_id,
                    product_id: Some(p.id),
                    quantity: item.quantity,
                    unit_price: p.sale_price * exchange_rate,
                    subtotal,
                };
                sale_items.push(sale_item);

                // Update stock
                // Update stock
                let _: sqlx::postgres::PgQueryResult = sqlx::query!(
                    "UPDATE products SET quantity = quantity - $1 WHERE id = $2",
                    item.quantity,
                    p.id
                )
                .execute(&mut *tx)
                .await
                .expect("Failed to update stock");
            }
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": format!("Product {} not found", item.product_id)}));
            }
        }
    }

    // Insert sale
    let sale: Sale = sqlx::query_as!(
        Sale,
        "INSERT INTO sales (id, total_amount, payment_method, promotion_code, currency_code, exchange_rate, status) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
        sale_id,
        total_amount,
        body.payment_method,
        body.promotion_code,
        currency,
        exchange_rate,
        status
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert sale");

    // Insert sale items
    // Insert sale items
    for si in sale_items {
        let _: sqlx::postgres::PgQueryResult = sqlx::query!(
            "INSERT INTO sale_items (id, sale_id, product_id, quantity, unit_price, subtotal) VALUES ($1, $2, $3, $4, $5, $6)",
            si.id, si.sale_id, si.product_id, si.quantity, si.unit_price, si.subtotal
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert sale item");
    }

    tx.commit().await.expect("Failed to commit transaction");

    HttpResponse::Ok().json(sale)
}

#[utoipa::path(
    get,
    path = "/api/sales",
    params(
        SalesQuerySchema
    ),
    responses(
        (status = 200, description = "List all sales with filters", body = [Sale]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Sales",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_sales(
    query: web::Query<crate::models::sale::SalesQuerySchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> =
        sqlx::QueryBuilder::new("SELECT * FROM sales WHERE 1=1 ");

    if let Some(status) = &query.status {
        query_builder.push(" AND status = ");
        query_builder.push_bind(status);
    }

    query_builder.push(" ORDER BY created_at DESC");

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20);
    let offset = (page - 1) * page_size;

    query_builder.push(" LIMIT ");
    query_builder.push_bind(page_size);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset);

    let result = query_builder
        .build_query_as::<Sale>()
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(sales) => {
            log::info!(
                "Sales fetched successfully (page: {}, size: {})",
                page,
                page_size
            );
            HttpResponse::Ok().json(sales)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    patch,
    path = "/api/sales/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Sale Database ID")
    ),
    request_body = UpdateSaleStatusSchema,
    responses(
        (status = 200, description = "Sale status updated", body = Sale),
        (status = 404, description = "Sale not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Sales",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_sale_status(
    path: web::Path<Uuid>,
    body: web::Json<UpdateSaleStatusSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let sale_id = path.into_inner();

    let result: Result<Option<Sale>, sqlx::Error> = sqlx::query_as!(
        Sale,
        "UPDATE sales SET status = $1 WHERE id = $2 RETURNING *",
        body.status,
        sale_id
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(sale)) => HttpResponse::Ok().json(sale),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Sale not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
