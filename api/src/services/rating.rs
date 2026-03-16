use crate::{
    AppState,
    dtos::rating::{RatingDto, UpdateRatingRequest},
    error::AppError,
};
use entity::{idea, idea_status::IdeaStatus, prelude::Rating, rating, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel, QueryFilter, Set,
    TransactionTrait, prelude::Uuid, sea_query::Expr,
};
use validator::Validate;

pub struct RatingService;

impl RatingService {
    pub async fn get_all(
        state: &AppState,
        idea_id: Uuid,
        expert_filter: Option<Expr>,
    ) -> Vec<RatingDto> {
        Rating::find()
            .left_join(users::Entity)
            .filter(rating::Column::IdeaId.eq(idea_id))
            .filter(Condition::all().add_option(expert_filter))
            .into_partial_model()
            .all(&state.conn)
            .await
            .unwrap_or_default()
    }

    pub async fn get_all_by_expert(
        state: &AppState,
        user_id: Uuid,
        idea_id: Uuid,
    ) -> Vec<RatingDto> {
        Self::get_all(state, idea_id, Some(rating::Column::ExpertId.eq(user_id))).await
    }

    pub async fn update(state: &AppState, payload: UpdateRatingRequest) -> Result<(), AppError> {
        payload.validate()?;
        let avg = [
            payload.budget,
            payload.market_value,
            payload.originality,
            payload.suitability,
            payload.technical_realizability,
        ]
        .into_iter()
        .map(|v| v as f64)
        .sum::<f64>()
            / 5.0;
        let mut payload = payload.into_active_model();
        payload.rating = Set(avg);
        payload.update(&state.conn).await?;
        Ok(())
    }

    pub async fn confirm(state: &AppState, payload: UpdateRatingRequest) -> Result<(), AppError> {
        payload.validate()?;
        let avg = [
            payload.budget,
            payload.market_value,
            payload.originality,
            payload.suitability,
            payload.technical_realizability,
        ]
        .into_iter()
        .map(|v| v as f64)
        .sum::<f64>()
            / 5.0;
        let mut rating = payload.into_active_model();
        rating.rating = Set(avg);
        rating.is_confirmed = Set(true);

        let txn = state.conn.begin().await?;

        let rating = rating.update(&txn).await?;

        let ratings = Rating::find()
            .filter(rating::Column::IdeaId.eq(rating.idea_id))
            .all(&txn)
            .await?;

        if ratings.iter().all(|r| r.is_confirmed) {
            let avg_rating = ratings.iter().map(|r| r.rating).sum::<f64>() / ratings.len() as f64;
            idea::ActiveModel {
                id: Set(rating.idea_id),
                status: Set(IdeaStatus::Confirmed),
                rating: Set(avg_rating),
                ..Default::default()
            }
            .update(&txn)
            .await?;
        }

        txn.commit().await?;

        Ok(())
    }
}
