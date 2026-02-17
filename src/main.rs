use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::try_join;

const PROXY_ADDR: &str = "127.0.0.1:8080";
const SERVER_ADDR: &str = "127.0.0.1:9090";
const MAX_SIZE_BUFF: usize = 8192; // 8 KB

#[tokio::main]
async fn main() {
    let cli_listener = TcpListener::bind(PROXY_ADDR).await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (cli_stream, _) = cli_listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            let _ = handle_cli_connection(cli_stream).await;
        });
    }
}

async fn handle_cli_connection(mut cli: TcpStream) -> io::Result<()> {
    let mut srv = TcpStream::connect(SERVER_ADDR).await?;

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
