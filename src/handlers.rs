use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use tracing::info;

use crate::models::{StatusPagination, StatusRecord};
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
    let total_items = v.len();

    info!("{:?}", pagination);

    let pages = (total_items as f64 / pagination.per_page as f64).ceil() as usize;
    let start = (pagination.page - 1) * pagination.per_page;
    let end = (pagination.page * pagination.per_page).min(total_items);
    let statuses = if !v.is_empty() {
        let mut s = v;
        if pagination.sort == "desc" {
            s.reverse();
        }
        if start >= end {
            Vec::new()
        } else {
            s[start..end].to_vec()
        }
    } else {
        v
    };

    let record = StatusRecord::new(
        pagination.page,
        pagination.per_page,
        pages,
        total_items,
        statuses,
    );
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
