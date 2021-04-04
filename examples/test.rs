use libracher::responses::Response;

use futures::prelude::*;
use futures::stream::FuturesUnordered;
use serde_json::json;

const URL: &'static str = "http://127.0.0.1:9226";

async fn get(client: &reqwest::Client) -> reqwest::Result<Response> {
    client
        .post(format!("{}/get/testing", URL))
        .send()
        .await?
        .json()
        .await
}

async fn set(client: &reqwest::Client) -> reqwest::Result<Response> {
    client
        .post(format!("{}/set/testing", URL))
        .json(&json!({"override": 1}))
        .send()
        .await?
        .json()
        .await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let reqs = FuturesUnordered::new();
    let reqs2 = FuturesUnordered::new();
    let reqs3 = FuturesUnordered::new();

    for _ in 0..1000 {
        reqs.push(get(&client));
    }

    for _ in 0..1000 {
        reqs2.push(set(&client));
    }

    for _ in 0..1000 {
        reqs3.push(get(&client));
    }

    let responses = reqs
        .chain(reqs2)
        .chain(reqs3)
        .try_collect::<Vec<_>>()
        .await?;

    let mut a = None;
    for item in responses {
        a = Some(item);
    }
    println!("{:?}", a);

    // futures::join!(reqs, reqs2);
    Ok(())
}
