mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    role::Role,
    users::{Column as UsersColumn, Entity as Users},
};
use hits_api::test_support;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_current_user_returns_claims_user() {
    let (app, state) = setup().await;
    let email = unique_email("user-me");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/users")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["id"], user.id.to_string());
    assert_eq!(body["email"], email);
}

#[tokio::test]
#[serial]
async fn get_user_by_id_returns_requested_user() {
    let (app, state) = setup().await;
    let caller_email = unique_email("user-by-id-caller");
    let target_email = unique_email("user-by-id-target");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let target =
        test_support::seed_user(state.conn(), &target_email, PASSWORD, vec![Role::Teacher])
            .await
            .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/users/{}", target.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["email"], target_email);
}

#[tokio::test]
#[serial]
async fn get_all_users_respects_pagination_and_skips_deleted() {
    let (app, state) = setup().await;
    let caller_email = unique_email("user-all-caller");
    let first_email = unique_email("user-all-first");
    let second_email = unique_email("user-all-second");
    let deleted_email = unique_email("user-all-deleted");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_user(state.conn(), &first_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_user(state.conn(), &second_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let deleted =
        test_support::seed_user(state.conn(), &deleted_email, PASSWORD, vec![Role::Member])
            .await
            .unwrap();

    let mut deleted = deleted.into_active_model();
    deleted.is_deleted = sea_orm::ActiveValue::Set(true);
    deleted.update(state.conn()).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/users/all?page=0&page_size=2")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let users = body.as_array().unwrap();
    assert_eq!(users.len(), 2);
    assert!(users.iter().all(|user| user["email"] != deleted_email));
}

#[tokio::test]
#[serial]
async fn get_all_users_with_skills_includes_skill_arrays() {
    let (app, state) = setup().await;
    let caller_email = unique_email("user-skills-caller");
    let target_email = unique_email("user-skills-target");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let target = test_support::seed_user(state.conn(), &target_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let skill = test_support::seed_skill(
        state.conn(),
        target.id,
        "Rust",
        entity::skill_type::SkillType::Language,
    )
    .await
    .unwrap();

    entity::user_skill::ActiveModel {
        user_id: sea_orm::ActiveValue::Set(target.id),
        skill_id: sea_orm::ActiveValue::Set(skill.id),
    }
    .insert(state.conn())
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/users/all/with-skills")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let users = body.as_array().unwrap();
    let target_user = users
        .iter()
        .find(|user| user["email"] == target_email)
        .unwrap();

    assert_eq!(target_user["skills"].as_array().unwrap().len(), 1);
    assert_eq!(target_user["skills"][0]["name"], "Rust");
}

#[tokio::test]
#[serial]
async fn get_users_in_and_not_in_teams_split_correctly() {
    let (app, state) = setup().await;
    let caller_email = unique_email("user-teams-caller");
    let in_team_email = unique_email("user-teams-in");
    let not_in_team_email = unique_email("user-teams-out");

    let caller = test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let in_team =
        test_support::seed_user(state.conn(), &in_team_email, PASSWORD, vec![Role::Member])
            .await
            .unwrap();
    test_support::seed_user(
        state.conn(),
        &not_in_team_email,
        PASSWORD,
        vec![Role::Member],
    )
    .await
    .unwrap();

    let team = test_support::seed_team(state.conn(), caller.id, "Test Team")
        .await
        .unwrap();
    test_support::seed_team_member(state.conn(), team.id, in_team.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let in_team_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/users/all/in-team")
                .header(header::COOKIE, cookie_header.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let not_in_team_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/users/all/not-in-team")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(in_team_response.status(), StatusCode::OK);
    assert_eq!(not_in_team_response.status(), StatusCode::OK);

    let in_team_body = json_body(in_team_response).await;
    let not_in_team_body = json_body(not_in_team_response).await;

    assert!(
        in_team_body
            .as_array()
            .unwrap()
            .iter()
            .any(|user| user["email"] == in_team_email)
    );
    assert!(
        not_in_team_body
            .as_array()
            .unwrap()
            .iter()
            .any(|user| user["email"] == not_in_team_email)
    );
}

#[tokio::test]
#[serial]
async fn create_user_requires_admin_and_persists_user() {
    let (app, state) = setup().await;
    let admin_email = unique_email("user-create-admin");
    let new_email = unique_email("user-create-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": new_email,
                        "password": "new-password123",
                        "first_name": "Created",
                        "last_name": "User",
                        "roles": ["Member"],
                        "study_group": "PI-1",
                        "telephone": "+70000000001"
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
        "Успешное создание пользователя"
    );

    let stored = Users::find()
        .filter(UsersColumn::Email.eq(new_email.to_lowercase()))
        .one(state.conn())
        .await
        .unwrap();

    assert!(stored.is_some());
}

#[tokio::test]
#[serial]
async fn create_user_forbids_non_admin() {
    let (app, state) = setup().await;
    let email = unique_email("user-create-forbidden");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/users")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "email": unique_email("user-create-forbidden-target"),
                        "password": "new-password123",
                        "first_name": "Created",
                        "last_name": "User",
                        "roles": ["Member"],
                        "study_group": null,
                        "telephone": null
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn update_user_changes_fields_for_admin() {
    let (app, state) = setup().await;
    let admin_email = unique_email("user-update-admin");
    let target_email = unique_email("user-update-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let target = test_support::seed_user(state.conn(), &target_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/users")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": target.id,
                        "email": unique_email("user-update-new"),
                        "first_name": "Updated",
                        "last_name": "User",
                        "roles": ["Teacher"],
                        "study_group": "PI-2",
                        "telephone": "+79999999999"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated = Users::find_by_id(target.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.first_name, "Updated");
    assert_eq!(updated.last_name, "User");
    assert_eq!(updated.roles, vec![Role::Teacher]);
}

#[tokio::test]
#[serial]
async fn delete_and_restore_user_work_for_admin() {
    let (app, state) = setup().await;
    let admin_email = unique_email("user-delete-admin");
    let target_email = unique_email("user-delete-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let target = test_support::seed_user(state.conn(), &target_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let delete_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/users/{}", target.id))
                .header(header::COOKIE, cookie_header.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);

    let deleted = Users::find_by_id(target.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert!(deleted.is_deleted);

    let restore_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/users/restore/{}", target_email))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(restore_response.status(), StatusCode::OK);

    let restored = Users::find_by_id(target.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert!(!restored.is_deleted);
}
