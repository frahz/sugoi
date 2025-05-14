use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use tracing::info;

use crate::models::{get_record, StatusPagination};
use crate::templates::{RootTemplate, StatusPartialTemplate};
use crate::AppState;

pub async fn status(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<StatusPagination>,
) -> impl IntoResponse {
    let v = state
        .db
        .get_statuses()
        .await
        .expect("Couldn't get statuses");
    info!("{:?}", pagination);
    let record = get_record(v, pagination);
    if let Some(target_val) = headers.get("Hx-Target") {
        info!("Hx-Target is present: {:?}", target_val);
        if target_val == "status-table" {
            let temp = StatusPartialTemplate::new(record);
            return (StatusCode::OK, Html(temp.render().unwrap()));
        }
    }
    let temp = RootTemplate::new(record);

    (StatusCode::OK, Html(temp.render().unwrap()))
}
