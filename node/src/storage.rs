use std::path::PathBuf;

use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type Data = Vec<Fr>;

pub struct Storage {
    path: PathBuf,
    data: Data,
}

impl Storage {
    pub async fn new(path: &str) -> Self {
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

    pub async fn write(&mut self, data: Data) {
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

    pub async fn read(&self) -> Data {
        self.data.clone()
    }
}
