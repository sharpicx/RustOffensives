use clap::{Parser, Subcommand};
use futures::future::join_all;
use ipnetwork::IpNetwork;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Semaphore,
    time::timeout,
};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Listen {
        host: String,
        port: u16,
        #[arg(long)]
        oneshot: bool,
        #[arg(long)]
        verbose: bool,
    },
    Send {
        host: String,
        port: u16,
        data: String,
        #[arg(long, default_value_t = 3000)]
        timeout_ms: u64,
        #[arg(long)]
        verbose: bool,
    },
    Scan {
        host: String,
        start: u16,
        end: u16,
        #[arg(long, default_value_t = 500)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 500)]
        concurrency: usize,
        #[arg(long)]
        verbose: bool,
    },
    // HttpGet {
    //     url: String,
    // },
    // Hello,
    // #[cfg(target_os = "windows")]
    // MessageBox {
    //     title: String,
    //     message: String,
    // },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::Listen {
            host,
            port,
            oneshot,
            verbose,
        } => {
            run_listener(host, port, oneshot, verbose).await;
        }
        Commands::Send {
            host,
            port,
            data,
            timeout_ms,
            verbose,
        } => {
            send_tcp(host, port, data, timeout_ms, verbose).await;
        }
        Commands::Scan {
            host,
            start,
            end,
            timeout_ms,
            concurrency,
            verbose,
        } => {
            port_scan(host, start, end, timeout_ms, concurrency, verbose).await;
        } // Commands::HttpGet { url } => {
          //     let _ = reqwest::blocking::get(url);
          // }
          // Commands::Hello => {
          //     println!("Hello from Rust executable!");
          // }
          // #[cfg(target_os = "windows")]
          // Commands::MessageBox { title, message } => {
          //     show_message_box(&title, &message);
          // }
    }
}

async fn run_listener(host: String, port: u16, oneshot: bool, verbose: bool) {
    let bind = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&bind).await.expect("bind failed");

    if verbose {
        println!(
            "[INFO] listening on {} (mode={})",
            bind,
            if oneshot { "one-shot" } else { "persistent" }
        );
    }

    loop {
        let (mut socket, addr) = match listener.accept().await {
            Ok(v) => v,
            Err(e) => {
                if verbose {
                    println!("[ERR] accept error: {}", e);
                }
                break;
            }
        };

        if verbose {
            println!("[INFO] accepted {}", addr);
        }

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if socket.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        if oneshot {
            break;
        }
    }
}

async fn send_tcp(host: String, port: u16, data: String, timeout_ms: u64, verbose: bool) {
    let addr = format!("{}:{}", host, port);
    let dur = Duration::from_millis(timeout_ms);

    match timeout(dur, TcpStream::connect(&addr)).await {
        Ok(Ok(mut stream)) => {
            let _ = stream.write_all(data.as_bytes()).await;

            let mut buf = [0u8; 1024];
            if let Ok(Ok(n)) = timeout(Duration::from_secs(3), stream.read(&mut buf)).await {
                if verbose && n > 0 {
                    println!("[INFO] recv: {:?}", String::from_utf8_lossy(&buf[..n]));
                }
            }
        }
        _ => {
            if verbose {
                println!("[ERR] connect failed");
            }
        }
    }
}

async fn port_scan(
    target: String,
    start: u16,
    end: u16,
    timeout_ms: u64,
    concurrency: usize,
    verbose: bool,
) {
    let hosts = parse_targets(&target);
    let s = start.min(end);
    let e = start.max(end);

    let sem = Arc::new(Semaphore::new(concurrency));
    let open_count = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::new();
    let dur = Duration::from_millis(timeout_ms);

    for host in hosts {
        for port in s..=e {
            let permit = sem.clone().acquire_owned().await.unwrap();
            let host = host.clone();
            let open_count = open_count.clone();

            tasks.push(tokio::spawn(async move {
                let addr = format!("{}:{}", host, port);
                if let Ok(Ok(_)) = timeout(dur, TcpStream::connect(&addr)).await {
                    open_count.fetch_add(1, Ordering::Relaxed);
                    println!("[OPEN] {}:{}", host, port);
                }
                drop(permit);
            }));
        }
    }

    join_all(tasks).await;

    if verbose {
        println!(
            "[INFO] scan complete: open={}",
            open_count.load(Ordering::Relaxed)
        );
    }
}
fn parse_targets(target: &str) -> Vec<String> {
    if target.contains('/') {
        if let Ok(net) = target.parse::<IpNetwork>() {
            return net.iter().map(|ip| ip.to_string()).collect();
        }
    }
    vec![target.to_string()]
}

// #[cfg(target_os = "windows")]
// fn show_message_box(title: &str, message: &str) {
//     use windows::{
//         Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW},
//         core::PCWSTR,
//     };
//
//     let t: Vec<u16> = title.encode_utf16().chain(Some(0)).collect();
//     let m: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
//
//     unsafe {
//         MessageBoxW(None, PCWSTR(m.as_ptr()), PCWSTR(t.as_ptr()), MB_OK);
//     }
// }
