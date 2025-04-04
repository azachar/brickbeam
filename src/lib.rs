#![doc = r#"
# BRICKBEAM Library

**brickbeam** is an open source Rust library that enables you to transmit **LEGO® Power Functions**–style IR signals directly via the kernel’s modern LIRC/rc-core interface.
Specifically, it expects a `/dev/lircX` device (commonly `/dev/lirc0`) to be available, for example through overlays like `gpio-ir-tx` or `pwm-ir-tx`.

The crate implements multiple protocols as separate modules, including Extended, Single Output, Combo Direct, and Combo PWM. Each protocol is self-contained and exposes a remote controller API that encodes commands into the appropriate IR pulse sequence.

## Remote Controllers

**brickbeam** supports a variety of remote controllers, each designed for a specific type of LEGO® Power Functions command:

• Speed Remote Controller – Ideal for controlling train motors using the Single Output protocol. Configure PWM speeds (with negative values for reverse) or send discrete commands.

• Direct Remote Controller – Uses the Combo Direct protocol, allowing independent control of two outputs (such as setting one motor to “Forward” and the other to “Float”) on a single channel.

• Combo Speed Remote Controller – Leverages the Combo PWM protocol to simultaneously adjust PWM speeds on two outputs.

• Extended Remote Controller – Provides additional control features including braking, toggling speed increments/decrements, and address toggling.

## Usage Example

```rust
use brickbeam::{BrickBeam, Channel, Output, SingleOutputCommand, Result};

fn main() -> Result<()> {
    // Initialize the library with the IR transmit device path.
    let brick_beam = BrickBeam::new("/dev/lirc0")?;

    // Create a Speed Remote Controller for a train motor on Channel One using the red output.
    let mut motor = brick_beam.create_speed_remote_controller(Channel::One, Output::RED)?;

    // Set motor speed to 5 (values range from 0 to 7; negative numbers indicate reverse).
    motor.send(SingleOutputCommand::PWM(5))?;

    Ok(())
}
```

## Requirements

• **Linux kernel** with **LIRC (rc-core) support**, ensuring that `/dev/lircX` is available.

• A properly configured IR transmit overlay on your Raspberry Pi (e.g., `gpio-ir-tx` or `pwm-ir-tx`).

See the project's [README for hardware setup instructions](https://github.com/azachar/brickbeam#enabling-ir-on-the-raspberry-pi).

## Installation

1. Install prerequisites for building on Bookworm Raspberry Pi

    ```bash
    sudo apt-get update
    sudo apt-get install wget gnupg software-properties-common
    add-apt-repository "deb http://apt.llvm.org/bookworm/ llvm-toolchain-focal-17 main" && sudo apt-get update
    sudo apt-get install llvm-17-dev llvm-17-tools libffi-dev
    ```

    Ensure that `export LLVM_SYS_170_PREFIX="/usr/lib/llvm-17"` is available for cargo commands.

2. Add brickbeam as a cargo dependency

    ```bash
    export LLVM_SYS_170_PREFIX="/usr/lib/llvm-17"
    cargo add brickbeam
    ```

    1. If you are building for a native Linux environment
    and using the IR transmit device at `/dev/lirc0` for direct,
    hardware-accelerated IR signal transmission via the cir library,
    then use the default dependency configuration.
    This setting ensures that brickbeam uses dedicated IR hardware dependencies.

        ```ignore
        [dependencies]
        brickbeam = { version = "0.1.0"}
        ```

    2. For platforms such as macOS – where some of the IR hardware dependencies
    (used by the default "cir" feature) may not compile – you can build using only the emulator.
    To do so, disable the default features and enable the emulator feature as follows:

        ```ignore
        [dependencies]
        brickbeam = { version = "0.1.0", default-features = false}
        ```
        > **NOTE:**
        > Use the IR transmission emulator for **development** only (e.g., on macOS).
        > Do not use `default-features = false` in production!
        > In production, the cir feature must be enabled (this is the default setting).

For more complete examples, see the [examples](https://github.com/azachar/brickbeam/tree/main/examples) directory.

> **Disclaimer:**
> This project is **not** sponsored, authorized, or endorsed by the LEGO Group.
> “LEGO”® is a trademark of the LEGO Group. LEGO® and Power Functions™ are trademarks of the LEGO Group.
>
> The IRP protocols and data structures in this crate are re-expressions
> of publicly available timing information for LEGO® Power Functions signals.
> They are not copied verbatim from any confidential or proprietary document.
>
> **Acknowledgements:**
> Special thanks to my brother for his unwavering support throughout this project.
"#]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

mod controller;
mod device;
mod errors;
mod protocols;

pub use controller::*;
pub use device::{DefaultPulseTransmitter, PulseTransmitter};
pub use errors::{Error, Result};

pub use protocols::{
    Channel, ComboDirectCommand, ComboPwmCommand, DirectState, ExtendedCommand, Output,
    SingleOutputCommand, SingleOutputDiscrete,
};
