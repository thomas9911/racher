#![warn(rust_2018_idioms)]

use libracher;
use libracher::arguments::{Args, SubArg};
use libracher::cli;
use libracher::config::RuntimeConfigArc;

use std::error::Error;
use std::sync::Arc;

use structopt::StructOpt;
use tokio::sync::broadcast;
use tokio::{signal, task};
use tracing::{debug, error};

async fn inner_loop(args: &Args, config: RuntimeConfigArc) -> Result<(), Box<dyn Error>> {
    // if args.backup_args.backup_remove {
    //     return Ok(cli::remove_backups(config.clone()).await?);
    // };
    let arc_cache = match args.sub_cmd.as_ref() {
        Some(SubArg::RemoveBackup { backup_dir }) => {
            return Ok(cli::remove_backups(&backup_dir.path).await?);
        }
        Some(SubArg::Join { join_address, .. }) => {
            cli::join_cache(join_address.clone(), config.clone()).await?
        }
        None => {
            let cache = cli::load_from_backup(config.clone()).await?;
            Arc::new(cache)
        }
    };

    let (tx, rx1) = broadcast::channel(16);

    let config_clone = config.clone();
    let cache_clone = arc_cache.clone();
    let http_server = task::spawn(async { cli::http_server(config_clone, cache_clone, tx) });
    let config_clone = config.clone();
    let cache_clone = arc_cache.clone();
    let sync_to_fs = task::spawn(async { cli::sync_to_fs(config_clone, cache_clone) });
    let config_clone = config.clone();
    let server_sender = task::spawn(async { cli::server_sender(config_clone, rx1) });

    tokio::select!(
        Ok(()) = signal::ctrl_c() => {},
        Ok(()) = http_server.await? => {},
        Ok(()) = sync_to_fs.await? => {},
        Ok(()) = server_sender.await? => {},
    );

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
    println!("{:?}", args);

    let config = args.as_runtime_config();
    println!("{:?}", config);
    // let config = args.as_runtime_config().to_arc();
    let config = config.to_arc();
    main_loop(&args, config).await?;

    debug!("Exiting");

    Ok(())
}
