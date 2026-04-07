mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{idea::Entity as Idea, idea_status::IdeaStatus, rating::Entity as Rating, role::Role};
use hits_api::test_support;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_ratings_returns_ratings_for_idea() {
    let (app, state) = setup().await;
    let caller_email = unique_email("rating-list-caller");
    let initiator_email = unique_email("rating-list-initiator");
    let expert_email = unique_email("rating-list-expert");
    let project_office_email = unique_email("rating-list-po");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let initiator = test_support::seed_user(
        state.conn(),
        &initiator_email,
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert_user =
        test_support::seed_user(state.conn(), &expert_email, PASSWORD, vec![Role::Expert])
            .await
            .unwrap();
    let _project_office_user = test_support::seed_user(
        state.conn(),
        &project_office_email,
        PASSWORD,
        vec![Role::ProjectOffice],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea One",
    )
    .await
    .unwrap();
    let rating = test_support::seed_rating(state.conn(), idea.id, expert_user.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/rating/{}", idea.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let ratings = body.as_array().unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0]["id"], rating.id.to_string());
    assert_eq!(ratings[0]["expert"]["email"], expert_email);
}

#[tokio::test]
#[serial]
async fn get_ratings_by_expert_filters_by_current_user() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("rating-exp-initiator"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert_user = test_support::seed_user(
        state.conn(),
        &unique_email("rating-exp-user"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let other_expert = test_support::seed_user(
        state.conn(),
        &unique_email("rating-exp-other"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea Two",
    )
    .await
    .unwrap();
    let own_rating = test_support::seed_rating(state.conn(), idea.id, expert_user.id)
        .await
        .unwrap();
    test_support::seed_rating(state.conn(), idea.id, other_expert.id)
        .await
        .unwrap();

    let expert_login_email = UsersEmail::from_model(&expert_user);
    let cookie_header = login_and_get_cookies(app.clone(), &expert_login_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/rating/expert/{}", idea.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let ratings = body.as_array().unwrap();
    assert_eq!(ratings.len(), 1);
    assert_eq!(ratings[0]["id"], own_rating.id.to_string());
}

#[tokio::test]
#[serial]
async fn save_rating_requires_expert_like_role() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("rating-save-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let member_user = test_support::seed_user(
        state.conn(),
        &unique_email("rating-save-member"),
        PASSWORD,
        vec![Role::Member],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea Three",
    )
    .await
    .unwrap();
    let rating = test_support::seed_rating(state.conn(), idea.id, member_user.id)
        .await
        .unwrap();

    let member_email = UsersEmail::from_model(&member_user);
    let cookie_header = login_and_get_cookies(app.clone(), &member_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/rating/save")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": rating.id,
                        "market_value": 5,
                        "originality": 5,
                        "technical_realizability": 5,
                        "suitability": 5,
                        "budget": 5
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
async fn save_rating_updates_scores_and_average() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("rating-save2-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert_user = test_support::seed_user(
        state.conn(),
        &unique_email("rating-save2-exp"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea Four",
    )
    .await
    .unwrap();
    let rating = test_support::seed_rating(state.conn(), idea.id, expert_user.id)
        .await
        .unwrap();

    let expert_email = UsersEmail::from_model(&expert_user);
    let cookie_header = login_and_get_cookies(app.clone(), &expert_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/rating/save")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": rating.id,
                        "market_value": 4,
                        "originality": 5,
                        "technical_realizability": 3,
                        "suitability": 4,
                        "budget": 4
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
        "Рейтинг успешно сохранен"
    );

    let updated = Rating::find_by_id(rating.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.market_value, 4);
    assert_eq!(updated.originality, 5);
    assert_eq!(updated.technical_realizability, 3);
    assert_eq!(updated.suitability, 4);
    assert_eq!(updated.budget, 4);
    assert!((updated.rating - 4.0).abs() < f64::EPSILON);
    assert!(!updated.is_confirmed);
}

#[tokio::test]
#[serial]
async fn confirm_rating_marks_rating_confirmed() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("rating-confirm-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert_user = test_support::seed_user(
        state.conn(),
        &unique_email("rating-confirm-exp"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea Five",
    )
    .await
    .unwrap();
    let rating = test_support::seed_rating(state.conn(), idea.id, expert_user.id)
        .await
        .unwrap();

    let expert_email = UsersEmail::from_model(&expert_user);
    let cookie_header = login_and_get_cookies(app.clone(), &expert_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/rating/confirm")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": rating.id,
                        "market_value": 5,
                        "originality": 5,
                        "technical_realizability": 5,
                        "suitability": 5,
                        "budget": 5
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated = Rating::find_by_id(rating.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert!(updated.is_confirmed);
    assert!((updated.rating - 5.0).abs() < f64::EPSILON);
}

#[tokio::test]
#[serial]
async fn confirm_last_rating_updates_idea_status_and_average() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("rating-final-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let first_expert = test_support::seed_user(
        state.conn(),
        &unique_email("rating-final-exp1"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let second_expert = test_support::seed_user(
        state.conn(),
        &unique_email("rating-final-exp2"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office_group =
        test_support::seed_group(state.conn(), "Project Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        expert_group.id,
        project_office_group.id,
        "Idea Six",
    )
    .await
    .unwrap();
    let first_rating = test_support::seed_rating(state.conn(), idea.id, first_expert.id)
        .await
        .unwrap();
    let second_rating = test_support::seed_rating(state.conn(), idea.id, second_expert.id)
        .await
        .unwrap();

    let mut first_rating_model = first_rating.clone().into_active_model();
    first_rating_model.rating = sea_orm::Set(4.0);
    first_rating_model.market_value = sea_orm::Set(4);
    first_rating_model.originality = sea_orm::Set(4);
    first_rating_model.technical_realizability = sea_orm::Set(4);
    first_rating_model.suitability = sea_orm::Set(4);
    first_rating_model.budget = sea_orm::Set(4);
    first_rating_model.is_confirmed = sea_orm::Set(true);
    first_rating_model.update(state.conn()).await.unwrap();

    let second_expert_email = UsersEmail::from_model(&second_expert);
    let cookie_header = login_and_get_cookies(app.clone(), &second_expert_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/rating/confirm")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": second_rating.id,
                        "market_value": 2,
                        "originality": 2,
                        "technical_realizability": 2,
                        "suitability": 2,
                        "budget": 2
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated_idea = Idea::find_by_id(idea.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_idea.status, IdeaStatus::Confirmed);
    assert!((updated_idea.rating - 3.0).abs() < f64::EPSILON);
}

struct UsersEmail;

impl UsersEmail {
    fn from_model(user: &entity::users::Model) -> String {
        user.email.clone()
    }
}
