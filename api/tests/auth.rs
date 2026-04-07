mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, cookies_from_response, json_body, setup, unique_email};
use entity::{
    role::Role,
    users::Entity as Users,
    verification_code::{Column as VerificationCodeColumn, Entity as VerificationCode},
};
use hits_api::test_support;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn login_sets_cookies() {
    let (app, state) = setup().await;
    let email = unique_email("login");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": PASSWORD }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let cookie_header = cookies_from_response(&response);
    assert!(cookie_header.contains("access_token="));
    assert!(cookie_header.contains("refresh_token="));
}

#[tokio::test]
#[serial]
async fn login_rejects_wrong_password() {
    let (app, state) = setup().await;
    let email = unique_email("login-wrong-password");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": "wrong-password" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json_body(response).await["error"], "Wrong credentials");
}

#[tokio::test]
#[serial]
async fn login_validates_payload() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": "not-an-email", "password": "123" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = json_body(response).await;
    assert_validation_error(&body, "email");
    assert_validation_error(&body, "password");
}

#[tokio::test]
#[serial]
async fn registration_creates_user_from_invitation() {
    let (app, state) = setup().await;
    let email = unique_email("registration");
    let invitation = test_support::seed_invitation(state.conn(), &email, vec![Role::Initiator])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/auth/registration/{}", invitation.id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": "ignored@example.com",
                        "password": PASSWORD,
                        "first_name": "Alice",
                        "last_name": "Tester",
                        "study_group": "CS-101",
                        "telephone": "+70000000000"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let created_user = Users::find()
        .filter(entity::users::Column::Email.eq(email.clone()))
        .one(state.conn())
        .await
        .unwrap()
        .expect("registered user should be stored");

    assert_eq!(created_user.first_name, "Alice");
    assert_eq!(created_user.last_name, "Tester");
    assert_eq!(created_user.study_group.as_deref(), Some("CS-101"));
    assert_eq!(created_user.telephone.as_deref(), Some("+70000000000"));
    assert_eq!(created_user.roles, vec![Role::Initiator]);
}

#[tokio::test]
#[serial]
async fn registration_returns_not_found_for_unknown_invitation() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/auth/registration/{}",
                    sea_orm::prelude::Uuid::nil()
                ))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": "ignored@example.com",
                        "password": PASSWORD,
                        "first_name": "Alice",
                        "last_name": "Tester",
                        "study_group": "CS-101",
                        "telephone": "+70000000000"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["error"], "Not Found");
}

#[tokio::test]
#[serial]
async fn registration_validates_payload() {
    let (app, state) = setup().await;
    let email = unique_email("registration-invalid");
    let invitation = test_support::seed_invitation(state.conn(), &email, vec![Role::Initiator])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/auth/registration/{}", invitation.id))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": "bad-email",
                        "password": "123",
                        "first_name": "Alice",
                        "last_name": "Tester",
                        "study_group": null,
                        "telephone": null
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = json_body(response).await;
    assert_validation_error(&body, "email");
    assert_validation_error(&body, "password");
}

#[tokio::test]
#[serial]
async fn refresh_issues_new_access_cookie() {
    let (app, state) = setup().await;
    let email = unique_email("refresh");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": PASSWORD }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let cookie_header = cookies_from_response(&login_response);

    let refresh_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_response.status(), StatusCode::OK);

    let refreshed_cookie_header = cookies_from_response(&refresh_response);
    assert!(refreshed_cookie_header.contains("access_token="));
    assert!(refreshed_cookie_header.contains("refresh_token="));
}

#[tokio::test]
#[serial]
async fn refresh_requires_cookie() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json_body(response).await["error"], "Wrong credentials");
}

#[tokio::test]
#[serial]
async fn refresh_rejects_access_token_instead_of_refresh_token() {
    let (app, state) = setup().await;
    let email = unique_email("refresh-invalid");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let login_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": PASSWORD }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let access_cookie = login_response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|cookie| cookie.starts_with("access_token="))
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .header(
                    header::COOKIE,
                    access_cookie.replace("access_token=", "refresh_token="),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(json_body(response).await["error"], "Invalid token");
}

#[tokio::test]
#[serial]
async fn logout_clears_auth_cookies() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header(
                    header::COOKIE,
                    "access_token=test-access; refresh_token=test-refresh",
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let cookies = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>();

    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.contains("access_token="))
    );
    assert!(
        cookies
            .iter()
            .any(|cookie| cookie.contains("refresh_token="))
    );
}

#[tokio::test]
#[serial]
async fn request_password_reset_creates_verification_record() {
    let (app, state) = setup().await;
    let email = unique_email("password-request");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/auth/password/verification/{email}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let verification_id = body["id"].as_str().unwrap();
    let verification_id = verification_id.parse::<sea_orm::prelude::Uuid>().unwrap();

    let stored_code = VerificationCode::find()
        .filter(VerificationCodeColumn::Id.eq(verification_id))
        .one(state.conn())
        .await
        .unwrap();

    assert!(stored_code.is_some());
}

#[tokio::test]
#[serial]
async fn request_password_reset_returns_not_found_for_unknown_email() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/auth/password/verification/{}",
                    unique_email("missing-user")
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(json_body(response).await["error"], "Not Found");
}

#[tokio::test]
#[serial]
async fn confirm_password_reset_updates_password() {
    let (app, state) = setup().await;
    let email = unique_email("password-confirm");
    let new_password = "new-password123";

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let verification_code = test_support::seed_password_reset_code(state.conn(), &email, "123456")
        .await
        .unwrap();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/auth/password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": verification_code.id,
                        "code": "123456",
                        "password": new_password
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await["message"],
        "Успешное обновление пароля"
    );

    let login_response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({ "email": email, "password": new_password }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn confirm_password_reset_rejects_wrong_code() {
    let (app, state) = setup().await;
    let email = unique_email("password-wrong-code");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let verification_code = test_support::seed_password_reset_code(state.conn(), &email, "123456")
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/auth/password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": verification_code.id,
                        "code": "654321",
                        "password": "new-password123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(response).await["error"],
        "Ошибка, попробуйте еще раз"
    );
}

#[tokio::test]
#[serial]
async fn confirm_password_reset_validates_payload() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/auth/password")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": sea_orm::prelude::Uuid::nil(),
                        "code": "12",
                        "password": "123"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let body = json_body(response).await;
    assert_validation_error(&body, "code");
    assert_validation_error(&body, "password");
}

fn assert_validation_error(body: &Value, field: &str) {
    assert!(
        body["errors"].get(field).is_some(),
        "expected validation error for field {field}, body: {body}"
    );
}
