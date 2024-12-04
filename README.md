# MQTT Rust broker and client

An MQTT broker that handles simultaneous connections through the use of threads, enabling efficient concurrent processing. The client component is designed with concurrency capabilities, allowing it to listen for incoming packets, send pings, and process user-initiated actions. The implementation adheres to MQTT version 5.0, supporting the CONNECT, CONNACK, SUBSCRIBE, SUBACK, PINGREQ, PINGRESP, and DISCONNECT packets. Due to the steep learning curve of Rust and project time constraints, the remaining packet types were not implemented.

To see the whole development process, visit:

https://github.com/DanielaLopez777/Rust_MQTT_broker.git

## Download Rust in a Linux environment

1. Update the package list and their versions:

    `sudo apt update`

2. Install essential development tools (curl, C/C++ compiler, and build utilities) needed for some Rust dependencies:

    `sudo apt install curl gcc make build-essential -y`


3. Download and install Rust via rustup, Rust’s official toolchain installer and choose the option 1 to proceed with standard installation:

    `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

4. Set up the environment variables needed for Rust and cargo to work correctly in the current shell session, allowing to use Rust and Cargo commands in the terminal:

    `source $HOME/.cargo/env`


5. Verify rust and cargo varsions

    `rustc --version`

    `cargo --version`


6. Update all the associated Rust components:

    `rustup update`


7. Install git in Ubuntu

    `sudo apt-get install git`

## Running the code

removes build artifacts to free space or force a fresh build:

`cargo clean`

For compiling the code use:

`cargo build`

For running the server:

`cargo run --bin server`

For running the client:

`cargo run --bin client`

## References a further information

For more documentation about the internal structure of the project type in a terminal:

`cargo doc --open`

This code was base in the following previous MQTT project implemented in C made at the college:

https://github.com/DanielaLopez777/MQTT_2023_C.git

The errors presented during the development were solved with the help of chatgpt and for syntax support the following link was consulted:

https://www.rust-lang.org/learn

For packets implementation the following MQTT version 5.0 was consulted:

https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.pdf

