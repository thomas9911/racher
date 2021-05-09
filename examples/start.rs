const BINARY: &'static str = "D:\\Rust\\racher\\target\\release\\racher.exe";
use std::io::Error;
// use std::process::{Child, Command};
// use std::thread;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::time;

fn envs() -> Vec<(String, String)> {
    vec![
        // ("RACHER_DEVELOPMENT", "1"),
        ("RACHER_BACKUP_SKIP_LOADING", "1"),
        ("RACHER_BACKUP_AMOUNT", "0"),
        ("RACHER_BACKUP_INTERVAL", "120"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn start_first(url: &str) -> Result<Child, Error> {
    Command::new(BINARY).args(&["-a", url]).envs(envs()).spawn()
}

fn start_extras(main: &str, url: &str) -> Result<Child, Error> {
    Command::new(BINARY)
        .args(&["join", "-a", url, "-j", main])
        .envs(envs())
        .spawn()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut first = start_first("127.0.0.1:9226")?;
    time::sleep(Duration::from_millis(1000)).await;

    let mut a = Vec::new();
    for port in 9227..9231 {
        a.push(start_extras(
            "http://127.0.0.1:9226",
            &format!("127.0.0.1:{}", port),
        )?)
    }

    tokio::select!(
        Ok(()) = signal::ctrl_c() => {},
        Ok(_) = first.wait() => {},
    );

    Ok(())
}

// fn main() -> Result<(), Error> {
//     let mut first = start_first("127.0.0.1:9226")?;
//     thread::sleep(Duration::from_millis(1000));

//     let mut a = Vec::new();
//     for port in 9226..9231 {
//         a.push(start_extras(
//             "http://127.0.0.1:9226",
//             &format!("127.0.0.1:{}", port),
//         )?)
//     }

//     first.wait()?;
//     for mut x in a {
//         x.kill()?;
//     }

//     Ok(())
// }
