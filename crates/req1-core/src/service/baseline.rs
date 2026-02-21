use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
    Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{baseline, baseline_entry};

use crate::PaginatedResponse;
use crate::baseline as baseline_core;
use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct CreateBaselineInput {
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub baseline_set_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BaselineWithEntries {
    #[serde(flatten)]
    pub baseline: baseline::Model,
    pub entries: Vec<baseline_entry::Model>,
}

#[derive(Debug, Deserialize)]
pub struct DiffBaselineInput {
    pub a: Uuid,
    pub b: Uuid,
}

#[derive(Debug, Serialize)]
pub struct BaselineDiff {
    pub baseline_a: Uuid,
    pub baseline_b: Uuid,
    pub added: Vec<DiffEntry>,
    pub removed: Vec<DiffEntry>,
    pub modified: Vec<DiffModified>,
}

#[derive(Debug, Serialize)]
pub struct DiffEntry {
    pub object_id: Uuid,
    pub version: i32,
    pub heading: Option<String>,
    pub body: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DiffModified {
    pub object_id: Uuid,
    pub version_a: i32,
    pub version_b: i32,
    pub heading_a: Option<String>,
    pub heading_b: Option<String>,
    pub body_a: Option<String>,
    pub body_b: Option<String>,
    pub attributes_a: Option<serde_json::Value>,
    pub attributes_b: Option<serde_json::Value>,
}

pub struct BaselineService;

impl BaselineService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateBaselineInput,
    ) -> Result<BaselineWithEntries, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = baseline::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            description: Set(input.description),
            created_by: Set(None),
            created_at: Set(now),
            locked: Set(true),
            baseline_set_id: Set(input.baseline_set_id),
        };

        let bl = model.insert(db).await?;
        let entries = baseline_core::snapshot_baseline(db, id, input.module_id).await?;

        Ok(BaselineWithEntries {
            baseline: bl,
            entries,
        })
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<BaselineWithEntries, CoreError> {
        let bl = baseline::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("baseline {id} not found")))?;

        let entries = baseline_entry::Entity::find()
            .filter(baseline_entry::Column::BaselineId.eq(id))
            .all(db)
            .await?;

        Ok(BaselineWithEntries {
            baseline: bl,
            entries,
        })
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = baseline::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!("baseline {id} not found")));
        }
        Ok(())
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<baseline::Model>, CoreError> {
        let paginator = baseline::Entity::find()
            .filter(baseline::Column::ModuleId.eq(module_id))
            .paginate(db, limit);
        let total = paginator.num_items().await?;
        let page = offset / limit;
        let items = paginator.fetch_page(page).await?;

        Ok(PaginatedResponse {
            items,
            total,
            offset,
            limit,
        })
    }

    pub async fn diff(
        db: &impl ConnectionTrait,
        input: DiffBaselineInput,
    ) -> Result<BaselineDiff, CoreError> {
        let sql = r"
            SELECT
                COALESCE(a.object_id, b.object_id) AS object_id,
                a.version AS version_a,
                b.version AS version_b,
                ha.heading AS heading_a,
                hb.heading AS heading_b,
                ha.body AS body_a,
                hb.body AS body_b,
                ha.attribute_values AS attributes_a,
                hb.attribute_values AS attributes_b
            FROM
                (SELECT object_id, version FROM baseline_entry WHERE baseline_id = $1) a
            FULL OUTER JOIN
                (SELECT object_id, version FROM baseline_entry WHERE baseline_id = $2) b
                ON a.object_id = b.object_id
            LEFT JOIN object_history ha
                ON ha.object_id = a.object_id AND ha.version = a.version
            LEFT JOIN object_history hb
                ON hb.object_id = b.object_id AND hb.version = b.version
        ";

        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            sql,
            [input.a.into(), input.b.into()],
        );

        let rows = db.query_all(stmt).await?;

        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        for row in &rows {
            let object_id: Uuid = row.try_get("", "object_id")?;
            let version_a: Option<i32> = row.try_get("", "version_a")?;
            let version_b: Option<i32> = row.try_get("", "version_b")?;
            let heading_a: Option<String> = row.try_get("", "heading_a")?;
            let heading_b: Option<String> = row.try_get("", "heading_b")?;
            let body_a: Option<String> = row.try_get("", "body_a")?;
            let body_b: Option<String> = row.try_get("", "body_b")?;
            let attributes_a: Option<serde_json::Value> = row.try_get("", "attributes_a")?;
            let attributes_b: Option<serde_json::Value> = row.try_get("", "attributes_b")?;

            match (version_a, version_b) {
                (None, Some(vb)) => {
                    added.push(DiffEntry {
                        object_id,
                        version: vb,
                        heading: heading_b,
                        body: body_b,
                        attributes: attributes_b,
                    });
                }
                (Some(va), None) => {
                    removed.push(DiffEntry {
                        object_id,
                        version: va,
                        heading: heading_a,
                        body: body_a,
                        attributes: attributes_a,
                    });
                }
                (Some(va), Some(vb)) if va != vb => {
                    modified.push(DiffModified {
                        object_id,
                        version_a: va,
                        version_b: vb,
                        heading_a,
                        heading_b,
                        body_a,
                        body_b,
                        attributes_a,
                        attributes_b,
                    });
                }
                _ => {}
            }
        }

        Ok(BaselineDiff {
            baseline_a: input.a,
            baseline_b: input.b,
            added,
            removed,
            modified,
        })
    }
}
