mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{
    idea::Entity as Idea,
    idea_checked::{Column as IdeaCheckedColumn, Entity as IdeaChecked},
    idea_skill::{Column as IdeaSkillColumn, Entity as IdeaSkill},
    idea_status::IdeaStatus,
    rating::{Column as RatingColumn, Entity as Rating},
    role::Role,
};
use hits_api::test_support;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_idea_by_id_marks_idea_as_checked() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-get-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let viewer = test_support::seed_user(
        state.conn(),
        &unique_email("idea-get-viewer"),
        PASSWORD,
        vec![Role::Member],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Idea Checked",
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &viewer.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/idea/{}", idea.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["is_checked"], true);

    let checked_count = IdeaChecked::find()
        .filter(IdeaCheckedColumn::IdeaId.eq(idea.id))
        .filter(IdeaCheckedColumn::UserId.eq(viewer.id))
        .count(state.conn())
        .await
        .unwrap();
    assert_eq!(checked_count, 1);
}

#[tokio::test]
#[serial]
async fn get_all_and_get_all_by_initiator_return_expected_ideas() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-list-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let other_initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-list-other"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();

    test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Mine",
    )
    .await
    .unwrap();
    test_support::seed_idea(
        state.conn(),
        other_initiator.id,
        experts.id,
        project_office.id,
        "Other",
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &initiator.email, PASSWORD).await;

    let all_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/idea?page=0&page_size=20")
                .header(header::COOKIE, cookie_header.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(all_response.status(), StatusCode::OK);
    assert_eq!(json_body(all_response).await.as_array().unwrap().len(), 2);

    let mine_response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/idea/initiator?page=0&page_size=20")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mine_response.status(), StatusCode::OK);
    let mine = json_body(mine_response).await;
    assert_eq!(mine.as_array().unwrap().len(), 1);
    assert_eq!(mine[0]["name"], "Mine");
}

