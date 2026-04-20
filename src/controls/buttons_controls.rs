use super::{ControlCommand, InputDevice, ServoCommand};
use embedded_hal::digital::InputPin;

pub struct ButtonsControls<B1, B2, B3, B4>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
{
    up_button: B1,
    down_button: B2,
    next_servo: B3,
    previous_servo: B4,
    current_servo_index: u8,
    servos_count: u8,
}

impl<B1, B2, B3, B4> ButtonsControls<B1, B2, B3, B4>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
{
    pub fn new(up: B1, down: B2, next: B3, previous: B4) -> Self {
        Self {
            up_button: up,
            down_button: down,
            next_servo: next,
            previous_servo: previous,
            current_servo_index: 0,
            servos_count: 16,
        }
    }
}

impl<B1, B2, B3, B4> InputDevice for ButtonsControls<B1, B2, B3, B4>
where
    B1: InputPin,
    B2: InputPin,
    B3: InputPin,
    B4: InputPin,
{
    fn read_input(&mut self) -> Result<ControlCommand, ()> {
        let mut step = 0i8;
        if self.up_button.is_high().unwrap() {
            step += 1;
        }
        if self.down_button.is_high().unwrap() {
            step -= 1;
        }
        if self.next_servo.is_high().unwrap() {
            self.current_servo_index = (self.current_servo_index + 1) % self.servos_count;
        }
        if self.previous_servo.is_high().unwrap() {
            self.current_servo_index =
                (self.current_servo_index + self.servos_count - 1) % self.servos_count;
        }
        Ok(ControlCommand::Servo(ServoCommand {
            servo_index: self.current_servo_index,
            step,
        }))
    }
}
