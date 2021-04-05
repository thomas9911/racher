use crate::config::RuntimeConfigArc;
use crate::Db;

use std::error::Error;
use std::path::PathBuf;

use chrono::Utc;
use dashmap::DashMap;
use futures::future::{abortable, FutureExt};
use serde_value::Value;
use tokio::io::{stdout, AsyncWriteExt};
use tokio::{fs, io, signal, time};

pub async fn http_server(cfg: RuntimeConfigArc, cache: Db) -> Result<(), Box<dyn Error>> {
    let api = crate::create_api(cache);
    let address = cfg.read().unwrap().address;
    let (addr, server) = warp::serve(api).bind_with_graceful_shutdown(address, async {
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

pub async fn fs_loop(cfg: RuntimeConfigArc, cache: Db) -> io::Result<()> {
    let mut interval = {
        let read_cfg = cfg.read().unwrap();
        time::interval(time::Duration::from_secs(read_cfg.backup_interval))
    };
    loop {
        interval.tick().await;
        let mut file = {
            let read_cfg = cfg.read().unwrap();
            let mut file = read_cfg.backup_dir.clone();
            fs::create_dir_all(&file).await?;

            file.push(format!("racher-{}", Utc::now().format("%Y%m%dT%H%M%S%6f")));
            file.set_extension("json");

            fs::File::create(file).await?
        };

        let bytes = serde_json::to_vec(&*cache)?;
        file.write_all(&bytes).await?;
    }
}

pub async fn fetch_data_dir_filenames(backup_dir: &PathBuf) -> io::Result<Vec<std::ffi::OsString>> {
    let mut dir_contents = fs::read_dir(backup_dir).await?;

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
        let read_cfg = cfg.read().unwrap();
        time::interval(time::Duration::from_secs(read_cfg.backup_interval))
    };
    loop {
        interval.tick().await;

        let read_cfg = cfg.read().unwrap();
        let mut filenames = fetch_data_dir_filenames(&read_cfg.backup_dir).await?;
        filenames.reverse();

        for item in filenames.into_iter().skip(read_cfg.backup_amount) {
            let mut file = read_cfg.backup_dir.clone();
            file.push(item);
            // on error just continue
            fs::remove_file(file).await.ok();
        }
    }
}

pub async fn sync_to_fs(cfg: RuntimeConfigArc, cache: Db) -> Result<(), Box<dyn Error>> {
    let (fs_loop_future, fs_loop_handle) = abortable(fs_loop(cfg.clone(), cache));
    let (clean_data_future, clean_handle) = abortable(clean_data_dir(cfg.clone()));
    let (_, _, fs_loop_poll, clean_data_poll) = tokio::join!(
        signal::ctrl_c().then(|_| async move { fs_loop_handle.abort() }),
        signal::ctrl_c().then(|_| async move { clean_handle.abort() }),
        fs_loop_future,
        clean_data_future,
    );
    fs_loop_poll??;
    clean_data_poll??;

    Ok(())
}

pub async fn load_from_backup(
    config: RuntimeConfigArc,
) -> Result<DashMap<String, Value>, Box<dyn Error>> {
    let read_cfg = config.read().unwrap();
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

pub async fn remove_backups(cfg: RuntimeConfigArc) -> Result<(), Box<dyn Error>> {
    let read_cfg = cfg.read().unwrap();
    let filenames = fetch_data_dir_filenames(&read_cfg.backup_dir).await?;

    for item in filenames.into_iter() {
        let mut file = read_cfg.backup_dir.clone();
        file.push(item);
        fs::remove_file(file).await?;
    }

    Ok(())
}
