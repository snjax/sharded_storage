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
    base_path: PathBuf,
}

impl Storage {
    pub async fn new(base_path: &str) -> Self {
        let base_path = base_path.parse().unwrap();

        Self { base_path }
    }

    pub async fn write(&self, chunk: &Chunk) -> Result<()> {
        let path = self.base_path.join("data.bin");
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .open(&path)
            .await?;

        let mut buf = vec![];
        chunk
            .serialize_compressed(&mut buf)
            .map_err(|_| anyhow::anyhow!("Serialization error"))?;

        file.write_all(&buf).await?;

        Ok(())
    }

    pub async fn read(&self) -> Result<Chunk> {
        let path = self.base_path.join("data.bin");
        let mut file = tokio::fs::OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .open(&path)
            .await?;

        let mut buf = vec![];
        file.read_to_end(&mut buf).await.unwrap();

        Chunk::deserialize_compressed(&mut &buf[..])
            .map_err(|_| anyhow::anyhow!("Deserialization error. File: {:?}", path))
    }
}
