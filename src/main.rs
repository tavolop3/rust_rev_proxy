use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

mod buffer;
use buffer::ProxyBuffer;

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";

fn main() {
    let cli_listener = TcpListener::bind(PROXY_ADDR).expect("Failed to bind proxy address");

    for cli_stream in cli_listener.incoming() {
        let Ok(cli_stream) = cli_stream else {
            println!("Connection with client failed");
            continue;
        };
        println!("New client connected...");
        thread::spawn(move || {
            handle_cli_connection(cli_stream);
        });
    }
}

fn handle_cli_connection(mut cli_stream: TcpStream) {
    cli_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking cli stream");

    let mut serv_stream = TcpStream::connect(SERVER_ADDR).expect("Failed to connect to server"); // just panic for this version
    serv_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking server stream");

    let mut cli_buffer = ProxyBuffer::new();
    let mut serv_buffer = ProxyBuffer::new();

    loop {
        // Offset represents where unsent data starts in the buffer

        // cli data -> server
        let pending = cli_buffer.get_unsent();
        if !pending.is_empty() {
            match serv_stream.write(pending) {
                Ok(n) => {
                    cli_buffer.advance_offset(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => println!("Error while sending client data to the server: {e:?}"),
            }
        }

        // server data -> cli
        let pending = serv_buffer.get_unsent();
        if !pending.is_empty() {
            match cli_stream.write(pending) {
                Ok(n) => {
                    serv_buffer.advance_offset(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => println!("Error while sending server data to the client: {e:?}"),
            }
        }

        // read cli data
        if !cli_buffer.is_full() {
            match cli_stream.read(cli_buffer.get_available()) {
                Ok(0) => {
                    println!("Client disconnected");
                    break;
                }
                Ok(n) => {
                    cli_buffer.advance_data_len(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    println!("Error while reading from client: {e:?}");
                }
            };
        }

        // read server data
        if !serv_buffer.is_full() {
            match serv_stream.read(serv_buffer.get_available()) {
                Ok(0) => {
                    panic!("Server disconnected");
                }
                Ok(n) => {
                    serv_buffer.advance_data_len(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    println!("Error while reading from server: {e:?}");
                }
            };
        }
    }
}
