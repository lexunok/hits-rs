mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use common::{PASSWORD, json_body, login_and_get_cookies, setup, unique_email};
use entity::{role::Role, team_invitation, team_member, users};
use hits_api::test_support;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, Set};
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

#[tokio::test]
#[serial]
async fn delete_team_works_for_owner() {
    let (app, state) = setup().await;
    let owner_email = unique_email("team-delete-owner");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Team To Delete")
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &owner_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/team/{}", team.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let deleted_team = entity::team::Entity::find_by_id(team.id)
        .one(state.conn())
        .await
        .unwrap();
    assert!(deleted_team.is_none());
}

#[tokio::test]
#[serial]
async fn delete_team_is_forbidden_for_non_owner() {
    let (app, state) = setup().await;
    let owner_email = unique_email("team-delete-non-owner-owner");
    let member_email = unique_email("team-delete-non-owner-member");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Protected Team")
        .await
        .unwrap();
    test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &member_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/team/{}", team.id))
                .header(header::COOKIE, cookie_header)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn send_invites_to_users_works_for_team_leader() {
    let (app, state) = setup().await;
    let owner_email = unique_email("invite-owner");
    let leader_email = unique_email("invite-leader");
    let invitee_email = unique_email("invite-invitee");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let leader = test_support::seed_user(state.conn(), &leader_email, PASSWORD, vec![Role::TeamLeader])
        .await
        .unwrap();
    let invitee = test_support::seed_user(state.conn(), &invitee_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    
    let team = entity::team::ActiveModel {
        name: Set("Leaders Team".to_string()),
        owner_id: Set(owner.id),
        leader_id: Set(Some(leader.id)),
        ..Default::default()
    }.insert(state.conn()).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &leader_email, PASSWORD).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/team/invitations")
                .header(header::COOKIE, cookie_header)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!([{
                        "user_id": invitee.id,
                        "team_id": team.id,
                        "email": invitee.email,
                        "first_name": invitee.first_name,
                        "last_name": invitee.last_name
                    }])
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let invitation = team_invitation::Entity::find()
        .one(state.conn())
        .await
        .unwrap();
    assert!(invitation.is_some());
    let invitation = invitation.unwrap();
    assert_eq!(invitation.user_id, invitee.id);
    assert_eq!(invitation.team_id, team.id);
}

#[tokio::test]
#[serial]
async fn get_team_invitations_by_user_returns_only_my_invitations() {
    let (app, state) = setup().await;
    let owner_email = unique_email("inv-list-owner");
    let user_email = unique_email("inv-list-user");
    let other_user_email = unique_email("inv-list-other");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner])
        .await
        .unwrap();
    let user = test_support::seed_user(state.conn(), &user_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let other_user = test_support::seed_user(state.conn(), &other_user_email, PASSWORD, vec![Role::Member])
        .await
        .unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Invitation Team").await.unwrap();
    test_support::seed_team_invitation(state.conn(), user.id, team.id).await.unwrap();
    test_support::seed_team_invitation(state.conn(), other_user.id, team.id).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &user_email, PASSWORD).await;

    let response = app.oneshot(
        Request::builder()
            .method("GET")
            .uri("/api/team/invitations/my")
            .header(header::COOKIE, cookie_header)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    let invitations = body.as_array().unwrap();
    assert_eq!(invitations.len(), 1);
    assert_eq!(invitations[0]["user_id"], user.id.to_string());
}

