use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, PrimaryKeyTrait, QueryFilter,
};
use serde::Serialize;
use uuid::Uuid;

use crate::PaginatedResponse;
use crate::error::CoreError;

pub async fn get_by_id<E>(
    db: &impl ConnectionTrait,
    id: Uuid,
    entity_name: &str,
) -> Result<<E as EntityTrait>::Model, CoreError>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync,
    <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
    E::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| CoreError::not_found(format!("{entity_name} {id} not found")))
}

pub async fn delete_by_id<E>(
    db: &impl ConnectionTrait,
    id: Uuid,
    entity_name: &str,
) -> Result<(), CoreError>
where
    E: EntityTrait,
    <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
    let result = E::delete_by_id(id).exec(db).await?;
    if result.rows_affected == 0 {
        return Err(CoreError::not_found(format!(
            "{entity_name} {id} not found"
        )));
    }
    Ok(())
}

pub async fn list_all<E>(
    db: &impl ConnectionTrait,
    offset: u64,
    limit: u64,
) -> Result<PaginatedResponse<<E as EntityTrait>::Model>, CoreError>
where
    E: EntityTrait,
    <E as EntityTrait>::Model: Sync + Serialize,
{
    let paginator = E::find().paginate(db, limit);
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

pub async fn list_filtered<E, C>(
    db: &impl ConnectionTrait,
    column: C,
    parent_id: Uuid,
    offset: u64,
    limit: u64,
) -> Result<PaginatedResponse<<E as EntityTrait>::Model>, CoreError>
where
    E: EntityTrait,
    C: ColumnTrait,
    <E as EntityTrait>::Model: Sync + Serialize,
{
    let paginator = E::find().filter(column.eq(parent_id)).paginate(db, limit);
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

/// Generate `get`, `delete`, and `list` methods on a service struct.
///
/// Flat variant (no parent filter):
/// ```ignore
/// crud_service!(WorkspaceService, workspace::Entity, "workspace");
/// ```
///
/// Nested variant (parent column filter):
/// ```ignore
/// crud_service!(ViewService, view::Entity, "view", parent: view::Column::ModuleId);
/// ```
#[macro_export]
macro_rules! crud_service {
    // Flat: list without parent filter
    ($service:ty, $entity:ty, $name:expr) => {
        impl $service {
            pub async fn get(
                db: &impl sea_orm::ConnectionTrait,
                id: uuid::Uuid,
            ) -> Result<<$entity as sea_orm::EntityTrait>::Model, $crate::error::CoreError> {
                $crate::service::crud::get_by_id::<$entity>(db, id, $name).await
            }

            pub async fn delete(
                db: &impl sea_orm::ConnectionTrait,
                id: uuid::Uuid,
            ) -> Result<(), $crate::error::CoreError> {
                $crate::service::crud::delete_by_id::<$entity>(db, id, $name).await
            }

            pub async fn list(
                db: &impl sea_orm::ConnectionTrait,
                offset: u64,
                limit: u64,
            ) -> Result<
                $crate::PaginatedResponse<<$entity as sea_orm::EntityTrait>::Model>,
                $crate::error::CoreError,
            > {
                $crate::service::crud::list_all::<$entity>(db, offset, limit).await
            }
        }
    };

    // Nested: list with parent column filter
    ($service:ty, $entity:ty, $name:expr, parent: $column:expr) => {
        impl $service {
            pub async fn get(
                db: &impl sea_orm::ConnectionTrait,
                id: uuid::Uuid,
            ) -> Result<<$entity as sea_orm::EntityTrait>::Model, $crate::error::CoreError> {
                $crate::service::crud::get_by_id::<$entity>(db, id, $name).await
            }

            pub async fn delete(
                db: &impl sea_orm::ConnectionTrait,
                id: uuid::Uuid,
            ) -> Result<(), $crate::error::CoreError> {
                $crate::service::crud::delete_by_id::<$entity>(db, id, $name).await
            }

            pub async fn list(
                db: &impl sea_orm::ConnectionTrait,
                parent_id: uuid::Uuid,
                offset: u64,
                limit: u64,
            ) -> Result<
                $crate::PaginatedResponse<<$entity as sea_orm::EntityTrait>::Model>,
                $crate::error::CoreError,
            > {
                $crate::service::crud::list_filtered::<$entity, _>(
                    db, $column, parent_id, offset, limit,
                )
                .await
            }
        }
    };
}
