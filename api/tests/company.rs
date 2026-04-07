mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    company::{Column as CompanyColumn, Entity as Company},
    role::Role,
};
use hits_api::test_support;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_companies_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/company")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn get_all_companies_returns_existing_companies() {
    let (app, state) = setup().await;
    let caller_email = unique_email("company-list-caller");
    let owner_email = unique_email("company-list-owner");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();
    let company = test_support::seed_company(state.conn(), "Acme", owner.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/company")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let companies = body.as_array().unwrap();
    assert_eq!(companies.len(), 1);
    assert_eq!(companies[0]["id"], company.id.to_string());
    assert_eq!(companies[0]["name"], "Acme");
    assert_eq!(companies[0]["owner"]["email"], owner_email);
}

#[tokio::test]
#[serial]
async fn get_company_by_id_returns_owner_and_members() {
    let (app, state) = setup().await;
    let caller_email = unique_email("company-get-caller");
    let owner_email = unique_email("company-get-owner");
    let member_email = unique_email("company-get-member");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let company = test_support::seed_company(state.conn(), "Globex", owner.id)
        .await
        .unwrap();
    test_support::seed_company_member(state.conn(), company.id, member.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/company/{}", company.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Globex");
    assert_eq!(body["owner"]["email"], owner_email);
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn get_company_members_returns_member_list() {
    let (app, state) = setup().await;
    let caller_email = unique_email("company-members-caller");
    let owner_email = unique_email("company-members-owner");
    let member_email = unique_email("company-members-member");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let company = test_support::seed_company(state.conn(), "Wayne", owner.id)
        .await
        .unwrap();
    test_support::seed_company_member(state.conn(), company.id, member.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/company/{}/members", company.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let members = body.as_array().unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn get_my_companies_returns_owned_and_member_companies() {
    let (app, state) = setup().await;
    let user_email = unique_email("company-my-user");
    let other_owner_email = unique_email("company-my-owner");

    let user = test_support::seed_user(state.conn(), &user_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let other_owner = test_support::seed_user(
        state.conn(),
        &other_owner_email,
        PASSWORD,
        vec![Role::Teacher],
    )
    .await
    .unwrap();

    let owned_company = test_support::seed_company(state.conn(), "Owned Co", user.id)
        .await
        .unwrap();
    let member_company = test_support::seed_company(state.conn(), "Member Co", other_owner.id)
        .await
        .unwrap();
    test_support::seed_company_member(state.conn(), member_company.id, user.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &user_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/company/my")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let companies = body.as_array().unwrap();
    assert_eq!(companies.len(), 2);
    assert!(
        companies
            .iter()
            .any(|company| company["id"] == owned_company.id.to_string())
    );
    assert!(
        companies
            .iter()
            .any(|company| company["id"] == member_company.id.to_string())
    );
}

#[tokio::test]
#[serial]
async fn create_company_requires_admin() {
    let (app, state) = setup().await;
    let caller_email = unique_email("company-create-forbidden-caller");
    let owner_email = unique_email("company-create-forbidden-owner");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/company")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Forbidden Co",
                        "owner_id": owner.id,
                        "members": []
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
async fn create_company_persists_owner_and_members_for_admin() {
    let (app, state) = setup().await;
    let admin_email = unique_email("company-create-admin");
    let owner_email = unique_email("company-create-owner");
    let member_email = unique_email("company-create-member");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/company")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Create Co",
                        "owner_id": owner.id,
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
    assert_eq!(body["name"], "Create Co");
    assert_eq!(body["owner"]["email"], owner_email);
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn update_company_changes_name_owner_and_members() {
    let (app, state) = setup().await;
    let admin_email = unique_email("company-update-admin");
    let old_owner_email = unique_email("company-update-old-owner");
    let new_owner_email = unique_email("company-update-new-owner");
    let member_email = unique_email("company-update-member");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let old_owner = test_support::seed_user(
        state.conn(),
        &old_owner_email,
        PASSWORD,
        vec![Role::Teacher],
    )
    .await
    .unwrap();
    let new_owner = test_support::seed_user(
        state.conn(),
        &new_owner_email,
        PASSWORD,
        vec![Role::Teacher],
    )
    .await
    .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let company = test_support::seed_company(state.conn(), "Old Co", old_owner.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/company")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": company.id,
                        "name": "Updated Co",
                        "owner_id": new_owner.id,
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
    assert_eq!(body["name"], "Updated Co");
    assert_eq!(body["owner"]["email"], new_owner_email);
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
}

#[tokio::test]
#[serial]
async fn delete_company_removes_company() {
    let (app, state) = setup().await;
    let admin_email = unique_email("company-delete-admin");
    let owner_email = unique_email("company-delete-owner");

    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Teacher])
        .await
        .unwrap();
    let company = test_support::seed_company(state.conn(), "Delete Co", owner.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/company/{}", company.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await["message"],
        "Компания успешно удалена"
    );

    let deleted = Company::find()
        .filter(CompanyColumn::Id.eq(company.id))
        .one(state.conn())
        .await
        .unwrap();

    assert!(deleted.is_none());
}
