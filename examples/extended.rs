//! # Example: Extended (Extended Protocol Controller)
//!
//! Demonstrates the `ExtendedRemoteController`, which supports additional commands such as
//! incrementing motor speed, braking, toggling addresses, and more. In this example, we
//! first increment the speed, and then brake the red output on Channel One.
//!
//! **Hardware setup instructions**:
//! See the project README.md at: [https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi](https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi)
//!
//! **Usage**:
//! ```bash
//! cargo run --example extended --features cir
//! ```

use brickbeam::{BrickBeam, Channel, ExtendedCommand, Result};
use figlet_rs::FIGfont;
use std::{thread, time::Duration};

fn welcome() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("Lego Extended Protocol").unwrap();
    println!("{}", figure);
}

fn main() -> Result<()> {
    welcome();

    println!("Initializing brickbeam library for LEGO Power Functions IR control...");
    let brick_beam = BrickBeam::new("/dev/lirc0")?;
    let mut motor = brick_beam.create_extended_remote_controller(Channel::One)?;

    println!("Incrementing the speed of the red motor on channel One...");
    motor.send(ExtendedCommand::IncrementSpeedOnRedOutput)?;
    thread::sleep(Duration::from_secs(1));

    println!("Braking and floating the red motor...");
    motor.send(ExtendedCommand::BrakeThenFloatOnRedOutput)?;

    println!("Done.");
    Ok(())
}
