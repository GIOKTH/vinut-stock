use crate::config::Config;
// Removed unused User import
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Claims {
    pub sub: String, // user_id
    pub role: String,
    pub exp: usize,
}

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}

pub fn create_jwt(user_id: String, role: String, config: &Config) -> String {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        role,
        exp: expiration as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
    )
    .expect("Token creation failed")
}

pub fn decode_jwt(token: &str, config: &Config) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}
