# rust-udp-perf-lab

A high-performance Rust lab for benchmarking UDP packet ingress. Compare standard single-packet receives (`recv_from`) against batched Linux syscalls (`recvmmsg`) under synthetic load—with future plans for kernel bypass via eBPF/XDP.

## Architecture

This crate contains two multi-threaded binaries:

### 1. Load Generator (`load_generator.rs`)

Spins up multiple threads, each binding a random local port, and floods `127.0.0.1:9000` with 64-byte UDP payloads. Used to saturate the receiver and measure throughput under pressure.

### 2. Multi-Mode Receiver (`receiver.rs`)

Listens on port `9000` and reports **packets per second (PPS)** via a background metrics thread. Mode is selectable from the CLI with `clap`.

#### Slow mode (`--mode slow`)

Uses `std::net::UdpSocket::recv_from` — one syscall per packet. Simple baseline that shows the cost of frequent user/kernel transitions at high packet rates.

#### Fast mode (`--mode fast`)

Uses `libc::recvmmsg` to receive up to **1024 packets per syscall**. Pre-allocates buffers and sets up `iovec` / `mmsghdr` structs so the kernel writes directly into Rust memory.

---

## Getting Started

### Prerequisites

- **Linux** (`recvmmsg` is not available on macOS/Windows)
- Rust 1.70+

### Run a benchmark

Clone the repo and build in release mode for meaningful throughput numbers:

```bash
git clone https://github.com/<your-username>/rust-udp-perf-lab.git
cd rust-udp-perf-lab
```

**Terminal 1** — start the receiver (try both modes):

```bash
cargo run --release --bin receiver -- --mode slow
cargo run --release --bin receiver -- --mode fast
```

**Terminal 2** — start the load generator:

```bash
cargo run --release --bin load_generator
```

Compare PPS between slow and fast mode while the generator is running.

---

## Roadmap

- **eBPF / XDP integration**
  - Deploy Rust eBPF programs (e.g. via `aya`) for early packet handling in the kernel
  - Reduce `sk_buff` allocation overhead on the standard network stack path
  - Export metrics from eBPF maps to user-space
