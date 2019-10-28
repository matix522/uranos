# RaspberryPi 3/4 OS
## To run
1) Clone this repo
2) Install Rust and execute this commands in main catalogue of repo 
    - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
    - `rustup default nightly-2019-10-04`
    - `rustup component add rust-src llvm-tools-preview clippy rustfmt`
    - `cargo install cargo-xbuild cargo-binutils`

3) Compile by use of `make` or run in qemu by `make qemu` provided you have Docker installed.



## TODO LIST 
Each group Ranked in inportance from top to bottom.
### Bugfixes 
 - [x] Task 0 is never scheduled
 - [x] If scheduling and interupt takse too long system hangs in next interupt.
 - [ ] Wrong resolution displaying when set to 1920x1080.
### Features 
 - [ ] Syscall interface ORWC: As an example for uart.
 - [ ] Charbuffer for hdmi output.
### Code Quality 
 - [ ] Unify interface of IntControler for RPI3 and RPI4. 
 - [ ] Unify interface for PL011 Uart and Aux Uart.
 - [ ] Create Device tree for physical and virtual devices. 