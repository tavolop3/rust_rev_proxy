use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread; // Required for spawning threads

const SERVER_ADDR: &str = "127.0.0.1:9090";

fn handle_proxy_connection(mut stream: TcpStream) {
    println!("Proxy connected from: {:?}", stream.peer_addr().unwrap());
    let mut buffer = [0u8; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Proxy ended connection");
                break;
            }
            Ok(bytes_received) => {
                let data_received = &buffer[..bytes_received];
                println!("Received: {:?} bytes", bytes_received);

                // Echo the data back
                if let Err(e) = stream.write_all(data_received) {
                    println!("Failed to write: {e}");
                    break;
                }
                let _ = stream.flush();
            }
            Err(err) => {
                println!("Error while reading from proxy: {err:?}");
                break;
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind(SERVER_ADDR).expect("Could not bind");
    println!("Server listening on {}", SERVER_ADDR);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_proxy_connection(stream);
                });
            }
            Err(err) => println!("Error accepting connection: {err:?}"),
        }
    }
}
