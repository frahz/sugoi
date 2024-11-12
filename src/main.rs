mod status;

use std::sync::Arc;

use axum::extract::State;
use axum::{extract::Path, routing::get, Router};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use status::Status;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tracing::{error, info};

const MAGIC_PACKET: u8 = 0x77;
const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";
const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress {
    bytes: [0x4C, 0xCC, 0x6A, 0xD0, 0x99, 0x22],
};

pub struct AppState {
    statuses: Arc<Mutex<Vec<Status>>>,
}

async fn wake(State(state): State<Arc<AppState>>, Path(mac_address): Path<String>) -> String {
    let mac_addr = MacAddress::parse(&mac_address).unwrap_or_else(|_| {
        error!("Invalid Mac Address: {mac_address}. Using default.");
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
                    mac_address
                ),
            )
        }
        Err(e) => {
            let msg = format!("Failed to send packet: {e}");
            error!(msg);
            (false, msg)
        }
    };
    let status = Status::new(status::CommandState::Wake, m, s);
    let mut v = state.statuses.lock().await;
    v.push(status);
    "Sent wake command to server\n".to_string()
}

async fn sleep(State(state): State<Arc<AppState>>, Path(address): Path<String>) -> String {
    // "127.0.0.1:8080"
    let (s, m) = match TcpStream::connect(address.as_str()).await {
        Ok(mut stream) => {
            info!("Sending sleep to server: {}", address);
            stream.write_u8(MAGIC_PACKET).await.unwrap();
            let msg = format!("Sent sleep command to server: {}", address);
            (true, msg)
        }
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            info!("Connection Refused (Server might be sleeping): {e}\n");
            let msg = format!("Server is already asleep: {e}");
            (false, msg)
        }
        Err(e) => {
            error!("{e}");
            let msg = format!("Got Error when connecting address: {e}");
            (false, msg)
        }
    };
    let status = Status::new(status::CommandState::Sleep, m.clone(), s);
    let mut v = state.statuses.lock().await;
    v.push(status);
    m
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting sugoi client");

    let shared_state = Arc::new(AppState {
        statuses: Arc::new(Mutex::new(Vec::new())),
    });

    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let app = Router::new()
        .route("/wake/:mac_address", get(wake))
        .route("/sleep/:address", get(sleep))
        .route("/status", get(status::status_root))
        .route("/status/refresh", get(status::status_refresh))
        .with_state(shared_state)
        .nest_service("/assets", ServeDir::new("assets"))
        .layer(compression_layer);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
