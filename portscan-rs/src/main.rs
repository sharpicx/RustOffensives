use clap::{Parser, Subcommand};
use ipnetwork::IpNetwork;
use std::{
    fs::OpenOptions,
    io::Write,
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
        start: Option<u16>,
        end: Option<u16>,
        #[arg(long)]
        common_ports: bool,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long, default_value_t = 500)]
        timeout_ms: u64,
        #[arg(long, default_value_t = 500)]
        concurrency: usize,
        #[arg(long)]
        verbose: bool,
    },
}

const COMMON_PORTS: &[u16] = &[
    80, 443, 8000, 8008, 8080, 8081, 8443, 8888, 9000, 9090, 21, 22, 23, 115, 3389, 5900, 5901,
    5985, 5986, 1433, 1434, 1521, 3306, 33060, 5432, 6379, 9200, 9300, 27017, 27018, 27019, 7000,
    7001, 9042, 389, 636, 3268, 3269, 135, 137, 138, 139, 445, 25, 465, 587, 110, 995, 143, 993,
    2375, 2376, 6443, 8080, 50000, 1883, 8883, 5672, 15672, 5683, 502, 161, 162, 514, 2049,
];

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
            common_ports,
            output,
            timeout_ms,
            concurrency,
            verbose,
        } => {
            port_scan(
                host,
                start,
                end,
                common_ports,
                output,
                timeout_ms,
                concurrency,
                verbose,
            )
            .await;
        }
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
    start: Option<u16>,
    end: Option<u16>,
    use_common: bool,
    output: Option<String>,
    timeout_ms: u64,
    concurrency: usize,
    verbose: bool,
) {
    let hosts = parse_targets(&target);
    let dur = Duration::from_millis(timeout_ms);
    let sem = Arc::new(Semaphore::new(concurrency));
    let open_count = Arc::new(AtomicUsize::new(0));

    let ports: Vec<u16> = if use_common {
        COMMON_PORTS.to_vec()
    } else {
        let s = start.unwrap_or(1);
        let e = end.unwrap_or(1000);
        (s.min(e)..=s.max(e)).collect()
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel(concurrency * 2);

    tokio::spawn(async move {
        for host in hosts {
            for &port in &ports {
                if tx.send((host.clone(), port)).await.is_err() {
                    break;
                }
            }
        }
    });

    while let Some((host, port)) = rx.recv().await {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let open_count = open_count.clone();
        let out_file = output.clone();

        tokio::spawn(async move {
            let addr = format!("{}:{}", host, port);
            let res = timeout(dur, TcpStream::connect(&addr)).await;

            if let Ok(Ok(stream)) = res {
                let _ = stream.set_nodelay(true);
                open_count.fetch_add(1, Ordering::Relaxed);
                println!("[OPEN] {}", addr);
                if let Some(path) = out_file {
                    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                        let _ = writeln!(file, "{}", addr);
                    }
                }
            }
            drop(permit);
        });
    }

    let _ = sem.acquire_many(concurrency as u32).await;

    if verbose {
        println!(
            "[INFO] scan complete. open={}",
            open_count.load(Ordering::Relaxed)
        );
    }
}

fn parse_targets(target: &str) -> Vec<String> {
    let mut all_ips = Vec::new();

    for part in target.split(',') {
        let part = part.trim();

        if part.contains('-') {
            if let Some(ips) = parse_ip_range(part) {
                all_ips.extend(ips);
                continue;
            }
        }

        if part.contains('/') {
            if let Ok(net) = part.parse::<IpNetwork>() {
                all_ips.extend(net.iter().map(|ip| ip.to_string()));
                continue;
            }
        }

        if !part.is_empty() {
            all_ips.push(part.to_string());
        }
    }

    all_ips
}

fn parse_ip_range(range_str: &str) -> Option<Vec<String>> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return None;
    }

    let start_ip_str = parts[0].trim();
    let end_val_str = parts[1].trim();

    let octets: Vec<&str> = start_ip_str.split('.').collect();
    if octets.len() != 4 {
        return None;
    }

    let base = format!("{}.{}.{}", octets[0], octets[1], octets[2]);
    let start_val: u8 = octets[3].parse().ok()?;
    let end_val: u8 = end_val_str.parse().ok()?;

    let (low, high) = if start_val <= end_val {
        (start_val, end_val)
    } else {
        (end_val, start_val)
    };

    Some(
        (low..=high)
            .map(|last| format!("{}.{}", base, last))
            .collect(),
    )
}
