use std::net::TcpStream;
use std::io::{Read, Write};
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    publish::PublishPacket,
    puback::PubAckPacket,
};

/// Sends a CONNECT packet to the MQTT server.
/// The CONNECT packet initiates the communication by providing client credentials and settings.
fn send_connect_packet(mut stream: TcpStream) {
    // Create the CONNECT packet with necessary details
    let connect_packet = ConnectPacket::new(
        "MQTT".to_string(),  // Protocol name
        5,                  // Protocol level (5 for MQTT)
        0b00000010,         // Flags (Clean Session enabled)
        60,                 // Keep Alive (in seconds)
        "client1".to_string(), // Client id
        None,               // Optional Will Topic
        None,               // Optional Will Message
        Some("user".to_string()), // Optional Username
        Some("password".to_string()), // Optional Password
    );

    // Encode the CONNECT packet into bytes for transmission
    let packet = connect_packet.encode();

    // Send the CONNECT packet to the server
    match stream.write(&packet) {
        Ok(_) => println!("CONNECT packet sent: {:?}", connect_packet),
        Err(e) => eprintln!("Failed to send CONNECT: {}", e),
    }
}

/// Receives and decodes a CONNACK packet from the server.
/// The CONNACK packet confirms whether the connection was successful or not.
fn receive_connack_packet(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    // Read the server's response, expecting a CONNACK packet
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Decode the CONNACK packet
            match ConnAckPacket::decode(&buffer[0..size]) {
                Ok(connack_packet) => {
                    println!("Received CONNACK packet: {:?}", connack_packet);
                }
                Err(e) => eprintln!("Failed to decode CONNACK: {}", e),
            }
        }
        Ok(_) => eprintln!("Empty package received"),
        Err(e) => eprintln!("Error reading the stream: {}", e),
    }
}

/// Sends a PUBLISH packet to the server.
/// The PUBLISH packet is used to send messages to other clients.
fn send_publish_packet(mut stream: TcpStream, topic: &str, message: &str) {
    // Create the PUBLISH packet with the provided topic and message
    let publish_packet = PublishPacket::new(
        topic.to_string(),         // Topic
        Some(1),                   // Message ID (Optional)
        0,                         // QoS level 0 (At most once)
        false,                     // Retain flag (not retained)
        false,                     // DUP flag (not a duplicate)
        message.as_bytes().to_vec(), // Payload (message content)
    );

    // Encode the PUBLISH packet into bytes for transmission
    let packet = publish_packet.encode();

    // Send the PUBLISH packet to the server
    match stream.write(&packet) {
        Ok(_) => println!("PUBLISH packet sent: {:?}", publish_packet),
        Err(e) => eprintln!("Failed to send PUBLISH: {}", e),
    }
}

/// Receives and decodes a PUBACK packet from the server.
/// The PUBACK packet acknowledges the receipt of a message with QoS 1.
fn receive_puback_packet(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    // Read the server's response, expecting a PUBACK packet
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Decode the PUBACK packet
            match PubAckPacket::decode(&buffer[0..size]) {
                Ok(puback_packet) => {
                    println!("Received PUBACK packet: {:?}", puback_packet);
                }
                Err(e) => eprintln!("Failed to decode PUBACK: {}", e),
            }
        }
        Ok(_) => eprintln!("Empty package received"),
        Err(e) => eprintln!("Error reading the stream: {}", e),
    }
}

/// Establishes a connection with the MQTT server and handles CONNECT, PUBLISH, and PUBACK packets.
fn start_client() {
    // Connect to the MQTT server at localhost on port 1883
    match TcpStream::connect("127.0.0.1:1883") {
        Ok(stream) => {
            println!("Connected to MQTT server at 127.0.0.1:1883");

            // Send the connect package via the stream
            send_connect_packet(stream.try_clone().expect("Error cloning the stream"));

            // Receive the response (CONNACK)
            receive_connack_packet(stream.try_clone().expect("Error cloning the stream"));

            // Send a PUBLISH packet
            send_publish_packet(stream.try_clone().expect("Error cloning the stream"), "test/topic", "Hello MQTT!");

            // Receive the PUBACK packet in response
            receive_puback_packet(stream);
        }
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}

/// Entry point for the MQTT client.
/// Calls the start_client function to begin communication.
fn main() {
    start_client();
}