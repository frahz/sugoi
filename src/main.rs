use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{extract::Path, routing::get, Router};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, info};

const MAGIC_PACKET: u8 = 0x77;
const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";
const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress {
    bytes: [0x4C, 0xCC, 0x6A, 0xD0, 0x99, 0x22],
};

#[derive(Template)]
#[template(path = "status.html")]
struct StatusTemplate<'a> {
    name: &'a str,
}

async fn wake(Path(mac_address): Path<String>) -> String {
    let mac_addr = MacAddress::parse(&mac_address).unwrap_or_else(|_| {
        error!("Invalid Mac Address: {mac_address}. Using default.");
        DEFAULT_MAC_ADDRESS
    });
    let wol_packet = WolPacket::create(&mac_addr);
    let socket = create_socket(DEFAULT_BROADCAST_IP).unwrap();
    match socket.send_to(&wol_packet.0, DEFAULT_BROADCAST_IP) {
        Ok(_) => info!("Sent wake to server packet len: {}", &wol_packet.0.len()),
        Err(e) => {
            error!("Failed to send packet: {e}");
        }
    }
    "Sent wake command to server\n".to_string()
}

async fn sleep(Path(address): Path<String>) -> String {
    // "127.0.0.1:8080"
    match TcpStream::connect(address.as_str()).await {
        Ok(mut stream) => {
            info!("Sending sleep to server: {}", address);
            stream.write_u8(MAGIC_PACKET).await.unwrap();
            "Sent sleep command to server\n".to_string()
        }
        Err(e) if e.kind() == std::io::ErrorKind::ConnectionRefused => {
            info!("Connection Refused (Server might be sleeping): {e}\n");
            format!("Server is already asleep: {e}\n")
        }
        Err(e) => {
            error!("{e}");
            format!("Got Error when connecting address: {e}")
        }
    }
}

async fn status() -> impl IntoResponse {
    let status = StatusTemplate { name: "John" };
    (StatusCode::OK, Html(status.render().unwrap()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Hello");

    let app = Router::new()
        .route("/wake/:mac_address", get(wake))
        .route("/sleep/:address", get(sleep))
        .route("/status", get(status));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
