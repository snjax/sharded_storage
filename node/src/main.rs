use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
};

use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use axum::{
    extract::{ConnectInfo, Path, State},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

type Data = Vec<Fr>;

struct Storage {
    path: PathBuf,
    data: Data,
}

impl Storage {
    async fn new(path: &str) -> Self {
        let path = path.parse().unwrap();
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .await
            .unwrap();

        let mut buf = vec![];
        file.read_to_end(&mut buf).await.unwrap();

        let data = if buf.is_empty() {
            vec![]
        } else {
            Data::deserialize_compressed(&mut &buf[..]).unwrap()
        };

        Self { path, data }
    }

    async fn write(&mut self, data: Data) {
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .create(true)
            .open(&self.path)
            .await
            .unwrap();

        let mut buf = vec![];
        data.serialize_compressed(&mut buf).unwrap();

        file.write_all(&buf).await.unwrap();

        self.data = data;
    }

    async fn read(&self) -> Data {
        self.data.clone()
    }
}

struct AppState {
    storage: Storage,
    // TODO: Replace with URL?
    peers: RwLock<HashSet<SocketAddr>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum P2PRequest {
    /// Connect request from a peer.
    Connect { addr: SocketAddr },
    /// New peer notification for other peers.
    NewPeer { addr: SocketAddr },
}

#[derive(Debug, Serialize, Deserialize)]
enum P2PResponse {
    Connected { other_peers: Vec<SocketAddr> },
    Success,
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "0.0.0.0:3000")]
    addr: SocketAddr,
    #[clap(short, long)]
    peer: Option<SocketAddr>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    tracing::info!("{:#?}", &args);

    let state = Arc::new(AppState {
        storage: Storage::new("data.bin").await,
        peers: RwLock::new(args.peer.into_iter().collect()),
    });

    let app = Router::new()
        .route("/", get(|| async { () }))
        .route("/data/:address", get(data_get))
        .route("/data", post(|| async { () }))
        // Shameful pseudo p2p. Rewrite with libp2p using the request/response behaviour.
        .route("/p2p", post(p2p))
        .with_state(state.clone());

    tracing::info!("Listening on {}", args.addr);
    let http = axum::Server::bind(&args.addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    let heartbeat = async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            let peers = state.peers.read().await.clone();
            for peer in peers {
                let res = reqwest::get(format!("http://{}/", peer)).await;

                match res {
                    Ok(res) => {
                        if res.status() == 200 {
                            tracing::info!("Peer {} is alive", peer);
                        } else {
                            tracing::warn!("Peer {} is dead: status code {}", peer, res.status());
                        }
                    }
                    Err(err) => {
                        tracing::warn!("Peer {} is dead: {}", peer, err);
                        state.peers.write().await.remove(&peer);
                    }
                }
            }
        }
    };

    if let Some(peer) = args.peer {
        tracing::info!("Connecting to peer {}", peer);
        let res = reqwest::Client::new()
            .post(format!("http://{}/p2p", peer))
            .json(&P2PRequest::Connect { addr: args.addr })
            .send()
            .await
            .unwrap();

        if res.status() != 200 {
            tracing::error!("Failed to connect to peer {}: {}", peer, res.status());
        }

        let json = res.json::<P2PResponse>().await.unwrap();

        if let P2PResponse::Connected { other_peers } = json {
            tracing::info!("Connected to peer {}", peer);
            tracing::info!("Other peers: {:?}", other_peers);

            let mut peers = state.peers.write().await;
            peers.extend(other_peers);
        } else {
            tracing::error!("Unexpected response from peer {}: {:?}", peer, json);
        }
    }

    tokio::select! {
        err = http => {
            tracing::error!("HTTP server error: {:?}", err);
        }
        _ = heartbeat => {
            tracing::error!("Heartbeat error");
        }
    }
}

async fn data_get(Path(address): Path<SocketAddr>) -> Json<Vec<String>> {
    Json(vec![])
}

async fn data_post() {}

async fn p2p(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<P2PRequest>,
) -> Json<P2PResponse> {
    match req {
        P2PRequest::Connect { mut addr } => {
            addr.set_ip(client_addr.ip());
            tracing::info!("Peer {} connected", addr);

            let mut peers = state.peers.write().await;
            let other_peers = peers.iter().copied().collect();
            peers.insert(addr);

            Json(P2PResponse::Connected { other_peers })
        }
        P2PRequest::NewPeer { addr } => {
            tracing::info!("Peer {} connected", addr);

            let mut peers = state.peers.write().await;
            peers.insert(addr);

            Json(P2PResponse::Success)
        }
    }
}
