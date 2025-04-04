use crate::device::PulseTransmitter;
use crate::Result;

// Note: PulseTransmitterEmulator is for development/testing on non-Linux platforms only.
pub struct PulseTransmitterEmulator;

impl PulseTransmitter for PulseTransmitterEmulator {
    fn send_pulses(&self, pulses: &[u32]) -> Result<()> {
        println!("Simulated send pulses: {:?}", pulses);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::PulseTransmitter;

    #[test]
    fn test_emulator_send_pulses_non_empty() {
        let emulator = PulseTransmitterEmulator;
        let pulses = vec![150, 300, 450];
        let result = emulator.send_pulses(&pulses);
        assert!(
            result.is_ok(),
            "Emulator should return Ok for non-empty pulses"
        );
    }

    #[test]
    fn test_emulator_send_pulses_empty() {
        // The emulator just prints "Simulated send pulses: []" and returns Ok
        let emulator = PulseTransmitterEmulator;
        let pulses = vec![];
        let result = emulator.send_pulses(&pulses);
        assert!(
            result.is_ok(),
            "Emulator should also return Ok for empty pulses"
        );
    }
}
