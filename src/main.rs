use std::collections::HashMap;
use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;

    let mut total_packets = 0u64;
    let mut source_counts = HashMap::new();

    println!("Listening on {}", socket.local_addr()?);

    let mut buffer = [0u8; 2048];

    loop {
        total_packets += 1;

        let (_bytes_recieved, src_addr) = socket.recv_from(&mut buffer)?;
        *source_counts.entry(src_addr.ip()).or_insert(0) += 1;
        println!("Total packets : {}", total_packets);
        println!("Unique sources : {}", source_counts.len());
    }
}
