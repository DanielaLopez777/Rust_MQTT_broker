use std::net::TcpStream;
use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::env;

use mqtt_broker::packets::{
    connect::ConnectPacket,
    connack::ConnAckPacket,
    publish::PublishPacket,
    subscribe::SubscribePacket,
    ping::PingReqPacket,
    disconnect::{DisconnectPacket, DisconnectReasonCode},
};

fn send_connect_packet(mut stream: TcpStream, client_id: String)
{
    let connect_packet = ConnectPacket::new(
        "MQTT".to_string(),
        5,
        0b00000010,
        60,
        client_id,
        None,
        None,
        Some("user".to_string()),
        Some("password".to_string()),
    );

    let packet = connect_packet.encode();
    let _ = stream.write(&packet);
}

fn receive_connack_packet(mut stream: TcpStream)
{
    let mut buffer = [0u8; 1024];
    let _ = stream.read(&mut buffer);
    let _ = ConnAckPacket::decode(&buffer);
}

fn send_publish_packet(mut stream: TcpStream, topic: &str, message: &str)
{
    let publish_packet = PublishPacket::new(
        topic.to_string(),
        1,
        1,
        false,
        false,
        message.as_bytes().to_vec(),
    );

    let packet = publish_packet.encode();
    let _ = stream.write(&packet);
}

fn send_subscribe_packet(mut stream: TcpStream, topic: &str)
{
    let subscribe_packet =
        SubscribePacket::new(1, vec![topic.to_string()], vec![1]);

    let packet = subscribe_packet.encode();
    let _ = stream.write(&packet);
}

fn send_disconnect_packet(stream: &mut TcpStream)
{
    let packet =
        DisconnectPacket::new(DisconnectReasonCode::NormalDisconnection)
            .encode();

    let _ = stream.write(&packet);
}

fn packets_listener(mut stream: TcpStream, shutdown_flag: Arc<Mutex<bool>>)
{
    let mut buffer = [0u8; 1024];

    loop {
        let ping = PingReqPacket.encode();
        let _ = stream.write(&ping);

        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                let packet_type = buffer[0] >> 4;

                if packet_type == 3 {
                    if let Ok(packet) =
                        PublishPacket::decode(&buffer[..size])
                    {
                        let _ =
                            String::from_utf8(packet.payload).unwrap_or_default();
                    }
                }
            }
            _ => {
                *shutdown_flag.lock().unwrap() = true;
                break;
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}

fn start_client()
{
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("sub");

    let shutdown_flag = Arc::new(Mutex::new(false));

    let client_id =
        format!("client-{}", std::process::id());

    let mut stream =
        TcpStream::connect("192.168.100.10:1883")
            .expect("Connection failed");

    send_connect_packet(stream.try_clone().unwrap(), client_id);
    receive_connack_packet(stream.try_clone().unwrap());

    if mode == "sub" {
        send_subscribe_packet(stream.try_clone().unwrap(), "test");
    }

    if mode == "pub" {

        let payload_size: usize =
            args[2].parse().unwrap();

        let execution_time: u64 =
            args[3].parse().unwrap();

        let frequency: u64 =
            args[4].parse().unwrap();

        let payload = "A".repeat(payload_size);

        let publish_interval =
            Duration::from_secs(frequency);

        let total_messages =
            execution_time / frequency;

        let mut message_count = 0;

        let start = Instant::now();

        while start.elapsed()
            < Duration::from_secs(execution_time)
            && message_count < total_messages
        {
            let publish_start = Instant::now();

            send_publish_packet(
                stream.try_clone().unwrap(),
                "test",
                &payload,
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

    send_disconnect_packet(&mut stream);
}

fn main() {
    start_client();
}