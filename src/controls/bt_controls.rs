use super::{ControlCommand, InputDevice, ServoCommand};
use crate::hal;

pub struct HC05<D, P>
where
    D: hal::uart::UartDevice,
    P: hal::uart::ValidUartPinout<D>,
{
    uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>,
}

impl<D, P> HC05<D, P>
where
    D: hal::uart::UartDevice,
    P: hal::uart::ValidUartPinout<D>,
{
    pub fn new(uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>) -> Self {
        Self { uart }
    }
}

impl<D, P> InputDevice for HC05<D, P>
where
    D: hal::uart::UartDevice,
    P: hal::uart::ValidUartPinout<D>,
{
    fn read_input(&mut self) -> Result<ControlCommand, ()> {
        let mut buffer = [0u8; 2];
        self.uart.read_full_blocking(&mut buffer).unwrap();
        Ok(ControlCommand::Servo(ServoCommand {
            servo_index: buffer[0],
            step: buffer[1] as i8,
        }))
    }
}
