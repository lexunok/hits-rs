use crate::{AppState, build_app, config::GLOBAL_CONFIG, utils::security::hash_password};
use anyhow::Context;
use axum::Router;
use chrono::{Duration, Local};
use entity::{
    company, company_member, group, group_member, idea, idea_checked, idea_skill,
    idea_status::IdeaStatus, invitation, market, market_status::MarketStatus, rating, role::Role,
    skill, skill_type::SkillType, team, team_member, team_wanted_skill, user_skill, users,
    verification_code,
};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ConnectionTrait, Database, DatabaseConnection, prelude::Uuid};
use tokio::sync::OnceCell;

static DB_RESET: OnceCell<()> = OnceCell::const_new();

fn ensure_env_loaded() {
    dotenvy::dotenv().ok();
    let _ = &*GLOBAL_CONFIG;
}

pub async fn prepare_clean_db_once() -> anyhow::Result<()> {
    ensure_env_loaded();

    DB_RESET
        .get_or_try_init(|| async {
            let conn = connect_db().await?;
            Migrator::up(&conn, None).await?;
            truncate_public_tables(&conn).await?;
            anyhow::Result::<()>::Ok(())
        })
        .await?;

    Ok(())
}

pub async fn prepare_clean_db() -> anyhow::Result<()> {
    ensure_env_loaded();

    let conn = connect_db().await?;
    Migrator::up(&conn, None).await?;
    truncate_public_tables(&conn).await?;

    Ok(())
}

pub async fn test_app() -> anyhow::Result<(Router, AppState)> {
    ensure_env_loaded();

    let conn = connect_db().await?;
    Migrator::up(&conn, None).await?;
    let redis_client = redis::Client::open(GLOBAL_CONFIG.redis_url.to_owned())?;
    let state = AppState::new(conn, redis_client);
    let app = build_app(state.clone())?;

    Ok((app, state))
}

pub async fn seed_user(
    conn: &DatabaseConnection,
    email: &str,
    password: &str,
    roles: Vec<Role>,
) -> anyhow::Result<users::Model> {
    let user = users::ActiveModel {
        email: Set(email.to_lowercase()),
        password: Set(hash_password(password)?),
        first_name: Set("Test".to_owned()),
        last_name: Set("User".to_owned()),
        roles: Set(roles),
        ..Default::default()
    };

    user.insert(conn)
        .await
        .context("failed to insert test user")
}

pub async fn seed_invitation(
    conn: &DatabaseConnection,
    email: &str,
    roles: Vec<Role>,
) -> anyhow::Result<invitation::Model> {
    let invitation = invitation::ActiveModel {
        email: Set(email.to_lowercase()),
        roles: Set(roles),
        expiry_date: Set((Local::now() + Duration::days(1)).into()),
        ..Default::default()
    };

    invitation
        .insert(conn)
        .await
        .context("failed to insert test invitation")
}

pub async fn seed_password_reset_code(
    conn: &DatabaseConnection,
    email: &str,
    code: &str,
) -> anyhow::Result<verification_code::Model> {
    let code = verification_code::ActiveModel {
        email: Set(email.to_lowercase()),
        code: Set(hash_password(code)?),
        expiry_date: Set((Local::now() + Duration::minutes(10)).into()),
        ..Default::default()
    };

    code.insert(conn)
        .await
        .context("failed to insert verification code")
}

pub async fn seed_skill(
    conn: &DatabaseConnection,
    creator_id: sea_orm::prelude::Uuid,
    name: &str,
    skill_type: SkillType,
) -> anyhow::Result<skill::Model> {
    let skill = skill::ActiveModel {
        name: Set(name.to_owned()),
        skill_type: Set(skill_type),
        confirmed: Set(true),
        creator_id: Set(creator_id),
        ..Default::default()
    };

    skill
        .insert(conn)
        .await
        .context("failed to insert test skill")
}

