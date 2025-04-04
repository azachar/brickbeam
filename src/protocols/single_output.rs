//! # Single Output Protocol
//!
//! Single Output mode messages let you control exactly one output (Red or Blue)
//! at a time, either with discrete commands or PWM speed steps. This is akin to
//! how the official “8879 Speed Remote” operates. We define an IRP (`LEGO_SINGLE_OUTPUT_IRP`)
//! specifying:
//!
//! - 38 kHz carrier frequency (about 26.3157 µs per cycle),
//! - 33% duty cycle (active portion of each cycle is 1/3),
//! - Bit encoding of 6 cycles for a “mark,” followed by either a short or long gap
//!   to distinguish “0” vs. “1” bits (via `<6,-10|6,-21>`).
//! - A final start/stop burst of `(6,-39)` to delimit each message.
//!
//! We compute a 4-bit LRC to ensure reliability. The protocol includes a “toggle bit”
//! that flips whenever a PWM command is transmitted, per LEGO Power Functions–style usage.
use irp::{Irp, Vartable};

use super::{map_speed, Channel, Output};
use crate::{Error, Result};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleOutputDiscrete {
    ToggleFullForward = 0b0000,
    ToggleDirection = 0b0001,
    IncrementNumericalPwm = 0b0010,
    DecrementNumericalPwm = 0b0011,
    IncrementPwm = 0b0100,
    DecrementPwm = 0b0101,
    FullForward = 0b0110,
    FullBackward = 0b0111,
    ToggleFullForwardBackward = 0b1000,
    ClearC1 = 0b1001,
    SetC1 = 0b1010,
    ToggleC1 = 0b1011,
    ClearC2 = 0b1100,
    SetC2 = 0b1101,
    ToggleC2 = 0b1110,
    ToggleFullBackward = 0b1111,
}

/// This enum represents the commands that can be sent to a controller using the Single Output protocol.
/// Commands can either be specified as a PWM (Pulse Width Modulation) value, which sets the speed and direction
/// of a motor, or as a discrete command that triggers a predefined operation (such as toggling direction).
#[derive(Debug, Clone, Copy)]
pub enum SingleOutputCommand {
    /// PWM command.
    ///
    /// This variant specifies a PWM value, where:
    /// - The signed integer value determines the speed of the motor.
    /// - Positive values indicate forward motion.
    /// - Negative values indicate reverse motion.
    /// - A value of 0 stops the motor, e.g. float
    /// - A value of 8 brake and float the motor.
    ///
    /// The acceptable values are from -7 to 8.
    PWM(i8),

    /// Discrete command.
    ///
    /// This variant sends a discrete command defined by the `SingleOutputDiscrete` enum.
    /// Discrete commands are used to trigger specific actions (such as toggling the motor’s direction)
    /// without directly setting a PWM value.
    Discrete(SingleOutputDiscrete),
}

/// Internal message for Single Output mode.
#[derive(Debug, Clone, Copy)]
struct SingleOutputMessage {
    toggle: u8,
    channel: u8,
    address: u8,
    mode: u8,   // 0 for PWM, 1 for Discrete
    output: u8, // 0 = Output A, 1 = Output B
    data: u8,
}

/// The SingleOutputProtocol encapsulates the IRP string, encoding logic, and its own toggle.
pub struct SingleOutputProtocol {
    irp: Irp,
    toggle: u8,
}

const LEGO_SINGLE_OUTPUT_IRP: &str = "\
{38k,33%,26.3157894737,msb}\
<6,-10|6,-21>\
(6,-39, T:1, 0:1, C:2, a:1, 1:1, M:1, O:1, D:4, L:4, 6,-39)\
{L = 0xF^((T*8+C)^((a<<3)|(1<<2)|(M<<1)|O)^D)}\
[T:0..1, C:0..3, a:0..1, M:0..1, O:0..1, D:0..15]\
";

