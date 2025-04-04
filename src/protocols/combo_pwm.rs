//! # Combo PWM Protocol
//!
//! Combo PWM allows simultaneously controlling both outputs (A/B) at various
//! speed steps. Internally, we define the IRP string `LEGO_COMBO_PWM_IRP`:
//! ```ignore
//! {38k,33%,26.3157894737,msb}
//! <6,-10|6,-21>
//! (6,-39, a:1, 1:1, C:2, B:4, A:4, L:4, 6,-39)
//! {L = ...}
//! ```
//! which indicates how to flash/gap bits at 38 kHz, a 33% duty cycle, etc.
//! We then map user-friendly `ComboPwmCommand` speeds (e.g. `speed_red=5`)
//! to the correct nibble for each output.

use super::{map_speed, Channel};
use crate::{Error, Result};
use irp::{Irp, Vartable};

/// Represents a Combo PWM command used for simultaneous control of two outputs
/// via the Combo PWM protocol.
#[derive(Debug, Clone, Copy)]
pub struct ComboPwmCommand {
    /// PWM speed for output A (red). Valid range is from -7 to 8.
    ///
    /// • A value of 0 sets the output to float.
    /// • A value of 8 applies braking before floating.
    pub speed_red: i8,

    /// PWM speed for output B (blue). Valid range is from -7 to 8.
    ///
    /// • A value of 0 sets the output to float.
    /// • A value of 8 applies braking before floating.
    pub speed_blue: i8,
}

struct ComboPwmMessage {
    address: u8,
    channel: u8,
    output_b: u8,
    output_a: u8,
}

pub struct ComboPwmProtocol {
    irp: Irp,
}

const LEGO_COMBO_PWM_IRP: &str = "\
{38k,33%,26.3157894737,msb}\
<6,-10|6,-21>\
(6,-39, a:1, 1:1, C:2, B:4, A:4, L:4, 6,-39)\
{L = 0xF^( ( (a<<3) | (1<<2) | C ) ^ B ^ A )}\
[a:0..1,C:0..3,B:0..15,A:0..15]\
";

impl ComboPwmProtocol {
    pub fn new() -> Result<Self> {
        let irp = Irp::parse(LEGO_COMBO_PWM_IRP).map_err(Error::ProtocolError)?;
        Ok(Self { irp })
    }

    fn encode_msg(&self, msg: ComboPwmMessage) -> Result<Vec<u32>> {
        let mut vars = Vartable::new();
        vars.set("a".into(), msg.address.into());
        vars.set("C".into(), msg.channel.into());
        vars.set("B".into(), msg.output_b.into());
        vars.set("A".into(), msg.output_a.into());
        self.irp
            .encode_raw(vars, 1)
            .map(|res| res.raw)
            .map_err(Error::ProtocolError)
    }

    /// Encodes a Combo PWM command.
    pub fn encode_cmd(&self, channel: Channel, cmd: ComboPwmCommand) -> Result<Vec<u32>> {
        let msg = ComboPwmMessage {
            address: 0,
            channel: channel as u8,
            output_b: map_speed(cmd.speed_blue),
            output_a: map_speed(cmd.speed_red),
        };
        self.encode_msg(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::Channel;
    #[test]
    fn test_combo_pwm_encode_cmd() {
        let proto = ComboPwmProtocol::new().unwrap();
        let cmd = ComboPwmCommand {
            speed_red: 5,
            speed_blue: -3,
        };
        let pulses = proto
            .encode_cmd(Channel::One, cmd)
            .expect("Encoding should succeed");
        assert!(!pulses.is_empty());

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 552, 157, 263, 157, 263, 157, 552, 157, 552, 157, 263, 157,
            552, 157, 263, 157, 552, 157, 263, 157, 552, 157, 263, 157, 263, 157, 552, 157, 552,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }
}
