/// MQTT Connect packet implementation for MQTT version 5.0.

/* 
The CONNECT packet is used to establish a connection between a client and a broker.
This packet includes several fields that identify the client, specify connection settings,
     and allow for the broker to acknowledge the connection.
*/

use std::io::{Read};
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};

/*
Implement traits for:
    Debug: To print the contents of an instance
    PartialEq: To compare different instances
    Clone: To create an independent instance
*/
#[derive(Debug, PartialEq, Clone)]
// The Connect packet's structure defined in MQTT 5.0
pub struct ConnectPacket {
    pub protocol_name: String,   // Name of the protocol (e.g., "MQTT")
    pub protocol_level: u8,      // Protocol level, should be 5 for MQTT v5.0
    pub connect_flags: u8,       // Flags that indicate the behavior of the connection
    pub keep_alive: u16,         // Maximum time interval between messages
    pub client_id: String,       // Unique identifier for the client
    //Option fields could take Some(value) or None
    pub will_topic: Option<String>,   // Will topic (optional)
    pub will_message: Option<String>, // Will message (optional)
    pub username: Option<String>,     // Username for authentication (optional)
    pub password: Option<String>,     // Password for authentication (optional)
    
}

impl ConnectPacket {
    // Constructor for a ConnectPacket, with all fields as parameters
    pub fn new(
        protocol_name: String,
        protocol_level: u8,
        connect_flags: u8,
        keep_alive: u16,
        client_id: String,
        will_topic: Option<String>,
        will_message: Option<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        ConnectPacket {
            protocol_name,
            protocol_level,
            connect_flags,
            keep_alive,
            client_id,
            will_topic,
            will_message,
            username,
            password,
        }
    }

    /// Encodes the Connect packet into bytes to send to the broker.
    pub fn encode(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        // Fixed header (first byte): Connect packet type (0x10)
        packet.push(0x10);

        // Variable header length calculation
        let mut remaining_length = 2 + self.protocol_name.len() as u16 + 1 // Protocol name & protocol level
            + 1 // Connect flags byte
            + 2 // Keep alive
            + 2 // Client ID len field
            + self.client_id.len() as u16; // Client ID

        //Evaluates if there are some optional fields
        if let Some(ref will_topic) = self.will_topic {
            //Will topic len field + will_topic len + will message len field + will_message len
            remaining_length += 2 + will_topic.len() as u16 + 2 + self.will_message.as_ref().unwrap().len() as u16;
        }

        if let Some(ref username) = self.username {
            //Username len field + username len
            remaining_length += 2 + username.len() as u16;
        }

        if let Some(ref password) = self.password {
            //Password len field + password len
            remaining_length += 2 + password.len() as u16;
        }

        // Encode the remaining length with VLQ codification
        let mut len_buffer = Vec::new();
        let mut length = remaining_length;
        loop {
            //Takes the 7 less significative bits.
            let mut byte = (length % 128) as u8;
            //Obtains the next 7 bits group
            length /= 128;
            //If there is another 7 bits group
            if length > 0 {
                /*Sets the most significant bit to 1 to
                indicate there are more bytes */
                byte |= 0x80; // 0x80 = 10000000
            }
            //Adds byte to the vector len_buffer
            len_buffer.push(byte);
            if length == 0 {
                break;
            }
        }

        // Add the remaining length bytes to the packet
        packet.extend(len_buffer);

        /*
        Using >>8 makes a right shift to leave only the 8 most significant bits
        & 0xFF applies a mask bit by bit to leave only the 8 least significant bits
        */

        // Protocol Name
        packet.push((self.protocol_name.len() >> 8) as u8); // Length of protocol name high byte
        packet.push(self.protocol_name.len() as u8 & 0xFF); // Length of protocol name low byte
        packet.extend_from_slice(self.protocol_name.as_bytes());

        // Protocol Level (always 5 for MQTT v5.0)
        packet.push(self.protocol_level);

        // Connect Flags
        packet.push(self.connect_flags);

        // Keep Alive
        packet.write_u16::<BigEndian>(self.keep_alive).unwrap();

        // Client ID length and value
        packet.push((self.client_id.len() >> 8) as u8); // High byte of client ID length
        packet.push(self.client_id.len() as u8 & 0xFF); // Low byte of client ID length
        packet.extend_from_slice(self.client_id.as_bytes());

        // Will Topic and Message (if present)
        if let Some(ref will_topic) = self.will_topic {
            packet.push((will_topic.len() >> 8) as u8);
            packet.push(will_topic.len() as u8 & 0xFF);
            packet.extend_from_slice(will_topic.as_bytes());

            let will_message = self.will_message.as_ref().unwrap();
            packet.push((will_message.len() >> 8) as u8);
            packet.push(will_message.len() as u8 & 0xFF);
            packet.extend_from_slice(will_message.as_bytes());
        }

        // Username (if present)
        if let Some(ref username) = self.username {
            packet.push((username.len() >> 8) as u8);
            packet.push(username.len() as u8 & 0xFF);
            packet.extend_from_slice(username.as_bytes());
        }

        // Password (if present)
        if let Some(ref password) = self.password {
            packet.push((password.len() >> 8) as u8);
            packet.push(password.len() as u8 & 0xFF);
            packet.extend_from_slice(password.as_bytes());
        }

        packet
    }

