use crate::db::AppState;
use crate::models::product::Product;
use crate::models::quotation::{
    CreateQuotationSchema, Quotation, QuotationItem, UpdateQuotationStatusSchema,
};
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/quotations",
    request_body = CreateQuotationSchema,
    responses(
        (status = 200, description = "Quotation created successfully", body = Quotation),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Quotations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_quotation(
    body: web::Json<CreateQuotationSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let quotation_id = Uuid::new_v4();
    let mut total_amount = Decimal::new(0, 2);

    let tax_rate = body.tax_rate.unwrap_or(Decimal::new(0, 0));
    let discount_amount = body.discount_amount.unwrap_or(Decimal::new(0, 0));

    let currency = body
        .currency_code
        .clone()
        .unwrap_or_else(|| "BASE".to_string());
    let exchange_rate = if currency == "BASE" {
        Decimal::new(1, 0)
    } else {
        let rate_result = sqlx::query!(
            "SELECT rate_to_base FROM exchange_rates WHERE currency_code = $1",
            currency
        )
        .fetch_optional(&data.db)
        .await;

        match rate_result {
            Ok(Some(r)) => r.rate_to_base,
            _ => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": format!("Currency {} not supported", currency)}));
            }
        }
    };

    let mut tx = data.db.begin().await.expect("Failed to start transaction");

    let mut quotation_items = Vec::new();

    for item in &body.items {
        let product = sqlx::query_as!(
            Product,
            "SELECT * FROM products WHERE id = $1",
            item.product_id
        )
        .fetch_one(&mut *tx)
        .await;

        match product {
            Ok(p) => {
                let subtotal_base = p.sale_price * Decimal::from(item.quantity);
                total_amount += subtotal_base;

                let q_item = QuotationItem {
                    id: Uuid::new_v4(),
                    quotation_id: Some(quotation_id),
                    product_id: Some(p.id),
                    quantity: item.quantity,
                    unit_price: p.sale_price, // Unit price in USD
                    subtotal: subtotal_base,  // Subtotal in USD
                };
                quotation_items.push(q_item);
            }
            Err(_) => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": format!("Product {} not found", item.product_id)}));
            }
        }
    }

    // Apply tax and discount to the total amount
    let unit_tax = total_amount * (tax_rate / Decimal::from(100));
    total_amount = (total_amount + unit_tax) - discount_amount;

    let status = body.status.clone().unwrap_or_else(|| "DRAFT".to_string());

    let quotation = sqlx::query_as!(
        Quotation,
        "INSERT INTO quotations (id, partner_name, total_amount, tax_rate, discount_amount, currency_code, exchange_rate, status) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        quotation_id,
        body.partner_name,
        total_amount, // Stored in USD base currency
        tax_rate,
        discount_amount,
        currency,
        exchange_rate,
        status
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert quotation");

    for qi in quotation_items {
        sqlx::query!(
            "INSERT INTO quotation_items (id, quotation_id, product_id, quantity, unit_price, subtotal) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            qi.id,
            qi.quotation_id,
            qi.product_id,
            qi.quantity,
            qi.unit_price,
            qi.subtotal
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert quotation item");
    }

    tx.commit().await.expect("Failed to commit transaction");

    HttpResponse::Ok().json(quotation)
}

#[utoipa::path(
    get,
    path = "/api/quotations",
    responses(
        (status = 200, description = "List all quotations", body = [Quotation]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Quotations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_quotations(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as!(
        Quotation,
        "SELECT * FROM quotations ORDER BY created_at DESC"
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(quotations) => HttpResponse::Ok().json(quotations),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    patch,
    path = "/api/quotations/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Quotation Database ID")
    ),
    request_body = UpdateQuotationStatusSchema,
    responses(
        (status = 200, description = "Quotation status updated", body = Quotation),
        (status = 404, description = "Quotation not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Quotations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_quotation_status(
    path: web::Path<Uuid>,
    body: web::Json<UpdateQuotationStatusSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let quotation_id = path.into_inner();

    let result = sqlx::query_as!(
        Quotation,
        "UPDATE quotations SET status = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING *",
        body.status,
        quotation_id
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(q)) => HttpResponse::Ok().json(q),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "Quotation not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    post,
    path = "/api/quotations/{id}/convert",
    params(
        ("id" = Uuid, Path, description = "Quotation Database ID")
    ),
    responses(
        (status = 200, description = "Quotation converted to sale successfully"),
        (status = 404, description = "Quotation not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Quotations",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn convert_to_sale(path: web::Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
    let quotation_id = path.into_inner();

    let mut tx = data.db.begin().await.expect("Failed to start transaction");

    // 1. Fetch Quotation
    let q = sqlx::query_as!(
        Quotation,
        "SELECT * FROM quotations WHERE id = $1",
        quotation_id
    )
    .fetch_optional(&mut *tx)
    .await;

    let quotation = match q {
        Ok(Some(quote)) => quote,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Quotation not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    if quotation.status.as_deref() == Some("ACCEPTED") {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Quotation already converted to sale"}));
    }

    // 2. Fetch Quotation Items
    let items = sqlx::query_as!(
        QuotationItem,
        "SELECT * FROM quotation_items WHERE quotation_id = $1",
        quotation_id
    )
    .fetch_all(&mut *tx)
    .await
    .unwrap_or_default();

    // 3. Check Stock for all items (with locking)
    for item in &items {
        if let Some(pid) = item.product_id {
            let product = sqlx::query!(
                "SELECT name, quantity, is_active FROM products WHERE id = $1 FOR UPDATE",
                pid
            )
            .fetch_one(&mut *tx)
            .await
            .expect("Failed to fetch product");

            if !product.is_active.unwrap_or(true) {
                return HttpResponse::BadRequest().json(json!({
                    "error": format!("Product '{}' is not active for sale", product.name)
                }));
            }

            if product.quantity <= 0 {
                return HttpResponse::BadRequest().json(json!({
                    "error": format!("Product '{}' is out of stock", product.name)
                }));
            }

            if product.quantity < item.quantity {
                return HttpResponse::BadRequest().json(json!({
                    "error": format!("Insufficient stock for product '{}'. Available: {}, Required: {}", product.name, product.quantity, item.quantity)
                }));
            }
        }
    }

    // 4. Create Sale
    let sale_id = Uuid::new_v4();
    let payment_amount = quotation.total_amount * quotation.exchange_rate;
    let payment_currency = quotation.currency_code.clone();

    // Use query_as! with all required fields to match schema
    sqlx::query!(
        "INSERT INTO sales (id, total_amount, currency_code, exchange_rate, status, payment_amount, payment_currency, payment_method) 
         VALUES ($1, $2, $3, $4, 'PENDING', $5, $6, 'CASH')",
        sale_id,
        quotation.total_amount, // Stored in USD base currency
        quotation.currency_code,
        quotation.exchange_rate,
        payment_amount,
        payment_currency,
    )
    .execute(&mut *tx)
    .await
    .expect("Failed to create sale");

    // 5. Create Sale Items and Update Stock
    for item in items {
        if let Some(pid) = item.product_id {
            sqlx::query!(
                "INSERT INTO sale_items (id, sale_id, product_id, quantity, unit_price, subtotal) 
                 VALUES ($1, $2, $3, $4, $5, $6)",
                Uuid::new_v4(),
                sale_id,
                pid,
                item.quantity,
                item.unit_price, // Already in USD from QuotationItem logic
                item.subtotal   // Already in USD from QuotationItem logic
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to insert sale item");

            sqlx::query!(
                "UPDATE products SET quantity = quantity - $1 WHERE id = $2",
                item.quantity,
                pid
            )
            .execute(&mut *tx)
            .await
            .expect("Failed to update stock");
        }
    }

    // 6. Update Quotation Status
    sqlx::query!(
        "UPDATE quotations SET status = 'ACCEPTED', updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        quotation_id
    )
    .execute(&mut *tx)
    .await
    .expect("Failed to update quotation");

    tx.commit().await.expect("Failed to commit transaction");

    HttpResponse::Ok().json(json!({
        "message": "Quotation successfully converted to sale",
        "sale_id": sale_id
    }))
}
