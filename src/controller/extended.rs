use crate::device::PulseTransmitter;
use crate::protocols::ExtendedCommand;
use crate::protocols::ExtendedProtocol;
use crate::{Channel, Result};

/// # ExtendedRemoteController
///
/// The Extended Remote Controller utilize Extended protocol for LEGO® Power Functions enabling additional commands like increment/decrement speed,
/// toggle-forward-or-float and brake-then-float operations.
///
/// # Fields
///
/// * `channel` - The channel on which the remote controller operates.
/// * `pulse_transmitter` - A reference to an object that implements the `PulseTransmitter` trait, used to send pulses.
/// * `protocol` - An instance of `ExtendedProtocol` used to encode commands.
///
/// # Thread Safety
///
/// This controller maintains internal mutable state (for example, a toggle bit and an address field) that are updated with each command.
/// As a result, the instance is not safe for concurrent use. Its API (e.g. the `send` method) requires a mutable reference,
/// and sharing an instance across threads without external synchronization (for instance, wrapping it in a `Mutex`) is not recommended.
///
/// # Errors
///
/// This controller's methods will return an error if the protocol fails to encode the command or if the pulse transmitter fails to send pulses.
pub struct ExtendedRemoteController<'a, T: PulseTransmitter> {
    channel: Channel,
    pulse_transmitter: &'a T,
    protocol: ExtendedProtocol,
}

impl<'a, T: PulseTransmitter> ExtendedRemoteController<'a, T> {
    pub fn new(pulse_transmitter: &'a T, channel: Channel) -> Result<Self> {
        let protocol = ExtendedProtocol::new()?;
        Ok(Self {
            protocol,
            pulse_transmitter,
            channel,
        })
    }

    pub fn send(&mut self, cmd: ExtendedCommand) -> Result<()> {
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
    fn test_extended_all_commands() {
        let transmitter = MockTransmitterSuccess;
        let mut controller = ExtendedRemoteController::new(&transmitter, Channel::One)
            .expect("Should create ExtendedRemoteController");

        // We test each ExtendedCommand variant
        let commands = [
            ExtendedCommand::BrakeThenFloatOnRedOutput,
            ExtendedCommand::IncrementSpeedOnRedOutput,
            ExtendedCommand::DecrementSpeedOnRedOutput,
            ExtendedCommand::ToggleForwardOrFloatOnBlueOutput,
            ExtendedCommand::ToggleAddress,
            ExtendedCommand::AlignToggle,
        ];

        for &cmd in &commands {
            let result = controller.send(cmd);
            assert!(result.is_ok(), "Extended command failed for {:?}", cmd);
        }
    }

    #[test]
    fn test_extended_toggle_address_sequence() {
        // Check that toggling address twice returns to original
        let transmitter = MockTransmitterSuccess;
        let mut controller = ExtendedRemoteController::new(&transmitter, Channel::One)
            .expect("Should create ExtendedRemoteController");

        // Send toggle address once
        controller.send(ExtendedCommand::ToggleAddress).unwrap();
        // ... internal address state is now 1, toggle bit flips

        // Send again
        controller.send(ExtendedCommand::ToggleAddress).unwrap();
        // ... internal address state is back to 0, toggle bit flips again

        // If we had a way to access controller.protocol.address or .toggle, we could check it.
        // For coverage, the main thing is verifying these calls don't fail or panic.
    }

    #[test]
    fn test_extended_send_fails() {
        let transmitter = MockTransmitterFail;
        let mut controller = ExtendedRemoteController::new(&transmitter, Channel::One)
            .expect("Should create ExtendedRemoteController");

        let result = controller.send(ExtendedCommand::BrakeThenFloatOnRedOutput);
        assert!(result.is_err(), "Expected error from failing transmitter");
        match result {
            Err(Error::Transmitting(msg)) => assert!(msg.contains("Mock failure")),
            _ => panic!("Unexpected error variant"),
        }
    }
}