    /// Decodes a byte slice into a Connect packet.
    ///
    /// # Arguments
    ///
    /// * `data` - The byte slice representing the CONNECT packet.
    ///
    /// # Returns
    ///
    /// This function returns a result containing either a decoded `ConnectPacket` or an error if decoding fails.
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        let mut cursor = std::io::Cursor::new(data);
        //Move two places due to the fixed header
        cursor.set_position(2);
 
        // Extracts the protocol name length 
        let protocol_name_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
        //Sets a mut vector of the size extracted
        let mut protocol_name = vec![0; protocol_name_len];
        //Parse the name into a string and saves it into the mut vector
        cursor.read_exact(&mut protocol_name).map_err(|e| e.to_string())?;
        let protocol_name = String::from_utf8(protocol_name).map_err(|e| e.to_string())?;

        // Extract the protocol level
        let protocol_level = cursor.read_u8().map_err(|e| e.to_string())?;

        // Extract the connect flags
        let connect_flags = cursor.read_u8().map_err(|e| e.to_string())?;

        // Extract keep alive time
        let keep_alive = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())?;

        // Read client ID length and value
        let client_id_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
        let mut client_id = vec![0; client_id_len];
        cursor.read_exact(&mut client_id).map_err(|e| e.to_string())?;
        let client_id = String::from_utf8(client_id).map_err(|e| e.to_string())?;

        // Parse optional fields: Will, Username, Password
        let mut will_topic = None;
        let mut will_message = None;
        let mut username = None;
        let mut password = None;

        // Will Topic and Message
        if connect_flags & 0x04 != 0 {
            let will_topic_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
            let mut will_topic_bytes = vec![0; will_topic_len];
            cursor.read_exact(&mut will_topic_bytes).map_err(|e| e.to_string())?;
            will_topic = Some(String::from_utf8(will_topic_bytes).map_err(|e| e.to_string())?);

            let will_message_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
            let mut will_message_bytes = vec![0; will_message_len];
            cursor.read_exact(&mut will_message_bytes).map_err(|e| e.to_string())?;
            will_message = Some(String::from_utf8(will_message_bytes).map_err(|e| e.to_string())?);
        }

        // Username
        if connect_flags & 0x80 != 0 {
            let username_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
            let mut username_bytes = vec![0; username_len];
            cursor.read_exact(&mut username_bytes).map_err(|e| e.to_string())?;
            username = Some(String::from_utf8(username_bytes).map_err(|e| e.to_string())?);
        }

        // Password
        if connect_flags & 0x40 != 0 {
            let password_len = cursor.read_u16::<BigEndian>().map_err(|e| e.to_string())? as usize;
            let mut password_bytes = vec![0; password_len];
            cursor.read_exact(&mut password_bytes).map_err(|e| e.to_string())?;
            password = Some(String::from_utf8(password_bytes).map_err(|e| e.to_string())?);
        }

        //Return the connect packet with the parsed information
        Ok(ConnectPacket {
            protocol_name,
            protocol_level,
            connect_flags,
            keep_alive,
            client_id,
            will_topic,
            will_message,
            username,
            password,
        })
    }
}