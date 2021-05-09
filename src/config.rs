// use rand::distributions::{Alphanumeric, Distribution};
use crate::arguments::{Args, SubArg};
use crate::sync::{Arc, RwLock};
use rand::thread_rng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use url::Url;

pub type RuntimeConfigArc = Arc<RwLock<RuntimeConfig>>;
const N: usize = 32;

#[derive(Debug, PartialEq)]
pub struct JoinCommand {
    pub address: Url,
}

impl JoinCommand {
    pub fn from_args(args: &Args) -> Option<JoinCommand> {
        if let Some(SubArg::Join {
            join_address: address,
            ..
        }) = args.sub_cmd.clone()
        {
            Some(JoinCommand { address })
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub address: SocketAddr,
    pub external_address: Url,
    pub backup_dir: PathBuf,
    pub backup_interval: u64,
    pub backup_amount: usize,
    pub backup_skip_loading: bool,
    pub neighbours: HashSet<Url>,
    pub base_code: String,
    pub identifier: String,
    // pub join_subcommand: Option<JoinCommand>,
}

impl RuntimeConfig {
    pub fn to_arc(self) -> Arc<RwLock<RuntimeConfig>> {
        Arc::new(RwLock::new(self))
    }

    pub fn generate_code() -> String {
        let mut rng = thread_rng();

        let mut buffer = [0u8; N];
        rng.fill(&mut buffer[..]);
        base64(buffer)
    }
}

impl Default for RuntimeConfig {
    fn default() -> RuntimeConfig {
        let identifier = Self::generate_code();
        RuntimeConfig {
            address: "127.0.0.1:9226".parse().unwrap(),
            external_address: "http://127.0.0.1:9226".parse().unwrap(),
            backup_dir: PathBuf::from("datadir"),
            backup_interval: 10,
            backup_amount: 10,
            backup_skip_loading: false,
            neighbours: HashSet::new(),
            base_code: base64_sha3(&identifier),
            identifier,
            // join_subcommand: None,
        }
    }
}

pub fn base64_sha3(input: &str) -> String {
    base64(Sha3_512::digest(input.as_bytes()))
}

pub fn base64<T: AsRef<[u8]>>(input: T) -> String {
    base64::encode_config(input, base64::URL_SAFE_NO_PAD)
}
