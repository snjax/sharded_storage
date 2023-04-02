use std::{
    collections::HashSet, net::SocketAddr, ops::Deref, path::PathBuf, str::FromStr, sync::Arc,
};

use anyhow::Result;
use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use axum::{
    extract::{ConnectInfo, Path, State},
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use shamir_ss::Domain;
use tokio::sync::RwLock;
use web3::types::Address;

use crate::{
    contract::RegistryContract,
    error::AppResult,
    storage::{Chunk, ChunkSerde, Storage},
};

// mod commitment;
mod contract;
mod error;
mod storage;
// mod commitment;

const CHUNK_SIZE: usize = 2;

struct AppState {
    storage: Storage,
    // TODO: Replace with URL?
    peers: RwLock<HashSet<SocketAddr>>,
    // contract: RegistryContract,
    domain: Domain,
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
    #[clap(short, long)]
    file: String,
    // #[clap(long)]
    // rpc_url: String,
    // #[clap(long)]
    // contract: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt::init();

    tracing::info!("{:#?}", &args);

    let domain = Domain::from_k(2);

    let state = Arc::new(AppState {
        storage: Storage::new(&args.file).await,
        peers: RwLock::new(args.peer.into_iter().collect()),
        // contract: RegistryContract::new(&args.rpc_url, &args.contract).unwrap(),
        domain,
    });

    let app = Router::new()
        .route("/", get(|| async { () }))
        .route("/data", get(get_data).post(set_data))
        .route(
            "/data/partial",
            get(get_partial_data).post(set_partial_data),
        )
        // Shameful pseudo p2p. Rewrite with libp2p using the request/response behaviour.
        .route("/p2p", post(p2p))
        .with_state(state.clone());

    tracing::info!("Listening on {}", args.addr);
    let http = axum::Server::bind(&args.addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>());

    let is_master = args.peer.is_none();
    let heartbeat = async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            // FIXME
            if !is_master {
                continue;
            }

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

async fn get_data(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<String>>> {
    let mut chunks = vec![state.storage.read().await.unwrap_or(Chunk::new())];

    for peer in state.peers.read().await.iter() {
        let res = reqwest::Client::new()
            .get(format!("http://{}/data/partial", peer))
            .send()
            .await;

        if let Ok(res) = res {
            let json = res
                .json::<ChunkSerde>()
                .await
                .map_err(|err| anyhow::anyhow!(err))?;
            let chunk = json.into();
            tracing::info!("! Got chunk from peer: {:?}", chunk);
            chunks.push(chunk);
        }
    }

    chunks.sort_by_key(|c| c.chunk);

    let mut elements: Vec<Option<Fr>> = vec![];
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.data.is_empty() {
            continue;
        }

        // if chunk.chunk != i as u32 {
        //     println!("Invalid chunk: {}", chunk.chunk);
        //     elements.extend(std::iter::repeat(None).take(CHUNK_SIZE));
        // } else {
        //     elements.extend(chunk.data.iter().map(|e| Some(*e)));
        // }

        elements.extend(chunk.data.iter().map(|e| Some(*e)));
    }

    tracing::info!("Elements: {:?}", elements);

    let elements = state
        .domain
        .decode(&elements)
        .ok_or_else(|| anyhow::anyhow!("Invalid data"))?
        .into_iter()
        .map(|e| e.to_string())
        .collect();

    Ok(Json(elements))
}

async fn get_partial_data(State(state): State<Arc<AppState>>) -> AppResult<Json<ChunkSerde>> {
    let data = state.storage.read().await.unwrap_or(Chunk::new()).into();

    Ok(Json(data))
}

async fn set_data(
    State(state): State<Arc<AppState>>,
    Json(data): Json<Vec<String>>,
) -> AppResult<()> {
    let data = data
        .into_iter()
        .map(|s| s.parse().map_err(|_| anyhow::anyhow!("Invalid element")))
        .collect::<Result<Vec<_>>>()?;

    let encoded = state.domain.encode(data);
    let num_peers = state.peers.read().await.len();
    let num_chunks = (encoded.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;

    if num_chunks > num_peers {
        return Err(anyhow::anyhow!(
            "Not enough peers to store data: expected at least {}, got {}",
            num_chunks,
            num_peers
        )
        .into());
    }

    let chunks = encoded
        .chunks(CHUNK_SIZE)
        .enumerate()
        .map(|(n, elements)| Chunk {
            chunk: n as u32,
            data: elements.to_vec(),
        })
        .collect::<Vec<_>>();

    state.storage.write(&chunks[0]).await?;
    // let address = Address::from_str(&address).map_err(|_| anyhow::anyhow!("Invalid address"))?;

    // FIXME: Calculate commitment and push it to the contract
    // state.contract.push_state(address, commit).await?;

    // Assuming none of the peers are disconnected
    let peers = state.peers.read().await.clone();
    for (chunk, peer) in chunks[1..].into_iter().zip(peers.into_iter()) {
        let res = reqwest::Client::new()
            .post(format!("http://{}/data/partial", peer))
            .json(&ChunkSerde::from(chunk.clone()))
            .send()
            .await;

        match res {
            Ok(res) => {
                if res.status() != 200 {
                    tracing::error!(
                        "Failed to send partial data to peer {}: {}",
                        peer,
                        res.status()
                    );
                }
            }
            Err(err) => {
                tracing::error!("Peer {} is dead: {}", peer, err);
                // state.peers.write().await.remove(&peer);
            }
        }
    }

    Ok(())
}

async fn set_partial_data(
    State(state): State<Arc<AppState>>,
    Json(data): Json<ChunkSerde>,
) -> AppResult<()> {
    let chunk = data.into();
    state.storage.write(&chunk).await?;

    // let address = Address::from_str(&address).map_err(|_| anyhow::anyhow!("Invalid address"))?;
    // state.contract.push_state(address, commit).await?;

    Ok(())
}

async fn p2p(
    ConnectInfo(client_addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Json(req): Json<P2PRequest>,
) -> Json<P2PResponse> {
    match req {
        P2PRequest::Connect { mut addr } => {
            // FIXME: Not good
            addr.set_ip(client_addr.ip());
            tracing::info!("Peer {} connected", addr);

            let mut peers = state.peers.write().await;
            let other_peers = peers.iter().copied().collect();
            peers.insert(addr);

            // TODO: notify other peers
            for peer in &other_peers {
                let res = reqwest::Client::new()
                    .post(format!("http://{}/p2p", peer))
                    .json(&P2PRequest::NewPeer { addr })
                    .send()
                    .await;

                if let Ok(res) = res {
                    if res.status() != 200 {
                        tracing::error!(
                            "Failed to notify peer {} about new peer {}: {}",
                            peer,
                            addr,
                            res.status()
                        );
                    }
                } else {
                    tracing::error!("Peer {} is dead", peer);
                }
            }

            tracing::info!("Connected peers: {:?}", peers);

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
