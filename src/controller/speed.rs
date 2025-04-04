use crate::{
    device::PulseTransmitter,
    protocols::{SingleOutputCommand, SingleOutputProtocol},
    Channel, Output, Result,
};

/// `SpeedRemoteController` is a struct that represents a remote controller for the LEGO® Power Functions Speed IR Remote Control 8879.
///
/// # Fields
///
/// * `channel` - The channel on which the remote controller operates.
/// * `output` - The output (e.g., RED or BLUE) that the remote controller controls.
/// * `pulse_transmitter` - A reference to an object that implements the `PulseTransmitter` trait, used to send pulses.
/// * `protocol` - An instance of `SingleOutputProtocol` used to encode commands.
///
/// # Thread Safety
///
/// This controller maintains mutable state (for example, a toggle bit within its protocol) that changes on every command transmission.
/// Additionally, its public methods require mutable access (e.g. the `send` method takes `&mut self`).
/// Therefore, sharing an instance across multiple threads is not allowed unless you wrap it in a synchronization primitive such as a `Mutex`.
///
/// # Errors
///
/// This controller's methods will return an error if the protocol fails to encode the command or if the pulse transmitter fails to send pulses.
///
/// # Example
/// ```rust
/// use brickbeam::{BrickBeam, Channel, Output, SingleOutputCommand, Result};
///
/// fn main() -> Result<()> {
///     let brick_beam = BrickBeam::new("/dev/lirc0")?;
///     let mut motor = brick_beam.create_speed_remote_controller(Channel::One, Output::RED)?;
///     motor.send(SingleOutputCommand::PWM(1));
///     Ok(())
/// }
/// ```
pub struct SpeedRemoteController<'a, T: PulseTransmitter> {
    channel: Channel,
    output: Output,
    pulse_transmitter: &'a T,
    protocol: SingleOutputProtocol,
}

impl<'a, T: PulseTransmitter> SpeedRemoteController<'a, T> {
    pub fn new(pulse_transmitter: &'a T, channel: Channel, output: Output) -> Result<Self> {
        let protocol = SingleOutputProtocol::new()?;
        Ok(Self {
            protocol,
            pulse_transmitter,
            channel,
            output,
        })
    }

    /// Sends a command to the motor.
    ///
    /// Accepts either a PWM value or a discrete command.
    pub fn send(&mut self, cmd: SingleOutputCommand) -> Result<()> {
        let pulses = self.protocol.encode_cmd(self.channel, self.output, cmd)?;
        self.pulse_transmitter.send_pulses(&pulses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::PulseTransmitter;
    use crate::Error;
    use crate::{Channel, Output};
    use crate::{SingleOutputCommand, SingleOutputDiscrete};

    struct MockTransmitterSuccess;
    impl PulseTransmitter for MockTransmitterSuccess {
        fn send_pulses(&self, pulses: &[u32]) -> crate::Result<()> {
            // Ensure a non-empty pulse sequence is provided.
            assert!(!pulses.is_empty());
            Ok(())
        }
    }

    struct MockTransmitterFail;
    impl PulseTransmitter for MockTransmitterFail {
        fn send_pulses(&self, _pulses: &[u32]) -> crate::Result<()> {
            Err(Error::Transmitting("Mock failure".to_string()))
        }
    }

    #[test]
    fn test_speed_remote_controller_pwm_success() {
        let transmitter = MockTransmitterSuccess;
        let mut controller = SpeedRemoteController::new(&transmitter, Channel::One, Output::RED)
            .expect("Should create SpeedRemoteController");
        let result = controller.send(SingleOutputCommand::PWM(5));
        assert!(result.is_ok());
    }

    #[test]
    fn test_speed_remote_controller_discrete_success() {
        let transmitter = MockTransmitterSuccess;
        let mut controller = SpeedRemoteController::new(&transmitter, Channel::One, Output::BLUE)
            .expect("Should create SpeedRemoteController");
        let result = controller.send(SingleOutputCommand::Discrete(
            SingleOutputDiscrete::ToggleDirection,
        ));
        assert!(result.is_ok());
    }

    #[test]
    fn test_speed_remote_controller_failure() {
        let transmitter = MockTransmitterFail;
        let mut controller = SpeedRemoteController::new(&transmitter, Channel::One, Output::RED)
            .expect("Should create SpeedRemoteController");
        let result = controller.send(SingleOutputCommand::PWM(5));
        assert!(result.is_err());
        if let Err(Error::Transmitting(msg)) = result {
            assert!(msg.contains("Mock failure"));
        } else {
            panic!("Unexpected error variant");
        }
    }
}
