mod load_balancer;

use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::try_join;

use load_balancer::{LoadBalanceStrategy, RoundRobinBalancer};

const BACKENDS: [&str; 3] = ["0.0.0.0:9090", "0.0.0.0:9091", "0.0.0.0:9092"]; // TODO: Config file
const PROXY_ADDR: &str = "0.0.0.0:8080";
const MAX_SIZE_BUFF: usize = 8192; // 8 KB

#[tokio::main]
async fn main() {
    let balancer: Arc<dyn LoadBalanceStrategy> = Arc::new(RoundRobinBalancer::new());
    let cli_listener = TcpListener::bind(PROXY_ADDR).await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (cli_stream, _) = cli_listener.accept().await.unwrap();
        let balancer = balancer.clone();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            let _ = handle_connection(cli_stream, balancer).await;
        });
    }
}

async fn handle_connection(
    mut cli: TcpStream,
    balancer: Arc<dyn LoadBalanceStrategy>,
) -> io::Result<()> {
    let srv_addr = balancer.next(&BACKENDS);
    let mut srv = TcpStream::connect(srv_addr).await?;

    let (mut cli_r, mut cli_w) = cli.split();
    let (mut srv_r, mut srv_w) = srv.split();

    let c2s = async {
        let mut buf = [0u8; MAX_SIZE_BUFF];
        loop {
            let n = cli_r.read(&mut buf).await?;
            if n == 0 {
                // Cli sent FIN (half close)
                srv_w.shutdown().await?;
                break;
            }

            srv_w.write_all(&buf[..n]).await?;
        }
        Ok::<_, io::Error>(())
    };

    let s2c = async {
        let mut buf = [0u8; MAX_SIZE_BUFF];
        loop {
            let n = srv_r.read(&mut buf).await?;
            if n == 0 {
                cli_w.shutdown().await?;
                break;
            }

            cli_w.write_all(&buf[..n]).await?;
        }
        Ok::<_, io::Error>(())
    };

    try_join!(c2s, s2c)?;
    Ok(())
}
