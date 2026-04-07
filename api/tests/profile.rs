mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    role::Role,
    skill_type::SkillType,
    users::{Column as UsersColumn, Entity as Users},
    verification_code::{Column as VerificationCodeColumn, Entity as VerificationCode},
};
use hits_api::test_support;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use serial_test::serial;
use std::{fs, path::Path};
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_profile_requires_auth() {
    let (app, state) = setup().await;
    let email = unique_email("profile-auth-required");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/profile/{}", user.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn profile_is_available_after_login() {
    let (app, state) = setup().await;
    let email = unique_email("profile-get");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/profile/{}", user.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["first_name"], "Test");
    assert_eq!(body["last_name"], "User");
}

#[tokio::test]
#[serial]
async fn profile_update_changes_saved_data() {
    let (app, state) = setup().await;
    let email = unique_email("profile-update");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profile")
                .header(header::COOKIE, cookie_header.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "first_name": "Updated",
                        "last_name": "Profile",
                        "study_group": "PI-42",
                        "telephone": "+79999999999"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);

    let profile_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/profile/{}", user.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(profile_response.status(), StatusCode::OK);

    let body = json_body(profile_response).await;
    assert_eq!(body["first_name"], "Updated");
    assert_eq!(body["last_name"], "Profile");
    assert_eq!(body["study_group"], "PI-42");
    assert_eq!(body["telephone"], "+79999999999");
}

#[tokio::test]
#[serial]
async fn update_skills_persists_profile_skills() {
    let (app, state) = setup().await;
    let email = unique_email("profile-skills");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let skill = test_support::seed_skill(state.conn(), user.id, "Rust", SkillType::Language)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let update_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profile/skills")
                .header(header::COOKIE, cookie_header.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!([skill.id]).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(update_response).await["message"],
        "Успешное обновление навыков"
    );

    let profile_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/profile/{}", user.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = json_body(profile_response).await;
    assert_eq!(body["skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["skills"][0]["name"], "Rust");
    assert_eq!(body["skills"][0]["type"], "Language");
}

#[tokio::test]
#[serial]
async fn upload_avatar_saves_webp_file() {
    let (app, state) = setup().await;
    let email = unique_email("profile-avatar");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let boundary = "X-BOUNDARY";
    let body = multipart_avatar_body(boundary, &valid_png_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/profile/avatar")
                .header(header::COOKIE, cookie_header)
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await["message"],
        "Аватар успешно обновлен"
    );

    let avatar_path = avatar_path_for(user.id);
    assert!(
        avatar_path.exists(),
        "avatar file should exist at {:?}",
        avatar_path
    );
    let _ = fs::remove_file(&avatar_path);
}

#[tokio::test]
#[serial]
async fn upload_avatar_rejects_missing_avatar_field() {
    let (app, state) = setup().await;
    let email = unique_email("profile-avatar-missing");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let boundary = "X-BOUNDARY";
    let body = multipart_named_file_body(boundary, "file", &valid_png_bytes());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/profile/avatar")
                .header(header::COOKIE, cookie_header)
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn request_email_change_creates_verification_record() {
    let (app, state) = setup().await;
    let current_email = unique_email("profile-email-current");
    let new_email = unique_email("profile-email-new");

    test_support::seed_user(state.conn(), &current_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &current_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/profile/email/verification/{new_email}"))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let verification_id = body["id"]
        .as_str()
        .unwrap()
        .parse::<sea_orm::prelude::Uuid>()
        .unwrap();

    let stored_code = VerificationCode::find()
        .filter(VerificationCodeColumn::Id.eq(verification_id))
        .one(state.conn())
        .await
        .unwrap()
        .expect("verification code should exist");

    assert_eq!(stored_code.email, new_email);
}

#[tokio::test]
#[serial]
async fn request_email_change_rejects_existing_email() {
    let (app, state) = setup().await;
    let current_email = unique_email("profile-email-existing-current");
    let occupied_email = unique_email("profile-email-existing-target");

    test_support::seed_user(state.conn(), &current_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_user(state.conn(), &occupied_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &current_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/profile/email/verification/{occupied_email}"))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        json_body(response).await["error"],
        "Пользователь с такой почтой уже существует!"
    );
}

#[tokio::test]
#[serial]
async fn confirm_email_change_updates_user_email() {
    let (app, state) = setup().await;
    let current_email = unique_email("profile-email-confirm-current");
    let new_email = unique_email("profile-email-confirm-new");

    test_support::seed_user(state.conn(), &current_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let verification_code =
        test_support::seed_password_reset_code(state.conn(), &new_email, "123456")
            .await
            .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &current_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profile/email")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": verification_code.id,
                        "code": "123456"
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
        "Успешное обновление почты"
    );

    let updated_user = Users::find()
        .filter(UsersColumn::Email.eq(new_email.clone()))
        .one(state.conn())
        .await
        .unwrap()
        .expect("user email should be updated");

    assert_eq!(updated_user.email, new_email);
}

#[tokio::test]
#[serial]
async fn confirm_email_change_rejects_wrong_code() {
    let (app, state) = setup().await;
    let current_email = unique_email("profile-email-wrong-current");
    let new_email = unique_email("profile-email-wrong-new");

    test_support::seed_user(state.conn(), &current_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let verification_code =
        test_support::seed_password_reset_code(state.conn(), &new_email, "123456")
            .await
            .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &current_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profile/email")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": verification_code.id,
                        "code": "654321"
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
async fn confirm_email_change_validates_payload() {
    let (app, state) = setup().await;
    let current_email = unique_email("profile-email-invalid");

    test_support::seed_user(state.conn(), &current_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &current_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profile/email")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": sea_orm::prelude::Uuid::nil(),
                        "code": "12"
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
}

fn assert_validation_error(body: &Value, field: &str) {
    assert!(
        body["errors"].get(field).is_some(),
        "expected validation error for field {field}, body: {body}"
    );
}

fn avatar_path_for(user_id: sea_orm::prelude::Uuid) -> std::path::PathBuf {
    let avatar_dir = std::env::var("AVATAR_PATH").unwrap();
    Path::new(&avatar_dir).join(format!("{user_id}.webp"))
}

fn multipart_avatar_body(boundary: &str, file_bytes: &[u8]) -> Vec<u8> {
    multipart_named_file_body(boundary, "avatar", file_bytes)
}

fn multipart_named_file_body(boundary: &str, field_name: &str, file_bytes: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"{field_name}\"; filename=\"avatar.png\"\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

fn valid_png_bytes() -> Vec<u8> {
    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255])));
    let mut cursor = std::io::Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png).unwrap();
    cursor.into_inner()
}
