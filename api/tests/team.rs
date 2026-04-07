mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::role::Role;
use hits_api::test_support;
use sea_orm::{ActiveModelTrait, EntityTrait};
use serde_json::json;
use serial_test::serial;
use tower::util::ServiceExt;

#[tokio::test]
#[serial]
async fn get_all_teams_requires_auth() {
    let (app, _) = setup().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/team")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn get_all_teams_returns_existing_teams() {
    let (app, state) = setup().await;
    let caller_email = unique_email("team-list-caller");
    let owner_email = unique_email("team-list-owner");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Backend Guild")
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/team")
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let teams = body.as_array().unwrap();
    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0]["id"], team.id.to_string());
    assert_eq!(teams[0]["name"], "Backend Guild");
    assert_eq!(teams[0]["owner"]["email"], owner_email);
}

#[tokio::test]
#[serial]
async fn get_my_teams_returns_owned_and_led_teams() {
    let (app, state) = setup().await;
    let user_email = unique_email("team-my-user");
    let other_owner_email = unique_email("team-my-other-owner");

    let user = test_support::seed_user(
        state.conn(),
        &user_email,
        PASSWORD,
        vec![Role::TeamOwner, Role::TeamLeader],
    )
    .await
    .unwrap();
    let other_owner = test_support::seed_user(
        state.conn(),
        &other_owner_email,
        PASSWORD,
        vec![Role::TeamOwner],
    )
    .await
    .unwrap();
    let expert_group = test_support::seed_group(state.conn(), "Experts", vec![Role::Expert])
        .await
        .unwrap();
    let office_group =
        test_support::seed_group(state.conn(), "Office", vec![Role::ProjectOffice])
            .await
            .unwrap();
    let idea = test_support::seed_idea(
        state.conn(),
        user.id,
        expert_group.id,
        office_group.id,
        "Marketplace idea",
    )
    .await
    .unwrap();

    let owned_team = test_support::seed_team(state.conn(), user.id, "Owned Team")
        .await
        .unwrap();
    let led_team = entity::team::ActiveModel {
        name: sea_orm::ActiveValue::Set("Led Team".to_owned()),
        description: sea_orm::ActiveValue::Set("Led Team description".to_owned()),
        owner_id: sea_orm::ActiveValue::Set(other_owner.id),
        leader_id: sea_orm::ActiveValue::Set(Some(user.id)),
        ..Default::default()
    }
    .insert(state.conn())
    .await
    .unwrap();
    let unrelated_team = test_support::seed_team(state.conn(), other_owner.id, "Other Team")
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &user_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/team/my/{}", idea.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let teams = body.as_array().unwrap();
    assert_eq!(teams.len(), 2);
    assert!(teams.iter().any(|team| team["id"] == owned_team.id.to_string()));
    assert!(teams.iter().any(|team| team["id"] == led_team.id.to_string()));
    assert!(!teams.iter().any(|team| team["id"] == unrelated_team.id.to_string()));
}

