use crate::client::Client;
use crate::config::RuntimeConfigArc;
use crate::routes::utils::move_object;
use crate::Db;

use warp::http::StatusCode;

use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_value::Value;
use std::convert::Infallible;
// use std::net::SocketAddr;
use tracing::{debug, error};
use url::Url;
use warp::filters::BoxedFilter;
use warp::reply;
use warp::{Filter, Reply};

use std::iter::FromIterator;

use crate::MAX_FILE_SIZE;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct JoinRequest {
    host: Url,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SyncRequest {
    code: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FanoutRequest {
    code: String,
    host: Url,
}

async fn fanout_to_neighbours(cfg: RuntimeConfigArc, host: Url) -> Result<(), String> {
    let mut client = Client::new();
    // if let Err(_) = client.ping(host.clone()).await {
    //      error!("host '{}' cannot be found", host);
    //      return Err(String::from("invalid host"))
    // }
    let (code, neighbours) = {
        let guard = cfg.read().await;
        let code = guard.base_code.clone();
        let neighbours = guard.neighbours.clone();
        (code, neighbours)
    };
    for neighbour in neighbours {
        debug!("adding host to'{}'", &neighbour);
        if let Err(_) = client.fanout(neighbour.clone(), host.clone(), &code).await {
            error!("failed adding host to '{}'", neighbour.clone())
        }
    }

    Ok(())
}

async fn inner_join(
    req: JoinRequest,
    cfg: RuntimeConfigArc,
) -> Result<impl warp::Reply, Infallible> {
    if req.host.cannot_be_a_base() {
        return Ok(reply::with_status(
            reply::json(&json!({ "error": "invalid host" })),
            StatusCode::BAD_REQUEST,
        ));
    }

    if let Err(txt) = fanout_to_neighbours(cfg.clone(), req.host.clone()).await {
        return Ok(reply::with_status(
            reply::json(&json!({ "error": txt })),
            StatusCode::BAD_REQUEST,
        ));
    }

    let (code, neighbours) = { 
        let mut guard = cfg.write().await;
        guard.neighbours.insert(req.host);
        (guard.base_code.clone(), Vec::from_iter(guard.neighbours.clone().into_iter()))
    };
    Ok(reply::with_status(
        reply::json(&json!({ "code": code, "neighbours": neighbours })),
        StatusCode::OK,
    ))
}

async fn inner_sync(
    req: SyncRequest,
    cache: Db,
    cfg: RuntimeConfigArc,
) -> Result<impl warp::Reply, Infallible> {
    let guard = cfg.read().await;
    if guard.base_code == req.code {
        return Ok(reply::with_status(reply::json(&cache), StatusCode::OK));
    }

    Ok(reply::with_status(
        reply::json(&json!({"error": "invalid code"})),
        StatusCode::BAD_REQUEST,
    ))
}

async fn inner_fanout(
    req: FanoutRequest,
    cfg: RuntimeConfigArc,
) -> Result<impl warp::Reply, Infallible> {
    // let mut client = Client::new();
    // if let Err(_) = client.ping(req.host.clone()).await {
    //     error!("host '{}' cannot be found", req.host);
    //     return Ok(reply::with_status(
    //         reply::json(&json!({"error": "invalid host"})),
    //         StatusCode::BAD_REQUEST,
    //     ))
    // }

    let base_code = {
        let guard = cfg.read().await;
        guard.base_code.clone()
    };

    if base_code == req.code {
        let mut guard = cfg.write().await;
        guard.neighbours.insert(req.host);
        return Ok(reply::with_status(
            reply::json(&json!({"fanout": "success"})),
            StatusCode::OK,
        ));
    }

    Ok(reply::with_status(
        reply::json(&json!({"error": "invalid code"})),
        StatusCode::BAD_REQUEST,
    ))
}

async fn inner_update(
    name: String,
    simple_map: Value,
    cache: Db,
) -> Result<impl warp::Reply, Infallible> {
    cache.insert(name, simple_map);
    Ok::<_, Infallible>(super::ok_reponse())
}

pub async fn inner_config(cfg: RuntimeConfigArc) -> Result<impl warp::Reply, Infallible> {
    let read_config = { cfg.read().await.clone() };
    Ok(reply::with_status(
        reply::json(&read_config),
        StatusCode::OK,
    ))
}

pub fn join(cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("join")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        // .map(move |req: JoinRequest| {
        //     let mut guard = block_on(cfg.write());
        //     guard.neighbours.insert(req.host);
        //     let code = guard.base_code.clone();
        //     reply::json(&json!({ "code": code }))
        // })
        .and(move_object(cfg))
        .and_then(inner_join)
        .boxed()
}

pub fn sync(cache: Db, cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("sync")
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        // .map(move |req: SyncRequest| {
        //     let guard = block_on(cfg.read());
        //     if guard.base_code == req.code {
        //         let json = reply::json(&cache);
        //         reply::with_status(json, StatusCode::OK)
        //     } else {
        //         let json = reply::json(&json!({"error": "invalid code"}));
        //         reply::with_status(json, StatusCode::BAD_REQUEST)
        //     }
        // })
        .and(move_object(cache))
        .and(move_object(cfg))
        .and_then(inner_sync)
        .with(warp::compression::gzip())
        .boxed()
}

pub fn update(cache: Db) -> BoxedFilter<(impl Reply,)> {
    warp::path!("update" / String)
        .and(warp::body::content_length_limit(MAX_FILE_SIZE))
        .and(warp::body::json())
        .and(move_object(cache))
        .and_then(inner_update)
        .boxed()
}

pub fn fanout(cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("fanout")
        .and(warp::body::json())
        .and(move_object(cfg))
        .and_then(inner_fanout)
        .boxed()
}

pub fn config(cfg: RuntimeConfigArc) -> BoxedFilter<(impl Reply,)> {
    warp::path!("config")
        .and(move_object(cfg))
        .and_then(inner_config)
        .boxed()
}
