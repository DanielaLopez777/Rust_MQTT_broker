/// MQTT Publish packet implementation for MQTT version 5.0.

/*
The PUBLISH packet is used to send messages from a client to a broker, or from a broker to a client.
This packet includes the message content, topic name, and various flags that control the message's behavior.
*/

use std::io::Read;
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/*
Implement traits for:
    Debug: To print the contents of an instance
    PartialEq: To compare different instances
    Clone: To create an independent instance
*/
#[derive(Debug, PartialEq, Clone)]
// The Publish packet's structure defined in MQTT 5.0
pub struct PublishPacket {
    pub topic_name: String,       // The topic to which the message is being sent
    pub message_id: u16,  // The message ID (optional, only used for QoS 1 and 2)
    pub qos: u8,                  // Quality of Service level (0, 1, or 2)
    pub retain: bool,             // Retain flag (whether the message should be retained by the broker)
    pub dup: bool,                // Duplicate delivery flag (for QoS 1 and 2)
    pub payload: Vec<u8>,         // The actual message payload (data)
}

impl PublishPacket {
    // Constructor for a PublishPacket, with all fields as parameters
    pub fn new(
        topic_name: String,
        message_id: u16,
        qos: u8,
        retain: bool,
        dup: bool,
        payload: Vec<u8>,
    ) -> Self {
        PublishPacket {
            topic_name,
            message_id,
            qos,
            retain,
            dup,
            payload,
        }
    }

    /// Encodes the Publish packet into bytes to send to the broker.
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Fixed header (first byte): Publish packet type (0x30)
        let mut first_byte = 0x30;

        // Set QoS level, retain, and dup flags
        first_byte |= (self.qos & 0x03) << 1; // QoS is stored in bits 1-2
        if self.retain {
            first_byte |= 0x01; // Retain flag is in bit 0
        }
        if self.dup {
            first_byte |= 0x08; // DUP flag is in bit 3
        }

        // Add the first byte to the packet
        packet.push(first_byte);

        // Variable header length calculation
        let mut remaining_length = 2 + self.topic_name.len() as u16 + self.payload.len() as u16;

        if self.qos > 0 {
            // Add message ID field (2 bytes) for QoS 1 and 2
            remaining_length += 2;
        }

        // Encode the remaining length with VLQ codification
        let mut len_buffer = Vec::new();
        let mut length = remaining_length;
        loop {
            let mut byte = (length % 128) as u8;
            length /= 128;
            if length > 0 {
                byte |= 0x80; // 0x80 = 10000000, indicates more bytes
            }
            len_buffer.push(byte);
            if length == 0 {
                break;
            }
        }

        // Add the remaining length bytes to the packet
        packet.extend(len_buffer);

        // Topic Name: Encode the topic length (2 bytes) followed by the topic itself
        packet.push((self.topic_name.len() >> 8) as u8); // High byte of topic length
        packet.push(self.topic_name.len() as u8 & 0xFF); // Low byte of topic length
        packet.extend_from_slice(self.topic_name.as_bytes());

        packet.write_u16::<BigEndian>(self.message_id).unwrap();

        // Payload: Add the actual message content
        packet.extend_from_slice(&self.payload);

        packet
    }

    /// Decodes a byte slice into a Publish packet.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice representing the PUBLISH packet.
    ///
    /// # Returns
    ///
    /// This function returns a result containing either a decoded `PublishPacket` or an error if decoding fails.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = std::io::Cursor::new(data);
    
        //Read the first byte (packet type and flags)
        let first_byte = cursor.read_u8().map_err(|e| e.to_string())?;
    
        //Decode the rest of the package in VLQ
        let mut remaining_length = 0u16;
        let mut multiplier = 1u16;
        loop {
            let byte = cursor.read_u8().map_err(|e| e.to_string())?;
            remaining_length += (byte & 127) as u16 * multiplier;
            multiplier *= 128;
            if (byte & 128) == 0 {
                break;
            }
        }
    
        //Read the topic lenght (2 bytes) and the topic name
        let topic_name_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
        let mut topic_name = vec![0; topic_name_len];
        cursor.read_exact(&mut topic_name).map_err(|e| e.to_string())?;
        let topic_name = String::from_utf8(topic_name).map_err(|e| e.to_string())?;
    
        //Read the message ID if qos is > 0)
        let qos = (first_byte >> 1) & 0x03;
        let message_id = if qos > 0 {
            cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?
        } else {
            0
        };
    
        // Read the payload (remaining data)
        let mut payload = Vec::new();
        cursor.read_to_end(&mut payload).map_err(|e| e.to_string())?;
    
        Ok(PublishPacket {
            topic_name,
            message_id,
            qos,
            retain: first_byte & 0x01 != 0,
            dup: first_byte & 0x08 != 0,
            payload,
        })
    }
}
