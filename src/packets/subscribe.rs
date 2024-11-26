use std::io::{Cursor, Read}; // Importing the `Read` trait
use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribePacket {
    pub packet_id: u16, // Packet ID
    pub topic_filters: Vec<String>, // Topics being subscribed to
    pub qos_values: Vec<u8>, // QoS values for each topic
}

impl SubscribePacket {
    /// Decodes a byte slice into a SUBSCRIBE packet.
    /// 
    /// # Arguments
    /// * `data` - A byte slice representing the SUBSCRIBE packet.
    /// 
    /// # Returns
    /// This function returns a Result that contains either the decoded `SubscribePacket` 
    /// or an error if the decoding fails.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = Cursor::new(data);

        // Read the fixed header (first byte), it should be 0x82 for SUBSCRIBE
        let packet_type = cursor.read_u8().map_err(|e| e.to_string())?;
        if packet_type != 0x82 {
            return Err(format!("Invalid packet type: 0x{:02x}", packet_type));
        }

        // Read the remaining length (variable length encoding)
        let remaining_length = read_remaining_length(&mut cursor)?;

        // Read the Packet Identifier (2 bytes)
        let packet_id = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;

        // Parse the topic filters and QoS values
        let mut topic_filters = Vec::new();
        let mut qos_values = Vec::new();
        let mut bytes_read = 2 + 2; // Starting from the packet ID and length field

        while bytes_read < remaining_length {
            // Read the length of the topic filter
            let topic_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;
            bytes_read += 2;

            // Ensure that the length is valid
            if topic_len == 0 {
                return Err("Topic length cannot be zero".to_string());
            }

            // Read the topic filter itself
            let mut topic_bytes = vec![0; topic_len as usize];
            cursor.read_exact(&mut topic_bytes).map_err(|e| e.to_string())?;
            bytes_read += topic_len as usize;

            let topic = String::from_utf8(topic_bytes).map_err(|e| e.to_string())?;

            // Read the QoS value (1 byte)
            let qos = cursor.read_u8().map_err(|e| e.to_string())?;
            bytes_read += 1;

            topic_filters.push(topic);
            qos_values.push(qos);
        }

        // Return the decoded SubscribePacket
        Ok(SubscribePacket {
            packet_id,
            topic_filters,
            qos_values,
        })
    }
}

/// Helper function to read the remaining length field (Variable Length Quantity encoding)
fn read_remaining_length(cursor: &mut Cursor<&[u8]>) -> Result<usize, String> {
    let mut multiplier = 1;
    let mut value = 0;

    loop {
        let byte = cursor.read_u8().map_err(|e| e.to_string())?;
        value += (byte & 0x7F) as usize * multiplier;
        if (byte & 0x80) == 0 {
            break;
        }
        multiplier *= 128;
    }

    Ok(value)
}
