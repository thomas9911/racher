#![warn(rust_2018_idioms)]

use libracher;
use libracher::cli;
use libracher::config::{RuntimeConfig, RuntimeConfigArc};

use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use structopt::StructOpt;
use tracing::{debug, error};
use tracing_subscriber::filter::LevelFilter;

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
    /// turn off all normal messages
    #[structopt(short, long)]
    pub quiet: bool,
    /// turn off all messages
    #[structopt(short = "Q", long)]
    pub ultra_quiet: bool,
    /// turn on debug messages
    #[structopt(short, long)]
    pub verbose: bool,
    /// turn on all messages
    #[structopt(long)]
    pub trace: bool,
    /// set the log level
    #[structopt(long, env = "RASHER_LOGGER_LEVEL", possible_values = &["ERROR", "WARN", "INFO", "DEBUG", "TRACE", "OFF"])]
    pub logger_level: Option<LevelFilter>,
    #[structopt(short, long, env = "RASHER_OUTPUT_FORMAT", default_value="compact", possible_values = &["compact", "json", "pretty"])]
    pub output: String,
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

    pub fn set_logger(&self) {
        let level = if self.ultra_quiet {
            LevelFilter::OFF
        } else if self.quiet {
            LevelFilter::ERROR
        } else if self.verbose {
            LevelFilter::DEBUG
        } else if self.trace {
            LevelFilter::TRACE
        } else {
            self.logger_level.unwrap_or(LevelFilter::INFO)
        };

        let builder = tracing_subscriber::fmt().with_max_level(level);
        match self.output.as_ref() {
            "json" => builder.json().init(),
            "compact" => builder.compact().init(),
            "pretty" => builder.pretty().init(),
            _ => unreachable!(),
        };
    }
}

async fn inner_loop(args: &Args, config: RuntimeConfigArc) -> Result<(), Box<dyn Error>> {
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
    fs_handler?;

    Ok(())
}

async fn main_loop(args: &Args, config: RuntimeConfigArc) -> Result<(), Box<dyn Error>> {
    loop {
        match inner_loop(args, config.clone()).await {
            Ok(_) => break,
            Err(e) => error!(%e),
        };
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = match Args::from_args_safe() {
        Ok(x) => x,
        Err(e) => return Ok(println!("{}", e)),
    };

    args.set_logger();

    let config = args.as_runtime_config().to_arc();
    main_loop(&args, config).await?;

    debug!("Exiting");

    Ok(())
}
