use axum::{
    body::Body,
    http::{Request, Response, header},
};
use hits_api::test_support;
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

pub const PASSWORD: &str = "password123";

pub async fn setup() -> (axum::Router, hits_api::AppState) {
    unsafe {
        std::env::set_var("DISABLE_SMTP", "true");
        std::env::set_var("DISABLE_REDIS_STREAM", "true");
    }
    test_support::prepare_clean_db().await.unwrap();
    test_support::test_app().await.unwrap()
}

pub fn unique_email(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{prefix}-{nanos}@example.com")
}

pub fn cookies_from_response(response: &Response<Body>) -> String {
    response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|cookie| cookie.split(';').next())
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .join("; ")
}

pub async fn json_body(response: Response<Body>) -> Value {
    use http_body_util::BodyExt;

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

pub async fn login_and_get_cookies(app: axum::Router, email: &str, password: &str) -> String {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({ "email": email, "password": password }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    cookies_from_response(&response)
}
