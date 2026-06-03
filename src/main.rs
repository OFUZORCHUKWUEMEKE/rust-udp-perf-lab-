use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:9000")?;
    println!("Listening on {}", socket.local_addr()?);

    let mut buffer = [0u8; 2048];

    loop {
        let (bytes_recieved, src_addr) = socket.recv_from(&mut buffer)?;
        println!("Recieved {} bytes from {}", bytes_recieved, src_addr);
    }
}
