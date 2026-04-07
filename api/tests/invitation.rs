mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::{Duration, Local};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    invitation::{Column as InvitationColumn, Entity as Invitation},
    role::Role,
};
use hits_api::test_support;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_invitation_returns_email_and_code() {
    let (app, state) = setup().await;
    let email = unique_email("invitation-get");
    let invitation = test_support::seed_invitation(state.conn(), &email, vec![Role::Member])
        .await
        .unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/invitation/{}", invitation.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["email"], email);
    assert_eq!(body["code"], invitation.id.to_string());
}

#[tokio::test]
#[serial]
async fn get_invitation_returns_not_found_for_expired_invitation() {
    let (app, state) = setup().await;
    let email = unique_email("invitation-expired");
    let invitation = test_support::seed_invitation(state.conn(), &email, vec![Role::Member])
        .await
        .unwrap();
    let invitation_id = invitation.id;

    let mut invitation_model = invitation.into_active_model();
    invitation_model.expiry_date = Set((Local::now() - Duration::minutes(1)).into());
    invitation_model.update(state.conn()).await.unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/invitation/{}", invitation_id))
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
async fn send_invitations_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [unique_email("invite-no-auth")],
                        "roles": ["Member"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn send_invitations_forbids_non_admin() {
    let (app, state) = setup().await;
    let email = unique_email("invite-non-admin");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [unique_email("invite-forbidden")],
                        "roles": ["Member"]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(json_body(response).await["error"], "Forbidden");
}

#[tokio::test]
#[serial]
async fn send_invitations_creates_records_for_admin() {
    let (app, state) = setup().await;
    let admin_email = unique_email("invite-admin");
    let first_invited = unique_email("invite-first");
    let second_invited = unique_email("invite-second");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [first_invited.clone(), second_invited.clone()],
                        "roles": ["Member"]
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
        "Новые приглашения успешно отправлены в кол-ве 2"
    );

    let invitations: Vec<String> = Invitation::find()
        .select_only()
        .column(InvitationColumn::Email)
        .filter(InvitationColumn::Email.is_in(vec![first_invited.clone(), second_invited.clone()]))
        .into_tuple()
        .all(state.conn())
        .await
        .unwrap();

    assert_eq!(invitations.len(), 2);
    assert!(invitations.contains(&first_invited));
    assert!(invitations.contains(&second_invited));
}

#[tokio::test]
#[serial]
async fn send_invitations_returns_error_for_existing_users() {
    let (app, state) = setup().await;
    let admin_email = unique_email("invite-existing-user-admin");
    let existing_user_email = unique_email("invite-existing-user-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    test_support::seed_user(
        state.conn(),
        &existing_user_email,
        PASSWORD,
        vec![Role::Member],
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [existing_user_email.clone()],
                        "roles": ["Member"]
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
        format!(
            "Следующие email уже зарегистрированы: {}",
            existing_user_email
        )
    );
}

#[tokio::test]
#[serial]
async fn send_invitations_reports_when_all_active_invitations_already_exist() {
    let (app, state) = setup().await;
    let admin_email = unique_email("invite-duplicates-admin");
    let already_invited_email = unique_email("invite-duplicates-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    test_support::seed_invitation(state.conn(), &already_invited_email, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [already_invited_email.clone()],
                        "roles": ["Member"]
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
        "Все приглашения по указанным email уже были отправлены ранее."
    );
}

#[tokio::test]
#[serial]
async fn send_invitations_creates_only_new_emails_when_partially_duplicated() {
    let (app, state) = setup().await;
    let admin_email = unique_email("invite-partial-admin");
    let existing_invitation_email = unique_email("invite-partial-old");
    let new_email = unique_email("invite-partial-new");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    test_support::seed_invitation(state.conn(), &existing_invitation_email, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/invitation")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "emails": [existing_invitation_email.clone(), new_email.clone()],
                        "roles": ["Member"]
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
        "Новые приглашения успешно отправлены в кол-ве 1"
    );

    let new_invitation_count = Invitation::find()
        .filter(InvitationColumn::Email.eq(new_email))
        .count(state.conn())
        .await
        .unwrap();

    assert_eq!(new_invitation_count, 1);
}
