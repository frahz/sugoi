use actix_web::{get, web, App, HttpServer, Responder};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tracing::{error, info};

const MAGIC_PACKET: u8 = 0x77;
const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";
const DEFAULT_MAC_ADDRESS: MacAddress = MacAddress {
    bytes: [0x4C, 0xCC, 0x6A, 0xD0, 0x99, 0x22],
};

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!\n")
}

#[get("/wake/{mac_address}")]
async fn wake(mac_address: web::Path<String>) -> impl Responder {
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

#[get("/sleep/{address}")]
async fn sleep(address: web::Path<String>) -> impl Responder {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Hello");
    HttpServer::new(|| App::new().service(greet).service(wake).service(sleep))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
