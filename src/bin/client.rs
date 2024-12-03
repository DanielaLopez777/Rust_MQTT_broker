use std::net::TcpStream;
use std::io::{Read, Write};
use std::io::{self};
use std::thread;
use std::time::{Duration, Instant};
use bytes::Bytes;
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    publish::PublishPacket,
    puback::PubAckPacket,
    subscribe::SubscribePacket, 
    suback::SubAckPacket, 
    ping::{PingRespPacket, PingReqPacket}
};

/// Sends a CONNECT packet to the MQTT server.
/// The CONNECT packet initiates the communication by providing client credentials and settings.
fn send_connect_packet(mut stream: TcpStream) 
{
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
fn receive_connack_packet(mut stream: TcpStream) 
{
    let mut buffer = [0u8; 1024];

    // Read the server's response, expecting a CONNACK packet
    match stream.read(&mut buffer) 
    {
        Ok(size) if size > 0 => 
        {
            // Decode the CONNACK packet
            match ConnAckPacket::decode(&buffer[0..size]) 
            {
                Ok(connack_packet) => 
                {
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
fn send_publish_packet(mut stream: TcpStream, topic: &str, message: &str) 
{
    // Create the PUBLISH packet with the provided topic and message
    let publish_packet = PublishPacket::new(
        topic.to_string(),         // Topic
        1,                   // Message ID (Optional)
        1,                         // QoS level
        false,                     // Retain flag (not retained)
        false,                     // DUP flag (not a duplicate)
        message.as_bytes().to_vec(), // Payload (message content)
    );

    // Encode the PUBLISH packet into bytes for transmission
    let packet = publish_packet.encode();

    // Send the PUBLISH packet to the server
    match stream.write(&packet) 
    {
        Ok(_) => println!("PUBLISH packet sent: {:?}", publish_packet),
        Err(e) => eprintln!("Failed to send PUBLISH: {}", e),
    }
}

/// Sends a SUBSCRIBE packet to the server.
/// The SUBSCRIBE packet allows the client to subscribe to topics.
fn send_subscribe_packet(mut stream: TcpStream, packet_id: u16, topic: &str) {
    // Predefined QoS values (you can adjust this as needed)
    let qos_values = vec![1];

    // Create the SUBSCRIBE packet
    let subscribe_packet = SubscribePacket::new(packet_id, vec![topic.to_string()], qos_values);

    // Encode the SUBSCRIBE packet into bytes for transmission
    let packet = subscribe_packet.encode();

    // Send the SUBSCRIBE packet to the server
    match stream.write(&packet) {
        Ok(_) => println!("SUBSCRIBE packet sent: {:?}", subscribe_packet),
        Err(e) => eprintln!("Failed to send SUBSCRIBE: {}", e),
    }
}

/// Displays the menu options and handles user input for actions.
fn display_menu() -> u8 {
    println!("Please select an option:");
    println!("1. Publish");
    println!("2. Subscribe");
    println!("3. Disconnect");

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).expect("Failed to read line");

    choice.trim().parse().unwrap_or(0) // Default to 0 if invalid input
}

fn packets_listener(mut stream: TcpStream) {
    let mut buffer = [0u8; 1024]; // Buffer to store incoming data
    //Starting ping time
    let mut last_ping_time = Instant::now();
    let mut pending_ping = false; // Flag to track if we are waiting for PINGRESP
    
    loop {
        // Send PINGREQ every 60 seconds if no other packets are being processed
        if last_ping_time.elapsed() > Duration::from_secs(60) && !pending_ping {
            let pingreq_packet = PingReqPacket;
            let pingreq_response = pingreq_packet.encode();
            match stream.write(&pingreq_response) {
                Ok(_) => {
                    println!("Sent PINGREQ");
                    pending_ping = true; // Mark that we're waiting for a PINGRESP
                },
                Err(e) => eprintln!("Error sending PINGREQ packet: {}\n", e),
            }
            last_ping_time = Instant::now();
        }

        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                // Determine packet type (for demonstration; replace with actual packet identification logic)
                let packet_type = buffer[0] >> 4; // MQTT packet type is in the top 4 bits of the first byte.

                match packet_type 
                {
                    3 => {
                        // PUBLISH packet
                        if let Ok(packet) = PublishPacket::decode(&buffer[..size]) {
                            let bytes = packet.payload;
                            let reconstructed_message = String::from_utf8(bytes).expect("Error al convertir bytes a string");
                            println!("Received PUBLISH packet from {:?} topic: {:?}\n", packet.topic_name, reconstructed_message);
                        }
                    }
                    4 => {
                        // PUBACK packet
                        if let Ok(packet) = PubAckPacket::decode(&buffer[..size]) {
                            println!("Received PUBACK packet: {:?}\n", packet);
                        }
                    }
                    9 => {
                        // SUBACK packet
                        if let Ok(packet) = SubAckPacket::decode(&buffer[..size]) {
                            println!("Received SUBACK packet: {:?}\n", packet);
                        }
                    }
                    13 =>
                    {
                        // Handle PINGRESP packet
                        if let Ok(_) = PingRespPacket::decode(&Bytes::copy_from_slice(&buffer[..size])) {
                            println!("Received valid PINGRESP packet.");
                            pending_ping = false; // Reset pending_ping flag after receiving PINGRESP
                        } else {
                            eprintln!("Invalid PINGRESP packet.\n");
                        }
                    }
                    _ => {
                        // Handle other packet types or log them
                        println!("Unhandled packet type: {}\n", packet_type);
                    }
                }
                if last_ping_time.elapsed() > Duration::from_secs(60) 
                {
                    println!("No PINGREQ received for over 60 seconds. Closing connection.");
                    break;
                }
            }

            Ok(_) => {
                println!("Client disconnected: {:?}\n", stream.peer_addr()); // Handle client disconnection
                break;
            }
            Err(e) => {
                println!("Error reading from stream: {}\n", e); // Log reading errors
                break;
            }
        }
    }
}

fn start_client() 
{
    // Connect to the MQTT server at localhost on port 1883
    match TcpStream::connect("127.0.0.1:1883") {
        Ok(mut stream) => 
        {
            println!("Connected to MQTT server at 127.0.0.1:1883");

            // Send the connect package via the stream
            send_connect_packet(stream.try_clone().expect("Error cloning the stream"));

            // Receive the response (CONNACK)
            receive_connack_packet(stream.try_clone().expect("Error cloning the stream"));

            // Start a background thread for listening to publications
            let listener_stream = stream.try_clone().expect("Error cloning the stream");
            
            thread::spawn(move || {
                packets_listener(listener_stream);
            });

            // Menu for user actions
            loop {
                let choice = display_menu();

                match choice {
                    1 => {
                        // Option 1: Publish message
                        let topic = "topic/1";
                        let message = "Hello MQTT!";
                        send_publish_packet(stream.try_clone().expect("Error cloning the stream"), topic, message);
                        thread::sleep(Duration::from_millis(100));
                    }
                    2 => {
                        // Option 2: Subscribe to a topic
                        let topics = vec!["topic/1", "topic/2", "topic/3"]; // Predefined topics
                        println!("Select a topic to subscribe to:");
                        for (index, topic) in topics.iter().enumerate() {
                            println!("{}: {}", index + 1, topic);
                        }
                    
                        let mut topic_choice = String::new();
                        io::stdin().read_line(&mut topic_choice).expect("Failed to read line");
                    
                        let topic_choice: usize = topic_choice.trim().parse().unwrap_or(0);
                        if topic_choice > 0 && topic_choice <= topics.len() {
                            let selected_topic = topics[topic_choice - 1];
                            send_subscribe_packet(stream.try_clone().expect("Error cloning the stream"), 1, selected_topic); // Packet ID set to 1 for this example
                            thread::sleep(Duration::from_millis(100));
                        } else {
                            println!("Invalid selection.");
                        }
                    }
                    3 => {
                        // Option 3: Disconnect (exit the loop)
                        println!("Disconnecting...");
                        break;
                    }
                    _ => {
                        println!("Invalid selection. Please try again.");
                    }
                }
            }
        }
        Err(e) => eprintln!("Failed to connect to server: {}", e),
    }
}

/// Entry point for the MQTT client.
/// Calls the start_client function to begin communication.
fn main() {
    start_client();
}