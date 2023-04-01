//! A naive, primitive implementation of a p2p node using TCP.
//! Use libp2p (or alternatives) for a more robust implementation.

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    net::SocketAddr,
    sync::Arc,
};

use anyhow::Result;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use clap::Parser;
use futures::future::BoxFuture;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpSocket,
    },
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        RwLock,
    },
    task::JoinHandle,
};

// Only need to broadcast a message and receive all responses.
pub struct Network<M> {
    state: Arc<State>,
    recv: RwLock<UnboundedReceiver<MessageWrapper<M>>>,
}

impl<M: Clone + Serialize> Network<M> {
    async fn broadcast(&self, message: M) -> Vec<M> {
        let mut peers = self.state.peers.write().await;
        let mut message_ids = HashSet::new();
        let mut responses = Vec::new();

        for peer in peers.values_mut() {
            let message_id = rand::random();
            let e = Event::Message(MessageWrapper {
                message_id,
                data: message.clone(),
            });

            event_proto::send(peer, &e).await.unwrap();

            message_ids.insert(message_id);
        }

        while message_ids.len() > 0 {
            let message = self.recv.write().await.recv().await.unwrap();

            if message_ids.remove(&message.message_id) {
                responses.push(message.data);
            }
        }

        responses
    }
}

struct Connection<M> {
    pub sender: UnboundedSender<M>,
    pub receiver: UnboundedReceiver<M>,
}

impl<M> Connection<M>
where
    M: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
{
    fn new() -> (Self, UnboundedSender<M>, UnboundedReceiver<M>) {
        let (s_send, s_recv) = mpsc::unbounded_channel();
        let (r_send, r_recv) = mpsc::unbounded_channel();
        let connection = Self {
            sender: s_send,
            receiver: r_recv,
        };
        (connection, r_send, s_recv)
    }

    pub async fn send(&self, message: M) {
        self.sender.send(message).unwrap();
    }

    pub async fn recv(&mut self) -> Option<M> {
        self.receiver.recv().await
    }
}

pub type PeerId = u32;

pub struct State {
    pub peer_id: PeerId,
    pub peers: RwLock<HashMap<PeerId, OwnedWriteHalf>>,
}

pub async fn start<M>(addr: SocketAddr, peer: Option<SocketAddr>) -> (Network<M>, JoinHandle<()>)
where
    M: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
{
    tracing::info!("Listening on {}", addr);
    let listener = TcpListener::bind(&addr).await.unwrap();

    let state = Arc::new(State {
        peer_id: rand::random(),
        peers: RwLock::new(HashMap::new()),
    });

    // Shared message receiver
    let (r_send, r_recv) = mpsc::unbounded_channel();

    let state_1 = state.clone();
    let r_send_1 = r_send.clone();
    let listener = tokio::spawn(async move {
        loop {
            let state = state_1.clone();
            let (stream, _) = listener.accept().await.unwrap();

            let (reader, writer) = stream.into_split();

            tokio::spawn(listen_for_events::<M>(
                reader,
                Some(writer),
                r_send_1.clone(),
                state,
            ));
        }
    });

    let state_2 = state.clone();
    let r_send_2 = r_send.clone();
    let handle = tokio::spawn(async move {
        if let Some(peer) = peer {
            let peer_connection = tokio::spawn(async move {
                use event_proto::send;

                tracing::info!("Connecting to {}", peer);
                let stream = TcpSocket::new_v4().unwrap().connect(peer).await.unwrap();
                let (reader, mut writer) = stream.into_split();

                send(&mut writer, &Event::<M>::ConnectRequest(rand::random()))
                    .await
                    .unwrap();

                let state = state_2.clone();
                listen_for_events::<M>(reader, Some(writer), r_send_2.clone(), state).await;
            });

            tokio::select! {
                _ = listener => {}
                _ = peer_connection => {}
            }
        } else {
            tokio::select! {
                _ = listener => {}
            }
        }
    });

    (
        Network {
            state,
            recv: RwLock::new(r_recv),
        },
        handle,
    )
}

