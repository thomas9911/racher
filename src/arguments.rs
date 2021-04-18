use crate::config::RuntimeConfig;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;
use tracing_subscriber::filter::LevelFilter;
use url::Url;

#[derive(Debug, Clone)]
/// wrapper around PathBuf that defaults to the temp_dir()
pub struct TempPathBuf {
    pub path: PathBuf,
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

fn parse_vec(input: &str) -> Result<Vec<Url>, url::ParseError> {
    let mut addresses = Vec::new();

    for item in input.split(",") {
        let addr = Url::from_str(item.trim())?;
        addresses.push(addr);
    }

    Ok(addresses)
}

///
///
///
///
#[derive(Debug, StructOpt)]
#[structopt(about)]
pub struct Args {
    #[structopt(flatten)]
    pub default_args: DefaultArgs,
    #[structopt(flatten)]
    pub logger_args: LoggerArgs,
    #[structopt(flatten)]
    pub backup_args: BackupArgs,
    #[structopt(subcommand)]
    pub sub_cmd: Option<SubArg>,
}

#[derive(Debug, Clone, StructOpt)]
pub struct LoggerArgs {
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
    #[structopt(long, env = "RACHER_LOGGER_LEVEL", possible_values = &["ERROR", "WARN", "INFO", "DEBUG", "TRACE", "OFF"])]
    pub logger_level: Option<LevelFilter>,
    /// set the output format
    #[structopt(short, long, env = "RACHER_OUTPUT_FORMAT", default_value="compact", possible_values = &["compact", "json", "pretty"])]
    pub output: String,
}

#[derive(Debug, Clone, StructOpt)]
pub struct BackupArgs {
    /// folder to write the backup files to
    #[structopt(long, default_value, env = "RACHER_BACKUP_DIR")]
    pub backup_dir: TempPathBuf,
    /// interval after to create a backup file in seconds
    #[structopt(long, default_value = "60", env = "RACHER_BACKUP_INTERVAL")]
    pub backup_interval: u64,
    /// amount of backups files that are kept
    #[structopt(long, default_value = "10", env = "RACHER_BACKUP_AMOUNT")]
    pub backup_amount: usize,
    ///
    #[structopt(long, env = "RACHER_BACKUP_SKIP_LOADING")]
    pub backup_skip_loading: bool,
    // /// removes all the backup files and exits
    // #[structopt(long)]
    // pub backup_remove: bool,
}

#[derive(Debug, Clone, StructOpt)]
pub struct DefaultArgs {
    /// address to bind to
    #[structopt(short, long, default_value = "127.0.0.1:9226", env = "RACHER_ADDRESS")]
    address: SocketAddr,
    ///
    #[structopt(long, env = "RACHER_NEIGHBOURS", parse(try_from_str = parse_vec))]
    pub neighbours: Vec<Vec<Url>>,
}

#[derive(Debug, Clone, StructOpt)]
pub enum SubArg {
    /// join already running racher
    Join {
        #[structopt(flatten)]
        default_args: DefaultArgs,
        #[structopt(flatten)]
        logger_args: LoggerArgs,
        #[structopt(flatten)]
        backup_args: BackupArgs,
        /// address to join
        #[structopt(short, long, env = "RACHER_JOIN_ADDRESS")]
        join_address: Url,
        /// address where the other racher can reach this instance, by default it takes the value from the address argument
        #[structopt(short, long, env = "RACHER_EXTERNAL_ADDRESS")]
        external_address: Option<Url>,
    },
    /// removes all the backup files and exits
    RemoveBackup {
        /// folder to write the backup files to
        #[structopt(long, default_value, env = "RACHER_BACKUP_DIR")]
        backup_dir: TempPathBuf,
    },
}

impl Args {
    pub fn as_runtime_config(&self) -> RuntimeConfig {
        let (default_args, backup_args, external_address) = if let Some(SubArg::Join {
            default_args,
            backup_args,
            external_address,
            ..
        }) = self.sub_cmd.clone()
        {
            let external_address = external_address.unwrap_or(addr_to_url(&default_args.address));
            (default_args, backup_args, external_address)
        } else {
            (
                self.default_args.clone(),
                self.backup_args.clone(),
                addr_to_url(&self.default_args.address),
            )
        };
        RuntimeConfig {
            address: default_args.address.clone(),
            backup_dir: backup_args.backup_dir.path.clone(),
            backup_interval: backup_args.backup_interval,
            backup_amount: backup_args.backup_amount,
            backup_skip_loading: backup_args.backup_skip_loading,
            external_address: external_address,
            neighbours: default_args
                .neighbours
                .clone()
                .into_iter()
                .flatten()
                .collect(),
            ..Default::default()
        }
    }

    pub fn set_logger(&self) {
        let args = if let Some(SubArg::Join { logger_args, .. }) = self.sub_cmd.clone() {
            logger_args
        } else {
            self.logger_args.clone()
        };
        let level = if args.ultra_quiet {
            LevelFilter::OFF
        } else if args.quiet {
            LevelFilter::ERROR
        } else if args.verbose {
            LevelFilter::DEBUG
        } else if args.trace {
            LevelFilter::TRACE
        } else {
            args.logger_level.unwrap_or(LevelFilter::INFO)
        };

        let builder = tracing_subscriber::fmt().with_max_level(level);
        match args.output.as_ref() {
            "json" => builder.json().init(),
            "compact" => builder.compact().init(),
            "pretty" => builder.pretty().init(),
            _ => unreachable!(),
        };
    }
}

fn addr_to_url(addr: &std::net::SocketAddr) -> url::Url {
    Url::parse(&format!("http://{}", addr)).unwrap()
}
