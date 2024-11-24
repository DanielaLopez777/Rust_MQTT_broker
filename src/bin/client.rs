use std::net::TcpStream;
use std::io::{Read, Write};
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    publish::PublishPacket,  // Import for PUBLISH packets
    puback::PubAckPacket,    // Import for PUBACK packets
};

/// Sends a CONNECT packet to the MQTT server.
/// The CONNECT packet initiates the communication by providing client credentials and settings.
fn send_connect_packet(mut stream: TcpStream) {
    // Create the CONNECT packet with necessary details
    let connect_packet = ConnectPacket::new(
        "MQTT".to_string(),  // Nombre del protocolo
        5,                  // Nivel del protocolo
        0b00000010,         // Flags (Clean Session habilitado)
        60,                 // Keep Alive (en segundos)
        "client1".to_string(), // ID del cliente
        None,               // Will Topic
        None,               // Will Message
        Some("user".to_string()), // Username (opcional)
        Some("password".to_string()), // Password (opcional)
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
        Ok(_) => eprintln!("Recibido paquete vacío"),
        Err(e) => eprintln!("Error al leer del stream: {}", e),
    }
}

/// Establishes a connection with the MQTT server and handles CONNECT, PUBLISH, and PUBACK packets.
fn start_client() {
    // Connect to the MQTT server at localhost on port 1883
    match TcpStream::connect("127.0.0.1:1883") {
        Ok(stream) => {
            println!("Connected to MQTT server at 127.0.0.1:1883");

            // Enviar paquete CONNECT
            send_connect_packet(stream.try_clone().expect("Error al clonar stream"));

            // Recibir y procesar el paquete CONNACK
            receive_connack_packet(stream);
        }
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}

/// Entry point for the MQTT client.
/// Calls the `start_client` function to begin communication.
fn main() {
    start_client();
}