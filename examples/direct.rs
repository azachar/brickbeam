//! # Example: Direct (Combo Direct Remote Controller)
//!
//! This example demonstrates the `DirectRemoteController` for the Combo Direct
//! protocol, which sets specific states (Forward, Backward, Brake, or Float)
//! on two outputs independently. Here, we run the red motor forward and the blue
//! motor backward, then brake both.
//!
//! **Hardware setup instructions**:
//! See the project README.md at: [https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi](https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi)
//!
//! **Usage**:
//! ```bash
//! cargo run --example direct --features cir
//! ```

use brickbeam::{BrickBeam, Channel, ComboDirectCommand, DirectState, Result};
use figlet_rs::FIGfont;
use std::{thread, time::Duration};

fn welcome() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font
        .convert("Lego Direct RC for 2 motors")
        .unwrap();
    println!("{}", figure);
}

fn main() -> Result<()> {
    welcome();

    println!("Initializing brickbeam library for LEGO Power Functions IR control...");
    let brick_beam = BrickBeam::new("/dev/lirc0")?;
    let mut motors = brick_beam.create_direct_remote_controller(Channel::One)?;

    println!("Running Red motor forward and Blue motor backward on Channel One...");
    motors.send(ComboDirectCommand {
        red: DirectState::Forward,
        blue: DirectState::Backward,
    })?;
    thread::sleep(Duration::from_secs(3));

    println!("Stopping both motors...");
    motors.send(ComboDirectCommand {
        red: DirectState::Brake,
        blue: DirectState::Brake,
    })?;

    println!("Motors stopped successfully. Done.");
    Ok(())
}
