/// Discovery routes: search listed halls.
///
/// Halls opt into discovery by setting is_listed to true. Searches match
/// against hall names and descriptions, ordered by approximate member count.

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use exom_core::DiscoveryResult;

use crate::state::AppState;
use super::auth::ErrorBody;

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorBody>) {
    (status, Json(ErrorBody { error: msg.to_string() }))
}

#[derive(Deserialize)]
pub struct DiscoveryQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize)]
pub struct DiscoveryResponse {
    pub hall_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_hash: Option<String>,
    pub splash_hash: Option<String>,
    pub category: Option<String>,
    pub member_count_approx: u32,
}

impl From<DiscoveryResult> for DiscoveryResponse {
    fn from(r: DiscoveryResult) -> Self {
        Self {
            hall_id: r.hall_id.to_string(),
            name: r.name,
            description: r.description,
            icon_hash: r.icon_hash,
            splash_hash: r.splash_hash,
            category: r.category,
            member_count_approx: r.member_count_approx,
        }
    }
}

/// GET /api/discovery
pub async fn search_halls(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DiscoveryQuery>,
) -> Result<Json<Vec<DiscoveryResponse>>, (StatusCode, Json<ErrorBody>)> {
    let search_term = query.q.unwrap_or_default();
    let limit = query.limit.unwrap_or(20).min(100);

    let db = state.db.lock().unwrap();
    let results = db.discovery().search(&search_term, limit)
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()))?;

    Ok(Json(results.into_iter().map(DiscoveryResponse::from).collect()))
}
