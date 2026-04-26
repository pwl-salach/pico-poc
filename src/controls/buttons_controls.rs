use super::{
    ConfigCommand, ControlCommand, EffectorCommand, EffectorDirection, InputDevice, ServoCommand,
};
use embedded_hal::digital::InputPin;

enum ControlMode {
    ServoControl,
    EffectorControl,
}

pub struct ButtonsControls<B1, B2, B3, B4, B5, B6, B7>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
    B6: InputPin,
    B7: InputPin,
{
    up_button: B1,
    down_button: B2,
    forward_button: B3,
    backward_button: B4,
    left_button: B5,
    right_button: B6,
    mode_switch: B7,
    current_servo_index: u8,
    servos_count: u8,
    servo_speed: u8,
    control_mode: ControlMode,
}

impl<B1, B2, B3, B4, B5, B6, B7> ButtonsControls<B1, B2, B3, B4, B5, B6, B7>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
    B6: InputPin,
    B7: InputPin,
{
    pub fn new(up: B1, down: B2, forward: B3, backward: B4, left: B5, right: B6, mode: B7) -> Self {
        Self {
            up_button: up,
            down_button: down,
            forward_button: forward,
            backward_button: backward,
            left_button: left,
            right_button: right,
            mode_switch: mode,
            current_servo_index: 0,
            servos_count: 16,
            servo_speed: 3,
            control_mode: ControlMode::ServoControl,
        }
    }
}

impl<B1, B2, B3, B4, B5, B6, B7> InputDevice for ButtonsControls<B1, B2, B3, B4, B5, B6, B7>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
    B5: InputPin,
    B6: InputPin,
    B7: InputPin,
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
                step += self.servo_speed as i8;
            }
            if self.down_button.is_high().unwrap() {
                step -= self.servo_speed as i8;
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
            let mut direction = EffectorDirection::Up;
            if self.up_button.is_high().unwrap() {
                direction = EffectorDirection::Up;
            } else if self.down_button.is_high().unwrap() {
                direction = EffectorDirection::Down;
            } else if self.forward_button.is_high().unwrap() {
                direction = EffectorDirection::Forward;
            } else if self.backward_button.is_high().unwrap() {
                direction = EffectorDirection::Backward;
            } else if self.left_button.is_high().unwrap() {
                direction = EffectorDirection::Left;
            } else if self.right_button.is_high().unwrap() {
                direction = EffectorDirection::Right;
            }
            Ok(ControlCommand::Effector(EffectorCommand {
                direction,
                step: self.servo_speed,
            }))
        }
    }
}