#[tokio::test]
#[serial]
async fn get_team_by_id_returns_members_and_skill_collections() {
    let (app, state) = setup().await;
    let caller_email = unique_email("team-get-caller");
    let owner_email = unique_email("team-get-owner");
    let member_email = unique_email("team-get-member");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let wanted_skill = test_support::seed_skill(
        state.conn(),
        owner.id,
        "Rust",
        entity::skill_type::SkillType::Language,
    )
    .await
    .unwrap();
    let member_skill = test_support::seed_skill(
        state.conn(),
        member.id,
        "Axum",
        entity::skill_type::SkillType::Framework,
    )
    .await
    .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Platform Team")
        .await
        .unwrap();
    test_support::seed_team_member(state.conn(), team.id, member.id)
        .await
        .unwrap();
    test_support::seed_team_wanted_skill(state.conn(), team.id, wanted_skill.id)
        .await
        .unwrap();
    test_support::seed_user_skill(state.conn(), member.id, member_skill.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/team/{}", team.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Platform Team");
    assert_eq!(body["owner"]["email"], owner_email);
    assert_eq!(body["leader"]["email"], owner_email);
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
    assert_eq!(body["wanted_skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["wanted_skills"][0]["name"], "Rust");
    assert_eq!(body["member_skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["member_skills"][0]["name"], "Axum");
}

#[tokio::test]
#[serial]
async fn create_team_requires_team_owner_or_admin() {
    let (app, state) = setup().await;
    let caller_email = unique_email("team-create-forbidden-caller");
    let owner_email = unique_email("team-create-forbidden-owner");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/team")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "Forbidden Team",
                        "description": "Should fail",
                        "is_closed": false,
                        "owner_id": owner.id,
                        "leader_id": owner.id,
                        "members": [],
                        "wanted_skills": []
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
async fn create_team_persists_members_wanted_skills_and_grants_leader_role() {
    let (app, state) = setup().await;
    let caller_email = unique_email("team-create-owner");
    let leader_email = unique_email("team-create-leader");
    let member_email = unique_email("team-create-member");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let leader = test_support::seed_user(state.conn(), &leader_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let skill = test_support::seed_skill(
        state.conn(),
        leader.id,
        "Postgres",
        entity::skill_type::SkillType::Database,
    )
    .await
    .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/team")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "name": "New Team",
                        "description": "Integration team",
                        "is_closed": true,
                        "owner_id": leader.id,
                        "leader_id": leader.id,
                        "members": [member.id],
                        "wanted_skills": [skill.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "New Team");
    assert_eq!(body["is_closed"], true);
    assert_eq!(body["owner"]["email"], leader_email);
    assert_eq!(body["leader"]["email"], leader_email);
    assert_eq!(body["members"].as_array().unwrap().len(), 1);
    assert_eq!(body["members"][0]["email"], member_email);
    assert_eq!(body["wanted_skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["wanted_skills"][0]["name"], "Postgres");

    let leader_after = entity::users::Entity::find_by_id(leader.id)
        .one(state.conn())
        .await
        .unwrap()
        .unwrap();
    assert!(leader_after.roles.contains(&Role::TeamLeader));
}

#[tokio::test]
#[serial]
async fn update_team_requires_admin_owner_or_leader() {
    let (app, state) = setup().await;
    let caller_email = unique_email("team-update-forbidden-caller");
    let owner_email = unique_email("team-update-forbidden-owner");

    test_support::seed_user(state.conn(), &caller_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Locked Team")
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &caller_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/team")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": team.id,
                        "name": "Updated Locked Team",
                        "description": "Still locked",
                        "is_closed": false,
                        "wanted_skills": []
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
async fn update_team_changes_basic_fields_and_wanted_skills() {
    let (app, state) = setup().await;
    let owner_email = unique_email("team-update-owner");

    let owner = test_support::seed_user(
        state.conn(),
        &owner_email,
        PASSWORD,
        vec![Role::TeamOwner, Role::TeamLeader],
    )
    .await
    .unwrap();
    let old_skill = test_support::seed_skill(
        state.conn(),
        owner.id,
        "Docker",
        entity::skill_type::SkillType::Devops,
    )
    .await
    .unwrap();
    let new_skill = test_support::seed_skill(
        state.conn(),
        owner.id,
        "Kubernetes",
        entity::skill_type::SkillType::Devops,
    )
    .await
    .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Delivery Team")
        .await
        .unwrap();
    test_support::seed_team_wanted_skill(state.conn(), team.id, old_skill.id)
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &owner_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/team")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "id": team.id,
                        "name": "Updated Delivery Team",
                        "description": "Updated description",
                        "is_closed": true,
                        "wanted_skills": [new_skill.id]
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["name"], "Updated Delivery Team");
    assert_eq!(body["description"], "Updated description");
    assert_eq!(body["is_closed"], true);
    assert_eq!(body["wanted_skills"].as_array().unwrap().len(), 1);
    assert_eq!(body["wanted_skills"][0]["name"], "Kubernetes");
}
