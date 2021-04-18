use crate::config::RuntimeConfigArc;
use crate::Db;

use std::convert::Infallible;
use std::error::Error;
use std::path::PathBuf;

use chrono::Utc;
use dashmap::DashMap;
use serde_value::Value;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast;
use tokio::{fs, io, signal, time};
use tracing::{debug, info};
use url::Url;
use warp::hyper::server::Server;
use warp::hyper::service::Service;
use warp::hyper::{Body, Request};

pub async fn http_server(
    cfg: RuntimeConfigArc,
    cache: Db,
    tx: broadcast::Sender<(String, Value)>,
) -> Result<(), Box<dyn Error>> {
    let address = cfg.read().await.address;
    // let (addr, server) = warp::serve(api).bind_with_graceful_shutdown(address, async {
    //     signal::ctrl_c().await.expect("failed to listen for event")
    // });
    let api = crate::create_api(cache.clone(), cfg.clone(), tx);

    let warp_svc = warp::service(api);
    let make_svc = warp::hyper::service::make_service_fn(move |_| {
        let warp_svc = warp_svc.clone();
        async move {
            let svc = warp::hyper::service::service_fn(move |req: Request<Body>| {
                let mut warp_svc = warp_svc.clone();
                warp_svc.call(req)
            });
            Ok::<_, Infallible>(svc)
        }
    });

    let server = Server::bind(&address).serve(make_svc);
    info!("address: http://{}", server.local_addr());
    let gracefull = server.with_graceful_shutdown(async {
        signal::ctrl_c().await.expect("failed to listen for event")
    });

    gracefull.await?;

    Ok(())
}

pub async fn server_sender(
    cfg: RuntimeConfigArc,
    mut rx: broadcast::Receiver<(std::string::String, serde_value::Value)>,
) -> Result<(), std::convert::Infallible> {
    let mut client = crate::client::Client::new();
    loop {
        if let Ok((key, value)) = rx.recv().await {
            let read_cfg = cfg.read().await;
            for host in read_cfg.neighbours.clone() {
                client.internal_update(host, &key, &value).await;
            }
        }
    }
}

pub async fn fs_loop(cfg: RuntimeConfigArc, cache: Db) -> io::Result<()> {
    let mut interval = {
        let read_cfg = cfg.read().await;
        time::interval(time::Duration::from_secs(read_cfg.backup_interval))
    };
    loop {
        interval.tick().await;
        let (mut file, path) = {
            let read_cfg = cfg.read().await;
            let mut file = read_cfg.backup_dir.clone();
            fs::create_dir_all(&file).await?;

            file.push(format!("racher-{}", Utc::now().format("%Y%m%dT%H%M%S%6f")));
            file.set_extension("json");

            (fs::File::create(&file).await?, file)
        };
        debug!("writing to file: {:?}", path);
        let bytes = serde_json::to_vec(&*cache)?;
        file.write_all(&bytes).await?;
    }
}

pub async fn fetch_data_dir_filenames(backup_dir: &PathBuf) -> io::Result<Vec<std::ffi::OsString>> {
    let mut dir_contents = match fs::read_dir(backup_dir).await {
        Err(_) => return Ok(Vec::new()),
        Ok(x) => x,
    };

    let mut filenames = Vec::new();

    while let Some(entry) = dir_contents.next_entry().await? {
        filenames.push(entry.file_name());
    }
    let mut filenames: Vec<_> = filenames
        .into_iter()
        .filter(|x| match x.to_str() {
            None => false,
            Some(string) => string.starts_with("racher"),
        })
        .collect();
    filenames.sort();
    Ok(filenames)
}

pub async fn clean_data_dir(cfg: RuntimeConfigArc) -> io::Result<()> {
    let mut interval = {
        let read_cfg = cfg.read().await;
        time::interval(time::Duration::from_secs(read_cfg.backup_interval))
    };
    loop {
        interval.tick().await;

        let read_cfg = cfg.read().await;
        let mut filenames = fetch_data_dir_filenames(&read_cfg.backup_dir).await?;
        filenames.reverse();
        debug!("started clean up old backups");

        for item in filenames.into_iter().skip(read_cfg.backup_amount) {
            let mut file = read_cfg.backup_dir.clone();
            file.push(item);
            // on error just continue
            fs::remove_file(file).await.ok();
        }

        debug!("done clean up old backups");
    }
}

pub async fn sync_to_fs(cfg: RuntimeConfigArc, cache: Db) -> Result<(), Box<dyn Error>> {
    tokio::select! {
        Ok(()) = signal::ctrl_c() => {}
        Ok(()) = fs_loop(cfg.clone(), cache) => {}
        Ok(()) = clean_data_dir(cfg.clone()) => {}
    };

    Ok(())
}

pub async fn load_from_backup(
    config: RuntimeConfigArc,
) -> Result<DashMap<String, Value>, Box<dyn Error>> {
    let read_cfg = config.read().await;
    if read_cfg.backup_skip_loading {
        return Ok(DashMap::new());
    }

    let mut filenames = fetch_data_dir_filenames(&read_cfg.backup_dir).await?;

    match filenames.pop() {
        Some(x) => {
            let mut file = read_cfg.backup_dir.clone();
            file.push(x);
            let contents = fs::read(file).await?;
            Ok(serde_json::from_slice(&contents)?)
        }
        None => Ok(DashMap::new()),
    }
}

pub async fn remove_backups(backup_dir: &PathBuf) -> Result<(), Box<dyn Error>> {
    let filenames = fetch_data_dir_filenames(backup_dir).await?;

    for item in filenames.into_iter() {
        let mut file = backup_dir.clone();
        file.push(item);
        fs::remove_file(file).await?;
    }

    Ok(())
}

pub async fn join_cache(join_address: Url, config: RuntimeConfigArc) -> Result<Db, Box<dyn Error>> {
    let mut client = crate::client::Client::new();
    let read_config = config.read().await;
    let addr = read_config.external_address.clone();
    let response = client.join(addr, join_address.clone()).await?;
    let code = response.code;
    let response = client.sync(join_address.clone(), &code).await?;
    let mut write_config = config.write().await;
    write_config.neighbours.insert(join_address);
    write_config.base_code = code;
    Ok(response)
}
