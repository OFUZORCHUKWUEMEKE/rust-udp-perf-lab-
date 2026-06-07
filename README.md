# UDP Throughput Benchmarker & Payload Analyzer 🚀

A high-performance systems experimentation repository demonstrating network optimization, built entirely in Rust. 

The scope of this project is to benchmark the extreme overhead of operating system context switching by comparing the standard Linux network stack (`recv_from`) against batched system calls (`recvmmsg`) using native `libc` bindings—with future plans for total kernel bypass using eBPF/XDP.

## Architecture

This crate contains two distinct multi-threaded binaries:

### 1. The Load Generator (`load_generator.rs`)
A dedicated benchmarking tool that spins up an arbitrary number of threads and binds random unprivileged host ports. It overrides blocking backpressure to aggressively saturate the loopback interface (`127.0.0.1`) with 64-byte or larger dummy UDP payload flooding, creating millions of packets per second of synthetic network pressure.

### 2. The Multi-Mode Receiver (`receiver.rs`)
The UDP Receiver runs in two dynamically selectable CLI modes through `clap`, measuring absolute **Packets Per Second (PPS)** through an atomic metrics thread that calculates absolute throughput deltas outside of the hot-receive loop.

#### 🐌 Mode A: Standard Ingestion (`--mode slow`)
The receiver uses standard `std::net::UdpSocket` polling. For every individual packet received, the Linux Kernel must perform a context switch out of User-Space to fetch the network buffer. This demonstrates the CPU exhaustion bottleneck caused by high-frequency context switching.

#### ⚡ Mode B: Batched Systems Calls (`--mode fast`)
The receiver leverages `libc::recvmmsg` ("Receive Multiple Messages"). It allocates a massive `1024` length array of contiguous `mmsghdr` structs, mapping their `iovec` memory slices directly into the Rust stack. It drops into a single unsafe C-binding system call to the Linux kernel, returning with up to 1,024 packets at once.
*Performance gains: Eliminates ~99.9% of context switches out of User Space.*

---

## 🛠️ Getting Started

### Prerequisites
- **Linux Environment** (macOS/Windows do not natively support `recvmmsg`)
- Rust `1.70+`

### Installation & Usage
Clone the repository and compile using `--release` for maximum throughput metrics.

First, spin up the receiver in Fast mode:
```bash
cargo run --release --bin receiver -- --mode fast
```

Next, open a second terminal and begin the synthetic load generation:
```bash
cargo run --release --bin load_generator
```

Observe exactly how the `recvmmsg` batch mapping exponentially outpaces standard `recv_from` packet ingestion.

---

## 🛣️ Roadmap

- **Stage 5: eBPF & eXpress Data Path (XDP)**
  - Integrate the `aya` crate to deploy 100% Rust-based eBPF bytecode directly into the Linux NIC driver.
  - Mitigate packets in Ring-0 before the Linux network stack allocates `sk_buff` structs, reaching absolute maximum theoretical throughput.
  - Use eBPF maps to pass metrics to the User-Space monitor.
