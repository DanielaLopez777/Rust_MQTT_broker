// connack.rs

/// MQTT ConnAck packet implementation for MQTT version 5.0.
///
/// The CONNACK packet is sent by the broker in response to a CONNECT packet from the client.
/// It indicates the success or failure of the connection attempt and provides additional
/// properties as per MQTT 5.0.

use std::io::{Read};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

/// Represents the CONNACK packet in MQTT v5.0.
#[derive(Debug, PartialEq, Clone)]
pub struct ConnAckPacket {
    pub session_present: bool,          // Indicates if the session is already present
    pub reason_code: ConnAckReasonCode, // The reason code for the connection result
    pub properties: Option<ConnAckProperties>, // Optional properties introduced in MQTT v5.0
}

/// Enum to represent the possible reason codes for a CONNACK packet.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConnAckReasonCode {
    Success = 0x00,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    UnsupportedProtocolVersion = 0x84,
    ClientIdentifierNotValid = 0x85,
    BadUserNameOrPassword = 0x86,
    NotAuthorized = 0x87,
    ServerUnavailable = 0x88,
    ServerBusy = 0x89,
    Banned = 0x8A,
    BadAuthenticationMethod = 0x8C,
    TopicNameInvalid = 0x90,
    PacketTooLarge = 0x95,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9A,
    QosNotSupported = 0x9B,
    UseAnotherServer = 0x9C,
    ServerMoved = 0x9D,
    ConnectionRateExceeded = 0x9F,
}

impl ConnAckReasonCode {
    /// Decodes a reason code from a byte.
    pub fn from_byte(byte: u8) -> Result<Self, String> {
        match byte {
            0x00 => Ok(ConnAckReasonCode::Success),
            0x80 => Ok(ConnAckReasonCode::UnspecifiedError),
            0x81 => Ok(ConnAckReasonCode::MalformedPacket),
            0x82 => Ok(ConnAckReasonCode::ProtocolError),
            0x83 => Ok(ConnAckReasonCode::ImplementationSpecificError),
            0x84 => Ok(ConnAckReasonCode::UnsupportedProtocolVersion),
            0x85 => Ok(ConnAckReasonCode::ClientIdentifierNotValid),
            0x86 => Ok(ConnAckReasonCode::BadUserNameOrPassword),
            0x87 => Ok(ConnAckReasonCode::NotAuthorized),
            0x88 => Ok(ConnAckReasonCode::ServerUnavailable),
            0x89 => Ok(ConnAckReasonCode::ServerBusy),
            0x8A => Ok(ConnAckReasonCode::Banned),
            0x8C => Ok(ConnAckReasonCode::BadAuthenticationMethod),
            0x90 => Ok(ConnAckReasonCode::TopicNameInvalid),
            0x95 => Ok(ConnAckReasonCode::PacketTooLarge),
            0x97 => Ok(ConnAckReasonCode::QuotaExceeded),
            0x99 => Ok(ConnAckReasonCode::PayloadFormatInvalid),
            0x9A => Ok(ConnAckReasonCode::RetainNotSupported),
            0x9B => Ok(ConnAckReasonCode::QosNotSupported),
            0x9C => Ok(ConnAckReasonCode::UseAnotherServer),
            0x9D => Ok(ConnAckReasonCode::ServerMoved),
            0x9F => Ok(ConnAckReasonCode::ConnectionRateExceeded),
            _ => Err(format!("Unknown reason code: {}", byte)),
        }
    }

    /// Encodes a reason code into a byte.
    pub fn to_byte(&self) -> u8 {
        (*self) as u8
    }
    
}

