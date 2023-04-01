use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
}

struct AppState {
    storage: Storage,
    peers: Vec<SocketAddr>,
}

#[derive(Debug, Serialize, Deserialize)]
enum P2PRequest {
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
enum P2PResponse {
    Pong,
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "0.0.0.0:3000")]
    addr: SocketAddr,
    #[clap(short, long)]
    peers: Option<Vec<SocketAddr>>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    tracing::info!("{:#?}", &args);

    let state = Arc::new(AppState {
        storage: Storage::new("data.bin").await,
        peers: args.peers.unwrap_or_default(),
    });

    let app = Router::new()
        .route("/", get(|| async { () }))
        .route("/data/:address", get(data_get))
        .route("/data", post(|| async { () }))
        // Pseudo p2p
        .route("/p2p", post(p2p))
        .with_state(state.clone());

    tracing::info!("Listening on {}", args.addr);
    let http = axum::Server::bind(&args.addr).serve(app.into_make_service());

    let heartbeat = async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            for peer in state.peers.iter() {
                let res = reqwest::get(format!("http://{}/p2p", peer)).await;

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
                    }
                }
            }
        }
    };

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

async fn p2p(State(state): State<Arc<AppState>>, Json(req): Json<P2PRequest>) -> Json<P2PResponse> {
    match req {
        P2PRequest::Ping => Json(P2PResponse::Pong),
    }
}
