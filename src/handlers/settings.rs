use crate::db::AppState;
use crate::models::user::UserResponse;
use crate::security::Claims;
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct ExchangeRate {
    pub currency_code: String,
    pub rate_to_base: Decimal,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct UpdateExchangeRateSchema {
    pub rate_to_base: Decimal,
}

#[utoipa::path(
    get,
    path = "/api/settings/exchange",
    responses(
        (status = 200, description = "List all exchange rates", body = [ExchangeRate]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_exchange_rates(data: web::Data<AppState>) -> impl Responder {
    let result: Result<Vec<ExchangeRate>, sqlx::Error> =
        sqlx::query_as!(ExchangeRate, "SELECT * FROM exchange_rates")
            .fetch_all(&data.db)
            .await;

    match result {
        Ok(rates) => HttpResponse::Ok().json(rates),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    post,
    path = "/api/settings/exchange/{currency}",
    params(
        ("currency" = String, Path, description = "Currency code (e.g. USD)")
    ),
    request_body = UpdateExchangeRateSchema,
    responses(
        (status = 200, description = "Exchange rate updated"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_exchange_rate(
    path: web::Path<String>,
    body: web::Json<UpdateExchangeRateSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let currency_code = path.into_inner();
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query!(
        "INSERT INTO exchange_rates (currency_code, rate_to_base) VALUES ($1, $2)
         ON CONFLICT (currency_code) DO UPDATE SET rate_to_base = EXCLUDED.rate_to_base",
        currency_code,
        body.rate_to_base
    )
    .execute(&data.db)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "Rate updated"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ChangeRoleSchema {
    pub role: String,
}

#[utoipa::path(
    get,
    path = "/api/settings/users",
    responses(
        (status = 200, description = "List all users for management", body = [UserResponse]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_users(data: web::Data<AppState>, claims: web::ReqData<Claims>) -> impl Responder {
    let me = Uuid::parse_str(&claims.into_inner().sub).ok();
    
    let result: Result<Vec<UserResponse>, sqlx::Error> = sqlx::query_as!(
        UserResponse,
        r#"SELECT 
            u.id, 
            u.username, 
            u.role, 
            u.is_blocked,
            EXISTS (SELECT 1 FROM sales s WHERE s.user_id = u.id) as "has_sales!"
           FROM users u 
           WHERE u.id != $1 
           ORDER BY u.created_at DESC"#,
        me
    )
    .fetch_all(&data.db)
    .await;

    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[utoipa::path(
    post,
    path = "/api/settings/users/{id}/block",
    params(
        ("id" = Uuid, Path, description = "User Database ID")
    ),
    responses(
        (status = 200, description = "User blocked successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn block_user(
    path: web::Path<Uuid>, 
    data: web::Data<AppState>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = path.into_inner();
    let me = Uuid::parse_str(&claims.into_inner().sub).ok();

    if Some(user_id) == me {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot block yourself!"}));
    }

    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> =
        sqlx::query!("UPDATE users SET is_blocked = TRUE WHERE id = $1", user_id)
            .execute(&data.db)
            .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "User blocked" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[utoipa::path(
    post,
    path = "/api/settings/users/{id}/unblock",
    params(
        ("id" = Uuid, Path, description = "User Database ID")
    ),
    responses(
        (status = 200, description = "User unblocked successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn unblock_user(
    path: web::Path<uuid::Uuid>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> =
        sqlx::query!("UPDATE users SET is_blocked = FALSE WHERE id = $1", user_id)
            .execute(&data.db)
            .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "User unblocked" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[utoipa::path(
    put,
    path = "/api/settings/users/{id}/role",
    params(
        ("id" = Uuid, Path, description = "User Database ID")
    ),
    request_body = ChangeRoleSchema,
    responses(
        (status = 200, description = "User role updated successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_user_role(
    path: web::Path<Uuid>,
    body: web::Json<ChangeRoleSchema>,
    data: web::Data<AppState>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = path.into_inner();
    let me = Uuid::parse_str(&claims.into_inner().sub).ok();

    if Some(user_id) == me {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot change your own role!"}));
    }

    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> = sqlx::query!(
        "UPDATE users SET role = $1 WHERE id = $2",
        body.role,
        user_id
    )
    .execute(&data.db)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "Role updated" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

#[utoipa::path(
    delete,
    path = "/api/settings/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User Database ID")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Settings",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    path: web::Path<Uuid>, 
    data: web::Data<AppState>,
    claims: web::ReqData<Claims>,
) -> impl Responder {
    let user_id = path.into_inner();
    let me = Uuid::parse_str(&claims.into_inner().sub).ok();

    if Some(user_id) == me {
        return HttpResponse::BadRequest().json(json!({"error": "Cannot delete yourself!"}));
    }

    // Check if user has sales records before deleting
    let has_sales = sqlx::query!(
        "SELECT id FROM sales WHERE user_id = $1 LIMIT 1",
        user_id
    )
    .fetch_optional(&data.db)
    .await;

    match has_sales {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Cannot delete user with existing sales records. Please block the user instead."
            }));
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
        Ok(None) => {}
    }

    let result: Result<sqlx::postgres::PgQueryResult, sqlx::Error> =
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&data.db)
            .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({ "message": "User deleted successfully" })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}
