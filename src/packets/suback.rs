/// MQTT SUBACK packet implementation for MQTT version 5.0.
///
/// The SUBACK packet is used to acknowledge a subscription request.
/// It is sent in response to a SUBSCRIBE packet from the client.
/// The SUBACK packet includes a Packet Identifier and a list of return codes
/// that indicate the result of the subscription request for each Topic Filter.
///
/// Return codes:
/// - 0x00: Success, QoS 0
/// - 0x01: Success, QoS 1
/// - 0x02: Success, QoS 2
/// - 0x80: Failure (Invalid Topic Filter)
///

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

#[derive(Debug, PartialEq, Clone)]
/// The SUBACK packet structure as defined in MQTT 5.0
pub struct SubAckPacket {
    pub packet_id: u16,          // Unique identifier for the subscription
    pub return_codes: Vec<u8>,   // List of return codes for each Topic Filter
}

impl SubAckPacket {
    /// Constructor for the SubAckPacket.
    ///
    /// # Arguments
    /// - `packet_id`: The Packet Identifier (u16)
    /// - `return_codes`: A vector of return codes (result of subscription for each topic filter)
    pub fn new(packet_id: u16, return_codes: Vec<u8>) -> Self {
        SubAckPacket {
            packet_id,
            return_codes,
        }
    }

    /// Encodes the SUBACK packet into bytes for transmission over the network.
    /// Converts the packet's fields into a byte sequence.
    ///
    /// # Returns
    /// A byte vector representing the SUBACK packet.
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Fixed header (first byte): SUBACK packet type (0x90) with reserved bits (1001)
        packet.push(0x90);

        // Variable header:
        // Packet Identifier (2 bytes)
        let mut variable_header = Vec::new();
        variable_header.write_u16::<BigEndian>(self.packet_id).unwrap();

        // Payload:
        // Return codes (1 byte for each topic filter's result)
        let mut payload = Vec::new();
        for return_code in &self.return_codes {
            payload.push(*return_code);
        }

        // Remaining length (Variable Header + Payload size)
        let remaining_length = variable_header.len() + payload.len();
        let mut len_buffer = Vec::new();
        let mut length = remaining_length;
        loop {
            let mut byte = (length % 128) as u8;
            length /= 128;
            if length > 0 {
                byte |= 0x80;
            }
            len_buffer.push(byte);
            if length == 0 {
                break;
            }
        }

        // Assemble the packet
        packet.extend(len_buffer); // Add remaining length
        packet.extend(variable_header); // Add variable header
        packet.extend(payload); // Add payload

        packet
    }

    /// Decodes a byte slice into a SUBACK packet.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice representing the SUBACK packet.
    ///
    /// # Returns
    /// This function returns a Result that contains either the decoded `SubAckPacket` 
    /// or an error if the decoding fails.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = std::io::Cursor::new(data);

        // Read the fixed header (first byte), it should be 0x90 for SUBACK
        let packet_type = cursor.read_u8().map_err(|e| e.to_string())?;
        if packet_type != 0x90 {
            return Err(format!("Invalid packet type: 0x{:02x}", packet_type));
        }

        // Read the remaining length
        let remaining_length = read_remaining_length(&mut cursor)?;

        // Read the Packet Identifier (2 bytes)
        let packet_id = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;

        // Read the payload (Return Codes)
        let mut return_codes = Vec::new();
        let mut bytes_read = 2; // Start with the 2 bytes of the packet_id
        while bytes_read < remaining_length {
            // Read each return code (1 byte per Topic Filter)
            let return_code = cursor.read_u8().map_err(|e| e.to_string())?;
            bytes_read += 1;
            return_codes.push(return_code);
        }

        // Return the decoded SubAckPacket
        Ok(SubAckPacket {
            packet_id,
            return_codes,
        })
    }
}

/// Helper function to read the remaining length field (Variable Length Quantity encoding)
fn read_remaining_length(cursor: &mut std::io::Cursor<&[u8]>) -> Result<usize, String> {
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