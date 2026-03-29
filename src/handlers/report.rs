use crate::db::AppState;
use crate::models::sale::{SaleDetailResponse, SaleItemResponse, SaleResponse, SalesQuerySchema};
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use chrono::{Utc};

#[utoipa::path(
    get,
    path = "/api/dashboard/summary",
    responses(
        (status = 200, description = "Dashboard summary fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_dashboard_summary(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query!("SELECT * FROM dashboard_stats")
        .fetch_one(&data.db)
        .await;

    match result {
        Ok(stats) => {
            // Get Top 5 best sellers today
            let top_sellers = sqlx::query!(
                "SELECT p.name, SUM(si.quantity) as total_qty 
                 FROM sale_items si 
                 JOIN products p ON si.product_id = p.id 
                 JOIN sales s ON si.sale_id = s.id 
                 WHERE s.created_at >= CURRENT_DATE AND s.status = 'COMPLETED'
                 GROUP BY p.name 
                 ORDER BY total_qty DESC 
                 LIMIT 5"
            )
            .fetch_all(&data.db)
            .await
            .unwrap_or_default();

            // Get sales and profit summary by currency today using actual payment amount
            let currency_summary = sqlx::query!(
                "SELECT 
                    COALESCE(s.payment_currency, 'USD') as currency_code, 
                    SUM(s.payment_amount) as total_sales,
                    SUM(s.payment_amount - (
                        SELECT COALESCE(SUM(si.quantity * p.cost_price), 0)
                        FROM sale_items si
                        JOIN products p ON si.product_id = p.id
                        WHERE si.sale_id = s.id
                    ) * s.exchange_rate) as total_profit
                 FROM sales s
                 WHERE s.created_at >= CURRENT_DATE AND s.status = 'COMPLETED'
                 GROUP BY s.payment_currency"
            )
            .fetch_all(&data.db)
            .await
            .unwrap_or_default();

            // Get low stock items details
            let low_stock_items = sqlx::query!(
                "SELECT name, quantity, low_stock_threshold 
                 FROM products 
                 WHERE quantity < low_stock_threshold"
            )
            .fetch_all(&data.db)
            .await
            .unwrap_or_default();

            // Get all exchange rates for the currency board
            let exchange_rates = sqlx::query!(
                "SELECT currency_code, rate_to_base FROM exchange_rates"
            )
            .fetch_all(&data.db)
            .await
            .unwrap_or_default();

            HttpResponse::Ok().json(json!({
                "daily_sales_total": stats.total_sales_today,
                "daily_profit_total": stats.total_profit_today,
                "best_selling_product": stats.best_selling_product,
                "low_stock_count": stats.low_stock_count,
                "top_5_best_sellers": top_sellers.into_iter().map(|s| json!({"name": s.name, "quantity": s.total_qty})).collect::<Vec<_>>(),
                "summary_by_currency": currency_summary.into_iter().map(|c| json!({
                    "currency": c.currency_code, 
                    "total_sales": c.total_sales,
                    "total_profit": c.total_profit
                })).collect::<Vec<_>>(),
                "low_stock_details": low_stock_items.into_iter().map(|i| json!({
                    "name": i.name,
                    "quantity": i.quantity,
                    "threshold": i.low_stock_threshold
                })).collect::<Vec<_>>(),
                "exchange_rates": exchange_rates.into_iter().map(|r| json!({
                    "currency": r.currency_code,
                    "rate": r.rate_to_base
                })).collect::<Vec<_>>(),
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/reports/products",
    responses(
        (status = 200, description = "Product performance reports fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_product_reports(data: web::Data<AppState>) -> impl Responder {
    let result: Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> =
        sqlx::query("SELECT * FROM product_performance ORDER BY total_sold DESC")
            .fetch_all(&data.db)
            .await;

    match result {
        Ok(rows) => {
            use sqlx::Row;
            let reports: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "product": r.get::<String, _>("product_name"),
                        "total_sold": r.get::<i64, _>("total_sold"),
                        "total_revenue": r.get::<Option<rust_decimal::Decimal>, _>("total_revenue"),
                        "total_profit": r.get::<Option<rust_decimal::Decimal>, _>("total_profit"),
                        "current_stock": r.get::<i32, _>("current_stock"),
                        "is_low_stock": r.get::<bool, _>("is_low_stock"),
                        "is_active": r.get::<Option<bool>, _>("is_active")
                    })
                })
                .collect();
            HttpResponse::Ok().json(reports)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/reports/low-stock",
    responses(
        (status = 200, description = "Low stock report fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_low_stock_report(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query!(
        "SELECT name, code, quantity, low_stock_threshold FROM products WHERE quantity <= low_stock_threshold"
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(rows) => {
            let products: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "name": r.name,
                        "code": r.code,
                        "quantity": r.quantity,
                        "low_stock_threshold": r.low_stock_threshold
                    })
                })
                .collect();
            HttpResponse::Ok().json(products)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/reports/sales/summary",
    params(SalesQuerySchema),
    responses(
        (status = 200, description = "Sales summary report fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports",
    security(("bearer_auth" = []))
)]
pub async fn get_sales_summary_report(
    query: web::Query<SalesQuerySchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_date = query
        .start_date
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

    let end_date = query
        .end_date
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|| Utc::now());

    let result = sqlx::query!(
        r#"SELECT 
            COALESCE(s.payment_currency, 'USD') as currency,
            COUNT(*) FILTER (WHERE s.status = 'COMPLETED') as completed_count,
            COUNT(*) FILTER (WHERE s.status = 'PENDING') as pending_count,
            SUM(s.payment_amount) as total_sales,
            SUM(s.payment_amount - (
                SELECT COALESCE(SUM(si.quantity * p.cost_price), 0)
                FROM sale_items si
                JOIN products p ON si.product_id = p.id
                WHERE si.sale_id = s.id
            ) * s.exchange_rate) as total_profit
         FROM sales s
         WHERE s.created_at >= $1 AND s.created_at <= $2
         GROUP BY COALESCE(s.payment_currency, 'USD')"#,
        start_date,
        end_date
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(rows) => {
            let summary: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "currency": r.currency,
                        "completed_count": r.completed_count,
                        "pending_count": r.pending_count,
                        "total_sales": r.total_sales,
                        "total_profit": r.total_profit
                    })
                })
                .collect();
            HttpResponse::Ok().json(summary)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/reports/sales/detailed",
    params(SalesQuerySchema),
    responses(
        (status = 200, description = "Detailed sales report fetched successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Reports",
    security(("bearer_auth" = []))
)]
pub async fn get_sales_detailed_report(
    query: web::Query<SalesQuerySchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_date = query
        .start_date
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

    let end_date = query
        .end_date
        .and_then(|d| d.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc())
        .unwrap_or_else(|| Utc::now());

    // Fetch sales headers
    let sales_result = sqlx::query_as!(
        SaleResponse,
        "SELECT s.id, s.user_id, u.username as \"username?\", s.total_amount, s.discount_amount, s.promotion_code, s.payment_method, s.currency_code, s.exchange_rate, s.status, s.payment_amount, s.payment_currency, s.created_at FROM sales s LEFT JOIN users u ON s.user_id = u.id WHERE s.created_at >= $1 AND s.created_at <= $2 ORDER BY s.created_at ASC",
        start_date,
        end_date
    )
    .fetch_all(&data.db)
    .await;

    match sales_result {
        Ok(sales) => {
            let mut report = Vec::new();
            for sale in sales {
                // Fetch items for each sale
                let items = sqlx::query!(
                    r#"SELECT si.id, si.sale_id, si.product_id, p.name as product_name, p.code as product_code, si.quantity, si.unit_price, si.subtotal 
                     FROM sale_items si 
                     LEFT JOIN products p ON si.product_id = p.id 
                     WHERE si.sale_id = $1"#,
                    sale.id
                )
                .fetch_all(&data.db)
                .await
                .unwrap_or_default();

                let item_responses: Vec<_> = items.into_iter().map(|item| SaleItemResponse {
                    id: item.id,
                    sale_id: item.sale_id.unwrap_or_default(),
                    product_id: item.product_id,
                    product_name: Some(item.product_name),
                    product_code: Some(item.product_code),
                    quantity: item.quantity,
                    unit_price: item.unit_price,
                    subtotal: item.subtotal,
                }).collect();

                report.push(SaleDetailResponse { sale, items: item_responses });
            }
            HttpResponse::Ok().json(report)
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
