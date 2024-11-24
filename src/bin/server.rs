use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use mqtt_broker::packets::{connect::ConnectPacket,
    connack::ConnAckPacket, connack::ConnAckReasonCode};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024];
    
    // Leer datos del cliente (paquete CONNECT)
    match stream.read(&mut buffer) {
        Ok(size) if size > 0 => {
            // Descodificar el paquete CONNECT
            match ConnectPacket::decode(&buffer[0..size]) {
                Ok(connect_packet) => {
                    println!("Recibido paquete CONNECT: {:?}", connect_packet);
                    
                    // Crear el paquete CONNACK de respuesta
                    let connack_packet = ConnAckPacket {
                        session_present: false,
                        reason_code: ConnAckReasonCode::Success,
                        properties: None,
                    };

                    // Codificar el paquete CONNACK
                    let response = connack_packet.encode();

                    // Enviar el paquete CONNACK al cliente
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

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Cliente conectado: {:?}", stream.peer_addr());
                handle_client(stream);
            },
            Err(e) => eprintln!("Error al aceptar conexión: {}", e),
        }
    }
}

fn main() {
    start_server();
}
