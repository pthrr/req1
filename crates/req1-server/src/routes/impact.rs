use std::collections::{HashMap, HashSet, VecDeque};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};
use entity::{link, link_type, object};

pub fn routes() -> Router<AppState> {
    Router::new().route("/object-impact/{id}", get(get_impact))
}

#[derive(Debug, Deserialize)]
struct ImpactQuery {
    direction: Option<String>,
    max_depth: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ImpactObject {
    id: Uuid,
    heading: Option<String>,
    level: String,
    depth: u32,
    link_type: Option<String>,
    module_id: Uuid,
}

#[derive(Debug, Serialize)]
struct ImpactEdge {
    source_id: Uuid,
    target_id: Uuid,
    link_type: Option<String>,
    suspect: bool,
}

#[derive(Debug, Serialize)]
struct ImpactResponse {
    root_id: Uuid,
    direction: String,
    max_depth: u32,
    objects: Vec<ImpactObject>,
    edges: Vec<ImpactEdge>,
}

struct QueueEntry {
    object_id: Uuid,
    depth: u32,
    link_type_id: Option<Uuid>,
}

/// Given a link and a node, return the neighbor in the given traversal direction.
fn link_neighbor(lnk: &link::Model, node_id: Uuid, direction: &str) -> Option<Uuid> {
    match direction {
        "forward" | "both" if lnk.source_object_id == node_id => Some(lnk.target_object_id),
        "backward" | "both" if lnk.target_object_id == node_id => Some(lnk.source_object_id),
        _ => None,
    }
}

/// `(object_id, depth, link_type_id)`
type BfsResult = Vec<(Uuid, u32, Option<Uuid>)>;
/// `(source, target, link_type_id, suspect)`
type BfsEdge = Vec<(Uuid, Uuid, Uuid, bool)>;

/// BFS traversal from `root_id` along links in the given `direction`, up to `max_depth`.
fn bfs_traverse(
    root_id: Uuid,
    direction: &str,
    max_depth: u32,
    all_links: &[link::Model],
) -> (BfsResult, BfsEdge) {
    let mut visited: HashSet<Uuid> = HashSet::new();
    let _ = visited.insert(root_id);

    let mut queue: VecDeque<QueueEntry> = VecDeque::new();
    let mut result_entries: Vec<(Uuid, u32, Option<Uuid>)> = Vec::new();
    let mut edge_entries: Vec<(Uuid, Uuid, Uuid, bool)> = Vec::new();

    // Seed BFS with neighbors of root
    for lnk in all_links {
        if let Some(nid) = link_neighbor(lnk, root_id, direction)
            && visited.insert(nid)
        {
            queue.push_back(QueueEntry {
                object_id: nid,
                depth: 1,
                link_type_id: Some(lnk.link_type_id),
            });
            edge_entries.push((
                lnk.source_object_id,
                lnk.target_object_id,
                lnk.link_type_id,
                lnk.suspect,
            ));
        }
    }

    while let Some(entry) = queue.pop_front() {
        result_entries.push((entry.object_id, entry.depth, entry.link_type_id));

        if entry.depth >= max_depth {
            continue;
        }

        for lnk in all_links {
            if let Some(nid) = link_neighbor(lnk, entry.object_id, direction)
                && visited.insert(nid)
            {
                queue.push_back(QueueEntry {
                    object_id: nid,
                    depth: entry.depth + 1,
                    link_type_id: Some(lnk.link_type_id),
                });
                edge_entries.push((
                    lnk.source_object_id,
                    lnk.target_object_id,
                    lnk.link_type_id,
                    lnk.suspect,
                ));
            }
        }
    }

    (result_entries, edge_entries)
}

async fn get_impact(
    State(state): State<AppState>,
    Path(root_id): Path<Uuid>,
    Query(query): Query<ImpactQuery>,
) -> Result<Json<ImpactResponse>, AppError> {
    let direction = query.direction.as_deref().unwrap_or("both");
    let max_depth = query.max_depth.unwrap_or(5).min(20);

    // Validate direction
    if !["forward", "backward", "both"].contains(&direction) {
        return Err(AppError::BadRequest(format!(
            "invalid direction '{direction}', must be one of: forward, backward, both"
        )));
    }

    // Verify root exists
    let _root = object::Entity::find_by_id(root_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("object {root_id} not found")))?;

    let all_links = link::Entity::find().all(&state.db).await?;
    let (result_entries, edge_entries) = bfs_traverse(root_id, direction, max_depth, &all_links);

    // Load object details for all collected IDs
    let object_ids: Vec<Uuid> = result_entries.iter().map(|(id, _, _)| *id).collect();
    let objects_map: HashMap<Uuid, object::Model> = if object_ids.is_empty() {
        HashMap::new()
    } else {
        object::Entity::find()
            .filter(object::Column::Id.is_in(object_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|o| (o.id, o))
            .collect()
    };

    // Load link type names
    let link_type_ids: HashSet<Uuid> = result_entries.iter().filter_map(|(_, _, lt)| *lt).collect();
    let link_types_map: HashMap<Uuid, String> = if link_type_ids.is_empty() {
        HashMap::new()
    } else {
        link_type::Entity::find()
            .filter(link_type::Column::Id.is_in(link_type_ids.into_iter().collect::<Vec<_>>()))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|lt| (lt.id, lt.name))
            .collect()
    };

    // Build response
    let impact_objects: Vec<ImpactObject> = result_entries
        .into_iter()
        .filter_map(|(id, depth, lt_id)| {
            let obj = objects_map.get(&id)?;
            Some(ImpactObject {
                id: obj.id,
                heading: obj.heading.clone(),
                level: obj.level.clone(),
                depth,
                link_type: lt_id.and_then(|lt| link_types_map.get(&lt).cloned()),
                module_id: obj.module_id,
            })
        })
        .collect();

    // Build edges
    let impact_edges: Vec<ImpactEdge> = edge_entries
        .into_iter()
        .map(|(src, tgt, lt_id, suspect)| ImpactEdge {
            source_id: src,
            target_id: tgt,
            link_type: link_types_map.get(&lt_id).cloned(),
            suspect,
        })
        .collect();

    Ok(Json(ImpactResponse {
        root_id,
        direction: direction.to_owned(),
        max_depth,
        objects: impact_objects,
        edges: impact_edges,
    }))
}
