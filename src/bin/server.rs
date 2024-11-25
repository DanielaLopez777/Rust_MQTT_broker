use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::{ConnAckPacket, ConnAckReasonCode},
    publish::PublishPacket,  
    puback::PubAckPacket,    
};

/// Handles communication with a connected MQTT client an all its processes.
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    // Read data from the client (expecting a CONNECT packet first)
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Attempt to decode the CONNECT packet
            match ConnectPacket::decode(&buffer[0..size]) {
                Ok(connect_packet) => {
                    println!("Received CONNECT packet: {:?}\n\n", connect_packet);

                    // Create a CONNACK packet as a response
                    let connack_packet = ConnAckPacket::new( 
                        session_present: false,
                        reason_code: ConnAckReasonCode::Success,
                        properties: None,
                    );

                    // Encode the CONNACK packet
                    let response = connack_packet.encode();

                    // Send the CONNACK packet back to the client
                    match stream.write(&response) {
                        Ok(_) => println!("Sent CONNACK package: {:?}", connack_packet),
                        Err(e) => eprintln!("Error sending the CONNACK package: {}", e),
            }
                },
                Err(e) => eprintln!("Error decoding CONNECT: {}", e),
                        }
        },
        Ok(_) => eprintln!("Empty package received"),
        Err(e) => eprintln!("Error reading the stream: {}", e),
    }
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:1883").expect("Error starting the server");
    println!("Servidor MQTT en 127.0.0.1:1883");

    // Accept and handle incoming client connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected: {:?}", stream.peer_addr());
                handle_client(stream);
            },
            Err(e) => eprintln!("Error accepting connection: {}", e),
        }
    }
}

/// Entry point for the MQTT server application.
/// Calls the `start_server` function to begin listening for client connections.
fn main() {
    start_server();
}
