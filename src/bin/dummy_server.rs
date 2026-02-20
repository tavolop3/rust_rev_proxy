use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;

const BASE_PORT: u16 = 9090;
const NUM_SERVERS: u16 = 3;
const BUFFER_SIZE: usize = 1024;

async fn handle_dummy_connection(mut stream: TcpStream, server_port: u16) {
    let client_addr = stream.peer_addr().unwrap();
    println!(
        "[Server {}] Dummy connected from {}",
        server_port, client_addr
    );

    let mut buf = [0u8; BUFFER_SIZE];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => {
                println!(
                    "[Server {}] Connection ended from {}",
                    server_port, client_addr
                );
                break;
            }
            Ok(n) => {
                println!(
                    "[Server {}] Received {} bytes from {}: {:?}",
                    server_port,
                    n,
                    client_addr,
                    &buf[..n]
                );
                if let Err(e) = stream.write_all(&buf[..n]).await {
                    println!(
                        "[Server {}] Failed to write to {}: {}",
                        server_port, client_addr, e
                    );
                    break;
                }
            }
            Err(e) => {
                println!(
                    "[Server {}] Error reading from {}: {}",
                    server_port, client_addr, e
                );
                break;
            }
        }
    }
}

async fn start_dummy_server(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Could not bind dummy server");
    println!("Dummy server listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // Spawn a new async task for each connection
                task::spawn(handle_dummy_connection(stream, port));
            }
            Err(e) => println!("Error accepting connection: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    // Start multiple dummy servers concurrently
    for i in 0..NUM_SERVERS {
        let port = BASE_PORT + i;
        tokio::spawn(start_dummy_server(port));
    }

    // Keep main alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
