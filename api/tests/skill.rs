mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    role::Role,
    skill::{Column as SkillColumn, Entity as Skill},
    skill_type::SkillType,
};
use hits_api::test_support;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_skills_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/skill")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn get_all_skills_returns_only_not_deleted() {
    let (app, state) = setup().await;
    let email = unique_email("skill-get-all");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let visible = test_support::seed_skill(state.conn(), user.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    let deleted = test_support::seed_skill(state.conn(), user.id, "OldSkill", SkillType::Devops)
        .await
        .unwrap();

    Skill::update_many()
        .col_expr(
            SkillColumn::DeletedAt,
            sea_orm::prelude::Expr::value(chrono::Local::now()),
        )
        .filter(SkillColumn::Id.eq(deleted.id))
        .exec(state.conn())
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let skills = body.as_array().unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["id"], visible.id.to_string());
}

#[tokio::test]
#[serial]
async fn get_skills_by_type_filters_results() {
    let (app, state) = setup().await;
    let email = unique_email("skill-by-type");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_skill(state.conn(), user.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    test_support::seed_skill(state.conn(), user.id, "Postgres", SkillType::Database)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/skill/type/Language")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let skills = body.as_array().unwrap();
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0]["name"], "Rust");
}

#[tokio::test]
#[serial]
async fn get_all_my_or_confirmed_groups_skills() {
    let (app, state) = setup().await;
    let owner_email = unique_email("skill-my");
    let other_email = unique_email("skill-other");
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let other = test_support::seed_user(state.conn(), &other_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    test_support::seed_skill(state.conn(), owner.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    test_support::seed_skill(state.conn(), other.id, "Postgres", SkillType::Database)
        .await
        .unwrap();
    let hidden = test_support::seed_skill(state.conn(), other.id, "Secret", SkillType::Framework)
        .await
        .unwrap();

    Skill::update_many()
        .col_expr(SkillColumn::Confirmed, sea_orm::prelude::Expr::value(false))
        .filter(SkillColumn::Id.eq(hidden.id))
        .exec(state.conn())
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &owner_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/skill/my")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["LANGUAGE"].as_array().unwrap().len(), 1);
    assert_eq!(body["DATABASE"].as_array().unwrap().len(), 1);
    assert!(body.get("FRAMEWORK").is_none());
}

#[tokio::test]
#[serial]
async fn member_create_skill_stores_unconfirmed_skill() {
    let (app, state) = setup().await;
    let email = unique_email("skill-create-member");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Axum",
                        "type": "Framework"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Axum");
    assert_eq!(body["type"], "Framework");
    assert_eq!(body["confirmed"], false);
    assert_eq!(body["creator_id"], user.id.to_string());
}

#[tokio::test]
#[serial]
async fn admin_create_skill_stores_confirmed_skill() {
    let (app, state) = setup().await;
    let email = unique_email("skill-create-admin");
    test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Docker",
                        "type": "Devops"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await["confirmed"], true);
}

#[tokio::test]
#[serial]
async fn create_skill_rejects_duplicate_name_and_type() {
    let (app, state) = setup().await;
    let email = unique_email("skill-duplicate");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    test_support::seed_skill(state.conn(), user.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Rust",
                        "type": "Language"
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
        "Навык с таким именем и типом уже существует."
    );
}

#[tokio::test]
#[serial]
async fn update_skill_requires_admin() {
    let (app, state) = setup().await;
    let email = unique_email("skill-update-forbidden");
    let user = test_support::seed_user(state.conn(), &email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let skill = test_support::seed_skill(state.conn(), user.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": skill.id,
                        "name": "RustLang"
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
async fn admin_update_skill_changes_fields() {
    let (app, state) = setup().await;
    let admin_email = unique_email("skill-update-admin");
    let creator_email = unique_email("skill-update-creator");
    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let creator =
        test_support::seed_user(state.conn(), &creator_email, PASSWORD, vec![Role::Member])
            .await
            .unwrap();
    let skill = test_support::seed_skill(state.conn(), creator.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/skill")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": skill.id,
                        "name": "RustLang",
                        "type": "Framework",
                        "confirmed": false
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
        "Навык успешно обновлен"
    );

    let updated = Skill::find_by_id(skill.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.name, "RustLang");
    assert_eq!(updated.skill_type, SkillType::Framework);
    assert!(!updated.confirmed);
    assert!(updated.updater_id.is_some());
}

#[tokio::test]
#[serial]
async fn admin_delete_skill_marks_it_deleted() {
    let (app, state) = setup().await;
    let admin_email = unique_email("skill-delete-admin");
    let creator_email = unique_email("skill-delete-creator");
    test_support::seed_user(state.conn(), &admin_email, PASSWORD, vec![Role::Admin])
        .await
        .unwrap();
    let creator =
        test_support::seed_user(state.conn(), &creator_email, PASSWORD, vec![Role::Member])
            .await
            .unwrap();
    let skill = test_support::seed_skill(state.conn(), creator.id, "Rust", SkillType::Language)
        .await
        .unwrap();
    let cookie_header = login_and_get_cookies(app.clone(), &admin_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/skill/{}", skill.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await["message"], "Навык успешно удален");

    let deleted = Skill::find_by_id(skill.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert!(deleted.deleted_at.is_some());
    assert!(deleted.deleter_id.is_some());
}
