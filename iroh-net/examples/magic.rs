use std::net::SocketAddr;

use clap::Parser;
use iroh_base::base32;
use iroh_net::{
    derp::{DerpMap, DerpMode},
    key::SecretKey,
    magic_endpoint::accept_conn,
    MagicEndpoint, NodeAddr,
};
use tracing::{debug, info};
use url::Url;

const EXAMPLE_ALPN: &[u8] = b"n0/iroh/examples/magic/0";

#[derive(Debug, Parser)]
struct Cli {
    #[clap(short, long)]
    secret: Option<String>,
    #[clap(short, long, default_value = "n0/iroh/examples/magic/0")]
    alpn: String,
    #[clap(short, long, default_value = "0")]
    bind_port: u16,
    #[clap(short, long)]
    derp_url: Option<Url>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Listen,
    ListenUnreliable,
    Connect {
        node_id: String,
        #[clap(long)]
        addrs: Option<Vec<SocketAddr>>,
        #[clap(long)]
        derp_url: Option<Url>,
    },
    ConnectUnreliable {
        node_id: String,
        #[clap(long)]
        addrs: Option<Vec<SocketAddr>>,
        #[clap(long)]
        derp_url: Option<Url>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let args = Cli::parse();
    let secret_key = match args.secret {
        None => {
            let secret_key = SecretKey::generate();
            println!("our secret key: {}", base32::fmt(secret_key.to_bytes()));
            secret_key
        }
        Some(key) => parse_secret(&key)?,
    };

    let derp_mode = match args.derp_url {
        None => DerpMode::Default,
        Some(url) => DerpMode::Custom(DerpMap::from_url(url)),
    };

    let endpoint = MagicEndpoint::builder()
        .secret_key(secret_key)
        .alpns(vec![args.alpn.to_string().into_bytes()])
        .derp_mode(derp_mode)
        .bind(args.bind_port)
        .await?;

    let me = endpoint.node_id();
    let local_addr = endpoint.local_endpoints().await?;
    println!("magic socket listening on {local_addr:?}");
    println!("our node id: {me}");

    match args.command {
        Command::Listen => {
            while let Some(conn) = endpoint.accept().await {
                let (node_id, alpn, conn) = accept_conn(conn).await?;
                info!(
                    "new connection from {node_id} with ALPN {alpn} (coming from {})",
                    conn.remote_address()
                );
                tokio::spawn(async move {
                    let (mut send, mut recv) = conn.accept_bi().await?;
                    debug!("accepted bi stream, waiting for data...");
                    let message = recv.read_to_end(1000).await?;
                    let message = String::from_utf8(message)?;
                    println!("received: {message}");

                    let message = format!("hi! you connected to {me}. bye bye");
                    send.write_all(message.as_bytes()).await?;
                    send.finish().await?;

                    Ok::<_, anyhow::Error>(())
                });
            }
        }
        Command::ListenUnreliable => {
            while let Some(conn) = endpoint.accept().await {
                let (node_id, alpn, conn) = accept_conn(conn).await?;
                info!(
                    "new (unreliable) connection from {node_id} with ALPN {alpn} (coming from {})",
                    conn.remote_address()
                );
                tokio::spawn(async move {
                    while let Ok(message) = conn.read_datagram().await {
                        let message = String::from_utf8(message.into())?;
                        println!("received: {message}");

                        let message = format!("hi! you connected to {me}. bye bye");
                        conn.send_datagram(message.as_bytes().to_vec().into())?;
                    }

                    Ok::<_, anyhow::Error>(())
                });
            }
        }
        Command::Connect {
            node_id,
            addrs,
            derp_url,
        } => {
            let addr = NodeAddr::from_parts(node_id.parse()?, derp_url, addrs.unwrap_or_default());
            let conn = endpoint.connect(addr, EXAMPLE_ALPN).await?;
            info!("connected");

            let (mut send, mut recv) = conn.open_bi().await?;

            let message = format!("hello here's {me}");
            send.write_all(message.as_bytes()).await?;
            send.finish().await?;
            let message = recv.read_to_end(100).await?;
            let message = String::from_utf8(message)?;
            println!("received: {message}");
        }
        Command::ConnectUnreliable {
            node_id,
            addrs,
            derp_url,
        } => {
            let addr = NodeAddr::from_parts(node_id.parse()?, derp_url, addrs.unwrap_or_default());
            let conn = endpoint.connect(addr, EXAMPLE_ALPN).await?;
            info!("connected");

            let message = format!("hello here's {me}");
            conn.send_datagram(message.as_bytes().to_vec().into())?;
            let message = conn.read_datagram().await?;
            let message = String::from_utf8(message.into())?;
            println!("received: {message}");
        }
    }
    Ok(())
}

fn parse_secret(secret: &str) -> anyhow::Result<SecretKey> {
    let bytes: [u8; 32] = base32::parse_array(secret)?;
    let key = SecretKey::from(bytes);
    Ok(key)
}
