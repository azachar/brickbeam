//! # Extended Protocol
//!
//! This module implements the Extended protocol. It sets up an IRP string that describes the timing,
//! carrier frequency, and bit format required to generate the appropriate pulse sequence.
//!
//! Note: The IRP string used here was built based on available public documentation and reverse‑engineering.
//!
//! The protocol supports commands such as braking, toggling, and adjusting speed. The internal state (toggle
//! and address) is maintained between calls to support multiple commands on the same channel.

use super::Channel;
use crate::{Error, Result};
use irp::{Irp, Vartable};

/// Represents an extended command for the Extended protocol.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendedCommand {
    BrakeThenFloatOnRedOutput = 0b0000,
    IncrementSpeedOnRedOutput = 0b0001,
    DecrementSpeedOnRedOutput = 0b0010,
    ToggleForwardOrFloatOnBlueOutput = 0b0100,
    ToggleAddress = 0b0110,
    // Get in sync
    AlignToggle = 0b0111,
    // Reserved = 0b1000,
}

#[derive(Debug, Clone, Copy)]
struct ExtendedMessage {
    toggle: u8,
    channel: u8,
    address: u8,
    function: u8,
}

pub struct ExtendedProtocol {
    irp: Irp,
    toggle: u8,
    address: u8, // initial value 0; toggled by ToggleAddress
}

/// This IRP string now uses an explicit unit equal to the period of a 38 kHz carrier,
/// i.e. 1000000/38000 µs (~26.315789 µs). This makes the stream values an exact
/// multiple of the carrier period:
///
/// - General spec: {38k,33%,1000000/38000,msb}
///
///  • Carrier: 38 kHz
///
///  • Duty cycle: 33%
///
///  • Unit: 1000000/38000 µs (exactly the period of a 38 kHz carrier)
///
///  • Bit order: most-significant-bit first
///
/// - Bit spec: <6,-10|6,-21>
///
///  • Logical 0: flash 6 units and gap 10 units (6×26.315789 ≈ 157.89 µs; 10×26.315789 ≈ 263.16 µs)
///
///  • Logical 1: flash 6 units and gap 21 units (6×26.315789 ≈ 157.89 µs; 21×26.315789 ≈ 552.63 µs)
///
/// - Stream: (6,-39, T:1, E:1, C:2, A:1, M:3, D:4, L:4, 6,-39)
///
///  • Start and stop bursts: flash 6 units, gap 39 units
///
///  • Payload fields: Toggle (T:1), Escape (E:1), Channel (C:2),
///       Address (A:1), Mode (M:3), Data (D:4) and LRC (L:4)
///
/// - Definition: {L = 0xF^( (T*8+E*4+C)^(A*8+M)^D )}
///
///  • Computes LRC from the other fields
///
/// - Parameter spec: [T:0..1,E:0..1,C:0..3,A:0..1,M:0..7,D:0..15]
///
///  • Note: L is omitted here because it’s computed.
pub const LEGO_EXTENDED_IRP: &str = "\
{38k,33%,26.3157894737,msb}\
<6,-10|6,-21>\
(6,-39, T:1, E:1, C:2, a:1, M:3, F:4, L:4, 6,-39)\
{L = 0xF^( (T*8+E*4+C)^(a*8+M)^F )}\
[T:0..1,E:0..1,C:0..3,a:0..1,M:0..7,F:0..15]\
";

impl ExtendedProtocol {
    pub fn new() -> Result<Self> {
        let irp = Irp::parse(LEGO_EXTENDED_IRP).map_err(Error::ProtocolError)?;
        Ok(Self {
            irp,
            toggle: 0,
            address: 0,
        })
    }

    fn encode_msg(&self, msg: ExtendedMessage) -> Result<Vec<u32>> {
        let mut vars = Vartable::new();
        vars.set("T".into(), msg.toggle.into());
        vars.set("E".into(), 0u8.into());
        vars.set("C".into(), msg.channel.into());
        vars.set("a".into(), msg.address.into());
        vars.set("M".into(), 0u8.into());
        vars.set("F".into(), msg.function.into());
        self.irp
            .encode_raw(vars, 1)
            .map(|res| res.raw)
            .map_err(Error::ProtocolError)
    }

    /// Encodes an Extended command.
    pub fn encode_cmd(&mut self, channel: Channel, cmd: ExtendedCommand) -> Result<Vec<u32>> {
        let msg = ExtendedMessage {
            toggle: self.toggle,
            channel: channel as u8,
            address: self.address,
            function: cmd as u8,
        };
        let pulses = self.encode_msg(msg)?;
        self.toggle ^= 1;
        if cmd == ExtendedCommand::ToggleAddress {
            self.address = 1 - self.address;
        }
        Ok(pulses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocols::Channel;
    #[test]
    fn test_extended_encode_cmd() {
        let mut proto = ExtendedProtocol::new().unwrap();
        let pulses = proto
            .encode_cmd(Channel::One, ExtendedCommand::BrakeThenFloatOnRedOutput)
            .expect("Encoding should succeed");
        assert!(!pulses.is_empty());
    }
}

// In brickbeam/src/protocols/extended.rs (or in a dedicated test module)
#[cfg(test)]
mod extended_protocol_tests {
    use super::*;
    use crate::protocols::Channel;
    use crate::protocols::ExtendedCommand;

    #[test]
    fn test_extended_brake_command_structure() {
        let mut proto = ExtendedProtocol::new().unwrap();
        let pulses = proto
            .encode_cmd(Channel::One, ExtendedCommand::BrakeThenFloatOnRedOutput)
            .expect("Encoding should succeed");
        assert!(!pulses.is_empty());

        assert_eq!(pulses.len(), 36, "Unexpected pulse sequence length");

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157,
            263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 552, 157, 552, 157, 552, 157, 552,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }

    #[test]
    fn test_extended_toggle_forward_command_structure() {
        let mut proto = ExtendedProtocol::new().unwrap();
        let pulses = proto
            .encode_cmd(
                Channel::One,
                ExtendedCommand::ToggleForwardOrFloatOnBlueOutput,
            )
            .expect("Encoding should succeed");
        assert!(!pulses.is_empty());

        assert_eq!(pulses.len(), 36, "Unexpected pulse sequence length");

        let expected: Vec<u32> = vec![
            157, 1026, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157, 263, 157,
            263, 157, 263, 157, 552, 157, 263, 157, 263, 157, 552, 157, 263, 157, 552, 157, 552,
            157, 1026,
        ];
        assert_eq!(pulses, expected, "Pulse sequence does not match expected");
    }

    #[test]
    fn test_extended_toggle_address_changes_internal_state() {
        let mut proto = ExtendedProtocol::new().unwrap();
        let initial_address = proto.address;
        // Invoke ToggleAddress command and verify that internal address is toggled.
        let pulses = proto
            .encode_cmd(Channel::One, ExtendedCommand::ToggleAddress)
            .expect("Encoding should succeed");
        // Ensure that pulses are produced.
        assert!(!pulses.is_empty());
        // Check that the address has been toggled.
        assert_eq!(
            proto.address,
            1 - initial_address,
            "ToggleAddress should invert the internal address"
        );

        // Invoke the command a second time to toggle it back.
        let _ = proto
            .encode_cmd(Channel::One, ExtendedCommand::ToggleAddress)
            .expect("Encoding should succeed");
        assert_eq!(
            proto.address, initial_address,
            "ToggleAddress should invert the internal address back to its original state"
        );
    }
}
