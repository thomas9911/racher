use crate::config::RuntimeConfigArc;
use crate::transport;
use crate::transport::Message;
use crate::Db;

use std::convert::Infallible;
use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use chrono::Utc;
use dashmap::DashMap;
use serde_value::Value;
use tokio::io::AsyncWriteExt;
use tokio::{fs, io, signal, time};
use tower::ServiceBuilder;
use tracing::{debug, info};
use url::Url;
use warp::hyper::server::Server;

pub async fn http_server(
    cfg: RuntimeConfigArc,
    cache: Db,
    tx: transport::Sender,
) -> Result<(), Box<dyn Error>> {
    let address = { cfg.read().await.address.clone() };
    // let (addr, server) = warp::serve(api).bind_with_graceful_shutdown(address, async {
    //     signal::ctrl_c().await.expect("failed to listen for event")
    // });
    let api = crate::create_api(cache.clone(), cfg.clone(), tx);

    let warp_svc = warp::service(api);
    // let make_svc = warp::hyper::service::make_service_fn(move |_| {
    //     let warp_svc = warp_svc.clone();
    //     async move {
    //         let svc = warp::hyper::service::service_fn(move |req: Request<Body>| {
    //             let mut warp_svc = warp_svc.clone();
    //             warp_svc.call(req)
    //         });
    //         Ok::<_, Infallible>(svc)
    //     }
    // });

    let f = move |_: &warp::hyper::server::conn::AddrStream| {
        let svc = warp_svc.clone();
        async move { Ok::<_, Infallible>(svc) }
    };

    let x = ServiceBuilder::new()
        // .buffer(100)
        // .layer(tower::buffer::BufferLayer::new(100))
        // .concurrency_limit(10)
        .layer(tower::limit::ConcurrencyLimitLayer::new(10000))
        .layer(tower::timeout::TimeoutLayer::new(Duration::from_millis(
            15000,
        )))
        .service_fn(f);

    let server = Server::bind(&address).serve(x);
    info!("address: http://{}", server.local_addr());
    let gracefull = server.with_graceful_shutdown(async {
        signal::ctrl_c().await.expect("failed to listen for event")
    });

    gracefull.await?;

    Ok(())
}

pub async fn server_sender(
    cfg: RuntimeConfigArc,
    mut rx: transport::Receiver,
) -> Result<(), std::convert::Infallible> {
    let mut client = crate::client::Client::new();
    loop {
        if let Ok(message) = rx.recv().await {
            let (key, value) = match message {
                Message::Created(key, value) => (key, value),
                Message::Deleted(key) => (key, Value::Unit),
            };
            let neighbours = {
                let read_cfg = cfg.read().await;
                read_cfg.neighbours.clone()
            };
            for host in neighbours {
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
            let mut file = {
                let read_cfg = cfg.read().await;
                read_cfg.backup_dir.clone()
            };
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

        let (backup_amount, backup_dir) = {
            let read_cfg = cfg.read().await;

            (read_cfg.backup_amount, read_cfg.backup_dir.clone())
        };
        let mut filenames = fetch_data_dir_filenames(&backup_dir).await?;
        filenames.reverse();
        debug!("started clean up old backups");

        for item in filenames.into_iter().skip(backup_amount) {
            let mut file = backup_dir.clone();
            file.push(item);
            // on error just continue
            fs::remove_file(file).await.ok();
        }

        debug!("done clean up old backups");
    }
}

pub async fn sync_neighbours(cfg: RuntimeConfigArc) -> Result<(), Box<dyn Error>> {
    time::interval(time::Duration::from_secs(10)).tick().await;

    let mut interval = { time::interval(time::Duration::from_secs(60)) };
    loop {
        let (neighbours, me) = {
            let read_cfg = cfg.read().await;
            let mut neighbours = read_cfg.neighbours.clone();
            let me = read_cfg.external_address.clone();
            neighbours.remove(&me);
            (neighbours, me)
        };
        let mut client = crate::client::Client::new();
        let neighbours = client.ping_all(neighbours).await?;
        {
            let mut write_cfg = cfg.write().await;
            write_cfg.neighbours = neighbours.clone();
        }
        client.join_all(me, neighbours).await?;

        interval.tick().await;
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
    client.ping(join_address.clone()).await?;
    let addr = {
        let read_config = config.read().await;
        read_config.external_address.clone()
    };
    let response = client.join(addr, join_address.clone()).await?;
    let code = response.code;
    let neighbours = response.neighbours;
    let response = client.sync(join_address.clone(), &code).await?;
    let mut write_config = config.write().await;
    write_config.neighbours.extend(neighbours);
    write_config.neighbours.insert(join_address);
    write_config.base_code = code;
    Ok(response)
}