/// Basic p2p protocol event.
#[derive(Serialize, Deserialize, Debug)]
enum Event<M> {
    ConnectRequest(PeerId),
    ConnectResponse {
        peer_id: PeerId,
        other_peers: Vec<(PeerId, SocketAddr)>,
    },
    Message(MessageWrapper<M>),
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageWrapper<M> {
    message_id: u32,
    data: M,
}

async fn listen_for_events<M>(
    mut reader: OwnedReadHalf,
    mut writer: Option<OwnedWriteHalf>,
    r_send: UnboundedSender<MessageWrapper<M>>,
    state: Arc<State>,
) -> Result<()>
where
    M: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
{
    use event_proto::recv;

    loop {
        match recv::<_, Event<M>>(&mut reader).await {
            Ok(event) => {
                tracing::info!("Got event: {:?}", event);

                match event {
                    // Response to the Connect event.
                    Event::ConnectResponse {
                        other_peers,
                        peer_id,
                    } => {
                        let mut peers = state.peers.write().await;
                        peers.insert(peer_id, writer.take().unwrap());

                        for (id, addr) in other_peers {
                            if peers.get(&id).is_some() {
                                continue;
                            }

                            let stream = TcpSocket::new_v4().unwrap().connect(addr).await.unwrap();
                            let (reader, writer) = stream.into_split();

                            state.peers.write().await.insert(id, writer);

                            let state = state.clone();

                            #[inline(always)]
                            fn listen_for_events_recursive<M>(
                                reader: OwnedReadHalf,
                                writer: Option<OwnedWriteHalf>,
                                r_send: UnboundedSender<MessageWrapper<M>>,
                                state: Arc<State>,
                            ) -> BoxFuture<'static, Result<()>>
                            where
                                M: Serialize + DeserializeOwned + Debug + Send + Sync + 'static,
                            {
                                Box::pin(listen_for_events::<M>(
                                    reader,
                                    writer,
                                    r_send.clone(),
                                    state,
                                ))
                            }

                            tokio::spawn(listen_for_events_recursive::<M>(
                                reader,
                                None,
                                r_send.clone(),
                                state,
                            ));
                        }
                    }
                    // New peer requested a connection
                    Event::ConnectRequest(id) => {
                        let mut writer = writer.take().unwrap();
                        // Broadcast to all peers that we have connected to this peer.
                        let mut peers = state.peers.write().await;

                        if peers.get(&id).is_some() {
                            tracing::warn!("Peer {} already connected", id);
                            return Ok(());
                        }

                        let other_peers = peers
                            .iter()
                            .map(|(id, _)| (*id, reader.peer_addr().unwrap()))
                            .collect::<Vec<_>>();
                        let event = Event::<M>::ConnectResponse {
                            peer_id: state.peer_id,
                            other_peers,
                        };
                        event_proto::send(&mut writer, &event).await.unwrap();

                        peers.insert(id, writer);
                        tracing::info!("Peer {} connected", id);
                    }
                    Event::Message(msg) => {
                        tracing::info!("Got message: {:?}", msg);
                        r_send.send(msg).unwrap();
                    }
                }
            }
            Err(err) => {
                tracing::error!("Connection closed: {:?}", err);
            }
        }
    }
}

mod event_proto {
    use std::io;

    use bincode::{deserialize, serialize};
    use serde::{de::DeserializeOwned, Serialize};
    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

    pub async fn send<S: AsyncWrite + Unpin, T: Serialize>(
        stream: &mut S,
        msg: &T,
    ) -> io::Result<()> {
        let buf = serialize(msg).unwrap();
        stream.write_u32(buf.len() as u32).await?;
        stream.write_all(&buf).await?;
        Ok(())
    }

    pub async fn recv<S: AsyncRead + Unpin, T: DeserializeOwned>(stream: &mut S) -> io::Result<T> {
        let len = stream.read_u32().await? as usize;
        let mut buf = vec![0; len];
        stream.read_exact(&mut buf).await?;
        deserialize(&buf[..]).map_err(|_| io::ErrorKind::Other.into())
    }
}
