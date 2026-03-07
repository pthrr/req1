use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use entity::attachment;

use crate::error::CoreError;

pub struct AttachmentService;

impl AttachmentService {
    pub async fn create(
        db: &impl ConnectionTrait,
        object_id: Uuid,
        file_name: String,
        content_type: String,
        data: &[u8],
        upload_dir: &str,
    ) -> Result<attachment::Model, CoreError> {
        let id = Uuid::now_v7();
        let size_bytes = i64::try_from(data.len())
            .map_err(|_| CoreError::bad_request("file too large".to_owned()))?;

        // Compute SHA-256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256 = format!("{:x}", hasher.finalize());

        // Store to filesystem: {upload_dir}/{object_id}/{filename}
        let dir = format!("{upload_dir}/{object_id}");
        std::fs::create_dir_all(&dir)
            .map_err(|e| CoreError::internal(format!("failed to create upload dir: {e}")))?;
        let storage_path = format!("{dir}/{id}_{file_name}");
        std::fs::write(&storage_path, data)
            .map_err(|e| CoreError::internal(format!("failed to write file: {e}")))?;

        let now = chrono::Utc::now().fixed_offset();
        let model = attachment::ActiveModel {
            id: Set(id),
            object_id: Set(object_id),
            file_name: Set(file_name),
            content_type: Set(content_type),
            size_bytes: Set(size_bytes),
            storage_path: Set(storage_path),
            sha256: Set(Some(sha256)),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        object_id: Uuid,
    ) -> Result<Vec<attachment::Model>, CoreError> {
        let items = attachment::Entity::find()
            .filter(attachment::Column::ObjectId.eq(object_id))
            .all(db)
            .await?;
        Ok(items)
    }

    pub async fn get(db: &impl ConnectionTrait, id: Uuid) -> Result<attachment::Model, CoreError> {
        attachment::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found(format!("attachment {id} not found")))
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let existing = Self::get(db, id).await?;

        // Remove file from filesystem
        if std::fs::remove_file(&existing.storage_path).is_err() {
            tracing::warn!(
                "failed to remove attachment file: {}",
                existing.storage_path
            );
        }

        let result = attachment::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::not_found(format!("attachment {id} not found")));
        }
        Ok(())
    }

    pub fn read_file(storage_path: &str) -> Result<Vec<u8>, CoreError> {
        std::fs::read(storage_path)
            .map_err(|e| CoreError::internal(format!("failed to read file: {e}")))
    }

    pub fn verify_integrity(attachment: &attachment::Model, data: &[u8]) -> bool {
        let Some(ref expected_sha) = attachment.sha256 else {
            return true; // No SHA stored, cannot verify — assume ok
        };
        let mut hasher = Sha256::new();
        hasher.update(data);
        let actual_sha = format!("{:x}", hasher.finalize());
        actual_sha == *expected_sha
    }
}
