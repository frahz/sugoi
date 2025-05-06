use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use jiff::Zoned;
use rusqlite::types::{FromSql, FromSqlError, ToSqlOutput};
use rusqlite::ToSql;
use serde::Deserialize;
use tracing::info;

use crate::AppState;

#[derive(Clone, Copy, Debug)]
pub enum CommandState {
    Wake,
    Sleep,
}

impl Display for CommandState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandState::Wake => write!(f, "Wake"),
            CommandState::Sleep => write!(f, "Sleep"),
        }
    }
}

impl ToSql for CommandState {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        Ok(ToSqlOutput::from(match self {
            CommandState::Wake => "Wake",
            CommandState::Sleep => "Sleep",
        }))
    }
}

impl FromSql for CommandState {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().and_then(|s| match s {
            "Wake" => Ok(CommandState::Wake),
            "Sleep" => Ok(CommandState::Sleep),
            _ => Err(FromSqlError::InvalidType),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(Zoned);

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = self.0.strftime("%Y-%m-%d %H:%M:%S %Z");
        write!(f, "{}", ts)
    }
}

impl ToSql for Timestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput> {
        let iso_string = self.0.to_string();
        Ok(iso_string.into())
    }
}

impl FromSql for Timestamp {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().and_then(|s| match Zoned::from_str(s) {
            Ok(zoned) => Ok(Self(zoned)),
            Err(err) => Err(FromSqlError::Other(Box::new(err))),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Status {
    pub timestamp: Timestamp,
    pub cmd: CommandState,
    pub msg: String,
    pub status: bool,
}

impl Status {
    pub fn new(cmd: CommandState, msg: String, status: bool) -> Self {
        Self {
            timestamp: Timestamp(Zoned::now()),
            cmd,
            msg,
            status,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct StatusPagination {
    sort: String,
    page: usize,
    per_page: usize,
}

impl Default for StatusPagination {
    fn default() -> Self {
        Self {
            sort: "desc".to_string(),
            page: 1,
            per_page: 10,
        }
    }
}

#[derive(Template)]
#[template(path = "status.html")]
struct StatusRootTemplate {
    statuses: Vec<Status>,
    rows: usize,
    current_page: usize,
    total_pages: usize,
}

impl StatusRootTemplate {
    fn new(statuses: Vec<Status>, current_page: usize, total_pages: usize) -> Self {
        let rows = statuses.len();
        Self {
            statuses,
            rows,
            current_page,
            total_pages,
        }
    }
}

#[derive(Template)]
#[template(path = "status/partial.html")]
struct StatusPartialTemplate {
    statuses: Vec<Status>,
    rows: usize,
    current_page: usize,
    total_pages: usize,
}

impl StatusPartialTemplate {
    fn new(statuses: Vec<Status>, current_page: usize, total_pages: usize) -> Self {
        let rows = statuses.len();
        Self {
            statuses,
            rows,
            current_page,
            total_pages,
        }
    }
}
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

    let pages = (v.len() as f64 / pagination.per_page as f64).ceil() as usize;
    let start = (pagination.page - 1) * pagination.per_page;
    let end = (pagination.page * pagination.per_page).min(v.len());
    let s = if !v.is_empty() {
        let mut s = v;
        if pagination.sort == "desc" {
            s.reverse();
        }
        s[start..end].to_vec()
    } else {
        v
    };

    if let Some(target_val) = headers.get("Hx-Target") {
        info!("Hx-Target is present: {:?}", target_val);
        if target_val == "status-table" {
            let temp = StatusPartialTemplate::new(s, pagination.page, pages);
            return (StatusCode::OK, Html(temp.render().unwrap()));
        }
    }
    let temp = StatusRootTemplate::new(s, pagination.page, pages);

    (StatusCode::OK, Html(temp.render().unwrap()))
}
