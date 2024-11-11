use std::fmt::Display;
use std::sync::Arc;

use askama::Template;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{extract::Path, routing::get, Router};
use jiff::Zoned;
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tower_http::services::ServeFile;
use tracing::{error, info};

const MAGIC_PACKET: u8 = 0x77;
const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";
const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress {
    bytes: [0x4C, 0xCC, 0x6A, 0xD0, 0x99, 0x22],
};

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate<'a> {
    statuses: &'a [Status],
}

enum CommandState {
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

struct Status {
    timestamp: String,
    command: CommandState,
    message: String,
    status: bool,
}

struct AppState {
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
    let status = Status {
        timestamp: get_time(),
        command: CommandState::Wake,
        message: m,
        status: s,
    };
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
    let status = Status {
        timestamp: get_time(),
        command: CommandState::Sleep,
        message: m.clone(),
        status: s,
    };
    let mut v = state.statuses.lock().await;
    v.push(status);
    m
}

async fn status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

fn get_time() -> String {
    Zoned::now().strftime("%Y-%m-%d %H:%M:%S %Z").to_string()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting sugoi client");

    let shared_state = Arc::new(AppState {
        statuses: Arc::new(Mutex::new(Vec::new())),
    });

    let app = Router::new()
        .route("/wake/:mac_address", get(wake))
        .route("/sleep/:address", get(sleep))
        .route("/status", get(status))
        .with_state(shared_state)
        .nest_service("/favicon.webp", ServeFile::new("assets/favicon.webp"));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
