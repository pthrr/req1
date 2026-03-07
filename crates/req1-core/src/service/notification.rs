use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use entity::notification;

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ListNotificationsFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub unread_only: bool,
}

const fn default_limit() -> u64 {
    50
}

pub struct NotificationService;

impl NotificationService {
    pub async fn list(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        filter: ListNotificationsFilter,
    ) -> Result<PaginatedResponse<notification::Model>, CoreError> {
        let mut select = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .order_by(notification::Column::CreatedAt, Order::Desc);

        if filter.unread_only {
            select = select.filter(notification::Column::Read.eq(false));
        }

        let paginator = select.paginate(db, filter.limit);
        let total = paginator.num_items().await?;
        let page = filter.offset / filter.limit;
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset: filter.offset,
            limit: filter.limit,
        })
    }

    pub async fn unread_count(db: &impl ConnectionTrait, user_id: Uuid) -> Result<u64, CoreError> {
        let count = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::Read.eq(false))
            .count(db)
            .await?;
        Ok(count)
    }

    pub async fn mark_read(
        db: &impl ConnectionTrait,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<notification::Model, CoreError> {
        let existing = notification::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("notification {id} not found")))?;

        if existing.user_id != user_id {
            return Err(CoreError::unauthorized(
                "cannot mark another user's notification".to_owned(),
            ));
        }

        let mut active: notification::ActiveModel = existing.into();
        active.read = Set(true);
        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn mark_all_read(db: &impl ConnectionTrait, user_id: Uuid) -> Result<u64, CoreError> {
        let result = notification::Entity::update_many()
            .col_expr(
                notification::Column::Read,
                sea_orm::sea_query::Expr::value(true),
            )
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::Read.eq(false))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }
}
