use super::{
    ConfigCommand, ControlCommand, EffectorCommand, EffectorDirection, InputDevice, ServoCommand,
};
use embedded_hal::digital::InputPin;

enum ControlMode {
    ServoControl,
    EffectorControl,
}

pub struct ButtonsControls<B1, B2, B3, B4, B5>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
{
    up_button: B1,
    down_button: B2,
    left_button: B3,
    right_button: B4,
    mode_switch: B5,
    current_servo_index: u8,
    servos_count: u8,
    control_mode: ControlMode,
}

impl<B1, B2, B3, B4, B5> ButtonsControls<B1, B2, B3, B4, B5>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
{
    pub fn new(up: B1, down: B2, left: B3, right: B4, mode: B5) -> Self {
        Self {
            up_button: up,
            down_button: down,
            left_button: left,
            right_button: right,
            mode_switch: mode,
            current_servo_index: 0,
            servos_count: 16,
            control_mode: ControlMode::ServoControl,
        }
    }
}

impl<B1, B2, B3, B4, B5> InputDevice for ButtonsControls<B1, B2, B3, B4, B5>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
{
    fn read_input(&mut self) -> Result<ControlCommand, ()> {
        if self.mode_switch.is_high().unwrap() {
            if let ControlMode::ServoControl = self.control_mode {
                self.control_mode = ControlMode::EffectorControl;
            } else {
                self.control_mode = ControlMode::ServoControl;
            }
            return Ok(ControlCommand::Config(ConfigCommand::default()));
        }
        if let ControlMode::ServoControl = self.control_mode {
            let mut step = 0i8;
            if self.up_button.is_high().unwrap() {
                step += 1;
            }
            if self.down_button.is_high().unwrap() {
                step -= 1;
            }
            if self.left_button.is_high().unwrap() {
                self.current_servo_index = (self.current_servo_index + 1) % self.servos_count;
            }
            if self.right_button.is_high().unwrap() {
                self.current_servo_index =
                    (self.current_servo_index + self.servos_count - 1) % self.servos_count;
            }
            Ok(ControlCommand::Servo(ServoCommand {
                servo_index: self.current_servo_index,
                step,
            }))
        } else {
            let step = 5u8;
            let mut direction = EffectorDirection::Up;
            if self.up_button.is_high().unwrap() {
                direction = EffectorDirection::Up;
            } else if self.down_button.is_high().unwrap() {
                direction = EffectorDirection::Down;
            } else if self.left_button.is_high().unwrap() {
                direction = EffectorDirection::Left;
            } else if self.right_button.is_high().unwrap() {
                direction = EffectorDirection::Right;
            }
            Ok(ControlCommand::Effector(EffectorCommand {
                direction,
                step,
            }))
        }
    }
}
