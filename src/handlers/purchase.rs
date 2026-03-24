use crate::db::AppState;
use crate::models::purchase::{CreatePurchaseSchema, Purchase};
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/purchases",
    request_body = CreatePurchaseSchema,
    responses(
        (status = 200, description = "Purchase created successfully", body = Purchase),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Purchases",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_purchase(
    body: web::Json<CreatePurchaseSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let purchase_id = Uuid::new_v4();

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

    // Calculate totals
    let mut total_quantity = 0;
    let mut total_amount_foreign = Decimal::new(0, 2);

    for item in &body.items {
        total_quantity += item.quantity;
        total_amount_foreign += item.buy_price * Decimal::from(item.quantity);
    }

    if total_quantity == 0 {
        return HttpResponse::BadRequest()
            .json(json!({"error": "Purchase must have at least one item with quantity > 0"}));
    }

    let shipping_cost = body.shipping_cost.unwrap_or(Decimal::new(0, 0));
    let tax_rate = body.tax_rate.unwrap_or(Decimal::new(0, 0));

    // Start transaction
    let mut tx = data.db.begin().await.expect("Failed to start transaction");

    let status = "PENDING".to_string();

    let purchase_result: Result<Purchase, sqlx::Error> = sqlx::query_as!(
        Purchase,
        "INSERT INTO purchases (id, supplier_id, total_amount, currency_code, exchange_rate, shipping_cost, tax_rate, status) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *",
        purchase_id,
        body.supplier_id,
        total_amount_foreign,
        currency,
        exchange_rate,
        shipping_cost,
        tax_rate,
        status
    )
    .fetch_one(&mut *tx)
    .await;

    let purchase = match purchase_result {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Failed to insert purchase header: {}", e)}));
        }
    };

    // Process Items
    for item in &body.items {
        // Distribute shipping cost per unit: total_shipping / total_quantity
        let unit_shipping_foreign = shipping_cost / Decimal::from(total_quantity);

        // Calculate Line Item Subtotal (Foreign)
        let subtotal_foreign = item.buy_price * Decimal::from(item.quantity);

        // Calculate Landed Cost PER UNIT (Local Currency)
        // Unit Tax = Unit Price * (Tax Rate / 100)
        let unit_tax_foreign = item.buy_price * (tax_rate / Decimal::from(100));

        // Landed Cost (Foreign) = Price + Tax + Shipping
        let landed_cost_foreign = item.buy_price + unit_tax_foreign + unit_shipping_foreign;

        // Landed Cost (Local) = Landed Cost (Foreign) / Exchange Rate  (Since Rate is Local/Foreign? No, wait.
        // If 1 USD = 30 THB. Rate is 30.
        // Price included in foreign (e.g. 10 USD).
        // Local Price = 10 * 30 = 300 THB.
        // Previous logic: "landed_unit_cost = ... / exchange_rate".
        // Let's check update_exchange_rate logic. "1 USD = 30 THB". rate_to_base = 30?
        // IF rate_to_base is "How many Base units for 1 Foreign unit", then multiply.
        // IF rate_to_base is "How many Foreign units for 1 Base unit", then divide.
        // Standard finance: Exchange Rate usually "Local per Foreign". 1 USD = 35 THB.
        // Let's assume multiplication is correct for converting Foreign -> Base.
        // WAIT. The previous code did DIVIDE. `(body.buy_price ... ) / exchange_rate;`
        // Let's look at `simulate_data.rs`: `let total_sale_amount_local = total_sale_amount_base * exchange_rate;`
        // Validation:
        // Sales: Base -> Local. Multiply. (Price in Base * Rate = Price in Local).
        // Purchases: Local (Foreign) -> Base. Divide?
        // No, if I buy in USD (Foreign), and Base is THB.
        // I pay 10 USD. Rate 35. Cost is 350 THB.
        // So Foreign * Rate = Base.
        // Previous code: `let landed_unit_cost = (body.buy_price + ... ) / exchange_rate;`
        // If Buy Price 350 THB (Local/Foreign?), Rate 35. Result 10 USD (Base).
        // Ah, maybe the previous code assumed "Foreign" meant "The currency of the transaction", and Base is "System Base".
        // If transaction is in THB (Rate 1), and Base is THB. Cost = Price / 1. Correct.
        // If transaction is in USD (Rate 30), and Base is THB.
        // Cost (THB) = Price (USD) * 30.
        // So it should be MULTIPLICATION.
        // Why did previous code divide?
        // `get_product_purchases` logic isn't there yet.
        // Let's look at `sales`.
        // `total_sale_amount_local = total_sale_amount_base * exchange_rate;`
        // Sale is stored in local? `INSERT INTO sales ... total_amount ...`
        // If I sell for 100 USD. Rate 30.
        // Base value?
        // Let's stick to: BASE CURRENCY is the accounting currency (e.g. LAK/THB).
        // If I buy in USD.
        // Cost in Base = Cost in USD * Exchange Rate.
        // I will use MULTIPLICATION.

        // Correcting logic: Cost in Base = Foreign Cost * Rate.
        let landed_cost_base = landed_cost_foreign * exchange_rate;

        // Insert Purchase Item with dynamic pricing attributes saved for later
        sqlx::query!(
            "INSERT INTO purchase_items (id, purchase_id, product_id, quantity, buy_price, subtotal, 
             new_sale_price, new_commission_price, new_promotion_price, landed_cost_base) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            Uuid::new_v4(),
            purchase_id,
            item.product_id,
            item.quantity,
            item.buy_price,
            subtotal_foreign,
            item.new_sale_price,
            item.new_commission_price,
            item.new_promotion_price,
            landed_cost_base
        )
        .execute(&mut *tx)
        .await
        .expect("Failed to insert purchase item");
    }

    tx.commit().await.expect("Failed to commit transaction");

    HttpResponse::Ok().json(purchase)
}

