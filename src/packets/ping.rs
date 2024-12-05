use bytes::Bytes;
use bytes::{BytesMut, BufMut};

/// MQTT Packet Type
const PINGREQ: u8 = 0b1100_0000; // Packet type for PINGREQ with flags (0b1100)
const PINGRESP: u8 = 0b1101_0000; // Packet type for PINGRESP with flags (0b1101)

/// Represents an MQTT PINGREQ Packet
pub struct PingReqPacket;

impl PingReqPacket {
    /// Encodes the PINGREQ packet into bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u8(PINGREQ); // Fixed header byte 1
        buf.put_u8(0x00);    // Remaining length is 0 for PINGREQ
        buf
    }
}

/// Represents an MQTT PINGRESP Packet
pub struct PingRespPacket;

impl PingRespPacket {
    /// Encodes the PINGRESP packet into bytes
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_u8(PINGRESP); // Fixed header byte 1
        buf.put_u8(0x00);     // Remaining length is 0 for PINGRESP
        buf
    }

    /// Decodes a PINGRESP packet from bytes
    pub fn decode(bytes: &Bytes) -> Result<Self, &'static str> {
        if bytes.len() != 2 {
            return Err("Invalid PINGRESP packet length");
        }
        if bytes[0] != PINGRESP || bytes[1] != 0x00 {
            return Err("Invalid PINGRESP packet format");
        }
        Ok(PingRespPacket)
    }
}