//! # Device Layer
//!
//! This module deals with transmitting the raw IR pulses to the hardware.
//! - On Linux with the `cir` feature, `CirPulseTransmitter` uses `/dev/lirc<X>`.
//! - On other platforms (or if `cir` is disabled), it uses `PulseTransmitterEmulator`,
//!   which simply prints pulses for testing or development.
//!
//! `DefaultPulseTransmitter` is aliased to whichever implementation is active
//! on your platform/features.

mod api;

#[cfg(feature = "cir")]
mod cir;
#[cfg(not(feature = "cir"))]
mod emulator;

/// On non–Linux platforms, the `send_pulses` functions simply print the encoded pulse sequence, acting as a development/testing emulator.
/// The library abstracts the underlying hardware differences by using the `DefaultPulseTransmitter`:
///
/// • On Linux, this corresponds to the `CirPulseTransmitter`, which uses the `/dev/lirc0` interface.
///
/// • On other platforms, it uses an emulator (`PulseTransmitterEmulator`) that mimics the interface while doing nothing.
///
pub use api::PulseTransmitter;

#[cfg(feature = "cir")]
pub use cir::CirPulseTransmitter; // See note below.
#[cfg(not(feature = "cir"))]
// Note: PulseTransmitterEmulator is for development/testing on non-Linux platforms only.
pub use emulator::PulseTransmitterEmulator;

/// Default PulseTransmitter implementation.
/// On Linux, this is the actual IR transmitter; on other platforms, it is simulated.
#[cfg(feature = "cir")]
pub type DefaultPulseTransmitter = crate::device::CirPulseTransmitter;
#[cfg(not(feature = "cir"))]
pub type DefaultPulseTransmitter = crate::device::PulseTransmitterEmulator;
