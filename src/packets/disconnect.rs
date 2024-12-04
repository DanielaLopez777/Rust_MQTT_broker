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
        let variable_header_len = 1 + self.properties.iter().map(|(k, v)| 1 + v.len()).sum::<usize>();
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

    /// Decode a disconnect packet from bytes
    pub fn decode(packet: &[u8]) -> Result<Self, &'static str> {
        let reason_code = DisconnectReasonCode::from_u8(packet[2])
            .ok_or("Invalid reason code in Disconnect packet")?;

        let mut properties = HashMap::new();
        let mut index = 3; // Start after reason code
        while index < packet.len() {
            let property_id = packet[index];
            index += 1;
            let value = vec![packet[index]];
            index += 1;
            properties.insert(property_id, value);
        }

        Ok(Self {
            reason_code,
            properties,
        })
    }
}