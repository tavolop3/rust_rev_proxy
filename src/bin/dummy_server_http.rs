use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const BASE_PORT: u16 = 9090;
const NUM_SERVERS: u16 = 3;
const BUFFER_SIZE: usize = 1024;

const HTTP_200_OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";

async fn handle_dummy_connection(mut stream: TcpStream) {
    let mut buf = [0u8; BUFFER_SIZE];

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(_n) => {
                if stream.write_all(HTTP_200_OK).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

async fn start_dummy_server(port: u16) {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Could not bind dummy server");

    println!("Dummy HTTP Server listening in {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_dummy_connection(stream));
            }
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

#[tokio::main]
async fn main() {
    println!("Seting up backends...");

    for i in 0..NUM_SERVERS {
        let port = BASE_PORT + i;
        tokio::spawn(start_dummy_server(port));
    }

    tokio::signal::ctrl_c().await.unwrap();
    println!("\nShuting down dummy servers...");
}
