use crate::db::AppState;
use crate::models::user::{CreateUserSchema, LoginSchema, User, UserResponse};
use crate::security::{create_jwt, decode_jwt, hash_password, verify_password};
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = CreateUserSchema,
    responses(
        (status = 200, description = "User registered successfully", body = UserResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn register_user(
    body: web::Json<CreateUserSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let hashed_password = hash_password(&body.password);
    let user_id = Uuid::new_v4();

    let query_result: Result<User, sqlx::Error> = sqlx::query_as!(
        User,
        "INSERT INTO users (id, username, password_hash, role, is_blocked) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        user_id,
        body.username,
        hashed_password,
        body.role,
        false
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                role: user.role,
                is_blocked: user.is_blocked,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            // Check for unique constraint violation (code 23505)
            // Error handling is a bit specific with sqlx, keeping it simple for now
            HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginSchema,
    responses(
        (status = 200, description = "Login successful", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn login_user(body: web::Json<LoginSchema>, data: web::Data<AppState>) -> impl Responder {
    let user_result = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = $1",
        body.username
    )
    .fetch_optional(&data.db)
    .await;

    match user_result {
        Ok(Some(user)) => {
            if user.is_blocked.unwrap_or(false) {
                return HttpResponse::Forbidden().json(json!({"error": "Your account is blocked"}));
            }

            if verify_password(&body.password, &user.password_hash) {
                let token = create_jwt(user.id.to_string(), user.role, &data.env);
                HttpResponse::Ok().json(json!({"token": token}))
            } else {
                HttpResponse::Unauthorized().json(json!({"error": "Invalid credentials"}))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user info fetched successfully", body = UserResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_me(req: actix_web::HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let auth_header = match req.headers().get("Authorization") {
        Some(h) => match h.to_str() {
            Ok(v) => v,
            Err(_) => {
                return HttpResponse::Unauthorized().json(json!({"error": "Invalid auth header"}))
            }
        },
        None => return HttpResponse::Unauthorized().json(json!({"error": "No auth header found"})),
    };

    let token = auth_header.replace("Bearer ", "");
    let claims = match decode_jwt(&token, &data.env) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::Unauthorized()
                .json(json!({"error": format!("Invalid token: {}", e)}))
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": "ID in token is invalid"}))
        }
    };

    let user_result = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_optional(&data.db)
        .await;

    match user_result {
        Ok(Some(user)) => {
            let response = UserResponse {
                id: user.id,
                username: user.username,
                role: user.role,
                is_blocked: user.is_blocked,
            };
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(json!({"error": "User not found"})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e.to_string()})),
    }
}
