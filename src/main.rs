#![warn(rust_2018_idioms)]

use libracher;
use libracher::cli;
use libracher::config::RuntimeConfig;

use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::io::{stdout, AsyncWriteExt};

#[derive(Debug, Clone)]
/// wrapper around PathBuf that defaults to the temp_dir()
struct TempPathBuf {
    path: PathBuf,
}

impl Default for TempPathBuf {
    fn default() -> Self {
        TempPathBuf {
            path: std::env::temp_dir(),
        }
    }
}

impl std::fmt::Display for TempPathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl std::str::FromStr for TempPathBuf {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from_str(s)?;
        Ok(TempPathBuf { path })
    }
}

///
///
///
///
#[derive(Debug, StructOpt)]
#[structopt(about)]
struct Args {
    /// address to bind to
    #[structopt(short, long, default_value = "127.0.0.1:9226", env = "RASHER_ADDRESS")]
    address: SocketAddr,
    /// folder to write the backup files to
    #[structopt(long, default_value, env = "RASHER_BACKUP_DIR")]
    pub backup_dir: TempPathBuf,
    /// interval after to create a backup file in seconds
    #[structopt(long, default_value = "60", env = "RASHER_BACKUP_INTERVAL")]
    pub backup_interval: u64,
    /// amount of backups files that are kept
    #[structopt(long, default_value = "10", env = "RASHER_BACKUP_AMOUNT")]
    pub backup_amount: usize,
    #[structopt(long, env = "RASHER_BACKUP_SKIP_LOADING")]
    pub backup_skip_loading: bool,
    /// removes all the backup files and exists
    #[structopt(long)]
    pub backup_remove: bool,
}

impl Args {
    pub fn as_runtime_config(&self) -> RuntimeConfig {
        RuntimeConfig {
            address: self.address.clone(),
            backup_dir: self.backup_dir.path.clone(),
            backup_interval: self.backup_interval,
            backup_amount: self.backup_amount,
            backup_skip_loading: self.backup_skip_loading,
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = match Args::from_args_safe() {
        Ok(x) => x,
        Err(e) => return Ok(println!("{}", e)),
    };

    let config = args.as_runtime_config().to_arc();
    if args.backup_remove {
        return Ok(cli::remove_backups(config.clone()).await?);
    };

    let cache = cli::load_from_backup(config.clone()).await?;
    let arc_cache = Arc::new(cache);

    let (http, fs_handler) = tokio::join!(
        cli::http_server(config.clone(), arc_cache.clone()),
        cli::sync_to_fs(config.clone(), arc_cache.clone())
    );

    http?;
    let mut stdout = stdout();

    match fs_handler {
        Ok(_) => (),
        Err(e) => stdout.write_all(format!("{}\n", e).as_bytes()).await?,
    };

    // tokio::select!{
    //     _ = http_server(args, arc_cache.clone()) => {},
    //     _ = sync_to_fs(arc_cache.clone()) => {},
    // };

    stdout.write_all(b"Bye").await?;

    Ok(())
}