#[utoipa::path(
    get,
    path = "/api/purchases",
    responses(
        (status = 200, description = "List all purchases", body = [Purchase]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Purchases",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_purchases(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query_as!(Purchase, "SELECT * FROM purchases ORDER BY created_at DESC")
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(purchases) => HttpResponse::Ok().json(purchases),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PurchaseDetailResponse {
    pub purchase: Purchase,
    pub items: Vec<PurchaseItemDetail>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct PurchaseItemDetail {
    pub id: Uuid,
    pub product_name: String,
    pub product_code: String,
    pub quantity: i32,
    pub buy_price: Decimal,
    pub subtotal: Decimal,
}

#[utoipa::path(
    get,
    path = "/api/purchases/{id}",
    params(
        ("id" = Uuid, Path, description = "Purchase ID")
    ),
    responses(
        (status = 200, description = "Purchase details fetched"),
        (status = 404, description = "Purchase not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Purchases",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_purchase_details(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let purchase_id = path.into_inner();

    let purchase = sqlx::query_as!(Purchase, "SELECT * FROM purchases WHERE id = $1", purchase_id)
        .fetch_optional(&data.db)
        .await;

    let purchase = match purchase {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Purchase not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    };

    let items = sqlx::query_as!(
        PurchaseItemDetail,
        r#"SELECT 
            pi.id, 
            p.name as product_name, 
            p.code as product_code, 
            pi.quantity, 
            pi.buy_price, 
            pi.subtotal 
           FROM purchase_items pi
           JOIN products p ON pi.product_id = p.id
           WHERE pi.purchase_id = $1"#,
        purchase_id
    )
    .fetch_all(&data.db)
    .await;

    match items {
        Ok(items) => HttpResponse::Ok().json(PurchaseDetailResponse { purchase, items }),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

/// Receive a PENDING purchase, updating inventory and product prices.
#[utoipa::path(
    patch,
    path = "/api/purchases/{id}/receive",
    responses(
        (status = 200, description = "Purchase items received and inventory updated", body = Purchase),
        (status = 404, description = "Purchase not found"),
        (status = 400, description = "Purchase is not in PENDING state")
    )
)]
pub async fn receive_purchase(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let purchase_id = path.into_inner();

    let purchase = sqlx::query_as!(
        Purchase,
        "SELECT * FROM purchases WHERE id = $1",
        purchase_id
    )
    .fetch_optional(&data.db)
    .await;

    let purchase = match purchase {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Purchase not found"})),
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to fetch purchase: {}", e)})),
    };

    if purchase.status.as_deref() != Some("PENDING") {
        return HttpResponse::BadRequest().json(json!({"error": "Purchase is already received or cancelled"}));
    }

    let mut tx = data.db.begin().await.expect("Failed to begin transaction");

    // Fetch items to update inventory and cost prices
    // Because we just added new fields, we need to SELECT them
    let items = sqlx::query!(
        "SELECT product_id, quantity, new_sale_price, new_commission_price, new_promotion_price, landed_cost_base 
         FROM purchase_items WHERE purchase_id = $1 AND product_id IS NOT NULL",
        purchase_id
    )
    .fetch_all(&mut *tx)
    .await;

    let items = match items {
        Ok(i) => i,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to fetch items: {}", e)}));
        }
    };

    for item in items {
        if let Some(product_id) = item.product_id {
            let mut product_update_query = "UPDATE products SET quantity = quantity + $1, cost_price = $2".to_string();
            let mut param_index = 3;

            if item.new_sale_price.is_some() {
                product_update_query.push_str(&format!(", sale_price = ${}", param_index));
                param_index += 1;
            }
            if item.new_commission_price.is_some() {
                product_update_query.push_str(&format!(", commission_price = ${}", param_index));
                param_index += 1;
            }
            if item.new_promotion_price.is_some() {
                product_update_query.push_str(&format!(", promotion_price = ${}", param_index));
                param_index += 1;
            }

            product_update_query.push_str(&format!(", updated_at = CURRENT_TIMESTAMP WHERE id = ${}", param_index));

            let mut q = sqlx::query(&product_update_query)
                .bind(item.quantity)
                .bind(item.landed_cost_base.unwrap_or(rust_decimal::Decimal::new(0, 0)));

            if let Some(sp) = item.new_sale_price {
                q = q.bind(sp);
            }
            if let Some(cp) = item.new_commission_price {
                q = q.bind(cp);
            }
            if let Some(pp) = item.new_promotion_price {
                q = q.bind(pp);
            }

            let update_result = q.bind(product_id).execute(&mut *tx).await;

            if let Err(e) = update_result {
                let _ = tx.rollback().await;
                return HttpResponse::InternalServerError().json(
                    json!({"error": format!("Failed to update product {}: {}", product_id, e)}),
                );
            }
        }
    }

    // Mark purchase as RECEIVED
    let updated_purchase = sqlx::query_as!(
        Purchase,
        "UPDATE purchases SET status = 'RECEIVED' WHERE id = $1 RETURNING *",
        purchase_id
    )
    .fetch_one(&mut *tx)
    .await;

    let updated_purchase = match updated_purchase {
        Ok(p) => p,
        Err(e) => {
            let _ = tx.rollback().await;
            return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to update purchase status: {}", e)}));
        }
    };

    tx.commit().await.expect("Failed to commit transaction");

    HttpResponse::Ok().json(updated_purchase)
}
