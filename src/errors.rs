use thiserror::Error;

/// The library’s specialized `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

/// Possible errors while encoding commands or transmitting pulses.
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Pulse sending error: {0}")]
    Transmitting(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_display_io() {
        let io_err = Error::Io(io::Error::new(io::ErrorKind::Other, "test error"));
        assert!(io_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_display_protocol() {
        let proto_err = Error::ProtocolError("encoding failed".to_string());
        assert!(proto_err.to_string().contains("Protocol error"));
    }

    #[test]
    fn test_error_display_transmitting() {
        let tx_err = Error::Transmitting("transmission failed".to_string());
        assert!(tx_err.to_string().contains("Pulse sending error"));
    }
}
