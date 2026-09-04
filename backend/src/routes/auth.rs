use axum::{
    extract::{Query, State},
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
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=email%20profile&access_type=offline",
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

async fn google_callback(
    Query(query): Query<AuthRequest>,
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> impl IntoResponse {
    let success_html = r#"
        <html><head><script>
            if (window.opener) { window.opener.postMessage('login_success', '*'); window.close(); }
            else { window.location.href = 'http://localhost:8080/'; }
        </script></head><body>Đăng nhập thành công, vui lòng đợi...</body></html>
    "#;

    if query.error.is_some() {
        let err_html = r#"
            <html><head><script>
                if (window.opener) { window.opener.postMessage('login_failed', '*'); window.close(); }
                else { window.location.href = 'http://localhost:8080/login?error=access_denied'; }
            </script></head><body>Đăng nhập thất bại.</body></html>
        "#;
        return (jar, axum::response::Html(err_html)).into_response();
    }

    if let Some(code) = query.code {
        let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_else(|_| "MOCK_SECRET_FOR_DEV".to_string());
        
        // Cố gắng gọi API lấy token, nếu không có secret thật, mock thành công
        if client_secret == "MOCK_SECRET_FOR_DEV" {
            // Mock Login (Phát triển cục bộ)
            let email = "admin@otahub.local";
            let google_id = "mock_google_id_123";
            let name = "Hiếu (Mock)";
            let picture = "https://ui-avatars.com/api/?name=Hieu&background=random";

            let _ = state.storage.execute_query({
                let email = email.to_string();
                let google_id = google_id.to_string();
                move |conn| {
                    conn.execute(
                        "INSERT OR IGNORE INTO users (email, google_id, role, is_verified) VALUES (?1, ?2, 'admin', 1)",
                        [&email, &google_id]
                    )?;
                    Ok(())
                }
            }).await;

            let token = crate::auth::token::generate_user_token(1, "admin".to_string(), &state.server_sk_bytes).unwrap_or_default();
            
            // Set JWT as cookie
            let cookie = Cookie::build(("auth_token", token))
                .path("/")
                .http_only(false) // Cho phép frontend JS đọc để lấy role/ID nếu cần
                .same_site(SameSite::Lax)
                .build();
            
            return (jar.add(cookie), axum::response::Html(success_html)).into_response();
        }

        // Thực tế: Lấy Token từ Google (nếu có GOOGLE_CLIENT_SECRET thật)
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| "http://localhost:7424/api/auth/google/callback".to_string());
        
        let client = Client::new();
        let params = [
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ];

        let res = client.post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await;

        if let Ok(resp) = res {
            if let Ok(token_res) = resp.json::<TokenResponse>().await {
                let user_res = client.get("https://www.googleapis.com/oauth2/v1/userinfo?alt=json")
                    .bearer_auth(token_res.access_token)
                    .send()
                    .await;
                
                if let Ok(user_info_resp) = user_res {
                    if let Ok(user_info) = user_info_resp.json::<UserInfo>().await {
                        let _ = state.storage.execute_query({
                            let email = user_info.email.clone();
                            let google_id = user_info.id.clone();
                            move |conn| {
                                conn.execute(
                                    "INSERT OR IGNORE INTO users (email, google_id, role, is_verified) VALUES (?1, ?2, 'user', 1)",
                                    [&email, &google_id]
                                )?;
                                Ok(())
                            }
                        }).await;

                        let user_id: i64 = state.storage.execute_query({
                            let email = user_info.email.clone();
                            move |conn| {
                                conn.query_row("SELECT id FROM users WHERE email = ?1", [&email], |r| r.get(0))
                            }
                        }).await.unwrap_or(1);

                        let token = crate::auth::token::generate_user_token(user_id, "user".to_string(), &state.server_sk_bytes).unwrap_or_default();
                        
                        let cookie = Cookie::build(("auth_token", token.clone()))
                            .path("/")
                            .http_only(false) 
                            .same_site(SameSite::Lax)
                            .build();

                        let user_cookie = Cookie::build(("user_info", format!("{}|{}", user_info.name, user_info.picture)))
                            .path("/")
                            .http_only(false)
                            .same_site(SameSite::Lax)
                            .build();
                        
                        return (jar.add(cookie).add(user_cookie), axum::response::Html(success_html)).into_response();
                    }
                }
            }
        }
    }

    let err_html = r#"
        <html><head><script>
            if (window.opener) { window.opener.postMessage('login_failed', '*'); window.close(); }
            else { window.location.href = 'http://localhost:8080/login?error=auth_failed'; }
        </script></head><body>Đăng nhập thất bại.</body></html>
    "#;
    (jar, axum::response::Html(err_html)).into_response()
}

async fn get_me(jar: CookieJar) -> axum::response::Json<serde_json::Value> {
    if let Some(user_cookie) = jar.get("user_info") {
        let parts: Vec<&str> = user_cookie.value().split('|').collect();
        if parts.len() == 2 {
            return axum::response::Json(serde_json::json!({
                "name": parts[0],
                "picture": parts[1]
            }));
        }
    }
    
    // Fallback if MOCK
    if jar.get("auth_token").is_some() {
        return axum::response::Json(serde_json::json!({
            "name": "Hiếu (Mock)",
            "picture": "https://ui-avatars.com/api/?name=Hieu&background=random"
        }));
    }

    axum::response::Json(serde_json::json!({ "error": "Not logged in" }))
}

async fn logout(jar: CookieJar) -> impl IntoResponse {
    let mut clean_jar = jar.clone();
    clean_jar = clean_jar.remove(Cookie::from("auth_token"));
    clean_jar = clean_jar.remove(Cookie::from("user_info"));
    
    (clean_jar, Redirect::to("http://localhost:8080/login"))
}
