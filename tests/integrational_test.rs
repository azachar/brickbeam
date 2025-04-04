#[cfg(test)]
mod integration_test {

    use brickbeam::{
        BrickBeam, Channel, ComboDirectCommand, ComboPwmCommand, DirectState, ExtendedCommand,
        Output, Result, SingleOutputCommand, SingleOutputDiscrete,
    };

    #[test]
    fn test_extended_send() -> Result<()> {
        let brick_beam = BrickBeam::new("/dev/lirc0")?;
        let mut extended = brick_beam.create_extended_remote_controller(Channel::One)?;
        extended.send(ExtendedCommand::BrakeThenFloatOnRedOutput)?;
        Ok(())
    }

    #[test]
    fn test_speed_remote_controller_pwm_send() -> Result<()> {
        let brick_beam = BrickBeam::new("/dev/lirc0")?;
        let mut motor = brick_beam.create_speed_remote_controller(Channel::Two, Output::RED)?;
        motor.send(SingleOutputCommand::PWM(5))?;
        Ok(())
    }

    #[test]
    fn test_speed_remote_controller_discrete_send() -> Result<()> {
        let brick_beam = BrickBeam::new("/dev/lirc0")?;
        let mut motor = brick_beam.create_speed_remote_controller(Channel::Two, Output::RED)?;
        motor.send(SingleOutputCommand::Discrete(
            SingleOutputDiscrete::ToggleDirection,
        ))?;
        Ok(())
    }

    #[test]
    fn test_direct_remote_controller_send() -> Result<()> {
        let brick_beam = BrickBeam::new("/dev/lirc0")?;
        let mut motors = brick_beam.create_direct_remote_controller(Channel::Three)?;
        let cmd = ComboDirectCommand {
            red: DirectState::Forward,
            blue: DirectState::Float,
        };
        motors.send(cmd)?;
        Ok(())
    }

    #[test]
    fn test_combo_speed_remote_controller_send() -> Result<()> {
        let brick_beam = BrickBeam::new("/dev/lirc0")?;
        let mut motors = brick_beam.create_combo_speed_remote_controller(Channel::Four)?;
        let cmd = ComboPwmCommand {
            speed_red: 5,
            speed_blue: -3,
        };
        motors.send(cmd)?;
        Ok(())
    }
}
