use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Form, Json, Router};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{error, info};

use crate::status::{CommandState, Status};
use crate::AppState;

const MAGIC_PACKET: u8 = 0x77;
const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";
const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress {
    bytes: [0x4C, 0xCC, 0x6A, 0xD0, 0x99, 0x22],
};

#[derive(Deserialize)]
struct Wake {
    mac_address: String,
}

#[derive(Deserialize)]
struct Sleep {
    address: String,
}

#[derive(Serialize)]
struct ApiResponse {
    message: String,
}

impl ApiResponse {
    fn new(message: String) -> Self {
        Self { message }
    }
}

pub fn get_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/wake", post(wake))
        .route("/sleep", post(sleep))
}

async fn wake(
    State(state): State<Arc<AppState>>,
    Form(wake_form): Form<Wake>,
) -> Json<ApiResponse> {
    let mac_addr = MacAddress::parse(&wake_form.mac_address).unwrap_or_else(|_| {
        error!(
            "Invalid Mac Address: {}. Using default.",
            wake_form.mac_address
        );
        DEFAULT_MAC_ADDRESS
    });
    let wol_packet = WolPacket::create(&mac_addr);
    let socket = create_socket(DEFAULT_BROADCAST_IP).unwrap();
    let (s, m) = match socket.send_to(&wol_packet.0, DEFAULT_BROADCAST_IP) {
        Ok(_) => {
            info!("Sent wake to server packet len: {}", &wol_packet.0.len());
            (
                true,
                format!(
                    "Sent wake command to following MAC Address: {}",
                    wake_form.mac_address
                ),
            )
        }
        Err(e) => {
            let msg = format!("Failed to send packet: {e}");
            error!(msg);
            (false, msg)
        }
    };
    let status = Status::new(CommandState::Wake, m.clone(), s);
    state.db.add_status(status).await.expect("Couldn't write to the database");
    Json(ApiResponse::new(m))
}

async fn sleep(
    State(state): State<Arc<AppState>>,
    Form(sleep_form): Form<Sleep>,
) -> Json<ApiResponse> {
    // "127.0.0.1:8080"
    let (s, m) = match TcpStream::connect(sleep_form.address.as_str()).await {
        Ok(mut stream) => {
            info!("Sending sleep to server: {}", sleep_form.address);
            stream.write_u8(MAGIC_PACKET).await.unwrap();
            let msg = format!("Sent sleep command to server: {}", sleep_form.address);
            (true, msg)
        }
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            info!("Connection Refused (Server might be sleeping): {e}");
            let msg = format!("Server is already asleep: {e}");
            (false, msg)
        }
        Err(e) => {
            error!("{e}");
            let msg = format!("Got Error when connecting address: {e}");
            (false, msg)
        }
    };
    let status = Status::new(CommandState::Sleep, m.clone(), s);
    state.db.add_status(status).await.expect("Couldn't write to the database");
    Json(ApiResponse::new(m))
}
