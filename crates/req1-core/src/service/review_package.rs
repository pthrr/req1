use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entity::{review_assignment, review_package};

use crate::PaginatedResponse;
use crate::error::CoreError;
use crate::service::e_signature::{ESignatureService, SignInput};

#[derive(Debug, Serialize)]
pub struct VotingSummary {
    pub package_id: Uuid,
    pub package_name: String,
    pub package_status: String,
    pub total_assignments: u64,
    pub approved: u64,
    pub rejected: u64,
    pub abstained: u64,
    pub pending: u64,
}

const VALID_STATUSES: &[&str] = &[
    "draft",
    "open",
    "in_review",
    "approved",
    "rejected",
    "closed",
];

#[derive(Debug, Deserialize)]
pub struct CreateReviewPackageInput {
    #[serde(default)]
    pub module_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReviewPackageInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
}

pub struct ReviewPackageService;

impl ReviewPackageService {
    pub async fn create(
        db: &impl ConnectionTrait,
        input: CreateReviewPackageInput,
    ) -> Result<review_package::Model, CoreError> {
        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = review_package::ActiveModel {
            id: Set(id),
            module_id: Set(input.module_id),
            name: Set(input.name),
            description: Set(input.description),
            status: Set("draft".to_owned()),
            created_by: Set(input.created_by),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    pub async fn update(
        db: &impl ConnectionTrait,
        id: Uuid,
        input: UpdateReviewPackageInput,
    ) -> Result<review_package::Model, CoreError> {
        let existing = review_package::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_package {id} not found")))?;

        let mut active: review_package::ActiveModel = existing.into();
        if let Some(name) = input.name {
            active.name = Set(name);
        }
        if let Some(description) = input.description {
            active.description = Set(Some(description));
        }
        if let Some(ref status) = input.status {
            if !VALID_STATUSES.contains(&status.as_str()) {
                return Err(CoreError::BadRequest(format!(
                    "invalid status '{status}', must be one of: {VALID_STATUSES:?}"
                )));
            }
            active.status = Set(status.clone());
        }
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }

    pub async fn delete(db: &impl ConnectionTrait, id: Uuid) -> Result<(), CoreError> {
        let result = review_package::Entity::delete_by_id(id).exec(db).await?;
        if result.rows_affected == 0 {
            return Err(CoreError::NotFound(format!(
                "review_package {id} not found"
            )));
        }
        Ok(())
    }

    pub async fn get(
        db: &impl ConnectionTrait,
        id: Uuid,
    ) -> Result<review_package::Model, CoreError> {
        review_package::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_package {id} not found")))
    }

    pub async fn list(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        offset: u64,
        limit: u64,
    ) -> Result<PaginatedResponse<review_package::Model>, CoreError> {
        let paginator = review_package::Entity::find()
            .filter(review_package::Column::ModuleId.eq(module_id))
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

    pub async fn voting_summary(
        db: &impl ConnectionTrait,
        module_id: Uuid,
    ) -> Result<Vec<VotingSummary>, CoreError> {
        let packages = review_package::Entity::find()
            .filter(review_package::Column::ModuleId.eq(module_id))
            .all(db)
            .await?;

        let mut summaries = Vec::new();
        for pkg in &packages {
            let all_assignments = review_assignment::Entity::find()
                .filter(review_assignment::Column::PackageId.eq(pkg.id))
                .all(db)
                .await?;

            let total = all_assignments.len() as u64;
            let approved = all_assignments
                .iter()
                .filter(|a| a.status == "approved")
                .count() as u64;
            let rejected = all_assignments
                .iter()
                .filter(|a| a.status == "rejected")
                .count() as u64;
            let abstained = all_assignments
                .iter()
                .filter(|a| a.status == "abstained")
                .count() as u64;
            let pending = all_assignments
                .iter()
                .filter(|a| a.status == "pending")
                .count() as u64;

            summaries.push(VotingSummary {
                package_id: pkg.id,
                package_name: pkg.name.clone(),
                package_status: pkg.status.clone(),
                total_assignments: total,
                approved,
                rejected,
                abstained,
                pending,
            });
        }

        Ok(summaries)
    }

    /// Transition a review package status with optional e-signature enforcement.
    pub async fn transition_status(
        db: &impl ConnectionTrait,
        id: Uuid,
        new_status: &str,
        signer_id: Uuid,
        sign_input: Option<SignInput>,
    ) -> Result<review_package::Model, CoreError> {
        if !VALID_STATUSES.contains(&new_status) {
            return Err(CoreError::BadRequest(format!(
                "invalid status '{new_status}', must be one of: {VALID_STATUSES:?}"
            )));
        }

        let existing = review_package::Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("review_package {id} not found")))?;

        // Check if signature is required
        let (needs_sig, needs_four_eyes) = ESignatureService::requires_signature(
            db,
            existing.module_id,
            &existing.status,
            new_status,
        )
        .await?;

        if needs_sig {
            let input = sign_input.ok_or_else(|| {
                CoreError::BadRequest(
                    "this transition requires an e-signature (password + meaning)".to_owned(),
                )
            })?;

            if needs_four_eyes {
                ESignatureService::check_four_eyes(db, signer_id, "review_package", id).await?;
            }

            let _sig = ESignatureService::sign(db, signer_id, "review_package", id, input).await?;
        }

        let mut active: review_package::ActiveModel = existing.into();
        active.status = Set(new_status.to_owned());
        active.updated_at = Set(chrono::Utc::now().fixed_offset());

        let result = active.update(db).await?;
        Ok(result)
    }
}
