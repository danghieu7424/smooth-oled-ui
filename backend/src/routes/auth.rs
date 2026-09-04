use axum::{
    extract::State,
    response::{Redirect, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;
use serde::Deserialize;
use reqwest::Client;
use crate::core::state::AppState;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/google", get(google_login))
        .route("/google/callback", get(google_callback))
        .route("/login_with_token", axum::routing::post(login_with_token))
        .route("/me", get(get_me))
        .route("/logout", get(logout))
}

#[derive(Deserialize)]
struct AuthRequest {
    code: Option<String>,
    error: Option<String>,
}

async fn google_login() -> impl IntoResponse {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:7424/api/auth/google/callback".to_string());
    
    let url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=token&scope=email%20profile",
        client_id, redirect_uri
    );
    
    Redirect::to(&url)
}

#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct UserInfo {
    id: String,
    email: String,
    name: String,
    picture: String,
}

async fn google_callback() -> impl IntoResponse {
    let html = r#"
        <html><head><script>
            if (window.opener) { window.opener.postMessage('login_success' + window.location.hash, '*'); window.close(); }
            else { window.location.href = 'http://localhost:8080/'; }
        </script></head><body>Đang xử lý đăng nhập...</body></html>
    "#;
    axum::response::Html(html)
}

#[derive(Deserialize)]
struct TokenLoginReq {
    access_token: String,
}

async fn login_with_token(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    axum::extract::Json(payload): axum::extract::Json<TokenLoginReq>,
) -> impl IntoResponse {
    let client = Client::new();
    let user_res = client.get("https://www.googleapis.com/oauth2/v1/userinfo?alt=json")
        .bearer_auth(payload.access_token)
        .send()
        .await;

    if let Ok(user_info_resp) = user_res {
        if let Ok(user_info) = user_info_resp.json::<UserInfo>().await {
            let _ = state.storage.execute_query({
                let email = user_info.email.clone();
                let google_id = user_info.id.clone();
                
                // Parse a substring of google_id into u64 to ensure it's stable and fits in u64
                let hash_val = google_id.chars().take(18).collect::<String>().parse::<u64>().unwrap_or(0);
                let new_suid = crate::helpers::suid::to_base62_ordered(hash_val);
                
                let name = user_info.name.clone();
                let picture = user_info.picture.clone();
                
                move |conn| {
                    conn.execute(
                        "INSERT OR IGNORE INTO users (email, google_id, role, is_verified, suid, name, picture) VALUES (?1, ?2, 'user', 1, ?3, ?4, ?5)",
                        [&email, &google_id, &new_suid, &name, &picture]
                    )?;
                    conn.execute("UPDATE users SET suid = ?1, name = ?3, picture = ?4 WHERE email = ?2", [&new_suid, &email, &name, &picture])?;
                    Ok(())
                }
            }).await;

            let user_id: i64 = state.storage.execute_query({
                let email = user_info.email.clone();
                move |conn| {
                    conn.query_row("SELECT id FROM users WHERE email = ?1", [&email], |r| r.get(0))
                }
            }).await.unwrap_or(1);

            let user_suid: String = state.storage.execute_query({
                let email = user_info.email.clone();
                move |conn| {
                    conn.query_row("SELECT suid FROM users WHERE email = ?1", [&email], |r| r.get(0))
                }
            }).await.unwrap_or_else(|_| crate::helpers::suid::suid());

            let token = crate::auth::token::generate_user_token(user_id, user_suid.clone(), "user".to_string(), &state.server_sk_bytes).unwrap_or_default();
            
            let cookie = Cookie::build(("auth_token", token))
                .path("/")
                .http_only(false) 
                .same_site(SameSite::Lax)
                .build();

            // We no longer need the user_info cookie, just auth_token is enough
            return (jar.add(cookie), axum::response::Json(serde_json::json!({"success": true}))).into_response();
        }
    }
    
    (axum::http::StatusCode::UNAUTHORIZED, jar, axum::response::Json(serde_json::json!({"error": "Invalid token"}))).into_response()
}

async fn get_me(
    State(state): State<Arc<AppState>>,
    jar: CookieJar
) -> axum::response::Json<serde_json::Value> {
    if let Some(cookie) = jar.get("auth_token") {
        if let Ok(token_data) = crate::auth::token::verify_user_token(cookie.value()) {
            let suid = token_data.claims.suid;
            
            let user_data: Result<(String, String), _> = state.storage.execute_query({
                let s = suid.clone();
                move |conn| {
                    conn.query_row("SELECT name, picture FROM users WHERE suid = ?1", [&s], |r| {
                        Ok((
                            r.get::<_, Option<String>>(0)?.unwrap_or_else(|| "User".to_string()),
                            r.get::<_, Option<String>>(1)?.unwrap_or_default()
                        ))
                    })
                }
            }).await;

            if let Ok((name, picture)) = user_data {
                return axum::response::Json(serde_json::json!({
                    "name": name,
                    "picture": picture,
                    "id": suid
                }));
            }
        }
    }
    
    axum::response::Json(serde_json::json!({ "error": "Not logged in" }))
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let mut clean_jar = jar.clone();
    clean_jar = clean_jar.remove(Cookie::from("auth_token"));
    clean_jar = clean_jar.remove(Cookie::from("user_info"));
    
    (clean_jar, Redirect::to("http://localhost:8080/login"))
}