pub async fn seed_team(
    conn: &DatabaseConnection,
    owner_id: sea_orm::prelude::Uuid,
    name: &str,
) -> anyhow::Result<team::Model> {
    let team = team::ActiveModel {
        name: Set(name.to_owned()),
        description: Set(format!("{name} description")),
        owner_id: Set(owner_id),
        leader_id: Set(Some(owner_id)),
        ..Default::default()
    };

    team.insert(conn)
        .await
        .context("failed to insert test team")
}

pub async fn seed_team_member(
    conn: &DatabaseConnection,
    team_id: sea_orm::prelude::Uuid,
    user_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<team_member::Model> {
    let team_member = team_member::ActiveModel {
        team_id: Set(team_id),
        user_id: Set(user_id),
        ..Default::default()
    };

    team_member
        .insert(conn)
        .await
        .context("failed to insert test team member")
}

pub async fn seed_team_wanted_skill(
    conn: &DatabaseConnection,
    team_id: sea_orm::prelude::Uuid,
    skill_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<team_wanted_skill::Model> {
    let wanted_skill = team_wanted_skill::ActiveModel {
        team_id: Set(team_id),
        skill_id: Set(skill_id),
    };

    wanted_skill
        .insert(conn)
        .await
        .context("failed to insert test team wanted skill")
}

pub async fn seed_team_invitation(
    conn: &DatabaseConnection,
    user_id: sea_orm::prelude::Uuid,
    team_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<entity::team_invitation::Model> {
    let invitation = entity::team_invitation::ActiveModel {
        user_id: Set(user_id),
        team_id: Set(team_id),
        status: Set(entity::request_status::RequestStatus::New),
        ..Default::default()
    };

    invitation
        .insert(conn)
        .await
        .context("failed to insert test team invitation")
}

pub async fn seed_user_skill(
    conn: &DatabaseConnection,
    user_id: sea_orm::prelude::Uuid,
    skill_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<user_skill::Model> {
    let user_skill = user_skill::ActiveModel {
        user_id: Set(user_id),
        skill_id: Set(skill_id),
    };

    user_skill
        .insert(conn)
        .await
        .context("failed to insert test user skill")
}

pub async fn seed_group(
    conn: &DatabaseConnection,
    name: &str,
    roles: Vec<Role>,
) -> anyhow::Result<group::Model> {
    let group = group::ActiveModel {
        name: Set(name.to_owned()),
        roles: Set(roles),
        ..Default::default()
    };

    group
        .insert(conn)
        .await
        .context("failed to insert test group")
}

pub async fn seed_group_member(
    conn: &DatabaseConnection,
    group_id: sea_orm::prelude::Uuid,
    user_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<group_member::Model> {
    let group_member = group_member::ActiveModel {
        group_id: Set(group_id),
        user_id: Set(user_id),
    };

    group_member
        .insert(conn)
        .await
        .context("failed to insert test group member")
}

pub async fn seed_company(
    conn: &DatabaseConnection,
    name: &str,
    owner_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<company::Model> {
    let company = company::ActiveModel {
        name: Set(name.to_owned()),
        owner_id: Set(owner_id),
        ..Default::default()
    };

    company
        .insert(conn)
        .await
        .context("failed to insert test company")
}

pub async fn seed_company_member(
    conn: &DatabaseConnection,
    company_id: sea_orm::prelude::Uuid,
    user_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<company_member::Model> {
    let company_member = company_member::ActiveModel {
        company_id: Set(company_id),
        user_id: Set(user_id),
    };

    company_member
        .insert(conn)
        .await
        .context("failed to insert test company member")
}

pub async fn seed_idea(
    conn: &DatabaseConnection,
    initiator_id: sea_orm::prelude::Uuid,
    group_expert_id: sea_orm::prelude::Uuid,
    group_project_office_id: sea_orm::prelude::Uuid,
    company_id: Uuid,
    name: &str,
) -> anyhow::Result<idea::Model> {
    let idea = idea::ActiveModel {
        initiator_id: Set(initiator_id),
        group_expert_id: Set(group_expert_id),
        group_project_office_id: Set(group_project_office_id),
        name: Set(name.to_owned()),
        status: Set(IdeaStatus::OnConfirmation),
        problem: Set("Problem".to_owned()),
        solution: Set("Solution".to_owned()),
        result: Set("Result".to_owned()),
        company_id: Set(company_id),
        description: Set("Description".to_owned()),
        suitability: Set(1),
        budget: Set(1),
        max_team_size: Set(5),
        min_team_size: Set(1),
        pre_assessment: Set(0.0),
        ..Default::default()
    };

    idea.insert(conn)
        .await
        .context("failed to insert test idea")
}

pub async fn seed_rating(
    conn: &DatabaseConnection,
    idea_id: sea_orm::prelude::Uuid,
    expert_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<rating::Model> {
    let rating = rating::ActiveModel {
        idea_id: Set(idea_id),
        expert_id: Set(expert_id),
        ..Default::default()
    };

    rating
        .insert(conn)
        .await
        .context("failed to insert test rating")
}

pub async fn seed_market(
    conn: &DatabaseConnection,
    name: &str,
    start_date: chrono::NaiveDate,
    finish_date: chrono::NaiveDate,
    status: MarketStatus,
) -> anyhow::Result<market::Model> {
    let market = market::ActiveModel {
        name: Set(name.to_owned()),
        start_date: Set(start_date),
        finish_date: Set(finish_date),
        status: Set(status),
        ..Default::default()
    };

    market
        .insert(conn)
        .await
        .context("failed to insert test market")
}

pub async fn seed_idea_checked(
    conn: &DatabaseConnection,
    idea_id: sea_orm::prelude::Uuid,
    user_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<idea_checked::Model> {
    let checked = idea_checked::ActiveModel {
        idea_id: Set(idea_id),
        user_id: Set(user_id),
    };

    checked
        .insert(conn)
        .await
        .context("failed to insert idea_checked")
}

pub async fn seed_idea_skill(
    conn: &DatabaseConnection,
    idea_id: sea_orm::prelude::Uuid,
    skill_id: sea_orm::prelude::Uuid,
) -> anyhow::Result<idea_skill::Model> {
    let idea_skill = idea_skill::ActiveModel {
        idea_id: Set(idea_id),
        skill_id: Set(skill_id),
    };

    idea_skill
        .insert(conn)
        .await
        .context("failed to insert idea_skill")
}

async fn connect_db() -> anyhow::Result<DatabaseConnection> {
    Database::connect(GLOBAL_CONFIG.db_url.to_owned())
        .await
        .context("failed to connect to database")
}

async fn truncate_public_tables(conn: &DatabaseConnection) -> anyhow::Result<()> {
    const TABLES: &[&str] = &[
        "\"company\"",
        "\"company_member\"",
        "\"favorite_idea\"",
        "\"group\"",
        "\"group_member\"",
        "\"idea\"",
        "\"idea_checked\"",
        "\"idea_market\"",
        "\"idea_market_advertisement\"",
        "\"idea_market_refused\"",
        "\"idea_skill\"",
        "\"invitation\"",
        "\"market\"",
        "\"project\"",
        "\"rating\"",
        "\"skill\"",
        "\"team\"",
        "\"team_invitation\"",
        "\"team_market_request\"",
        "\"team_member\"",
        "\"team_refused\"",
        "\"team_wanted_skill\"",
        "\"user_skill\"",
        "\"users\"",
        "\"verification_code\"",
    ];

    let truncate_sql = format!(
        "TRUNCATE TABLE {} RESTART IDENTITY CASCADE",
        TABLES.join(", ")
    );
    conn.execute_unprepared(&truncate_sql).await?;

    Ok(())
}
