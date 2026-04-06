use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x40; // Default I2C address for PCA9685
// const ADDR: u8 = 0x1F;
pub struct Pca9685<I2C> {
    i2c: I2C,
}

impl<I2C: I2c> Pca9685<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn init(&mut self) -> Result<(), I2C::Error> {
        // Reset the PCA9685
        self.i2c.write(ADDR, &[0x00, 0x80])?; // MODE1 register: normal mode
        // Set MODE2 to OUTDRV (totem-pole output)
        self.i2c.write(ADDR, &[0x01, 0x04])?; // MODE2 register: OUTDRV
        Ok(())
    }

    pub fn set_pwm_freq(&mut self, mut freq_hz: f32) -> Result<(), I2C::Error> {
        // Clamp frequency to datasheet limits (see PCA9685 datasheet)
        if freq_hz < 24.0 {
            freq_hz = 24.0; // lowest possible with prescale=255
        }
        if freq_hz > 1526.0 {
            freq_hz = 1526.0; // highest possible with prescale=3
        }

        // Calculate prescale value with proper rounding
        let prescaleval = (25_000_000.0 / (freq_hz * 4096.0)) - 1.0;
        let prescale = prescaleval as u8;
        debug_assert!(
            prescale >= 3 && prescale <= 255,
            "Prescale out of range: {}",
            prescale
        );

        // Read current MODE1
        let mut oldmode = [0u8];
        self.i2c.write(ADDR, &[0x00])?; // Set register pointer
        // If your I2C trait supports read, use it here. Otherwise, assume default 0x20 (AI enabled)
        // self.i2c.read(ADDR, &mut oldmode)?;
        // For portability, use 0x20 as default
        oldmode[0] = 0x20;

        let newmode = (oldmode[0] & !0x80) | 0x10; // Clear RESTART, set SLEEP
        self.i2c.write(ADDR, &[0x00, newmode])?; // Go to sleep
        self.i2c.write(ADDR, &[0xFE, prescale])?; // Set prescaler
        self.i2c.write(ADDR, &[0x00, oldmode[0]])?; // Wake up
        // Delay 5ms (use busy-wait for no_std)
        for _ in 0..300_000 {
            cortex_m::asm::nop();
        }
        // Set RESTART and AI bits
        self.i2c.write(ADDR, &[0x00, oldmode[0] | 0x80 | 0x20])?;
        Ok(())
    }

    pub fn set_pwm(&mut self, channel: u8, on: u16, off: u16) -> Result<(), I2C::Error> {
        // Write the register address and 4 bytes for ON/OFF
        let reg = 0x06 + 4 * channel;
        let data = [
            reg,
            (on & 0xFF) as u8,
            (on >> 8) as u8,
            (off & 0xFF) as u8,
            (off >> 8) as u8,
        ];
        self.i2c.write(ADDR, &data)?;
        Ok(())
    }

    /// Set servo angle (0-180 degrees) on a given channel.
    /// min_pulse and max_pulse are in microseconds (e.g., 1000, 2000).
    /// Typical servos use 1000us (0 deg) to 2000us (180 deg) at 50Hz.
    pub fn set_servo_angle(
        &mut self,
        channel: u8,
        angle: f32,
        min_pulse: u16,
        max_pulse: u16,
    ) -> Result<(), I2C::Error> {
        // Clamp angle
        let angle = angle.clamp(0.0, 180.0);
        // Pulse width in us
        let pulse = min_pulse as f32 + (max_pulse as f32 - min_pulse as f32) * (angle / 180.0);
        // At 50Hz, period = 20_000us, 4096 steps
        let period_us = 20_000.0;
        let ticks = ((pulse / period_us) * 4096.0) as u16;
        self.set_pwm(channel, 0, ticks)
    }
}
