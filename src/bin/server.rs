use std::io::{Read, Write};
use std::net::TcpListener;

const SERVER_ADDR: &str = "127.0.0.1:9090";

fn main() {
    let listener = TcpListener::bind(SERVER_ADDR).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0u8; 1024];
                loop {
                    let bytes_received = match stream.read(&mut buffer) {
                        Ok(0) => panic!("Proxy ended connection"),
                        Ok(bytes_received) => bytes_received,
                        Err(err) => {
                            println!("Error while reading from proxy: {err:?}");
                            break;
                        }
                    };
                    let data_received = &buffer[..bytes_received];
                    println!("R:{data_received:?}");
                    let _ = stream.write_all(data_received);
                    let _ = stream.flush();
                }
            }
            Err(err) => println!("Error: {err:?}"),
        }
    }
}
