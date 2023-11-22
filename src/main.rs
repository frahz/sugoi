use actix_web::{get, web, App, HttpServer, Responder};
use tracing::info;
use std::process::Command;

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!\n")
}

#[get("/wake")]
async fn wake() -> impl Responder {
    let output = Command::new("/usr/sbin/etherwake")
        .arg("-i")
        .arg("eth0")
        .arg("4C:CC:6A:D0:99:22")
        .output()
        .unwrap();
    info!("wake output: {} err: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    format!("Sent wake command to server. status: {}\n", output.status)
}

#[get("/sleep")]
async fn sleep() -> impl Responder {
    let output = Command::new("/bin/ssh")
        .arg("server")
        .arg("sudo systemctl suspend")
        .output()
        .unwrap();
    info!("sleep output: {} err: {}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
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