impl SingleOutputProtocol {
    pub fn new() -> Result<Self> {
        let irp = Irp::parse(LEGO_SINGLE_OUTPUT_IRP).map_err(Error::ProtocolError)?;
        Ok(Self { irp, toggle: 0 })
    }

    fn encode_msg(&self, msg: SingleOutputMessage) -> Result<Vec<u32>> {
        let mut vars = Vartable::new();
        vars.set("T".into(), msg.toggle.into());
        vars.set("C".into(), msg.channel.into());
        vars.set("a".into(), msg.address.into());
        vars.set("M".into(), msg.mode.into());
        vars.set("O".into(), msg.output.into());
        vars.set("D".into(), msg.data.into());
        self.irp
            .encode_raw(vars, 1)
            .map(|res| res.raw)
            .map_err(Error::ProtocolError)
    }

    /// Encodes a Single Output command.
    pub fn encode_cmd(
        &mut self,
        channel: Channel,
        output: Output,
        cmd: SingleOutputCommand,
    ) -> Result<Vec<u32>> {
        let (mode, data) = match cmd {
            SingleOutputCommand::PWM(speed) => (0, map_speed(speed)),
            SingleOutputCommand::Discrete(discrete) => (1, discrete as u8),
        };
        let msg = SingleOutputMessage {
            toggle: self.toggle,
            channel: channel as u8,
            address: 0,
            mode,
            output: output as u8,
            data,
        };
        let pulses = self.encode_msg(msg)?;
        if mode == 0 {
            self.toggle ^= 1;
        }
        Ok(pulses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::{Channel, Output};
    #[test]
    fn test_single_output_pwm_encode_cmd() {
        let mut proto = SingleOutputProtocol::new().unwrap();
        let pulses = proto
            .encode_cmd(Channel::One, Output::RED, SingleOutputCommand::PWM(5))
            .expect("PWM encoding should succeed");
        assert!(!pulses.is_empty());

        assert_eq!(pulses.len(), 36, "Unexpected pulse sequence length");

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 552, 157, 263, 157,
            263, 157, 263, 157, 552, 157, 263, 157, 552, 157, 552, 157, 552, 157, 552, 157, 263,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }

    #[test]
    fn test_single_output_discrete_encode_cmd() {
        let mut proto = SingleOutputProtocol::new().unwrap();
        let pulses = proto
            .encode_cmd(
                Channel::One,
                Output::BLUE,
                SingleOutputCommand::Discrete(SingleOutputDiscrete::ToggleDirection),
            )
            .expect("Discrete encoding should succeed");
        assert!(!pulses.is_empty());

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 552, 157, 552, 157,
            552, 157, 263, 157, 263, 157, 263, 157, 552, 157, 552, 157, 263, 157, 263, 157, 552,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }

    #[test]
    fn test_single_output_pwm_full_range() {
        let mut proto = SingleOutputProtocol::new().unwrap();
        for speed in -7..=8 {
            let pulses =
                proto.encode_cmd(Channel::One, Output::RED, SingleOutputCommand::PWM(speed));
            assert!(pulses.is_ok(), "Encoding failed for speed={}", speed);
        }
    }

    #[test]
    fn test_single_output_discrete_commands() {
        let mut proto = SingleOutputProtocol::new().unwrap();
        let commands = [
            SingleOutputDiscrete::ToggleFullForward,
            SingleOutputDiscrete::ToggleDirection,
            SingleOutputDiscrete::IncrementNumericalPwm,
            SingleOutputDiscrete::DecrementNumericalPwm,
            SingleOutputDiscrete::FullForward,
            SingleOutputDiscrete::FullBackward,
            // ... etc
        ];
        for cmd in commands {
            let pulses = proto.encode_cmd(
                Channel::One,
                Output::BLUE,
                SingleOutputCommand::Discrete(cmd),
            );
            assert!(pulses.is_ok(), "Encoding failed for discrete cmd={:?}", cmd);
        }
    }
}
