use crate::{
    device::PulseTransmitter,
    protocols::{ComboDirectCommand, ComboDirectProtocol},
    Channel, Result,
};

/// `DirectRemoteController` is a struct that represents a remote controller for the LEGO® Power Functions IR Remote Control 8885.
///
/// # Fields
///
/// * `channel` - The channel on which the remote controller operates.
/// * `pulse_transmitter` - A reference to an object that implements the `PulseTransmitter` trait, used to send pulses.
/// * `protocol` - An instance of `ComboDirectProtocol` used to encode commands.
///
/// # Thread Safety
///
/// Although the internal protocol used by `DirectRemoteController` is stateless (it does not maintain mutable state),
/// the public API requires a mutable reference (i.e. the `send` method takes `&mut self`), which prevents concurrent use.
/// If you must share an instance across threads, wrap the controller (or the underlying transmitter) in a synchronization primitive (e.g. a `Mutex`).
///
/// # Errors
///
/// This struct's methods will return an error if the protocol fails to encode the command or if the pulse transmitter fails to send pulses.
pub struct DirectRemoteController<'a, T: PulseTransmitter> {
    channel: Channel,
    pulse_transmitter: &'a T,
    protocol: ComboDirectProtocol,
}

impl<'a, T: PulseTransmitter> DirectRemoteController<'a, T> {
    pub fn new(pulse_transmitter: &'a T, channel: Channel) -> Result<Self> {
        let protocol = ComboDirectProtocol::new()?;
        Ok(Self {
            protocol,
            pulse_transmitter,
            channel,
        })
    }

    pub fn send(&mut self, cmd: ComboDirectCommand) -> Result<()> {
        let pulses = self.protocol.encode_cmd(self.channel, cmd)?;
        self.pulse_transmitter.send_pulses(&pulses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::PulseTransmitter;
    use crate::protocols::Channel;
    use crate::{DirectState, Error, Result};

    /// A mock transmitter that always succeeds.
    struct MockTransmitterSuccess;

    impl PulseTransmitter for MockTransmitterSuccess {
        fn send_pulses(&self, pulses: &[u32]) -> Result<()> {
            // Check that pulses are not empty
            assert!(!pulses.is_empty());
            Ok(())
        }
    }

    /// A mock transmitter that always fails.
    struct MockTransmitterFail;

    impl PulseTransmitter for MockTransmitterFail {
        fn send_pulses(&self, _pulses: &[u32]) -> Result<()> {
            Err(Error::Transmitting("Mock failure".to_string()))
        }
    }

    #[test]
    fn test_combo_direct_all_states() {
        // This covers all pairs of (red, blue) states.
        let transmitter = MockTransmitterSuccess;
        let mut controller = DirectRemoteController::new(&transmitter, Channel::One)
            .expect("Should create DirectRemoteController");

        let states = [
            DirectState::Float,
            DirectState::Forward,
            DirectState::Backward,
            DirectState::Brake,
        ];

        for &red_state in &states {
            for &blue_state in &states {
                let cmd = ComboDirectCommand {
                    red: red_state,
                    blue: blue_state,
                };
                let result = controller.send(cmd);
                assert!(
                    result.is_ok(),
                    "Command failed for red={:?} blue={:?}",
                    red_state,
                    blue_state
                );
            }
        }
    }

    #[test]
    fn test_combo_direct_send_fails() {
        // Ensure we handle transmitter errors gracefully
        let transmitter = MockTransmitterFail;
        let mut controller = DirectRemoteController::new(&transmitter, Channel::One)
            .expect("Should create DirectRemoteController");

        let cmd = ComboDirectCommand {
            red: DirectState::Forward,
            blue: DirectState::Float,
        };
        let result = controller.send(cmd);
        assert!(result.is_err(), "Expected error from failing transmitter");
        match result {
            Err(Error::Transmitting(msg)) => assert!(msg.contains("Mock failure")),
            _ => panic!("Unexpected error variant"),
        }
    }
}
