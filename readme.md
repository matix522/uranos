# UranOS
## Unified resource allocator, not Operating System
## To run
1) Clone this repo
2) Install Rust and execute this commands in main catalogue of repo 
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - `rustup default nightly-2020-06-07`
    - `rustup component add rust-src llvm-tools-preview clippy rustfmt`
    - `cargo install cargo-xbuild cargo-binutils`
3) You need docker installed and configured

4) Compile by use of `make` or run in qemu by `make qemu` provided you have Docker installed.
