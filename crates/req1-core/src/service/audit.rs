use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use entity::audit_log;

use crate::PaginatedResponse;
use crate::error::CoreError;

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct AuditLogFilter {
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub user_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action: Option<String>,
}

pub struct AuditService;

impl AuditService {
    pub async fn log(
        db: &impl ConnectionTrait,
        user_id: Option<Uuid>,
        action: &str,
        entity_type: &str,
        entity_id: Option<Uuid>,
        details: Option<serde_json::Value>,
        ip_address: Option<String>,
    ) -> Result<(), CoreError> {
        let model = audit_log::ActiveModel {
            user_id: Set(user_id),
            action: Set(action.to_string()),
            entity_type: Set(entity_type.to_string()),
            entity_id: Set(entity_id),
            details: Set(details),
            ip_address: Set(ip_address),
            created_at: Set(chrono::Utc::now().fixed_offset()),
            ..Default::default()
        };

        let _ = model.insert(db).await?;
        Ok(())
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        filter: AuditLogFilter,
    ) -> Result<PaginatedResponse<audit_log::Model>, CoreError> {
        let mut select =
            audit_log::Entity::find().order_by(audit_log::Column::CreatedAt, Order::Desc);

        if let Some(user_id) = filter.user_id {
            select = select.filter(audit_log::Column::UserId.eq(user_id));
        }
        if let Some(ref entity_type) = filter.entity_type {
            select = select.filter(audit_log::Column::EntityType.eq(entity_type.as_str()));
        }
        if let Some(entity_id) = filter.entity_id {
            select = select.filter(audit_log::Column::EntityId.eq(entity_id));
        }
        if let Some(ref action) = filter.action {
            select = select.filter(audit_log::Column::Action.eq(action.as_str()));
        }

        let paginator = select.paginate(db, filter.limit);
        let total = paginator.num_items().await?;
        let page = filter.offset.checked_div(filter.limit).unwrap_or(0);
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset: filter.offset,
            limit: filter.limit,
        })
    }
}
