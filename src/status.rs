use std::fmt::Display;
use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use jiff::Zoned;

use crate::AppState;

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

pub struct Status {
    timestamp: String,
    cmd: CommandState,
    msg: String,
    status: bool,
}

impl Status {
    pub fn new(cmd: CommandState, msg: String, status: bool) -> Self {
        Self {
            timestamp: get_time(),
            cmd,
            msg,
            status,
        }
    }
}

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate<'a> {
    statuses: &'a [Status],
}

#[derive(Template)]
#[template(path = "status-tbody.html")]
struct StatusVectorTemplate<'a> {
    statuses: &'a [Status],
}

pub async fn status_root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // let mut v = Vec::new();
    // for i in 0..22 {
    //     let s = if i % 2 == 0 {
    //         Status {
    //             timestamp: get_time(),
    //             command: CommandState::Wake,
    //             message: "Sent Wake to Mac Address: AA:BB:CC:DD:EE:FF".to_string(),
    //             status: true,
    //         }
    //     } else {
    //         Status {
    //             timestamp: get_time(),
    //             command: CommandState::Sleep,
    //             message: "Sent sleep to server: inari:8253".to_string(),
    //             status: false,
    //         }
    //     };
    //     v.push(s);
    // }
    let v = state.statuses.lock().await;
    let status = StatusTemplate { statuses: &v };
    (StatusCode::OK, Html(status.render().unwrap()))
}

pub async fn status_refresh(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let v = state.statuses.lock().await;
    let status = StatusVectorTemplate { statuses: &v };
    (StatusCode::OK, Html(status.render().unwrap()))
}


fn get_time() -> String {
    Zoned::now().strftime("%Y-%m-%d %H:%M:%S %Z").to_string()
}
