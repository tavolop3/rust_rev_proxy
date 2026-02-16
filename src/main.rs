use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;

mod buffer;
use buffer::ProxyBuffer;

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";

struct ConnectionHalf {
    read_open: bool,
    write_open: bool,
}
impl ConnectionHalf {
    pub fn new() -> ConnectionHalf {
        ConnectionHalf {
            read_open: true,  // Can read from this half
            write_open: true, // Can write to this half
        }
    }
}

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
    if let Err(e) = cli_stream.set_nonblocking(true) {
        eprintln!("Failed to set nonblocking client stream: {}", e);
        return;
    }

    let mut serv_stream = match TcpStream::connect(SERVER_ADDR) {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("Could not connect to server {}: {}", SERVER_ADDR, e);
            return;
        }
    };
    if let Err(e) = serv_stream.set_nonblocking(true) {
        eprintln!("Failed to set nonblocking server stream: {}", e);
        return;
    }

    let mut cli_buffer = ProxyBuffer::new();
    let mut serv_buffer = ProxyBuffer::new();

    let mut cli_half = ConnectionHalf::new();
    let mut serv_half = ConnectionHalf::new();

    let mut connection_broken = false;

    loop {
        // writes firsts to drain buffers if necessary

        // cli data -> server
        if serv_half.write_open {
            let pending = cli_buffer.get_unsent();
            if !pending.is_empty() {
                match serv_stream.write(pending) {
                    Ok(n) => {
                        cli_buffer.advance_offset(n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => {
                        println!("Error while sending client data to the server: {e:?}");
                        connection_broken = true
                    }
                }
            } else if !cli_half.read_open {
                // cli FIN and drained buffer
                let _ = serv_stream.shutdown(Shutdown::Write); // Sends FIN to server
                serv_half.write_open = false;
            }
        }

        // server data -> cli
        if cli_half.write_open {
            let pending = serv_buffer.get_unsent();
            if !pending.is_empty() {
                match cli_stream.write(pending) {
                    Ok(n) => {
                        serv_buffer.advance_offset(n);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => {
                        println!("Error while sending server data to the client: {e:?}");
                        connection_broken = true;
                    }
                }
            } else if !serv_half.read_open {
                // serv FIN and drained buffer
                let _ = cli_stream.shutdown(Shutdown::Write);
                cli_half.write_open = false;
            }
        }

        // read cli data
        if cli_half.read_open && !cli_buffer.is_full() {
            match cli_stream.read(cli_buffer.get_available()) {
                Ok(0) => {
                    println!("Client sent FIN");
                    cli_half.read_open = false;
                }
                Ok(n) => {
                    cli_buffer.advance_data_len(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    println!("Error while reading from client: {e:?}");
                    connection_broken = true;
                }
            };
        }

        // read server data
        if serv_half.read_open && !serv_buffer.is_full() {
            match serv_stream.read(serv_buffer.get_available()) {
                Ok(0) => {
                    println!("Server sent FIN");
                    serv_half.read_open = false;
                }
                Ok(n) => {
                    serv_buffer.advance_data_len(n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => {
                    println!("Error while reading from server: {e:?}");
                    connection_broken = true;
                }
            };
        }

        // if both FIN and drained buffers then exit loop
        if connection_broken
            || !cli_half.read_open
                && !serv_half.read_open
                && cli_buffer.is_empty()
                && serv_buffer.is_empty()
        {
            println!("Closing proxy connection...");
            break;
        }
    }
}
