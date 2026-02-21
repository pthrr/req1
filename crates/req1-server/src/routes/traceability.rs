use std::collections::HashSet;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::{ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::{link, object};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/traceability-matrix", get(get_traceability_matrix))
        .route("/modules/{module_id}/coverage", get(get_coverage))
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Deserialize)]
struct TraceabilityMatrixQuery {
    source_module_id: Uuid,
    target_module_id: Uuid,
    link_type_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct MatrixObject {
    id: Uuid,
    heading: Option<String>,
    position: i32,
}

#[derive(Debug, Serialize)]
struct MatrixCell {
    source_id: Uuid,
    target_id: Uuid,
    link_id: Uuid,
    suspect: bool,
}

#[derive(Debug, Serialize)]
struct TraceabilityMatrixResponse {
    source_objects: Vec<MatrixObject>,
    target_objects: Vec<MatrixObject>,
    cells: Vec<MatrixCell>,
}

async fn get_traceability_matrix(
    State(state): State<AppState>,
    Query(params): Query<TraceabilityMatrixQuery>,
) -> Result<Json<TraceabilityMatrixResponse>, AppError> {
    // 1. Fetch source objects ordered by position
    let source_objects: Vec<object::Model> = object::Entity::find()
        .filter(object::Column::ModuleId.eq(params.source_module_id))
        .order_by(object::Column::Position, Order::Asc)
        .all(&state.db)
        .await?;

    // 2. Fetch target objects ordered by position
    let target_objects: Vec<object::Model> = object::Entity::find()
        .filter(object::Column::ModuleId.eq(params.target_module_id))
        .order_by(object::Column::Position, Order::Asc)
        .all(&state.db)
        .await?;

    let source_ids: Vec<Uuid> = source_objects.iter().map(|o| o.id).collect();
    let target_ids: Vec<Uuid> = target_objects.iter().map(|o| o.id).collect();

    // 3. Fetch links where (source in source_ids AND target in target_ids) OR vice versa
    let mut link_query = link::Entity::find().filter(
        link::Column::SourceObjectId
            .is_in(source_ids.clone())
            .and(link::Column::TargetObjectId.is_in(target_ids.clone()))
            .or(link::Column::SourceObjectId
                .is_in(target_ids.clone())
                .and(link::Column::TargetObjectId.is_in(source_ids.clone()))),
    );

    if let Some(lt) = params.link_type_id {
        link_query = link_query.filter(link::Column::LinkTypeId.eq(lt));
    }

    let links = link_query.all(&state.db).await?;

    // Build set for fast membership check
    let source_set: HashSet<Uuid> = source_ids.into_iter().collect();
    let target_set: HashSet<Uuid> = target_ids.into_iter().collect();

    // 4. Normalize direction: source always from source_module, target from target_module
    let cells: Vec<MatrixCell> = links
        .into_iter()
        .filter_map(|l| {
            if source_set.contains(&l.source_object_id) && target_set.contains(&l.target_object_id)
            {
                Some(MatrixCell {
                    source_id: l.source_object_id,
                    target_id: l.target_object_id,
                    link_id: l.id,
                    suspect: l.suspect,
                })
            } else if source_set.contains(&l.target_object_id)
                && target_set.contains(&l.source_object_id)
            {
                Some(MatrixCell {
                    source_id: l.target_object_id,
                    target_id: l.source_object_id,
                    link_id: l.id,
                    suspect: l.suspect,
                })
            } else {
                None
            }
        })
        .collect();

    let resp = TraceabilityMatrixResponse {
        source_objects: source_objects
            .into_iter()
            .map(|o| MatrixObject {
                id: o.id,
                heading: o.heading,
                position: o.position,
            })
            .collect(),
        target_objects: target_objects
            .into_iter()
            .map(|o| MatrixObject {
                id: o.id,
                heading: o.heading,
                position: o.position,
            })
            .collect(),
        cells,
    };

    Ok(Json(resp))
}

#[derive(Debug, Serialize)]
struct CoverageResponse {
    total_objects: u64,
    with_upstream: u64,
    with_downstream: u64,
    with_any_link: u64,
    upstream_pct: f64,
    downstream_pct: f64,
    any_link_pct: f64,
}

#[allow(clippy::cast_precision_loss)]
async fn get_coverage(
    State(state): State<AppState>,
    Path(module_id): Path<Uuid>,
) -> Result<Json<CoverageResponse>, AppError> {
    let objects = object::Entity::find()
        .filter(object::Column::ModuleId.eq(module_id))
        .filter(object::Column::DeletedAt.is_null())
        .all(&state.db)
        .await?;

    let object_ids: HashSet<Uuid> = objects.iter().map(|o| o.id).collect();
    let total = object_ids.len() as u64;

    if total == 0 {
        return Ok(Json(CoverageResponse {
            total_objects: 0,
            with_upstream: 0,
            with_downstream: 0,
            with_any_link: 0,
            upstream_pct: 0.0,
            downstream_pct: 0.0,
            any_link_pct: 0.0,
        }));
    }

    let id_vec: Vec<Uuid> = object_ids.iter().copied().collect();
    let links = link::Entity::find()
        .filter(
            link::Column::SourceObjectId
                .is_in(id_vec.clone())
                .or(link::Column::TargetObjectId.is_in(id_vec)),
        )
        .all(&state.db)
        .await?;

    let mut has_downstream: HashSet<Uuid> = HashSet::new(); // objects appearing as source
    let mut has_upstream: HashSet<Uuid> = HashSet::new(); // objects appearing as target

    for lnk in &links {
        if object_ids.contains(&lnk.source_object_id) {
            let _ = has_downstream.insert(lnk.source_object_id);
        }
        if object_ids.contains(&lnk.target_object_id) {
            let _ = has_upstream.insert(lnk.target_object_id);
        }
    }

    let with_any: HashSet<Uuid> = has_downstream.union(&has_upstream).copied().collect();
    let total_f = total as f64;

    Ok(Json(CoverageResponse {
        total_objects: total,
        with_upstream: has_upstream.len() as u64,
        with_downstream: has_downstream.len() as u64,
        with_any_link: with_any.len() as u64,
        upstream_pct: (has_upstream.len() as f64 / total_f * 100.0),
        downstream_pct: (has_downstream.len() as f64 / total_f * 100.0),
        any_link_pct: (with_any.len() as f64 / total_f * 100.0),
    }))
}
