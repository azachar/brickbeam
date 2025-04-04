use crate::{
    device::PulseTransmitter,
    protocols::{ComboPwmCommand, ComboPwmProtocol},
    Channel, Result,
};

/// `ComboSpeedRemoteController` is a struct that represents a remote controller for the LEGO® Power Functions Speed IR Remote Control 8879.
///
/// # Fields
///
/// * `channel` - The channel on which the remote controller operates.
/// * `pulse_transmitter` - A reference to an object that implements the `PulseTransmitter` trait, used to send pulses.
/// * `protocol` - An instance of `ComboPwmProtocol` used to encode commands.
///
/// # Thread Safety
///
/// This controller does not internally maintain mutable state (its protocol is stateless regarding toggle values),
/// but its API requires mutable access (the `send` method takes `&mut self`), meaning that concurrent use is not allowed.
/// If concurrent access is required, consider wrapping the instance in a synchronization primitive such as a `Mutex`.
///
/// # Errors
///
/// This struct's methods will return an error if the protocol fails to encode the command or if the pulse transmitter fails to send pulses.
pub struct ComboSpeedRemoteController<'a, T: PulseTransmitter> {
    channel: Channel,
    pulse_transmitter: &'a T,
    protocol: ComboPwmProtocol,
}

impl<'a, T: PulseTransmitter> ComboSpeedRemoteController<'a, T> {
    pub fn new(pulse_transmitter: &'a T, channel: Channel) -> Result<Self> {
        let protocol = ComboPwmProtocol::new()?;
        Ok(Self {
            protocol,
            pulse_transmitter,
            channel,
        })
    }

    pub fn send(&mut self, cmd: ComboPwmCommand) -> Result<()> {
        let pulses = self.protocol.encode_cmd(self.channel, cmd)?;
        self.pulse_transmitter.send_pulses(&pulses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::PulseTransmitter;
    use crate::protocols::Channel;
    use crate::{Error, Result};

    struct MockTransmitterSuccess;

    impl PulseTransmitter for MockTransmitterSuccess {
        fn send_pulses(&self, pulses: &[u32]) -> Result<()> {
            assert!(!pulses.is_empty());
            Ok(())
        }
    }

    struct MockTransmitterFail;

    impl PulseTransmitter for MockTransmitterFail {
        fn send_pulses(&self, _pulses: &[u32]) -> Result<()> {
            Err(Error::Transmitting("Mock failure".to_string()))
        }
    }

    #[test]
    fn test_combo_speed_various_speeds() {
        let transmitter = MockTransmitterSuccess;
        let mut controller = ComboSpeedRemoteController::new(&transmitter, Channel::One)
            .expect("Should create ComboSpeedRemoteController");

        // Test boundary and typical speeds on each output
        let speeds = [-7, -1, 0, 1, 7, 8];
        for &red_speed in &speeds {
            for &blue_speed in &speeds {
                let cmd = ComboPwmCommand {
                    speed_red: red_speed,
                    speed_blue: blue_speed,
                };
                let result = controller.send(cmd);
                assert!(
                    result.is_ok(),
                    "ComboPwmCommand failed for red={} blue={}",
                    red_speed,
                    blue_speed
                );
            }
        }
    }

    #[test]
    fn test_combo_speed_send_fails() {
        let transmitter = MockTransmitterFail;
        let mut controller = ComboSpeedRemoteController::new(&transmitter, Channel::One)
            .expect("Should create ComboSpeedRemoteController");

        let cmd = ComboPwmCommand {
            speed_red: 5,
            speed_blue: -3,
        };
        let result = controller.send(cmd);
        assert!(result.is_err(), "Expected error from failing transmitter");
        match result {
            Err(Error::Transmitting(msg)) => assert!(msg.contains("Mock failure")),
            _ => panic!("Unexpected error variant"),
        }
    }
}
