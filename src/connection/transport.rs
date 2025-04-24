pub(crate) struct TransportLayer {
    protocol: Protocol,
    state: ConnectionState,
}

pub enum Protocol {
    SSH,
    HTTPS,
}
