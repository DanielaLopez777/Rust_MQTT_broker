use std::sync::{Arc, Mutex}; // Provides thread-safe sharing of data between threads
use std::net::{TcpListener, TcpStream}; // Provides TCP networking capabilities
use std::thread; // Provides threading utilities for concurrent execution
use std::io::{Read, Write}; // Provides I/O traits for reading and writing
use mqtt_broker::packets::{
    connect::ConnectPacket, // For handling MQTT CONNECT packets
    connack::{ConnAckPacket, ConnAckReasonCode}, // For creating CONNACK response packets
    publish::PublishPacket, // For handling MQTT PUBLISH packets
    puback::PubAckPacket,
    subscribe::SubscribePacket,
    suback::SubAckPacket,
};

// Function to handle each connected client
fn handle_client(stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) 
{
    let mut stream = stream; // Make the TcpStream mutable to read/write data
    let mut buffer = [0u8; 1024]; // Buffer to store incoming data

    // Initial read to check for a CONNECT packet from the client
    match stream.read(&mut buffer) 
    {
        Ok(size) if size > 0 => 
        {
            // Decode the received data as a CONNECT packet
            match ConnectPacket::decode(&buffer[0..size]) 
            {
                Ok(connect_packet) => 
                {
                    println!("Received CONNECT packet: {:?}\n", connect_packet);

                    // Create a CONNACK packet as a response
                    let connack_packet = ConnAckPacket::new(
                        false, // Session Present flag
                        ConnAckReasonCode::Success, // Success response code
                        None, // Optional properties (none in this case)
                    );

                    let response = connack_packet.encode(); // Encode the CONNACK packet

                    // Send the CONNACK packet back to the client
                    match stream.write(&response) 
                    {
                        Ok(_) => println!("Sent CONNACK package: {:?}\n", connack_packet),
                        Err(e) => eprintln!("Error sending the CONNACK package: {}\n\n", e),
                    }
                }
                Err(e) => eprintln!("Error decoding CONNECT: {}\n", e), // Log decoding errors
            }
        }
        Ok(_) => println!("Client disconnected: {:?}\n", stream.peer_addr()), // Handle empty read (disconnection)
        Err(e) => println!("Error reading from stream: {}\n", e), // Log reading errors
    }

    // Enter a loop to continuously read packets from the client
    loop 
    {
        match stream.read(&mut buffer) 
        {
            Ok(size) if size > 0 => 
            {
                // Decode the data as a PUBLISH packet
                if let Ok(packet) = PublishPacket::decode(&buffer[..size]) 
                {
                    println!("Received PUBLISH packet: {:?}\n", packet);
                    
                    // Send PUBACK packet
                    let puback_packet = PubAckPacket::new(packet.message_id);
                    let puback_response = puback_packet.encode();
                    match stream.write(&puback_response) {
                        Ok(_) => println!("Sent PUBACK packet for message ID: {}\n", packet.message_id),
                        Err(e) => eprintln!("Error sending PUBACK packet: {}\n", e),
                    }
                    
                    let encoded_packet = packet.encode(); // Encode the packet for broadcasting
                    let clients_guard = clients.lock().unwrap(); // Lock the shared client list

                    // Send the packet to all connected clients except the sender
                    for mut client in clients_guard.iter() 
                    {
                        if client.peer_addr().unwrap() != stream.peer_addr().unwrap() 
                        {
                            let _ = client.write(&encoded_packet);
                        }
                    }
                } 
                if let Ok(packet) = SubscribePacket::decode(&buffer[..size]) 
                {
                    println!("Received SUBSCRIBE packet: {:?}\n", packet);
                    
                    // Prepare return codes for the subscription
                    let return_codes: Vec<u8> = packet
                        .qos_values
                        .iter()
                        .map(|&qos| {
                            if qos <= 2 {
                                qos // Grant requested QoS if valid (0, 1, 2)
                            } else {
                                0x80 // Return 0x80 for invalid QoS values
                            }
                        })
                        .collect();

                    // Create a SUBACK packet as a response
                    let suback_packet = SubAckPacket {
                        packet_id: packet.packet_id, // Echo the packet_id from the SUBSCRIBE packet
                        return_codes,                // Use the computed return codes
                    };

                    // Encode the SUBACK packet (assume an `encode` method exists)
                    let suback_response = suback_packet.encode(); 

                    // Send the SUBACK packet back to the client
                    match stream.write(&suback_response) 
                    {
                        Ok(_) => println!("Sent SUBACK : {:?}\n", suback_response),
                        Err(e) => eprintln!("Error sending SUBACK packet: {}\n", e),
                    }
                }
            }
            Ok(_) => 
            {
                println!("Client disconnected: {:?}\n", stream.peer_addr()); // Handle client disconnection
                break;
            }
            Err(e) => 
            {
                println!("Error reading from stream: {}\n", e); // Log reading errors
                break;
            }
        }
    }

    // Remove the disconnected client from the shared client list
    let mut clients_guard = clients.lock().unwrap();
    if let Some(pos) = clients_guard.iter().position(|x| 
        {
        match x.peer_addr()
         {
            Ok(addr) => addr == stream.peer_addr().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap()), // Fallback to default address if error
            Err(_) => false, // Ignore if peer address retrieval fails
        }
    }) 
    {
        clients_guard.remove(pos);
    }
}

// Function to start the MQTT server
fn start_server() 
{
    // Bind the server to a local address and port
    let listener = TcpListener::bind("127.0.0.1:1883").expect("Error starting the server"); 
    println!("\nMQTT server started on 127.0.0.1:1883\n");

    // Shared list of connected clients
    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new())); 

    // Accept incoming connections in a loop
    for stream in listener.incoming() 
    {
        match stream 
        {
            Ok(stream) => 
            {
                println!("Client connected: {:?}\n", stream.peer_addr());

                // Lock the client list for modification
                let mut clients_guard = clients.lock().unwrap(); 
                // Add the new client to the list
                clients_guard.push(stream.try_clone().unwrap()); 

                // Create a clone of the client list for the new thread
                let clients_clone = Arc::clone(&clients); 
                thread::spawn(move || {
                    // Handle the client in a separate thread
                    handle_client(stream, clients_clone); 
                });
            }
            Err(e) => 
            {
                println!("Error accepting connection: {}\n", e); // Log errors during connection acceptance
            }
        }
    }
}

// Entry point of the application
fn main() {
    start_server(); // Start the MQTT server
}
