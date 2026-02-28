mod load_balancer;

use bytes::BytesMut;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{Arc, atomic::AtomicUsize};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::try_join;

use load_balancer::{Balancer, ServerConnections};

use crate::load_balancer::ConnectionGuard;

const PROXY_ADDR: &str = "0.0.0.0:8080";
const MAX_SIZE_BUFF: usize = 8192; // 8 KB

#[tokio::main]
async fn main() {
    let raw_servers = vec![
        "0.0.0.0:9090".to_string(),
        "0.0.0.0:9091".to_string(),
        "0.0.0.0:9092".to_string(),
    ]; // TODO: Config file
    let mut servers = Vec::new();

    // Process each server address to a socket at startup to avoid doing it in every connection
    // inside tasks
    for s in raw_servers {
        let addr: SocketAddr = s.parse().expect("Invalid IP address in config");
        servers.push(ServerConnections {
            addr,
            active_conns: AtomicUsize::new(0),
            generation: 0, // TODO: if hot reload modifies, update generation
        });
    }
    let balancer = Arc::new(Balancer::LeastConnections { servers });

    let cli_listener = TcpListener::bind(PROXY_ADDR).await.unwrap();

    println!("Reverse Proxy listening on {}...", PROXY_ADDR);

    loop {
        match cli_listener.accept().await {
            Ok((cli_stream, _)) => {
                let balancer_clone = Arc::clone(&balancer);

                tokio::spawn(async move {
                    if let Err(e) = handle_connection(cli_stream, balancer_clone).await {
                        eprint!("Error handle_connection(): {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error cli_listener.accept(): {}", e);
            }
        }
    }
}

async fn handle_connection(mut cli: TcpStream, balancer: Arc<Balancer>) -> io::Result<()> {
    let (srv_addr, srv_id) = balancer
        .next()
        .ok_or_else(|| Error::other("No servers available"))?;
    let _guard = ConnectionGuard {
        balancer: Arc::clone(&balancer),
        server_id: srv_id,
    };
    let mut srv = TcpStream::connect(srv_addr).await?;

    let (mut cli_r, mut cli_w) = cli.split();
    let (mut srv_r, mut srv_w) = srv.split();

    let c2s = async {
        let mut buf = BytesMut::with_capacity(MAX_SIZE_BUFF);
        loop {
            let n = cli_r.read_buf(&mut buf).await?;
            if n == 0 {
                // Cli sent FIN (half close)
                srv_w.shutdown().await?;
                break;
            }

            srv_w.write_all(&buf[..n]).await?;
            // reset the cursor of buf to the start
            buf.clear();
        }
        Ok::<_, io::Error>(())
    };

    let s2c = async {
        let mut buf = BytesMut::with_capacity(MAX_SIZE_BUFF);
        loop {
            let n = srv_r.read_buf(&mut buf).await?;
            if n == 0 {
                cli_w.shutdown().await?;
                break;
            }

            cli_w.write_all(&buf[..n]).await?;
            buf.clear();
        }
        Ok::<_, io::Error>(())
    };

    try_join!(c2s, s2c)?;

    Ok(())
}
