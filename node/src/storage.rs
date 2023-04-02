use std::path::PathBuf;

use anyhow::Result;
use ark_bn254::Fr;
use ark_ff::Field;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkSerde {
    pub chunk: u32,
    pub data: Vec<String>,
}

impl From<Chunk> for ChunkSerde {
    fn from(chunk: Chunk) -> Self {
        Self {
            chunk: chunk.chunk,
            data: chunk.data.iter().map(|x| x.to_string()).collect(),
        }
    }
}

#[derive(Clone, CanonicalSerialize, CanonicalDeserialize, Debug)]
pub struct Chunk {
    pub chunk: u32,
    pub data: Vec<Fr>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            chunk: 0,
            data: vec![],
        }
    }

    pub fn new_empty(chunk: u32, size: usize) -> Self {
        Self {
            chunk,
            data: vec![Fr::ZERO; size],
        }
    }
}

impl From<ChunkSerde> for Chunk {
    fn from(chunk: ChunkSerde) -> Self {
        Self {
            chunk: chunk.chunk,
            data: chunk.data.iter().map(|x| x.parse().unwrap()).collect(),
        }
    }
}

pub struct Storage {
    path: PathBuf,
}

impl Storage {
    pub async fn new(path: &str) -> Self {
        let path = path.parse().unwrap();

        Self { path }
    }

    pub async fn write(&self, chunk: &Chunk) -> Result<()> { ;
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .open(&self.path)
            .await?;

        let mut buf = vec![];
        chunk
            .serialize_compressed(&mut buf)
            .map_err(|_| anyhow::anyhow!("Serialization error"))?;

        file.write_all(&buf).await?;

        Ok(())
    }

    pub async fn read(&self) -> Option<Chunk> {
        let mut file = match tokio::fs::OpenOptions::new().read(true).open(&self.path).await {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!("Error opening file: {}", err);
                return None;
            }
        };

        let mut buf = vec![];
        if let Err(_) = file.read_to_end(&mut buf).await { 
            return None;
        }

        Chunk::deserialize_compressed(&mut &buf[..]).ok()
    }
}
