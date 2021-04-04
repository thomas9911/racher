#![warn(rust_2018_idioms)]

use libracher;

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;

use dashmap::DashMap;
use serde_value::Value;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::{signal, time, io};
// use futures::poll;
// use futures::FutureExt;
use futures::future::FutureExt;
use futures::future::abortable;

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
}

async fn http_server(args: Args, cache: libracher::Db) -> Result<(), Box<dyn Error>> {
    let api = libracher::create_api(cache);
    let (addr, server) = warp::serve(api).bind_with_graceful_shutdown(args.address, async {
        signal::ctrl_c().await.expect("failed to listen for event")
    });

    let mut stdout = stdout();
    stdout
        .write_all(format!("Listening on: http://{}\n", addr).as_bytes())
        .await?;
    stdout.flush().await?;

    server.await;

    Ok(())
}

async fn fs_loop(cache: libracher::Db) -> io::Result<()> {
    let mut interval = time::interval(time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let mut file = File::create("foo.json").await?;

        let bytes = serde_json::to_vec(&*cache)?;
        file.write_all(&bytes).await?;
    }
}

async fn sync_to_fs(cache: libracher::Db) -> Result<(), Box<dyn Error>> {
    let (future, handle) = abortable(fs_loop(cache));
    let (_, poll) = tokio::join!(
        signal::ctrl_c().then(|_| async move {handle.abort()}),
        future
    );
    poll??;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args_safe()?;

    let cache = DashMap::new();
    cache.insert(String::from("testing"), Value::Bool(true));
    let arc_cache = Arc::new(cache);

    let (http, fs_handler) = tokio::join!(
        http_server(args, arc_cache.clone()),
        sync_to_fs(arc_cache.clone())
    );

    http?;
    fs_handler.ok();

    // tokio::select!{
    //     _ = http_server(args, arc_cache.clone()) => {},
    //     _ = sync_to_fs(arc_cache.clone()) => {},
    // };

    let mut stdout = stdout();
    stdout.write_all(b"Bye").await?;

    Ok(())
}
