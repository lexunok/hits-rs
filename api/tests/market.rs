mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use chrono::NaiveDate;
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    market::{Column as MarketColumn, Entity as Market},
    market_status::MarketStatus,
    role::Role,
};
use hits_api::test_support;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_markets_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/market")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn get_all_markets_returns_existing_records() {
    let (app, state) = setup().await;
    let email = unique_email("market-list");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let market = test_support::seed_market(
        state.conn(),
        "Spring Market",
        date(2026, 1, 1),
        date(2026, 1, 31),
        MarketStatus::New,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/market")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let markets = body.as_array().unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0]["id"], market.id.to_string());
    assert_eq!(markets[0]["status"], "New");
}

#[tokio::test]
#[serial]
async fn get_active_markets_filters_only_active() {
    let (app, state) = setup().await;
    let email = unique_email("market-active");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_market(
        state.conn(),
        "Active Market",
        date(2026, 2, 1),
        date(2026, 2, 28),
        MarketStatus::Active,
    )
    .await
    .unwrap();
    test_support::seed_market(
        state.conn(),
        "New Market",
        date(2026, 3, 1),
        date(2026, 3, 31),
        MarketStatus::New,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/market/active")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let markets = body.as_array().unwrap();
    assert_eq!(markets.len(), 1);
    assert_eq!(markets[0]["name"], "Active Market");
}

#[tokio::test]
#[serial]
async fn get_market_by_id_returns_market() {
    let (app, state) = setup().await;
    let email = unique_email("market-by-id");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let market = test_support::seed_market(
        state.conn(),
        "Detail Market",
        date(2026, 4, 1),
        date(2026, 4, 30),
        MarketStatus::New,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/market/{}", market.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Detail Market");
}

#[tokio::test]
#[serial]
async fn create_market_requires_project_office_or_admin() {
    let (app, state) = setup().await;
    let email = unique_email("market-create-forbidden");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/market")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Create Market",
                        "start_date": "2026-05-01",
                        "finish_date": "2026-05-31"
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
async fn project_office_can_create_and_update_market() {
    let (app, state) = setup().await;
    let email = unique_email("market-po");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::ProjectOffice])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/market")
                .header(header::COOKIE, cookie_header.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Create Market",
                        "start_date": "2026-05-01",
                        "finish_date": "2026-05-31"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);
    let created = json_body(create_response).await;
    let market_id = created["id"].as_str().unwrap();

    let update_response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/market")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": market_id,
                        "name": "Updated Market",
                        "start_date": "2026-06-01",
                        "finish_date": "2026-06-30"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let updated = json_body(update_response).await;
    assert_eq!(updated["name"], "Updated Market");
}

#[tokio::test]
#[serial]
async fn update_market_status_allows_active_for_project_office() {
    let (app, state) = setup().await;
    let email = unique_email("market-status-po");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::ProjectOffice])
        .await
        .unwrap();
    let market = test_support::seed_market(
        state.conn(),
        "Status Market",
        date(2026, 7, 1),
        date(2026, 7, 31),
        MarketStatus::New,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/market/status")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": market.id,
                        "status": "Active"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = Market::find_by_id(market.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, MarketStatus::Active);
}

#[tokio::test]
#[serial]
async fn update_market_status_forbids_non_admin_to_set_new() {
    let (app, state) = setup().await;
    let email = unique_email("market-status-forbidden");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::ProjectOffice])
        .await
        .unwrap();
    let market = test_support::seed_market(
        state.conn(),
        "Status Market",
        date(2026, 8, 1),
        date(2026, 8, 31),
        MarketStatus::Active,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/market/status")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": market.id,
                        "status": "New"
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
async fn admin_can_set_status_back_to_new_and_delete_market() {
    let (app, state) = setup().await;
    let email = unique_email("market-admin");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let market = test_support::seed_market(
        state.conn(),
        "Admin Market",
        date(2026, 9, 1),
        date(2026, 9, 30),
        MarketStatus::Active,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;
    let status_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/market/status")
                .header(header::COOKIE, cookie_header.clone())
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": market.id,
                        "status": "New"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_response.status(), StatusCode::OK);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/market/{}", market.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::OK);
    assert_eq!(
        json_body(delete_response).await["message"],
        "Маркет успешно удален"
    );

    let deleted = Market::find()
        .filter(MarketColumn::Id.eq(market.id))
        .one(state.conn())
        .await
        .unwrap();
    assert!(deleted.is_none());
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}
