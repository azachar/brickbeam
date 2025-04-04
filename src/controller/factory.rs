use crate::{
    controller::{
        ComboSpeedRemoteController, DirectRemoteController, ExtendedRemoteController,
        SpeedRemoteController,
    },
    device::{DefaultPulseTransmitter, PulseTransmitter},
    Result,
};
use crate::{Channel, Output};
use std::path::Path;

/// The primary API for creating various remote controllers for LEGO IR transmission.
///
/// This struct abstracts the details of the underlying `PulseTransmitter`.
/// On Linux, with the default Cargo feature `cir`, it uses [cir crate](https://docs.rs/cir) that uses the Linux kernel's LIRC (rc-core) drivers for IR transmitter.
/// on other platforms, it uses an emulator that is intended only for quick and easy compilation, not for production use.
///
/// Once initialized, you can create remote controllers that wrap the underlying LEGO® IR transmission protocols.
///
/// Specifically, BrickBeam provides methods to obtain a remote controller
/// * for the Single Output protocol via create_speed_remote_controller(),
/// * for the Combo PWM protocol via create_combo_speed_remote_controller(),
/// * for the Combo Direct protocol via create_direct_remote_controller(),
/// * and for the Extended protocol via create_extended_remote_controller().
///
/// # Examples
/// ```rust
/// use brickbeam::{BrickBeam, Channel, Output, SingleOutputCommand, Result};
///
/// fn main() -> Result<()> {
///     let brick_beam = BrickBeam::new("/dev/lirc0")?;
///     let mut motor = brick_beam.create_speed_remote_controller(Channel::One, Output::RED)?;
///     motor.send(SingleOutputCommand::PWM(5))?;
///     Ok(())
/// }
/// ```
pub struct BrickBeam<T: PulseTransmitter = DefaultPulseTransmitter> {
    pulse_transmitter: T,
}

impl BrickBeam<DefaultPulseTransmitter> {
    #[cfg(feature = "cir")]
    /// Creates a new `BrickBeam` instance using the Linux Kernel's LIRC (rc-core) IR transmitter.
    ///
    /// This function initializes a `BrickBeam` instance by setting up the Linux-specific IR transmitter.
    ///
    /// # Arguments
    ///
    /// * `tx_device_path` - A path reference to the kernel transmission device, such as /dev/lirc0.
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A result containing the new `BrickBeam` instance or an error.
    pub fn new(tx_device_path: impl AsRef<Path>) -> Result<Self> {
        let pulse_transmitter = crate::device::CirPulseTransmitter::new(tx_device_path)?;
        Ok(Self { pulse_transmitter })
    }

    #[cfg(not(feature = "cir"))]
    /// Creates a new `BrickBeam` instance for non‑Linux platforms using a simulated IR transmitter.
    ///
    /// # Arguments
    ///
    /// * `_tx_device_path` - A path reference to the transmission device (unused on non-Linux platforms).
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A result containing the new `BrickBeam` instance or an error.
    pub fn new(_tx_device_path: impl AsRef<Path>) -> Result<Self> {
        let pulse_transmitter = crate::device::PulseTransmitterEmulator;
        Ok(Self { pulse_transmitter })
    }
}

impl<T: PulseTransmitter> BrickBeam<T> {
    /// Creates a Speed Remote Controller using the Single Output protocol.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel (1 to 4) to be used for the controller.
    /// * `output` - The output (Red, Blue) to be used for the controller.
    ///
    /// # Returns
    ///
    /// * `Result<SpeedRemoteController<T>>` - A result containing the new `SpeedRemoteController` instance or an error.
    pub fn create_speed_remote_controller(
        &self,
        channel: Channel,
        output: Output,
    ) -> Result<SpeedRemoteController<T>> {
        SpeedRemoteController::new(&self.pulse_transmitter, channel, output)
    }

    /// Creates a Combo Speed Remote Controller using the Combo PWM protocol.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel (1 to 4) to be used for the controller.
    ///
    /// # Returns
    ///
    /// * `Result<ComboSpeedRemoteController<T>>` - A result containing the new `ComboSpeedRemoteController` instance or an error.
    pub fn create_combo_speed_remote_controller(
        &self,
        channel: Channel,
    ) -> Result<ComboSpeedRemoteController<T>> {
        ComboSpeedRemoteController::new(&self.pulse_transmitter, channel)
    }

    /// Creates a Direct Remote Controller using the Combo Direct protocol.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel (1 to 4) to be used for the controller.
    ///
    /// # Returns
    ///
    /// * `Result<DirectRemoteController<T>>` - A result containing the new `DirectRemoteController` instance or an error.
    pub fn create_direct_remote_controller(
        &self,
        channel: Channel,
    ) -> Result<DirectRemoteController<T>> {
        DirectRemoteController::new(&self.pulse_transmitter, channel)
    }

    /// Creates an Extended Remote Controller.
    ///
    /// # Arguments
    ///
    /// * `channel` - The channel (1 to 4) to be used for the controller.
    ///
    /// # Returns
    ///
    /// * `Result<ExtendedRemoteController<T>>` - A result containing the new `ExtendedRemoteController` instance or an error.
    pub fn create_extended_remote_controller(
        &self,
        channel: Channel,
    ) -> Result<ExtendedRemoteController<T>> {
        ExtendedRemoteController::new(&self.pulse_transmitter, channel)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Channel, Error, Output, PulseTransmitter, SingleOutputCommand};

    use super::BrickBeam;

    #[test]
    fn test_brick_beam_factory() {
        // On a non-Linux system or with no cir feature, this just uses the emulator.
        let beam = BrickBeam::new("/dev/lirc0").unwrap();
        beam.create_speed_remote_controller(Channel::One, Output::RED)
            .unwrap();
        beam.create_combo_speed_remote_controller(Channel::Two)
            .unwrap();
        beam.create_direct_remote_controller(Channel::Three)
            .unwrap();
        beam.create_extended_remote_controller(Channel::Four)
            .unwrap();
        // pass if all created successfully
    }

    struct FailingTransmitter;
    impl PulseTransmitter for FailingTransmitter {
        fn send_pulses(&self, _pulses: &[u32]) -> crate::Result<()> {
            Err(Error::Transmitting("Mocked failure".to_string()))
        }
    }

    #[test]
    fn test_send_fails() {
        let beam = BrickBeam {
            pulse_transmitter: FailingTransmitter,
        };
        let mut motor = beam
            .create_speed_remote_controller(Channel::One, Output::RED)
            .unwrap();
        let result = motor.send(SingleOutputCommand::PWM(5));
        assert!(result.is_err());
        match result {
            Err(Error::Transmitting(msg)) => assert!(msg.contains("Mocked failure")),
            _ => panic!("Expected Transmitting error"),
        }
    }
}
