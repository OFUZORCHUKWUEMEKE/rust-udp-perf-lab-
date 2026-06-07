use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let target = "127.0.0.1:9000";
    let threads = 4;

    println!(
        "Starting load generator with {} thread blasin {}",
        threads, target
    );

    let mut handles = vec![];

    for id in 0..threads {
        let handle = thread::spawn(move || {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind");

            // A small dummy payload (64 bytes of zeroes)
            let payload = [0u8; 64];
            println!("Thread {} started sending...", id);
            loop {
                // Ignore errors (like the receiver buffer getting full and dropping packets)
                let _ = socket.send_to(&payload, target);
            }
        });
        handles.push(handle);
    }

    // Keep the main thread alive while the spawned threads do the work
    for handle in handles {
        handle.join().unwrap();
    }
    Ok(())
}
