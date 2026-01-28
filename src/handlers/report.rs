use crate::db::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

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

            // Get sales and profit summary by currency today
            let currency_summary = sqlx::query!(
                "SELECT 
                    s.currency_code, 
                    SUM(s.total_amount) as total_sales,
                    SUM(si.quantity * (si.unit_price - (p.cost_price * s.exchange_rate))) as total_profit
                 FROM sales s
                 JOIN sale_items si ON s.id = si.sale_id
                 JOIN products p ON si.product_id = p.id
                 WHERE s.created_at >= CURRENT_DATE AND s.status = 'COMPLETED'
                 GROUP BY s.currency_code"
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
    let result = sqlx::query!("SELECT * FROM product_performance ORDER BY total_sold DESC")
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(rows) => {
            let reports: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    json!({
                        "product": r.product_name,
                        "total_sold": r.total_sold,
                        "total_revenue": r.total_revenue,
                        "total_profit": r.total_profit
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
