use std::str::FromStr;

use anyhow::Result;
use ark_bn254::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use secp256k1::SecretKey;
use web3::{
    api::{Eth, Namespace},
    contract::Contract,
    transports::Http,
    types::{Address, H256, U256},
    Web3,
};

use crate::storage::Data;

const CONTRACT_ABI: &[u8] = include_bytes!("StateRegistry.json");

pub struct RegistryContract {
    web3: Web3<Http>,
    contract: Contract<Http>,
}

impl RegistryContract {
    pub fn new(rpc_url: &str, address: &str) -> Result<Self> {
        let transport = Http::new(rpc_url)?;
        let web3 = Web3::new(transport.clone());
        let contract = Contract::from_json(Eth::new(transport), address.parse()?, CONTRACT_ABI)?;

        Ok(Self { web3, contract })
    }

    async fn push_state(&self, address: Address, data: &[Fr]) -> Result<H256> {
        let mut buf = vec![];
        data.serialize_compressed(&mut buf).unwrap();

        let accounts = self.web3.eth().accounts().await?;
        let hash = self
            .contract
            .call("pushState", (address, buf), accounts[0], Default::default())
            .await?;

        Ok(hash)
    }

    async fn get_state(&self, address: Address) -> Result<Vec<Fr>> {
        let state: Vec<u8> = self
            .contract
            .query("state", (address), None, Default::default(), None)
            .await?;

        let data = Vec::<Fr>::deserialize_compressed(&mut &state[..]).unwrap();

        Ok(data)
    }

    async fn get_state_height(&self, address: Address) -> Result<u64> {
        let state_height: U256 = self
            .contract
            .query("getStateHeight", (address), None, Default::default(), None)
            .await?;

        Ok(state_height.as_u64())
    }
}
