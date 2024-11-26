/// MQTT PUBACK packet implementation for MQTT version 5.0.

/// 
/// The PUBACK packet is used to acknowledge receipt of a published message.
/// When a client sends a message with QoS 1 (at least once delivery), 
/// it expects a PUBACK packet from the receiver (broker or client).
/// The PUBACK packet includes the message identifier (Packet ID) to match the message it acknowledges.
///

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/*
Implementing traits for:
    Debug: For printing the contents of the packet
    PartialEq: For comparing two instances of PUBACK packets
    Clone: To create a new independent instance of PUBACK
*/
#[derive(Debug, PartialEq, Clone)]
// The PUBACK packet structure as defined in MQTT 5.0
pub struct PubAckPacket {
    pub packet_id: u16, // Unique identifier for the message to acknowledge
}

impl PubAckPacket {
    // Constructor for the PubAckPacket, only requiring the packet_id.
    pub fn new(packet_id: u16) -> Self {
        PubAckPacket {
            packet_id,
        }
    }

    /// Encodes the PUBACK packet into bytes for transmission over the network.
    /// This method converts the packet's fields into a byte sequence.
    ///
    /// # Returns
    /// A byte vector representing the PUBACK packet.
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Fixed header (first byte): PUBACK packet type (0x40)
        packet.push(0x40);  // PUBACK packet type (MQTT Control Packet type for PUBACK)

        // Remaining length (2 bytes): The length of the variable header (just the packet ID)
        // In MQTT v5.0, this is a simple 2-byte integer (the packet ID).
        let remaining_length = 2; // Remaining length = 2 bytes for packet_id

        // Encode the remaining length with VLQ (Variable Length Quantity) encoding
        let mut len_buffer = Vec::new();
        let mut length = remaining_length;
        loop {
            let mut byte = (length % 128) as u8; // Get the least significant 7 bits
            length /= 128;
            if length > 0 {
                byte |= 0x80; // Set the most significant bit to indicate more bytes
            }
            len_buffer.push(byte);
            if length == 0 {
                break;
            }
        }

        // Add the remaining length bytes to the packet
        packet.extend(len_buffer);

        // The variable header contains the packet identifier (2 bytes)
        // The packet_id uniquely identifies the message being acknowledged
        packet.write_u16::<BigEndian>(self.packet_id).unwrap();

        // Return the encoded packet as a byte vector
        packet
    }

    /// Decodes a byte slice into a PUBACK packet.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice representing the PUBACK packet.
    ///
    /// # Returns
    ///
    /// This function returns a Result that contains either the decoded `PubAckPacket` 
    /// or an error if the decoding fails.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = std::io::Cursor::new(data);

        // Read the fixed header (first byte), it should be 0x40 for PUBACK
        let packet_type = cursor.read_u8().map_err(|e| e.to_string())?;
        if packet_type != 0x40 {
            return Err(format!("Invalid packet type: 0x{:02x}", packet_type));
        }

        // Read the remaining length (skip the length bytes in the header)
        let remaining_length = read_remaining_length(&mut cursor)?;

        // Check if the remaining length matches the expected value for PUBACK (2 bytes for packet_id)
        if remaining_length != 2 {
            return Err(format!("Invalid remaining length: {}", remaining_length));
        }

        // Read the Packet ID (2 bytes)
        let packet_id = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;

        // Return the decoded PUBACK packet
        Ok(PubAckPacket { packet_id })
    }
}

/// Helper function to read the remaining length field (Variable Length Quantity encoding)
/// This function reads the remaining length from the MQTT packet header.
/// 
/// # Arguments
///
/// * `cursor` - A cursor over the byte slice from which to read the remaining length.
///
/// # Returns
/// The remaining length value as an integer.
///
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