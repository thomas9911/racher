use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub type RuntimeConfigArc = Arc<RwLock<RuntimeConfig>>;

pub struct RuntimeConfig {
    pub address: SocketAddr,
    pub backup_dir: PathBuf,
    pub backup_interval: u64,
    pub backup_amount: usize,
    pub backup_skip_loading: bool,
}

impl RuntimeConfig {
    pub fn to_arc(self) -> Arc<RwLock<RuntimeConfig>> {
        Arc::new(RwLock::new(self))
    }
}

impl Default for RuntimeConfig {
    fn default() -> RuntimeConfig {
        RuntimeConfig {
            address: "127.0.0.1:9226".parse().unwrap(),
            backup_dir: PathBuf::from("datadir"),
            backup_interval: 10,
            backup_amount: 10,
            backup_skip_loading: false,
        }
    }
}
