use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";

struct ConnectionData {
    cli_stream: TcpStream,
    serv_stream: TcpStream, // First version will use an individual server socket for each cli connection
    cli_to_serv_buff: Vec<u8>,
    serv_to_cli_buff: Vec<u8>,
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

fn handle_cli_connection(cli_stream: TcpStream) {
    cli_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking cli stream");

    let serv_stream = TcpStream::connect(SERVER_ADDR).expect("Failed to connect to server"); // just panic for this version
    serv_stream
        .set_nonblocking(true)
        .expect("Failed to set nonblocking server stream");

    let mut conn_data = ConnectionData {
        cli_stream: cli_stream,
        serv_stream: serv_stream,
        cli_to_serv_buff: vec![],
        serv_to_cli_buff: vec![],
    };

    loop {
        // TODO: limit read
        // read cli data
        let mut temp_read_arr = [0u8; 4096]; // Temporary space
        let bytes_received = match conn_data.cli_stream.read(&mut temp_read_arr) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
            Err(e) => {
                println!("Error while reading from client: {e:?}");
                break;
            }
        };

        // write cli data to server only if cli data was sent
        if bytes_received > 0 {
            conn_data.cli_to_serv_buff = temp_read_arr[..bytes_received].to_vec();
            if let Err(e) = conn_data.serv_stream.write(&conn_data.cli_to_serv_buff) {
                println!("Error while sending client data to the server: {e:?}");
                break;
            }
        }

        // read server data
        let bytes_received = match conn_data.serv_stream.read(&mut temp_read_arr) {
            Ok(0) => panic!("Server disconnected"),
            Ok(n) => n,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => 0,
            Err(err) => {
                println!("Error while reading from server: {err:?}");
                break;
            }
        };

        // write server data to cli only if server sent data
        if bytes_received > 0 {
            conn_data.serv_to_cli_buff = temp_read_arr[..bytes_received].to_vec();
            if let Err(err) = conn_data.cli_stream.write(&conn_data.serv_to_cli_buff) {
                println!("Error while sending server data to the client: {err:?}");
                break;
            };
        }
    }
}
