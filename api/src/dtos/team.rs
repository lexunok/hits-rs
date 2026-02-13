use super::profile::UserDto;
use chrono::NaiveDate;
use entity::users;
use macros::IntoDataResponse;
use sea_orm::{
    sea_query::{Asterisk, Query, Expr, ExprTrait, },ColumnTrait,
    DbErr, DeriveIntoActiveModel, DerivePartialModel, EntityTrait, FromQueryResult, IdenStatic, Iterable, PartialModelTrait, QueryResult, TryGetError, prelude::{DateTimeLocal, Uuid}, sea_query::Expr
};
use entity::team_member;
use entity::team;
use entity::team::ActiveModel;
use serde::{Deserialize, Serialize};

#[derive(Serialize, IntoDataResponse, Debug, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::team::Entity")]
pub struct TeamDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_closed: bool,
    pub has_active_project: bool,
    pub created_at: NaiveDate,
    #[sea_orm(
        from_expr = r#"
            Expr::cust(
                "(SELECT COUNT(*) \
                FROM team_member \
                WHERE team_id = team.id \
                    AND finish_date IS NULL)"
            )
        "#
    )]
    pub members_count: i64,

    #[sea_orm(nested, alias = "leader")]
    pub leader: Option<UserDto>,
    #[sea_orm(nested, alias = "owner")]
    pub owner: UserDto,
    #[sea_orm(skip)]
    pub members: Vec<UserDto>,
}

#[derive(Deserialize, Debug)]
pub struct CreateTeamRequest {
    pub name: String,
    pub description: String,
    pub is_closed: bool,
    pub owner_id: Uuid,
    pub leader_id: Option<Uuid>, 
    pub members: Vec<Uuid>,
    pub wanted_skills: Vec<Uuid>
}