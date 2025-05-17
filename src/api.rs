use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{error, info};

use crate::models::{
    get_record, CommandState, ApiResponse, SleepForm, Status, StatusPagination, StatusRecord,
    WakeForm,
};
use crate::AppState;

pub(crate) const MAGIC_PACKET: u8 = 0x77;
pub(crate) const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";

pub fn get_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/wake", post(wake))
        .route("/sleep", post(sleep))
        .route("/status", get(status))
}

async fn wake(
    State(state): State<Arc<AppState>>,
    Form(wake_form): Form<WakeForm>,
) -> Json<ApiResponse> {
    let Ok(mac_addr) = MacAddress::parse(&wake_form.mac_address) else {
        let msg = format!("Invalid Mac Address: {}.", wake_form.mac_address);
        error!(msg);
        return Json(ApiResponse::new(msg));
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
    Json(ApiResponse::new(m))
}

async fn sleep(
    State(state): State<Arc<AppState>>,
    Form(sleep_form): Form<SleepForm>,
) -> Json<ApiResponse> {
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
    Json(ApiResponse::new(m))
}

async fn status(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<StatusPagination>,
) -> Json<StatusRecord> {
    let v = state
        .db
        .get_statuses()
        .await
        .expect("Couldn't get statuses");
    info!("{:?}", pagination);
    let record = get_record(v, pagination);
    Json(record)
}
