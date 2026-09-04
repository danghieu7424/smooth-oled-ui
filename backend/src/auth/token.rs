// src/utils/token.rs
#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use jsonwebtoken::{EncodingKey, DecodingKey, Header, Validation, encode, decode, TokenData, errors::Error as JwtError};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoClaims {
    pub sub: String,       // file path relative (ví dụ: "videos/abc.mp4")
    pub exp: usize,        // epoch seconds
    pub ip: Option<String> // optional bound IP as string
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: i64,          // user_id
    pub suid: String,      // suid
    pub role: String,      // role
    pub exp: usize,        // epoch seconds
}

fn jwt_secret() -> String {
    env::var("JWT_SECRET").unwrap_or_else(|_| "OTA_HUB_SECRET_KEY_FOR_DEV_987654321".to_string())
}

/// Tạo token với expiry_seconds và optional ip
pub fn create_signed_token(file_path: &str, expiry_seconds: i64, ip: Option<String>) -> Result<String, JwtError> {
    let secret = jwt_secret();
    let exp = (Utc::now() + Duration::seconds(expiry_seconds)).timestamp() as usize;
    let claims = VideoClaims {
        sub: file_path.to_string(),
        exp,
        ip,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_token(token: &str) -> Result<TokenData<VideoClaims>, JwtError> {
    let secret = jwt_secret();
    let validation = Validation::default();
    decode::<VideoClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
}

pub fn generate_user_token(user_id: i64, suid: String, role: String, _server_sk: &[u8]) -> Result<String, JwtError> {
    let secret = jwt_secret();
    let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
    let claims = UserClaims {
        sub: user_id,
        suid,
        role,
        exp,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub fn verify_user_token(token: &str) -> Result<TokenData<UserClaims>, JwtError> {
    let secret = jwt_secret();
    let validation = Validation::default();
    decode::<UserClaims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
}
