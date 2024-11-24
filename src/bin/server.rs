use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::{ConnAckPacket, ConnAckReasonCode},
    publish::PublishPacket,  // Import for handling PUBLISH packets
    puback::PubAckPacket,    // Import for sending PUBACK packets
};

/// Handles communication with a connected MQTT client.
/// Processes CONNECT, PUBLISH, and PUBACK packet flows.
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    // Read data from the client (expecting a CONNECT packet first)
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Attempt to decode the CONNECT packet
            match ConnectPacket::decode(&buffer[0..size]) {
                Ok(connect_packet) => {
                    println!("Received CONNECT packet: {:?}", connect_packet);

                    // Create a CONNACK packet as a response
                    let connack_packet = ConnAckPacket {
                        session_present: false,
                        reason_code: ConnAckReasonCode::Success,
                        properties: None,
                    };

                    // Encode the CONNACK packet
                    let response = connack_packet.encode();

                    // Send the CONNACK packet back to the client
                    match stream.write(&response) {
                        Ok(_) => println!("Enviado paquete CONNACK: {:?}", connack_packet),
                        Err(e) => eprintln!("Error al enviar CONNACK: {}", e),
            }
                },
                Err(e) => eprintln!("Error al decodificar CONNECT: {}", e),
                        }
        },
        Ok(_) => eprintln!("Recibido paquete vacío"),
        Err(e) => eprintln!("Error al leer del stream: {}", e),
    }
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:1883").expect("Error al iniciar el servidor");
    println!("Servidor MQTT en 127.0.0.1:1883");

    // Accept and handle incoming client connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Client connected: {:?}", stream.peer_addr());
                handle_client(stream);
            },
            Err(e) => eprintln!("Error al aceptar conexión: {}", e),
        }
    }
}

/// Entry point for the MQTT server application.
/// Calls the `start_server` function to begin listening for client connections.
fn main() {
    start_server();
}
