use crate::device::PulseTransmitter;
use crate::{Error, Result};
use cir::lirc::Lirc;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Transmits pulses to the kernel's /dev/lircX device using the cir library.
/// See README.md for information how to enable /dev/lircX device in the Linux kernel.
pub struct CirPulseTransmitter {
    tx_device: Arc<Mutex<Lirc>>,
}

impl CirPulseTransmitter {
    /// Creates a new CirPulseTransmitter instance.
    ///
    /// # Arguments
    ///
    /// * `tx_device_path` - A reference to the path of the transmission device. (e.g. /dev/lirc0)
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A result containing the new CirPulseTransmitter instance or an error.
    pub fn new(tx_device_path: impl AsRef<Path>) -> Result<Self> {
        let tx_device = cir::lirc::open(tx_device_path)?;
        Ok(Self {
            tx_device: Arc::new(Mutex::new(tx_device)),
        })
    }
}

impl PulseTransmitter for CirPulseTransmitter {
    /// Sends pulses to the transmission device.
    ///
    /// # Arguments
    ///
    /// * `pulses` - A slice of unsigned 32-bit integers representing the pulses to be sent.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - A result indicating success or failure.
    fn send_pulses(&self, pulses: &[u32]) -> Result<()> {
        let mut tx_device = self
            .tx_device
            .lock()
            .map_err(|e| Error::Transmitting(format!("Lock error: {}", e)))?;

        tx_device
            .send(pulses)
            .map_err(|e| Error::Transmitting(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "cir")]
mod tests {
    use super::*;

    #[test]
    fn test_cir_transmitter_send_pulses_non_empty() {
        // This test requires a valid /dev/lirc0 device.
        let transmitter = CirPulseTransmitter::new("/dev/lirc0").expect("Should open /dev/lirc0");
        let pulses = vec![157, 263, 157, 1026];
        let result = transmitter.send_pulses(&pulses);
        assert!(
            result.is_ok(),
            "Transmitter should return Ok for non-empty pulses"
        );
    }

    #[test]
    fn test_cir_transmitter_new_invalid_path() {
        let result = CirPulseTransmitter::new("/invalid/path");
        assert!(result.is_err());
    }
}
