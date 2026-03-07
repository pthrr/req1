use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use entity::app_user;

use crate::auth::{AuthUser, Claims};
use crate::error::CoreError;

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: app_user::Model,
}

pub struct AuthService;

impl AuthService {
    pub async fn register(
        db: &impl ConnectionTrait,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<app_user::Model, CoreError> {
        let existing = app_user::Entity::find()
            .filter(app_user::Column::Email.eq(email))
            .one(db)
            .await?;

        if existing.is_some() {
            return Err(CoreError::conflict(format!(
                "user with email '{email}' already exists"
            )));
        }

        let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| CoreError::internal(format!("failed to hash password: {e}")))?;

        let now = chrono::Utc::now().fixed_offset();
        let id = Uuid::now_v7();

        let model = app_user::ActiveModel {
            id: Set(id),
            email: Set(email.to_string()),
            display_name: Set(display_name.to_string()),
            password_hash: Set(Some(password_hash)),
            role: Set("viewer".to_string()),
            active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = model.insert(db).await?;
        Ok(result)
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn login_with_user(
        user: &app_user::Model,
        password: &str,
        jwt_secret: &str,
        exp_hours: u64,
    ) -> Result<LoginResponse, CoreError> {
        let hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| CoreError::unauthorized("account has no password set".to_string()))?;

        let valid = bcrypt::verify(password, hash)
            .map_err(|e| CoreError::internal(format!("bcrypt verify error: {e}")))?;

        if !valid {
            return Err(CoreError::unauthorized("invalid credentials".to_string()));
        }

        if !user.active {
            return Err(CoreError::unauthorized("account is disabled".to_string()));
        }

        let now = chrono::Utc::now();
        #[allow(clippy::cast_possible_wrap)]
        let exp = now + chrono::Duration::hours(exp_hours as i64);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
        )
        .map_err(|e| CoreError::internal(format!("JWT encode error: {e}")))?;

        Ok(LoginResponse {
            token,
            user: user.clone(),
        })
    }

    pub async fn login(
        db: &impl ConnectionTrait,
        email: &str,
        password: &str,
        jwt_secret: &str,
        exp_hours: u64,
    ) -> Result<LoginResponse, CoreError> {
        let user = app_user::Entity::find()
            .filter(app_user::Column::Email.eq(email))
            .one(db)
            .await?
            .ok_or_else(|| CoreError::unauthorized("invalid credentials".to_string()))?;

        Self::login_with_user(&user, password, jwt_secret, exp_hours)
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn verify_token(token: &str, jwt_secret: &str) -> Result<AuthUser, CoreError> {
        let data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(|e| CoreError::unauthorized(format!("invalid token: {e}")))?;

        let user_id: Uuid = data
            .claims
            .sub
            .parse()
            .map_err(|_| CoreError::unauthorized("invalid token subject".to_string()))?;

        Ok(AuthUser {
            id: user_id,
            email: data.claims.email,
            role: data.claims.role,
        })
    }

    pub async fn change_password(
        db: &impl ConnectionTrait,
        user_id: Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), CoreError> {
        let user = app_user::Entity::find_by_id(user_id)
            .one(db)
            .await?
            .ok_or_else(|| CoreError::not_found("user not found".to_string()))?;

        let hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| CoreError::unauthorized("account has no password set".to_string()))?;

        let valid = bcrypt::verify(old_password, hash)
            .map_err(|e| CoreError::internal(format!("bcrypt verify error: {e}")))?;

        if !valid {
            return Err(CoreError::unauthorized(
                "current password is incorrect".to_string(),
            ));
        }

        let new_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|e| CoreError::internal(format!("failed to hash password: {e}")))?;

        let mut active: app_user::ActiveModel = user.into();
        active.password_hash = Set(Some(new_hash));
        active.updated_at = Set(chrono::Utc::now().fixed_offset());
        let _ = active.update(db).await?;

        Ok(())
    }
}
