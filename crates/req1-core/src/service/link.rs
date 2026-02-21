use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::Deserialize;
use uuid::Uuid;

use entity::{link, link_type, object};

use crate::PaginatedResponse;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateLinkInput {
    pub source_object_id: Uuid,
    pub target_object_id: Uuid,
    pub link_type_id: Uuid,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkInput {
    pub suspect: Option<bool>,
    pub attributes: Option<serde_json::Value>,
}

const fn default_limit() -> u64 {
    50
}

#[derive(Debug, Deserialize)]
pub struct ListLinksFilter {
    pub source_object_id: Option<Uuid>,
    pub target_object_id: Option<Uuid>,
    pub link_type_id: Option<Uuid>,
    pub module_id: Option<Uuid>,
    #[serde(default)]
    pub offset: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkTypeInput {
    pub name: String,
    pub description: Option<String>,
}

pub struct LinkService;

impl LinkService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateLinkInput,
    ) -> Result<link::Model, CoreError> {
        if input.source_object_id == input.target_object_id {
            return Err(CoreError::BadRequest(
                "source and target must be different objects".to_owned(),
            ));
        }

        let existing = link::Entity::find()
            .filter(link::Column::SourceObjectId.eq(input.source_object_id))
            .filter(link::Column::TargetObjectId.eq(input.target_object_id))
            .filter(link::Column::LinkTypeId.eq(input.link_type_id))
            .one(db)
            .await?;
        if existing.is_some() {
            return Err(CoreError::BadRequest(
                "a link with this source, target, and type already exists".to_owned(),
            ));
        }

        // Capture current fingerprints from source and target objects
        let source = object::Entity::find_by_id(input.source_object_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                CoreError::NotFound(format!(
                    "source object {} not found",
                    input.source_object_id
                ))
            })?;
        let target = object::Entity::find_by_id(input.target_object_id)
            .one(db)
            .await?
            .ok_or_else(|| {
                CoreError::NotFound(format!(
                    "target object {} not found",
                    input.target_object_id
                ))
            })?;

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = link::ActiveModel {
            id: Set(id),
            source_object_id: Set(input.source_object_id),
            target_object_id: Set(input.target_object_id),
            link_type_id: Set(input.link_type_id),
            attributes: Set(input.attributes),
            suspect: Set(false),
            source_fingerprint: Set(source.content_fingerprint),
            target_fingerprint: Set(target.content_fingerprint),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateLinkInput,
    ) -> Result<link::Model, CoreError> {
        let existing = link::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("link {id} not found")))?;

        let mut active: link::ActiveModel = existing.clone().into();

        if let Some(suspect) = input.suspect {
            if !suspect {
                // Resolving suspect: update stored fingerprints to current object fingerprints
                let source = object::Entity::find_by_id(existing.source_object_id)
                    .one(db)
                    .await?;
                let target = object::Entity::find_by_id(existing.target_object_id)
                    .one(db)
                    .await?;
                if let Some(s) = source {
                    active.source_fingerprint = Set(s.content_fingerprint);
                }
                if let Some(t) = target {
                    active.target_fingerprint = Set(t.content_fingerprint);
                }
            }
            active.suspect = Set(suspect);
        }
        if let Some(attributes) = input.attributes {
            active.attributes = Set(Some(attributes));
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = link::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("link {id} not found")));
        }
        Ok(())
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<link::Model, CoreError> {
        link::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("link {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        filter: ListLinksFilter,
    ) -> Result<PaginatedResponse<link::Model>, CoreError> {
        let mut query = link::Entity::find();
        if let Some(source) = filter.source_object_id {
            query = query.filter(link::Column::SourceObjectId.eq(source));
        }
        if let Some(target) = filter.target_object_id {
            query = query.filter(link::Column::TargetObjectId.eq(target));
        }
        if let Some(lt) = filter.link_type_id {
            query = query.filter(link::Column::LinkTypeId.eq(lt));
        }
        if let Some(module_id) = filter.module_id {
            let object_ids: Vec<Uuid> = object::Entity::find()
                .filter(object::Column::ModuleId.eq(module_id))
                .all(db)
                .await?
                .into_iter()
                .map(|o| o.id)
                .collect();
            query = query.filter(
                link::Column::SourceObjectId
                    .is_in(object_ids.clone())
                    .or(link::Column::TargetObjectId.is_in(object_ids)),
            );
        }

        let paginator = query.paginate(db, filter.limit);
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

    pub async fn list_link_types(
        db: &impl ConnectionTrait,
    ) -> Result<Vec<link_type::Model>, CoreError> {
        let items = link_type::Entity::find().all(db).await?;
        Ok(items)
    }

    pub async fn create_link_type(
        db: &impl ConnectionTrait,
        input: CreateLinkTypeInput,
    ) -> Result<link_type::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = link_type::ActiveModel {
            id: Set(id),
            name: Set(input.name),
            description: Set(input.description),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }
}
