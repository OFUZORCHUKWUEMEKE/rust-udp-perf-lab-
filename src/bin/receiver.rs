use clap::{Parser, ValueEnum};
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use std::os::unix::io::AsRawFd;
use libc::{iovec, msghdr, mmsghdr};
use std::ptr;

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

fn run_fast_mode(socket: UdpSocket) -> std::io::Result<()> {
    // 1. Setup the exact same Metrics counter from Stage 3
    let total_packets = Arc::new(AtomicU64::new(0));
    let metrics_counter = Arc::clone(&total_packets);

    thread::spawn(move || {
        let mut last_count = 0;
        loop {
            thread::sleep(Duration::from_secs(1));
            let current_count = metrics_counter.load(Ordering::Relaxed);
            println!("Fast Mode Speed: {} PPS", current_count - last_count);
            last_count = current_count;
        }
    });

    println!("Starting fast mode loop...");

    // 2. We define our batch size. We want the kernel to grab 1024 packets at once.
    const BATCH_SIZE: usize = 1024;
    const PACKET_SIZE: usize = 2048;

    // 3. We have to pre-allocate memory for all 1024 packets
    let mut buffers = vec![[0u8; PACKET_SIZE]; BATCH_SIZE];
    
    // 4. We set up C-compatible struct pointers (iovec and mmsghdr) that 
    // point to our Rust buffers. This allows the Linux kernel to write directly into them.
    let mut iovecs: Vec<iovec> = buffers.iter_mut().map(|buf| {
        iovec {
            iov_base: buf.as_mut_ptr() as *mut libc::c_void,
            iov_len: buf.len(),
        }
    }).collect();

    let mut msgs: Vec<mmsghdr> = iovecs.iter_mut().map(|iov| {
        mmsghdr {
            msg_hdr: msghdr {
                msg_name: ptr::null_mut(),
                msg_namelen: 0,
                msg_iov: iov as *mut iovec,
                msg_iovlen: 1,
                msg_control: ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            },
            msg_len: 0,
        }
    }).collect();

    let fd = socket.as_raw_fd();

    loop {
        // 5. THE MAGIC: We make ONE system call to fetch up to 1024 packets
        let packets_received = unsafe {
            libc::recvmmsg(
                fd,
                msgs.as_mut_ptr(),
                BATCH_SIZE as u32,
                0, // no flags
                ptr::null_mut() // no timeout
            )
        };

        if packets_received > 0 {
            // We successfully pulled down a batch!
            total_packets.fetch_add(packets_received as u64, Ordering::Relaxed);
        } else if packets_received < 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("recvmmsg error: {}", err);
        }
    }
}

