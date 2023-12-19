use actix_web::{get, web, App, HttpServer, Responder};
use low::macaddr::MacAddress;
use low::wol::{create_socket, WolPacket};
use std::process::Command;
use tracing::{error, info};

const DEFAULT_BROADCAST_IP: &str = "255.255.255.255:9";

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!\n")
}

#[get("/wake")]
async fn wake() -> impl Responder {
    let mac_addr = MacAddress::parse("4C:CC:6A:D0:99:22").unwrap();
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

#[get("/sleep")]
async fn sleep() -> impl Responder {
    let output = Command::new("/bin/ssh")
        .arg("server")
        .arg("sudo systemctl suspend")
        .output()
        .unwrap();
    info!(
        "sleep output: {} err: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    format!("Sent sleep command to server. status: {}\n", output.status)
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
