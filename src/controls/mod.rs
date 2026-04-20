pub mod bt_controls;
pub mod buttons_controls;

pub struct ServoCommand {
    pub servo_index: u8,
    pub step: i8,
}

enum EffectorDirection {
    Up,
    Down,
    Left,
    Right,
}

pub struct EffectorCommand {
    pub direction: EffectorDirection,
    pub step: u8,
}

pub struct ConfigCommand {
    // Add fields for configuration parameters
}

pub enum ControlCommand {
    Servo(ServoCommand),
    Effector(EffectorCommand),
    Config(ConfigCommand),
}

pub trait InputDevice {
    fn read_input(&mut self) -> Result<ControlCommand, ()>;
}
