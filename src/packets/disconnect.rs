use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum DisconnectReasonCode {
    NormalDisconnection = 0x00,
    DisconnectWithWillMessage = 0x04,
    /*
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    ServerBusy = 0x89,*/
    ServerShuttingDown = 0x8B,
    KeepAliveTimeout = 0x8D,
    /*SessionTakenOver = 0x8E,
    TopicFilterInvalid = 0x8F,
    TopicNameInvalid = 0x90,
    ReceiveMaximumExceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9A,
    QoSNotSupported = 0x9B,
    UseAnotherServer = 0x9C,
    ServerMoved = 0x9D,
    SharedSubscriptionNotSupported = 0x9E,
    ConnectionRateExceeded = 0x9F,
    MaximumConnectTime = 0xA0,
    SubscriptionIdentifiersNotSupported = 0xA1,
    WildcardSubscriptionsNotSupported = 0xA2,*/
}

impl DisconnectReasonCode {
    pub fn from_u8(value: u8) -> Option<DisconnectReasonCode> {
        match value {
            0x00 => Some(DisconnectReasonCode::NormalDisconnection),
            0x04 => Some(DisconnectReasonCode::DisconnectWithWillMessage),
            0x8B => Some(DisconnectReasonCode::ServerShuttingDown),
            0x8D => Some(DisconnectReasonCode::KeepAliveTimeout),
            //Future cases ...
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct DisconnectPacket {
    reason_code: DisconnectReasonCode,
    properties: HashMap<u8, Vec<u8>>, // Key-value properties
}

impl DisconnectPacket {
    /// Create a new disconnect packet
    pub fn new(reason_code: DisconnectReasonCode) -> Self {
        Self {
            reason_code,
            properties: HashMap::new(),
        }
    }

    /// Add a property to the disconnect packet
    pub fn add_property(&mut self, property_identifier: u8, value: Vec<u8>) {
        self.properties.insert(property_identifier, value);
    }

    /// Encode the disconnect packet into bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = vec![];

        // Fixed header
        buffer.push(0xE0); // Disconnect packet type and flags
        let variable_header_len = 1 + self.properties.iter().map(|(_k, v)| 1 + v.len()).sum::<usize>();
        buffer.push(variable_header_len as u8);

        // Variable header
        buffer.push(self.reason_code.clone() as u8);

        // Properties
        for (key, value) in &self.properties {
            buffer.push(*key);
            buffer.extend(value);
        }

        buffer
    }

    /// Decode a disconnect packet from a byte slice
    pub fn decode(packet: &[u8]) -> Result<Self, &'static str> {
        if packet.len() < 3 {
            return Err("Packet too short to decode");
        }

        let mut index = 1; // Skip the fixed header byte

        // Get the length of the variable header
        let variable_header_len = packet[index] as usize;
        if packet.len() < variable_header_len + 2 {
            return Err("Packet length mismatch");
        }
        index += 1; // Move to the reason code

        // Extract the reason code (1 byte)
        let reason_code_value = packet[index];
        let reason_code = DisconnectReasonCode::from_u8(reason_code_value)
            .ok_or("Invalid reason code")?;
        index += 1; // Move to properties

        // Extract properties
        let mut properties = HashMap::new();
        while index < packet.len() {
            if index + 1 >= packet.len() {
                return Err("Property length missing");
            }

            let property_identifier = packet[index];
            index += 1;

            let property_length = packet[index] as usize;
            index += 1;

            if index + property_length > packet.len() {
                return Err("Property data out of bounds");
            }

            let property_value = packet[index..index + property_length].to_vec();
            properties.insert(property_identifier, property_value);
            index += property_length;
        }

        Ok(DisconnectPacket {
            reason_code,
            properties,
        })
    }
}