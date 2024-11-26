use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::{ConnAckPacket, ConnAckReasonCode},
    publish::PublishPacket,
};

fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    let mut stream = stream;
    let mut buffer = [0u8; 1024];

    match stream.read(&mut buffer) 
    {
        Ok(size) if size > 0 => 
        {
            match ConnectPacket::decode(&buffer[0..size]) 
            {
                Ok(connect_packet) => 
                {
                    println!("Received CONNECT packet: {:?}\n\n", connect_packet);

                    let connack_packet = ConnAckPacket::new(
                        false,
                        ConnAckReasonCode::Success,
                        None,
                    );

                    let response = connack_packet.encode();

                    match stream.write(&response) 
                    {
                        Ok(_) => println!("Sent CONNACK package: {:?}\n\n", connack_packet),
                        Err(e) => eprintln!("Error sending the CONNACK package: {}\n\n", e),
                    }
                }
                Err(e) => eprintln!("Error decoding CONNECT: {}\n\n", e),
            }
        }
        Ok(_) => println!("Cliente desconectado: {:?}", stream.peer_addr()),

        Err(e) => println!("Error al leer del stream: {}", e),
    }

    loop 
    {
        match stream.read(&mut buffer) 
        {
            Ok(size) if size > 0 => 
            {
                if let Ok(packet) = PublishPacket::decode(&buffer[..size]) 
                {
                    println!("Recibido paquete PUBLISH: {:?}", packet);

                    let encoded_packet = packet.encode();
                    let clients_guard = clients.lock().unwrap();
                    for mut client in clients_guard.iter() 
                    { 
                        if client.peer_addr().unwrap() != stream.peer_addr().unwrap() 
                        {
                            let _ = client.write(&encoded_packet);
                        }
                    }
                }
            }
            Ok(_) => 
            {
                println!("Cliente desconectado: {:?}", stream.peer_addr());
                break;
            }
            Err(e) => 
            {
                println!("Error al leer del stream: {}", e);
                break;
            }
        }
    }

    let mut clients_guard = clients.lock().unwrap();
    if let Some(pos) = clients_guard.iter().position(|x| {
        match x.peer_addr() {
            Ok(addr) => addr == stream.peer_addr().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()), // Reemplazar con dirección predeterminada si hay error
            Err(_) => false, // Si hay un error, no se puede comparar
        }
    }) {
        clients_guard.remove(pos);
    }
}


fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:1883").expect("Error iniciando el servidor");
    println!("Servidor MQTT iniciado en 127.0.0.1:1883");

    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Cliente conectado: {:?}", stream.peer_addr());

                let mut clients_guard = clients.lock().unwrap();
                clients_guard.push(stream.try_clone().unwrap());

                let clients_clone = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(stream, clients_clone);
                });
            }
            Err(e) => {
                println!("Error aceptando conexión: {}", e);
            }
        }
    }
}

fn main() {
    start_server();
}