#[tokio::test]
#[serial]
async fn get_all_on_confirmation_returns_unconfirmed_expert_ideas() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-conf-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert = test_support::seed_user(
        state.conn(),
        &unique_email("idea-conf-expert"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let other_expert = test_support::seed_user(
        state.conn(),
        &unique_email("idea-conf-other-expert"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Needs Confirmation",
    )
    .await
    .unwrap();
    let expert_rating = test_support::seed_rating(state.conn(), idea.id, expert.id)
        .await
        .unwrap();
    let mut other_rating = test_support::seed_rating(state.conn(), idea.id, other_expert.id)
        .await
        .unwrap()
        .into_active_model();
    other_rating.is_confirmed = sea_orm::Set(true);
    other_rating.update(state.conn()).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &expert.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/idea/on-confirmation?page=0&page_size=20")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let ideas = body.as_array().unwrap();
    assert_eq!(ideas.len(), 1);
    assert_eq!(ideas[0]["id"], idea.id.to_string());
    let own_rating_count = Rating::find()
        .filter(RatingColumn::Id.eq(expert_rating.id))
        .count(state.conn())
        .await
        .unwrap();
    assert_eq!(own_rating_count, 1);
}

#[tokio::test]
#[serial]
async fn initiator_can_create_idea_and_ratings_are_generated_for_experts() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-create-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let expert1 = test_support::seed_user(
        state.conn(),
        &unique_email("idea-create-exp1"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let expert2 = test_support::seed_user(
        state.conn(),
        &unique_email("idea-create-exp2"),
        PASSWORD,
        vec![Role::Expert],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    test_support::seed_group_member(state.conn(), experts.id, expert1.id)
        .await
        .unwrap();
    test_support::seed_group_member(state.conn(), experts.id, expert2.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &initiator.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/idea")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": null,
                        "name": "New Idea",
                        "status": "New",
                        "problem": "Problem",
                        "solution": "Solution",
                        "result": "Result",
                        "customer": "Customer",
                        "contact_person": "Contact",
                        "description": "Description",
                        "suitability": 8,
                        "budget": 6,
                        "max_team_size": 5,
                        "min_team_size": 2
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let idea_id = body["id"]
        .as_str()
        .unwrap()
        .parse::<sea_orm::prelude::Uuid>()
        .unwrap();
    assert_eq!(body["pre_assessment"], 7.0);
    assert_eq!(body["experts"]["id"], experts.id.to_string());
    assert_eq!(body["project_office"]["id"], project_office.id.to_string());

    let ratings_count = Rating::find()
        .filter(RatingColumn::IdeaId.eq(idea_id))
        .count(state.conn())
        .await
        .unwrap();
    assert_eq!(ratings_count, 2);
}

#[tokio::test]
#[serial]
async fn save_idea_rejects_invalid_team_size() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-invalid-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &initiator.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/idea")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": null,
                        "name": "Bad Idea",
                        "status": "New",
                        "problem": "Problem",
                        "solution": "Solution",
                        "result": "Result",
                        "customer": "Customer",
                        "contact_person": "Contact",
                        "description": "Description",
                        "suitability": 8,
                        "budget": 6,
                        "max_team_size": 1,
                        "min_team_size": 2
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn save_idea_skills_replaces_existing_skills() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-skill-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Idea Skills",
    )
    .await
    .unwrap();
    let old_skill = test_support::seed_skill(
        state.conn(),
        initiator.id,
        "OldSkill",
        entity::skill_type::SkillType::Language,
    )
    .await
    .unwrap();
    let new_skill = test_support::seed_skill(
        state.conn(),
        initiator.id,
        "NewSkill",
        entity::skill_type::SkillType::Framework,
    )
    .await
    .unwrap();
    test_support::seed_idea_skill(state.conn(), idea.id, old_skill.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &initiator.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/idea/skills")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": idea.id,
                        "skills": [{
                            "id": new_skill.id,
                            "name": new_skill.name,
                            "type": "Framework",
                            "confirmed": true,
                            "creator_id": initiator.id,
                            "updater_id": null,
                            "deleter_id": null
                        }]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let skill_links = IdeaSkill::find()
        .filter(IdeaSkillColumn::IdeaId.eq(idea.id))
        .all(state.conn())
        .await
        .unwrap();
    assert_eq!(skill_links.len(), 1);
    assert_eq!(skill_links[0].skill_id, new_skill.id);
}

#[tokio::test]
#[serial]
async fn initiator_can_send_to_approval_update_status_and_delete_own_idea() {
    let (app, state) = setup().await;
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-own-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Own Idea",
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &initiator.email, PASSWORD).await;
    let send_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/idea/send/{}", idea.id))
                .header(header::COOKIE, cookie_header.clone())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(send_response.status(), StatusCode::OK);

    let updated = Idea::find_by_id(idea.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, IdeaStatus::OnApproval);

    let delete_response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/idea/{}", idea.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::OK);

    let deleted = Idea::find_by_id(idea.id).one(state.conn()).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
#[serial]
async fn admin_can_update_status_of_any_idea() {
    let (app, state) = setup().await;
    let admin = test_support::seed_user(
        state.conn(),
        &unique_email("idea-admin"),
        PASSWORD,
        vec![Role::Admin],
    )
    .await
    .unwrap();
    let initiator = test_support::seed_user(
        state.conn(),
        &unique_email("idea-admin-init"),
        PASSWORD,
        vec![Role::Initiator],
    )
    .await
    .unwrap();
    let experts = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let project_office =
        test_support::seed_group(state.conn(), "ProjectOffice", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        initiator.id,
        experts.id,
        project_office.id,
        "Admin Status",
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &admin.email, PASSWORD).await;
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/idea/status")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": idea.id,
                        "status": "Confirmed"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = Idea::find_by_id(idea.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, IdeaStatus::Confirmed);
}
