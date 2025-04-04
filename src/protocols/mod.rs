//! # Protocols Module
//!
//! The protocols here define how to encode commands into IR pulse sequences
//! following LEGO® Power Functions–style messages. We rely on IRP (Infrared
//! Remote Protocol) definitions to represent the carrier frequency, duty cycle,
//! and bit encoding of each protocol.
//!
//! ## IRP Explanation
//!
//! Each submodule (`combo_direct`, `combo_pwm`, `extended`, `single_output`) defines
//! a string constant describing the waveforms using the `irp` crate syntax. For example,
//! `38k` indicates a ~38 kHz carrier frequency, and the rest of the string
//! (e.g., `{33%,26.3157894737,msb} <6,-10|6,-21> ...`) describes how many cycles
//! to flash for a logical “0” or “1.”
//!
//! **Note**: While these IRP strings are compatible with how LEGO® Power Functions
//! signals are generally understood, they are **not** copied from any confidential
//! specification. They are an independent re-expression of wave timings publicly
//! documented.
//!
//! ## Protocol Summaries
//! - **Combo Direct**: For controlling both outputs with discrete states (Forward/Backward/Brake/Float).
//! - **Combo PWM**: For controlling both outputs with PWM speed steps (for example ±7).
//! - **Extended**: Provides extended operations like brake-then-float, toggle address, etc.
//! - **Single Output**: For the “Speed Remote” behavior on one output (PWM or discrete toggles).
//!
//! The main re-exports let you access the command enums (e.g. `ComboPwmCommand`)
//! and their respective protocols.

mod combo_direct;
mod combo_pwm;
mod extended;
mod single_output;

pub(crate) use combo_direct::ComboDirectProtocol;
pub(crate) use combo_pwm::ComboPwmProtocol;
pub(crate) use extended::ExtendedProtocol;
pub(crate) use single_output::SingleOutputProtocol;

pub use combo_direct::{ComboDirectCommand, DirectState};
pub use combo_pwm::ComboPwmCommand;
pub use extended::ExtendedCommand;
pub use single_output::{SingleOutputCommand, SingleOutputDiscrete};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Output {
    RED = 0,  // A
    BLUE = 1, // B
}

/// Maps user-specified PWM speeds into protocol-specific command values.
///
/// Acceptable inputs are from -7 to 8.
/// - A value of 0 sets the output to float.
/// - A value of 8 applies braking before floating.
///
/// Inputs beyond this range are clamped to the nearest valid value.
/// (e.g., inputs greater than 8 become 7; inputs less than -7 become -7)
pub fn map_speed(speed: i8) -> u8 {
    if speed == 0 || speed == 8 {
        speed as u8
    } else if speed > 0 {
        if speed > 7 {
            7
        } else {
            speed as u8
        }
    } else {
        let s = (-speed) as u8;
        if s > 7 {
            9
        } else {
            16 - s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_output_values() {
        assert_eq!(Channel::One as u8, 0);
        assert_eq!(Output::RED as u8, 0);
    }

    #[test]
    fn test_map_speed_values() {
        assert_eq!(map_speed(0), 0);
        assert_eq!(map_speed(8), 8);

        assert_eq!(map_speed(1), 1);
        assert_eq!(map_speed(-1), 15);

        assert_eq!(map_speed(7), 7);
        assert_eq!(map_speed(9), 7);

        assert_eq!(map_speed(-6), 10);
        assert_eq!(map_speed(-7), 9);
        assert_eq!(map_speed(-8), 9);
    }

    #[test]
    fn test_map_speed_extreme_values() {
        assert_eq!(map_speed(100), 7); // Clamp excessive positive values to 7
        assert_eq!(map_speed(-100), 9); // Clamp excessive negative values to -7 (encoded as 9)
    }
}
