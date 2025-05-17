use std::sync::Arc;

use askama::Template;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::Form;
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{error, info};

use crate::api::{DEFAULT_BROADCAST_IP, MAGIC_PACKET};
use crate::models::{get_record, CommandState, SleepForm, Status, StatusPagination, WakeForm};
use crate::templates::{RootTemplate, StatusPartialTemplate, ToastFragment};
use crate::AppState;

pub async fn wake(
    State(state): State<Arc<AppState>>,
    Form(wake_form): Form<WakeForm>,
) -> impl IntoResponse {
    let Ok(mac_addr) = MacAddress::parse(&wake_form.mac_address) else {
        let msg = format!("Invalid Mac Address: {}.", wake_form.mac_address);
        error!(msg);
        let toast = ToastFragment::new(msg, false);
        return (StatusCode::OK, Html(toast.render().unwrap()));
    };
    let wol_packet = WolPacket::create(&mac_addr);
    let socket = create_socket(DEFAULT_BROADCAST_IP).unwrap();
    let (s, m) = match socket.send_to(&wol_packet.0, DEFAULT_BROADCAST_IP) {
        Ok(_) => {
            info!("Sent wake to server packet len: {}", &wol_packet.0.len());
            (true, format!("MAC Address: {}", wake_form.mac_address))
        }
        Err(e) => {
            let msg = format!("Failed to send packet: {e}");
            error!(msg);
            (false, msg)
        }
    };
    let status = Status::new(CommandState::Wake, m.clone(), s);
    state
        .db
        .add_status(status)
        .await
        .expect("Couldn't write to the database");
    let msg = if s {
        "Succesfully sent wake command".to_owned()
    } else {
        m
    };
    let toast = ToastFragment::new(msg, s);
    (StatusCode::OK, Html(toast.render().unwrap()))
}

pub async fn sleep(
    State(state): State<Arc<AppState>>,
    Form(sleep_form): Form<SleepForm>,
) -> impl IntoResponse {
    // "127.0.0.1:8080"
    let (s, m) = match TcpStream::connect(sleep_form.address.as_str()).await {
        Ok(mut stream) => {
            info!("Sending sleep to server: {}", sleep_form.address);
            stream.write_u8(MAGIC_PACKET).await.unwrap();
            let msg = format!("Server: {}", sleep_form.address);
            (true, msg)
        }
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            info!("Connection Refused (Server might be sleeping): {e}");
            let msg = format!("Server is already asleep: {e}");
            (false, msg)
        }
        Err(e) => {
            error!("{e}");
            let msg = format!("Got error when connecting address: {e}");
            (false, msg)
        }
    };
    let status = Status::new(CommandState::Sleep, m.clone(), s);
    state
        .db
        .add_status(status)
        .await
        .expect("Couldn't write to the database");
    let toast = ToastFragment::new(m, s);
    (StatusCode::OK, Html(toast.render().unwrap()))
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
