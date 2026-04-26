use super::Messenger;
use crate::hal;
// Compatibility layer for the `hd44780-driver` crate, which uses `embedded-hal-0-2` while the rest of the codebase uses `embedded-hal-1.0`.
// This is needed as there are no `embedded-hal-1.0` compatible LCD drivers at the moment.
use embedded_hal_0_2::blocking::i2c::Write;
use hd44780_driver::{HD44780, bus::I2CBus};

const LCD_I2C_ADDRESS: u8 = 0x27;

pub struct LcdMessenger<I2C>
where
    I2C: Write,
{
    lcd: HD44780<I2CBus<I2C>>,
    timer: hal::Timer,
}

impl<I2C> LcdMessenger<I2C>
where
    I2C: Write,
{
    pub fn new(i2c: I2C, mut timer: hal::Timer) -> Self {
        let mut lcd =
            HD44780::new_i2c(i2c, LCD_I2C_ADDRESS, &mut timer).expect("failed to initialize LCD");
        // Clear the screen
        lcd.reset(&mut timer).expect("failed to reset lcd screen");
        lcd.clear(&mut timer).expect("failed to clear the screen");
        Self { lcd, timer }
    }
}

impl<I2C> Messenger for LcdMessenger<I2C>
where
    I2C: Write,
{
    fn send_message(&mut self, message: &str) {
        // Clear the screen before writing the new message
        self.lcd
            .clear(&mut self.timer)
            .expect("failed to clear LCD");
        self.lcd
            .write_str(message, &mut self.timer)
            .expect("failed to write message to LCD");
    }
}