/// Properties specific to the CONNACK packet in MQTT v5.0.
#[derive(Debug, PartialEq, Clone)]
pub struct ConnAckProperties {
    pub session_expiry_interval: Option<u32>, // Optional session expiry interval
    pub receive_maximum: Option<u16>,        // Maximum number of QoS 1 or QoS 2 messages
    pub maximum_packet_size: Option<u32>,    // Maximum size of a packet
    pub assigned_client_identifier: Option<String>, // Assigned client ID from broker
    pub reason_string: Option<String>,       // Human-readable reason for connection result
    pub server_keep_alive: Option<u16>,      // Server-determined keep-alive interval
    pub response_information: Option<String>, // Optional response information
    pub server_reference: Option<String>,    // Alternate server address
    pub authentication_method: Option<String>, // Optional authentication method
    pub authentication_data: Option<Vec<u8>>,  // Optional authentication data
}

impl ConnAckPacket {
    /// Encodes the CONNACK packet into bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Fixed header: CONNACK packet type (0x20) and reserved flags (0x00)
        packet.push(0x20);

        // Placeholder for remaining length (calculated later)
        let mut variable_header = Vec::new();

        // Session Present flag (1 byte)
        variable_header.push(if self.session_present { 1 } else { 0 });

        // Reason code (1 byte)
        variable_header.push(self.reason_code.to_byte());

        // Properties (if any)
        let mut properties = Vec::new();
        if let Some(ref props) = self.properties {
            if let Some(interval) = props.session_expiry_interval {
                properties.push(0x11); // Property identifier for session expiry interval
                properties.write_u32::<BigEndian>(interval).map_err(|e| e.to_string()).unwrap();
            }

            if let Some(maximum) = props.receive_maximum {
                properties.push(0x21); // Property identifier for receive maximum
                properties.write_u16::<BigEndian>(maximum).map_err(|e| e.to_string()).unwrap();
            }

            if let Some(size) = props.maximum_packet_size {
                properties.push(0x27); // Property identifier for maximum packet size
                properties.write_u32::<BigEndian>(size).map_err(|e| e.to_string()).unwrap();
            }

            if let Some(ref client_id) = props.assigned_client_identifier {
                properties.push(0x12); // Property identifier for assigned client ID
                properties.push(client_id.len() as u8);
                properties.extend_from_slice(client_id.as_bytes());
            }

            // Additional properties can be added similarly...
        }

        // Add properties length and properties to variable header
        variable_header.push(properties.len() as u8);
        variable_header.extend_from_slice(&properties);

        // Calculate remaining length
        let remaining_length = variable_header.len();
        packet.push(remaining_length as u8);

        // Add variable header to packet
        packet.extend(variable_header);

        packet
    }

    /// Decodes a CONNACK packet from bytes.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = std::io::Cursor::new(data);
        cursor.set_position(2);
        // Read session present flag
        let session_present = match cursor.read_u8().map_err(|e| e.to_string())? {
            0 => false,
            1 => true,
            _ => return Err("Invalid session present flag".to_string()),
        };


        // Read reason code
        let reason_code = ConnAckReasonCode::from_byte(cursor.read_u8().map_err(|e| e.to_string())?)?;

        // Read properties (if any)
        let mut properties = None;
        let properties_length = cursor.read_u8().map_err(|e| e.to_string())? as usize;
        if properties_length > 0 {
            let mut properties_data = vec![0; properties_length];
            cursor.read_exact(&mut properties_data).map_err(|e| e.to_string())?;
            // Decode properties (similar to encoding logic)
            properties = Some(ConnAckProperties {
                session_expiry_interval: None, // Decode as needed
                receive_maximum: None,        // Decode as needed
                maximum_packet_size: None,    // Decode as needed
                assigned_client_identifier: None, // Decode as needed
                reason_string: None,          // Decode as needed
                server_keep_alive: None,      // Decode as needed
                response_information: None,   // Decode as needed
                server_reference: None,       // Decode as needed
                authentication_method: None,  // Decode as needed
                authentication_data: None,    // Decode as needed
            });
        }

        Ok(ConnAckPacket {
            session_present,
            reason_code,
            properties,
        })
    }
}