#[tokio::test]
#[serial]
async fn add_team_member_works_for_leader() {
    let (app, state) = setup().await;
    let owner_email = unique_email("add-mem-owner");
    let leader_email = unique_email("add-mem-leader");
    let member_email = unique_email("add-mem-member");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner]).await.unwrap();
    let leader = test_support::seed_user(state.conn(), &leader_email, PASSWORD, vec![Role::TeamLeader]).await.unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member]).await.unwrap();
    let team = entity::team::ActiveModel {
        name: Set("Leaders Team".to_string()),
        owner_id: Set(owner.id),
        leader_id: Set(Some(leader.id)),
        ..Default::default()
    }.insert(state.conn()).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &leader_email, PASSWORD).await;

    let response = app.oneshot(
        Request::builder()
            .method("POST")
            .uri(format!("/api/team/members/add/{}/{}", team.id, member.id))
            .header(header::COOKIE, cookie_header)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let team_member = entity::team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team.id))
        .filter(team_member::Column::UserId.eq(member.id))
        .one(state.conn())
        .await
        .unwrap();

    assert!(team_member.is_some());
}

#[tokio::test]
#[serial]
async fn kick_member_works_for_leader() {
    let (app, state) = setup().await;
    let owner_email = unique_email("kick-owner");
    let leader_email = unique_email("kick-leader");
    let member_email = unique_email("kick-member");
    
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner]).await.unwrap();
    let leader = test_support::seed_user(state.conn(), &leader_email, PASSWORD, vec![Role::TeamLeader]).await.unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member]).await.unwrap();
    let team = entity::team::ActiveModel {
        name: Set("Leaders Team".to_string()),
        owner_id: Set(owner.id),
        leader_id: Set(Some(leader.id)),
        ..Default::default()
    }.insert(state.conn()).await.unwrap();
    test_support::seed_team_member(state.conn(), team.id, member.id).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &leader_email, PASSWORD).await;
    
    let response = app.oneshot(
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/team/members/kick/{}/{}", team.id, member.id))
            .header(header::COOKIE, cookie_header)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let members_count = entity::team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team.id))
        .count(state.conn())
        .await
        .unwrap();

    assert_eq!(members_count, 0);
}

#[tokio::test]
#[serial]
async fn leave_team_works_for_member() {
    let (app, state) = setup().await;
    let owner_email = unique_email("leave-owner");
    let member_email = unique_email("leave-member");

    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner]).await.unwrap();
    let member = test_support::seed_user(state.conn(), &member_email, PASSWORD, vec![Role::Member]).await.unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "A Team to Leave").await.unwrap();
    test_support::seed_team_member(state.conn(), team.id, member.id).await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &member_email, PASSWORD).await;

    let response = app.oneshot(
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/team/leave/{}", team.id))
            .header(header::COOKIE, cookie_header)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let members_count = entity::team_member::Entity::find()
        .filter(team_member::Column::TeamId.eq(team.id))
        .count(state.conn())
        .await
        .unwrap();
    
    assert_eq!(members_count, 0);
}

#[tokio::test]
#[serial]
async fn update_team_leader_works_for_owner() {
    let (app, state) = setup().await;
    let owner_email = unique_email("leader-update-owner");
    let new_leader_email = unique_email("leader-update-new");
    
    let owner = test_support::seed_user(state.conn(), &owner_email, PASSWORD, vec![Role::TeamOwner]).await.unwrap();
    let new_leader = test_support::seed_user(state.conn(), &new_leader_email, PASSWORD, vec![Role::Member]).await.unwrap();
    let team = test_support::seed_team(state.conn(), owner.id, "Team With New Leader").await.unwrap();

    let cookie_header = login_and_get_cookies(app.clone(), &owner_email, PASSWORD).await;

    let response = app.oneshot(
        Request::builder()
            .method("PUT")
            .uri(format!("/api/team/leader/{}/{}", team.id, new_leader.id))
            .header(header::COOKIE, cookie_header)
            .body(Body::empty())
            .unwrap()
    ).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let updated_team = entity::team::Entity::find_by_id(team.id).one(state.conn()).await.unwrap().unwrap();
    assert_eq!(updated_team.leader_id, Some(new_leader.id));

    let new_leader_user = users::Entity::find_by_id(new_leader.id).one(state.conn()).await.unwrap().unwrap();
    assert!(new_leader_user.roles.contains(&Role::TeamLeader));
}
