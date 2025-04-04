//! # Example: Combo (Combo Speed Remote Controller)
//!
//! This example demonstrates using the `ComboSpeedRemoteController` (Combo PWM protocol)
//! to drive two outputs at different speeds simultaneously. In this scenario, we assume both
//! a “red” and “blue” motor are connected to the same IR receiver on Channel One. We set
//! them running in different directions, then stop them.
//!
//! **Hardware setup instructions**:
//! See the project README.md at: [https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi](https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi)
//!
//! **Usage**:
//! ```bash
//! cargo run --example combo --features cir
//! ```

use brickbeam::{BrickBeam, Channel, ComboPwmCommand, Result};
use figlet_rs::FIGfont;
use std::{thread, time::Duration};

fn welcome() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("Lego Speed RC for 2 motors").unwrap();
    println!("{}", figure);
}

fn main() -> Result<()> {
    welcome();

    println!("Initializing brickbeam library for LEGO Power Functions IR control...");
    let brick_beam = BrickBeam::new("/dev/lirc0")?;
    let mut motors = brick_beam.create_combo_speed_remote_controller(Channel::One)?;

    println!("Running a Red Motor with speed 7 and a Blue Motor Backwards with speed -7...");
    motors.send(ComboPwmCommand {
        speed_red: 7,
        speed_blue: -7,
    })?;
    thread::sleep(Duration::from_secs(3));

    println!("Stopping both motors...");
    motors.send(ComboPwmCommand {
        speed_red: 0,
        speed_blue: 0,
    })?;

    println!("Done.");
    Ok(())
}
