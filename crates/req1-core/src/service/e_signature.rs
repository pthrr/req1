use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use entity::{app_user, e_signature, module, review_package};

use crate::error::CoreError;

#[derive(Debug, Deserialize)]
pub struct SignInput {
    pub password: String,
    pub meaning: String,
    pub ip_address: Option<String>,
}

pub struct ESignatureService;

impl ESignatureService {
    /// Create an e-signature by re-authenticating the user with their password.
    pub async fn sign(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        input: SignInput,
    ) -> Result<e_signature::Model, CoreError> {
        // Re-authenticate: load user and verify password
        let user = app_user::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound("user not found".to_owned()))?;

        let hash = user.password_hash.as_deref().ok_or_else(|| {
            CoreError::Unauthorized("account has no password set".to_owned())
        })?;

        let valid = bcrypt::verify(&input.password, hash)
            .map_err(|e| CoreError::Internal(format!("bcrypt verify error: {e}")))?;

        if !valid {
            return Err(CoreError::Unauthorized(
                "invalid password for e-signature".to_owned(),
            ));
        }

        // Compute signature hash: SHA-256 of (user_id + entity_type + entity_id + meaning + timestamp)
        let now = chrono::Utc::now().fixed_offset();
        let sig_data = format!(
            "{user_id}:{entity_type}:{entity_id}:{}:{}",
            input.meaning,
            now.to_rfc3339()
        );
        let signature_hash = format!("{:x}", Sha256::digest(sig_data.as_bytes()));

        let id = Uuid::now_v7();
        let model = e_signature::ActiveModel {
            id: Set(id),
            user_id: Set(user_id),
            entity_type: Set(entity_type.to_owned()),
            entity_id: Set(entity_id),
            meaning: Set(input.meaning),
            signature_hash: Set(signature_hash),
            ip_address: Set(input.ip_address),
            created_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    /// Check four-eyes principle: signer must not be the creator of the entity.
    pub async fn check_four_eyes(
        db: &impl ConnectionTrait,
        signer_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<(), CoreError> {
        if entity_type == "review_package" {
            let pkg = review_package::Entity::find_by_id(entity_id)
                .one(db)
                .await?
                .ok_or_else(|| {
                    CoreError::NotFound(format!("review_package {entity_id} not found"))
                })?;
            if pkg.created_by == Some(signer_id) {
                return Err(CoreError::BadRequest(
                    "four-eyes principle: signer cannot be the creator".to_owned(),
                ));
            }
        }
        Ok(())
    }

    /// Check if a transition requires a signature based on `module.signature_config`.
    /// Returns (`needs_signature`, `needs_four_eyes`).
    pub async fn requires_signature(
        db: &impl ConnectionTrait,
        module_id: Uuid,
        from_state: &str,
        to_state: &str,
    ) -> Result<(bool, bool), CoreError> {
        let mod_entity = module::Entity::find_by_id(module_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("module {module_id} not found")))?;

        let config = &mod_entity.signature_config;
        let transition_key = format!("{from_state}->{to_state}");

        let needs_sig = config
            .get("require_signature_transitions")
            .and_then(|v| v.as_array())
            .is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some(&transition_key)));

        let needs_four_eyes = config
            .get("require_four_eyes")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Ok((needs_sig, needs_sig && needs_four_eyes))
    }

    /// List all signatures for a given entity.
    pub async fn list_for_entity(
        db: &impl ConnectionTrait,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<e_signature::Model>, CoreError> {
        let results = e_signature::Entity::find()
            .filter(e_signature::Column::EntityType.eq(entity_type))
            .filter(e_signature::Column::EntityId.eq(entity_id))
            .all(db)
            .await?;
        Ok(results)
    }
}
