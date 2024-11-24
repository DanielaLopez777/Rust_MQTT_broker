use std::net::{TcpStream};
use std::io::{Read, Write};
use mqtt_broker::packets::{connect::ConnectPacket,
connack::ConnAckPacket};

fn send_connect_packet(mut stream: TcpStream) {
    // Crear el paquete CONNECT
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

    // Codificar el paquete CONNECT
    let packet = connect_packet.encode();

    // Enviar el paquete CONNECT al servidor
    match stream.write(&packet) {
        Ok(_) => println!("Paquete CONNECT enviado: {:?}", connect_packet),
        Err(e) => eprintln!("Error al enviar CONNECT: {}", e),
    }
}

fn receive_connack_packet(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];

    // Leer respuesta del servidor (paquete CONNACK)
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Decodificar el paquete CONNACK
            match ConnAckPacket::decode(&buffer[0..size]) {
                Ok(connack_packet) => {
                    println!("Recibido paquete CONNACK: {:?}", connack_packet);
                }
                Err(e) => eprintln!("Error al decodificar CONNACK: {}", e),
            }
        }
        Ok(_) => eprintln!("Recibido paquete vacío"),
        Err(e) => eprintln!("Error al leer del stream: {}", e),
    }
}

fn start_client() {
    match TcpStream::connect("127.0.0.1:1883") {
        Ok(stream) => {
            println!("Conectado al servidor MQTT en 127.0.0.1:1883");

            // Enviar paquete CONNECT
            send_connect_packet(stream.try_clone().expect("Error al clonar stream"));

            // Recibir y procesar el paquete CONNACK
            receive_connack_packet(stream);
        }
        Err(e) => eprintln!("No se pudo conectar al servidor: {}", e),
    }
}

fn main() {
    start_client();
}