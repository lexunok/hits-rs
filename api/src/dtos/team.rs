use super::profile::UserDto;
use chrono::NaiveDate;
use entity::users;
use macros::IntoDataResponse;
use sea_orm::{
    DbErr, DerivePartialModel, EntityTrait, FromQueryResult, IdenStatic, Iterable,
    PartialModelTrait, QueryResult, TryGetError,
    prelude::{DateTimeLocal, Uuid},
    sea_query::Expr,
};
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

    #[sea_orm(nested, alias = "leader")]
    pub leader: UserDto,
    #[sea_orm(nested, alias = "owner")]
    pub owner: UserDto,
    #[sea_orm(skip)]
    pub members: Vec<UserDto>,
}
