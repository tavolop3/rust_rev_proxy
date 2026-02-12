use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";
const MAX_SIZE_BUFF: usize = 1024;

fn main() {
    let mut server_stream = TcpStream::connect(SERVER_ADDR).expect("Failed to connect to server");
    let cli_listener = TcpListener::bind(PROXY_ADDR).unwrap();

    for cli_stream in cli_listener.incoming() {
        match cli_stream {
            Ok(mut cli_stream) => {
                let mut buffer = [0u8; MAX_SIZE_BUFF];
                let bytes_received = match cli_stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(bytes_received) => bytes_received,
                    Err(err) => {
                        println!("Error while reading from client: {err:?}");
                        break;
                    }
                };

                let cli_data_received = &buffer[..bytes_received];
                match server_stream.write_all(cli_data_received) {
                    Ok(_) => {
                        let mut buffer = [0u8; MAX_SIZE_BUFF];
                        let bytes_received = match server_stream.read(&mut buffer) {
                            Ok(0) => panic!("Server disconnected"),
                            Ok(bytes_received) => bytes_received,
                            Err(err) => {
                                println!("Error while reading from server: {err:?}");
                                break;
                            }
                        };
                        let server_data_received = &buffer[..bytes_received];
                        if let Err(err) = cli_stream.write_all(server_data_received) {
                            println!("Error while sending server data to the client: {err:?}");
                            break;
                        };
                        let _ = cli_stream.flush();
                    }

                    Err(err) => {
                        println!("Error while sending client data to the server: {err:?}");
                        break;
                    }
                }
            }
            Err(err) => {
                println!("Connection failed: {err:?}")
            }
        }
    }
}
