//! # Example: Speed (Single Output Remote Controller)
//!
//! Shows how to use the `SpeedRemoteController` (Single Output protocol).
//! We gradually ramp up the speed of a single motor (on Channel One, RED output),
//! then stop it.
//!
//! **Hardware setup instructions**:
//! See the project README.md at: [https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi](https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi)
//!
//! **Usage**:
//! ```bash
//! cargo run --example speed --features cir
//! ```

use brickbeam::{BrickBeam, Channel, Output, Result, SingleOutputCommand};
use figlet_rs::FIGfont;
use std::{thread, time::Duration};

fn welcome() {
    let standard_font = FIGfont::standard().unwrap();
    let figure = standard_font.convert("Lego Speed RC for 1 Motor").unwrap();
    println!("{}", figure);
}

fn main() -> Result<()> {
    welcome();

    println!("Initializing brickbeam library for LEGO Power Functions IR control...");
    let brick_beam = BrickBeam::new("/dev/lirc0")?;
    let mut motor = brick_beam.create_speed_remote_controller(Channel::One, Output::RED)?;

    println!("Running a Red motor forward with speed 1 on Channel One for 2 seconds...");
    motor.send(SingleOutputCommand::PWM(1))?;
    thread::sleep(Duration::from_secs(2));

    println!("Increasing the Red motor speed to 7 for 3 seconds...");
    motor.send(SingleOutputCommand::PWM(7))?;
    thread::sleep(Duration::from_secs(3));

    println!("Stopping the Red motor...");
    motor.send(SingleOutputCommand::PWM(0))?;

    println!("Done.");
    Ok(())
}
