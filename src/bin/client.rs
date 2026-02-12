use std::net::TcpStream;
use std::io::{Read, Write};
use std::io::{self};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::env;
use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    publish::PublishPacket,
    puback::PubAckPacket,
    subscribe::SubscribePacket, 
    suback::SubAckPacket, 
    ping:: PingReqPacket,
    disconnect::{DisconnectPacket, DisconnectReasonCode}
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
        Ok(_) => println!("[+]CONNECT packet sent: {:?}\n", connect_packet),
        Err(e) => eprintln!("[-]Failed to send CONNECT: {}\n", e),
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
                    println!("[+]Received CONNACK packet: {:?}\n", connack_packet);
                }
                Err(e) => eprintln!("[-]Failed to decode CONNACK: {}\n", e),
            }
        }
        Ok(_) => eprintln!("[-]Empty package received\n"),
        Err(e) => eprintln!("[-]Error reading the stream: {}\n", e),
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

    let _ = stream.write(&packet);
}

fn send_subscribe_packet(mut stream: TcpStream, packet_id: u16, topic: &str) {
    // Predefined QoS values (you can adjust this as needed)
    let qos_values = vec![1];

    // Create the SUBSCRIBE packet
    let subscribe_packet = SubscribePacket::new(packet_id, vec![topic.to_string()], qos_values);

    // Encode the SUBSCRIBE packet into bytes for transmission
    let packet = subscribe_packet.encode();

    // Send the SUBSCRIBE packet to the server
    match stream.write(&packet) {
        Ok(_) => println!("[+]SUBSCRIBE packet sent: {:?}\n", subscribe_packet),
        Err(e) => eprintln!("[-]Failed to send SUBSCRIBE: {}\n", e),
    }
}

fn send_disconnect_packet(stream: &mut TcpStream, reason_code: DisconnectReasonCode) {
    let mut disconnect_packet = DisconnectPacket::new(reason_code);
    disconnect_packet.add_property(0x11, vec![0x01, 0x02]);
    let packet = disconnect_packet.encode();
    let _ = stream.write(&packet);
}

fn packets_listener(mut stream: TcpStream, shutdown_flag: Arc<Mutex<bool>>) {
    let mut buffer = [0u8; 1024];
    let mut last_ping_time = Instant::now();

    loop {
        let pingreq_packet = PingReqPacket;
        let pingreq_response = pingreq_packet.encode();
        let _ = stream.write(&pingreq_response);

        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let packet_type = buffer[0] >> 4;

                if packet_type == 3 {
                    if let Ok(packet) = PublishPacket::decode(&buffer[..size]) {
                        let msg = String::from_utf8(packet.payload).unwrap();
                        println!("{}: {}", packet.topic_name, msg);
                    }
                }

                last_ping_time = Instant::now();
            }
            _ => {
                let mut shutdown = shutdown_flag.lock().unwrap();
                *shutdown = true;
                break;
            }
        }

        if last_ping_time.elapsed() > Duration::from_secs(60) {
            break;
        }
    }
}

fn start_client() 
{
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("sub");

    let shutdown_flag = Arc::new(Mutex::new(false));

    match TcpStream::connect("192.168.100.10:1883") {
        Ok(mut stream) => 
        {
            send_connect_packet(stream.try_clone().unwrap());
            receive_connack_packet(stream.try_clone().unwrap());

            if mode == "sub" {
                send_subscribe_packet(stream.try_clone().unwrap(), 1, "test");
            }

            if mode == "pub" {
                let payload_size: usize = args.get(2)
                    .expect("Missing payload size")
                    .parse()
                    .expect("Invalid payload size");

                let execution_time: u64 = args.get(3)
                    .expect("Missing execution time")
                    .parse()
                    .expect("Invalid execution time");

                let publish_frequency: u64 = args.get(4)
                    .expect("Missing publish frequency")
                    .parse()
                    .expect("Invalid publish frequency");

                let publish_interval = Duration::from_secs(publish_frequency);

                let payload = "A".repeat(payload_size);

                // Cantidad máxima de mensajes esperados
                let total_messages = execution_time / publish_frequency;

                let mut message_count = 0;

                let program_start = Instant::now();

                while program_start.elapsed() < Duration::from_secs(execution_time)
                    && message_count < total_messages
                {
                    let publish_start = Instant::now();

                    send_publish_packet(
                        stream.try_clone().unwrap(),
                        "test",
                        &payload
                    );

                    message_count += 1;

                    let elapsed = publish_start.elapsed();

                    if elapsed < publish_interval {
                        thread::sleep(publish_interval - elapsed);
                    }
                }

                println!("Total messages sent: {}", message_count);
            }

            let listener_stream = stream.try_clone().unwrap();
            let listener_flag = Arc::clone(&shutdown_flag);

            thread::spawn(move || {
                packets_listener(listener_stream, listener_flag);
            });

            loop {
                if *shutdown_flag.lock().unwrap() {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }

            send_disconnect_packet(&mut stream, DisconnectReasonCode::NormalDisconnection);
        }
        Err(_) => {}
    }
}

fn main() {
    start_client();
}
