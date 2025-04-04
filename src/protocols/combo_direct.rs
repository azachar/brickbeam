//! # Combo Direct Protocol
//!
//! We reuse the `LEGO_EXTENDED_IRP` constant (from extended.rs) because the
//! base waveform timing is the same. The relevant bits for Combo Direct are
//! encoded as (Mode=1), toggling the F nibble for the two outputs, etc.

use super::Channel;
use crate::{Error, Result};
use irp::{Irp, Vartable};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectState {
    Float = 0b00,
    Forward = 0b01,
    Backward = 0b10,
    Brake = 0b11,
}

/// Represents a Combo Direct command used to control two outputs simultaneously
/// via the Combo Direct protocol.
#[derive(Debug, Clone, Copy)]
pub struct ComboDirectCommand {
    /// The state for output A (red).
    /// Controls the forward, reverse, brake or float actions for the A output.
    pub red: DirectState,

    /// The state for output B (blue).
    /// Controls the forward, reverse, brake or float actions for the B output.
    pub blue: DirectState,
}

struct ComboDirectMessage {
    channel: u8,
    data: u8,
}

pub struct ComboDirectProtocol {
    irp: Irp,
}

use crate::protocols::extended::LEGO_EXTENDED_IRP;

impl ComboDirectProtocol {
    pub fn new() -> Result<Self> {
        let irp = Irp::parse(LEGO_EXTENDED_IRP).map_err(Error::ProtocolError)?;
        Ok(Self { irp })
    }

    fn encode_msg(&self, msg: ComboDirectMessage) -> Result<Vec<u32>> {
        let mut vars = Vartable::new();
        vars.set("T".into(), 0u8.into());
        vars.set("E".into(), 0u8.into());
        vars.set("C".into(), msg.channel.into());
        vars.set("a".into(), 0u8.into());
        vars.set("M".into(), 1u8.into());
        vars.set("F".into(), msg.data.into());
        self.irp
            .encode_raw(vars, 1)
            .map(|res| res.raw)
            .map_err(Error::ProtocolError)
    }

    /// Encodes a Combo Direct command.
    pub fn encode_cmd(&self, channel: Channel, cmd: ComboDirectCommand) -> Result<Vec<u32>> {
        let msg = ComboDirectMessage {
            channel: channel as u8,
            data: ((cmd.blue as u8) << 2) | (cmd.red as u8),
        };
        self.encode_msg(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::Channel;
    #[test]
    fn test_combo_direct_encode_cmd() {
        let proto = ComboDirectProtocol::new().unwrap();
        let cmd = ComboDirectCommand {
            red: DirectState::Forward,
            blue: DirectState::Float,
        };
        let pulses = proto
            .encode_cmd(Channel::One, cmd)
            .expect("Encoding should succeed");
        assert!(!pulses.is_empty());

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157,
            552, 157, 263, 157, 263, 157, 263, 157, 552, 157, 552, 157, 552, 157, 552, 157, 552,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }

    #[test]
    fn test_combo_direct_all_states() {
        let proto = ComboDirectProtocol::new().unwrap();
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
                let pulses = proto.encode_cmd(Channel::One, cmd);
                assert!(
                    pulses.is_ok(),
                    "ComboDirect encoding failed for {:?} / {:?}",
                    red_state,
                    blue_state
                );
            }
        }
    }
}
