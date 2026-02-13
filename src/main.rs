use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";
const BUFFER_SIZE: usize = 4096; // 4 KiB

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
    let mut cli_buffer = [0u8; BUFFER_SIZE];
    let mut serv_buffer = [0u8; BUFFER_SIZE];
    let mut cli_offset: usize = 0;
    let mut cli_data_len: usize = 0;
    let mut serv_offset: usize = 0;
    let mut serv_data_len: usize = 0;

    cli_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking cli stream");

    let mut serv_stream = TcpStream::connect(SERVER_ADDR).expect("Failed to connect to server"); // just panic for this version
    serv_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking server stream");

    loop {
        // Offset represents where unsent data starts in the buffer

        // cli data -> server
        if cli_offset != cli_data_len {
            match serv_stream.write(&cli_buffer[cli_offset..cli_data_len]) {
                Ok(n) => {
                    cli_offset += n;
                    if cli_offset == cli_data_len {
                        cli_offset = 0;
                        cli_data_len = 0;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => println!("Error while sending client data to the server: {e:?}"),
            }
        }

        // server data -> cli
        if serv_offset != BUFFER_SIZE {
            match cli_stream.write(&serv_buffer[serv_offset..serv_data_len]) {
                Ok(n) => {
                    serv_offset += n;
                    if serv_offset == serv_data_len {
                        serv_offset = 0;
                        serv_data_len = 0;
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => println!("Error while sending server data to the client: {e:?}"),
            }
        }

        // read cli data
        if cli_data_len != BUFFER_SIZE {
            match cli_stream.read(&mut cli_buffer[cli_data_len..BUFFER_SIZE]) {
                Ok(0) => {
                    println!("Client disconnected");
                    break;
                }
                Ok(n) => {
                    cli_data_len += n;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    println!("Error while reading from client: {e:?}");
                }
            };
        }

        // read server data
        if serv_data_len != BUFFER_SIZE {
            match serv_stream.read(&mut serv_buffer[serv_data_len..BUFFER_SIZE]) {
                Ok(0) => {
                    panic!("Server disconnected");
                }
                Ok(n) => {
                    serv_data_len += n;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    println!("Error while reading from server: {e:?}");
                }
            };
        }
    }
}
