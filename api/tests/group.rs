mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    group::{Column as GroupColumn, Entity as Group},
    role::Role,
};
use hits_api::test_support;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_groups_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/group")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn get_all_groups_returns_existing_groups() {
    let (app, state) = setup().await;
    let email = unique_email("group-list");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let group = test_support::seed_group(state.conn(), "Backend", vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/group")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let groups = body.as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["id"], group.id.to_string());
    assert_eq!(groups[0]["name"], "Backend");
}

#[tokio::test]
#[serial]
async fn get_group_by_id_returns_members() {
    let (app, state) = setup().await;
    let caller_email = unique_email("group-get-caller");
    let member_email = unique_email("group-get-member");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let member =
        test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Teacher])
            .await
            .unwrap();
    let group = test_support::seed_group(state.conn(), "Design", vec![Role::Teacher])
        .await
        .unwrap();
    test_support::seed_group_member(state.conn(), group.id, member.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/group/{}", group.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Design");
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn create_group_requires_admin() {
    let (app, state) = setup().await;
    let email = unique_email("group-create-forbidden");
    let member_email = unique_email("group-create-member");

    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/group")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "New Group",
                        "roles": ["Member"],
                        "members": [member.id]
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
async fn create_group_persists_group_and_members_for_admin() {
    let (app, state) = setup().await;
    let admin_email = unique_email("group-create-admin");
    let member_email = unique_email("group-create-target");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let member =
        test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Teacher])
            .await
            .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/group")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "New Group",
                        "roles": ["Teacher"],
                        "members": [member.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "New Group");
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn update_group_changes_name_roles_and_members() {
    let (app, state) = setup().await;
    let admin_email = unique_email("group-update-admin");
    let old_member_email = unique_email("group-update-old");
    let new_member_email = unique_email("group-update-new");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let old_member = test_support::seed_user(
        state.conn(),
        &old_member_email,
        PASSWORD,
        vec![Role::Member],
    )
    .await
    .unwrap();
    let new_member = test_support::seed_user(
        state.conn(),
        &new_member_email,
        PASSWORD,
        vec![Role::Teacher],
    )
    .await
    .unwrap();
    let group = test_support::seed_group(state.conn(), "Old Group", vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_group_member(state.conn(), group.id, old_member.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/group")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": group.id,
                        "name": "Updated Group",
                        "roles": ["Teacher"],
                        "members": [new_member.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Updated Group");
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], new_member_email);
}

#[tokio::test]
#[serial]
async fn delete_group_removes_group() {
    let (app, state) = setup().await;
    let admin_email = unique_email("group-delete-admin");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let group = test_support::seed_group(state.conn(), "Delete Me", vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/group/{}", group.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await["message"],
        "Группа успешно удалена"
    );

    let deleted = Group::find()
        .filter(GroupColumn::Id.eq(group.id))
        .one(state.conn())
        .await
        .unwrap();

    assert!(deleted.is_none());
}
