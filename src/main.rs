use std::collections::HashMap;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:7000")?;

    let mut total_packets = 0u64;
    let mut source_counts = HashMap::new();

    let mut total_bytes = 0u64;
    let mut max_size = 0usize;
    let mut min_size = usize::MAX;

    println!("Listening on {}", socket.local_addr()?);

    let mut buffer = [0u8; 2048];

    loop {
        let (bytes_recieved, src_addr) = socket.recv_from(&mut buffer)?;

        total_packets += 1;
        total_bytes += bytes_recieved as u64;

        max_size = max_size.max(bytes_recieved);
        min_size = min_size.min(bytes_recieved);

        let avg_size = total_bytes as f64 / total_packets as f64;
        *source_counts.entry(src_addr.ip()).or_insert(0) += 1;

        println!(
            "Packet from {} | size={} | avg={:.2}",
            src_addr, bytes_recieved, avg_size
        );
    }
}
