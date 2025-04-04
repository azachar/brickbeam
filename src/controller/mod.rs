//! # Controllers
//!
//! This module defines the main “remote controller” structs corresponding to
//! each type of LEGO® Power Functions protocol. Each controller wraps its
//! respective protocol encoder and a `PulseTransmitter` to send IR signals.
//!
//! The submodules include:
//! - `combo_direct` for Combo Direct protocol (two outputs, discrete states),
//! - `combo_speed` for Combo PWM protocol (two outputs, PWM),
//! - `extended` for the Extended protocol (toggle bits, brake, etc.),
//! - `speed` for the Single Output protocol (commonly called “Speed Remote”),
//! - `factory` for the core `BrickBeam` struct that instantiates controllers.
//!
//! **Thread Safety**:
//!   All the controllers produce IR signals in a “send” method that requires `&mut self`.
//!   This design ensures no concurrent “send” from multiple threads. If multi-threaded
//!   access is needed, wrap your controller instance in a Mutex.
//!
mod combo_direct;
mod combo_speed;
mod extended;
mod factory;
mod speed;

pub use combo_direct::DirectRemoteController;
pub use combo_speed::ComboSpeedRemoteController;
pub use extended::ExtendedRemoteController;
pub use factory::BrickBeam;
pub use speed::SpeedRemoteController;
