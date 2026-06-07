use clap::{Parser, ValueEnum};
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(name = "UDP Receiver")]
struct Args {
    /// Which mode to run the receiver in
    #[arg(short, long, value_enum, default_value_t = Mode::Slow)]
    mode: Mode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Mode {
    Slow,
    Fast,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:9000")?;
    println!("Starting receiver in {:?} mode", args.mode);

    match args.mode {
        Mode::Slow => run_slow_mode(socket)?,
        Mode::Fast => println!("Fast mode not implemented yet!"),
    }
    Ok(())
}

fn run_slow_mode(socket: UdpSocket) -> std::io::Result<()> {
    let mut buffer = [0u8; 2048];

    // We use Arc (Atomic Reference Counted) so multiple threads can safely share this counter
    let total_packets = Arc::new(AtomicU64::new(0));

    // Clone the reference to the counter so we can give it to the metrics thread
    let metrics_counter = Arc::clone(&total_packets);

    // Spawn a background thread dedicated entirely to printing the PPS
    thread::spawn(move || {
        let mut last_count = 0;
        loop {
            thread::sleep(Duration::from_secs(1));
            // Read the current total
            let current_count = metrics_counter.load(Ordering::Relaxed);
            let pps = current_count - last_count;
            println!("Speed: {} PPS", pps);
            last_count = current_count;
        }
    });

    println!("Starting slow mode loop...");
    loop {
        match socket.recv_from(&mut buffer) {
            Ok(_) => {
                // Increment the atomic counter. 'Relaxed' means we don't need strict ordering
                total_packets.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }
}
