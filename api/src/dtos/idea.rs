use crate::dtos::{company::CompanyDto, group::GroupDto, skill::SkillDto, user::UserDto};
use entity::idea_status::IdeaStatus;
use macros::IntoDataResponse;
use sea_orm::{
    DerivePartialModel, FromQueryResult,
    prelude::{DateTimeLocal, Uuid},
};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

fn validate_create_idea_status(value: &IdeaStatus) -> Result<(), ValidationError> {
    match value {
        IdeaStatus::New | IdeaStatus::OnApproval => Ok(()),
        _ => Err(ValidationError::new("Неверный статус при создании идеи")),
    }
}

#[derive(Serialize, IntoDataResponse, Debug, Deserialize, DerivePartialModel)]
#[sea_orm(entity = "entity::idea::Entity")]
pub struct IdeaDto {
    pub id: Uuid,
    #[sea_orm(nested)]
    pub initiator: UserDto,
    pub name: String,
    #[sea_orm(nested)]
    pub experts: GroupDto,
    #[sea_orm(nested, alias = "project_office")]
    pub project_office: GroupDto,
    pub status: IdeaStatus,
    pub created_at: DateTimeLocal,
    pub modified_at: DateTimeLocal,
    pub is_active: bool,
    pub problem: String,
    pub solution: String,
    pub result: String,
    pub company_id: Uuid,
    pub description: String,
    pub suitability: i64,
    pub budget: i64,
    pub pre_assessment: f64,
    pub rating: f64,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Serialize, IntoDataResponse, Debug, Deserialize)]
pub struct IdeaWithChecked {
    pub id: Uuid,
    pub initiator: UserDto,
    pub name: String,
    pub experts: Option<GroupDto>,
    pub project_office: Option<GroupDto>,
    pub company: CompanyDto,
    pub is_checked: bool,
    pub status: IdeaStatus,
    pub created_at: DateTimeLocal,
    pub modified_at: DateTimeLocal,
    pub is_active: bool,
    pub problem: String,
    pub solution: String,
    pub result: String,
    pub description: String,
    pub suitability: i64,
    pub budget: i64,
    pub pre_assessment: f64,
    pub rating: f64,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Debug, FromQueryResult)]
pub struct IdeaQueryResult {
    pub id: Uuid,

    pub initiator_id: Uuid,
    pub initiator_email: String,
    pub initiator_first_name: String,
    pub initiator_last_name: String,

    pub experts_id: Option<Uuid>,
    pub experts_name: Option<String>,

    pub project_office_id: Option<Uuid>,
    pub project_office_name: Option<String>,

    pub name: String,
    pub is_checked: bool,
    pub status: IdeaStatus,
    pub created_at: DateTimeLocal,
    pub modified_at: DateTimeLocal,
    pub is_active: bool,
    pub problem: String,
    pub solution: String,
    pub result: String,

    pub company_id: Uuid,
    pub company_contact_person: String,
    pub company_name: String,

    pub description: String,
    pub suitability: i64,
    pub budget: i64,
    pub pre_assessment: f64,
    pub rating: f64,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

impl From<IdeaQueryResult> for IdeaWithChecked {
    fn from(value: IdeaQueryResult) -> Self {
        let project_office = value.project_office_id.map(|id| GroupDto {
            id,
            name: value.project_office_name.unwrap_or_default(),
            ..Default::default()
        });
        let experts = value.experts_id.map(|id| GroupDto {
            id,
            name: value.experts_name.unwrap_or_default(),
            ..Default::default()
        });
        Self {
            id: value.id,
            initiator: UserDto {
                id: value.initiator_id,
                email: value.initiator_email,
                first_name: value.initiator_first_name,
                last_name: value.initiator_last_name,
                ..Default::default()
            },
            name: value.name,
            experts,
            project_office,
            is_checked: value.is_checked,
            status: value.status,
            created_at: value.created_at,
            modified_at: value.modified_at,
            is_active: value.is_active,
            problem: value.problem,
            solution: value.solution,
            result: value.result,
            company: CompanyDto {
                id: value.company_id,
                name: value.company_name,
                contact_person: value.company_contact_person
            },
            description: value.description,
            suitability: value.suitability,
            budget: value.budget,
            pre_assessment: value.pre_assessment,
            rating: value.rating,
            max_team_size: value.max_team_size,
            min_team_size: value.min_team_size,
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct SaveIdeaRequest {
    pub id: Option<Uuid>,
    pub name: String,
    #[validate(custom(function = "validate_create_idea_status"))]
    pub status: IdeaStatus,
    pub problem: String,
    pub solution: String,
    pub result: String,
    pub company_id: Uuid,
    pub description: String,
    pub suitability: i64,
    pub budget: i64,
    pub max_team_size: i16,
    pub min_team_size: i16,
}

#[derive(Deserialize)]
pub struct IdeaStatusRequest {
    pub id: Uuid,
    pub status: IdeaStatus,
}

#[derive(Serialize, Deserialize)]
pub struct IdeaSkillRequest {
    pub id: Uuid,
    pub skills: Vec<SkillDto>,
}
