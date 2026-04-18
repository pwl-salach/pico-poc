use embedded_hal::i2c::I2c;

const DEFAULT_ADDR: u8 = 0x40; // Default I2C address for PCA9685
const DEFAULT_OSC_FREQ: f32 = 25_000_000.0; // Internal oscillator frequency (Hz)
const PWM_RESOLUTION: f32 = 4096.0; // 12-bit counter, fixed by hardware
const PWM_FREQ_HZ: f32 = 50.0; // Default PWM frequency for servos (Hz)
const MIN_PWM_FREQ_HZ: f32 = 40.0; // Datasheet practical low limit
const MAX_PWM_FREQ_HZ: f32 = 1000.0; // Datasheet practical high limit

#[derive(Debug, Clone, Copy)]
pub struct ServoConfig {
    pub min_pulse: u16, // in microseconds
    pub max_pulse: u16, // in microseconds
    pub current_angle: f32,
}

#[derive(Debug)]
pub enum Pca9685Error<I2CErr> {
    I2c(I2CErr),
    ServoNotConfigured,
}

impl<I2CErr> From<I2CErr> for Pca9685Error<I2CErr> {
    fn from(err: I2CErr) -> Self {
        Pca9685Error::I2c(err)
    }
}
pub struct Pca9685<I2C> {
    i2c: I2C,
    servos: [Option<ServoConfig>; 16],
    oscilator_freq: f32,
    addr: u8,
    pwm_freq_hz: f32,
}

impl<I2C: I2c> Pca9685<I2C> {
    pub fn new(
        i2c: I2C,
        servos: [Option<ServoConfig>; 16],
        pwm_freq_hz: Option<f32>,
        oscilator_freq: Option<f32>,
        addr: Option<u8>,
    ) -> Self {
        Self {
            i2c,
            servos: servos,
            oscilator_freq: oscilator_freq.unwrap_or(DEFAULT_OSC_FREQ),
            addr: addr.unwrap_or(DEFAULT_ADDR),
            pwm_freq_hz: pwm_freq_hz.unwrap_or(PWM_FREQ_HZ),
        }
    }

    pub fn new_default(i2c: I2C, servos: [Option<ServoConfig>; 16]) -> Self {
        Self::new(i2c, servos, None, None, None)
    }

    pub fn init(&mut self) -> Result<(), I2C::Error> {
        // Reset the PCA9685
        self.i2c.write(self.addr, &[0x00, 0x80])?; // MODE1 register: normal mode
        // Set MODE2 to OUTDRV (totem-pole output)
        self.i2c.write(self.addr, &[0x01, 0x04])?; // MODE2 register: OUTDRV

        self.set_pwm_freq(self.pwm_freq_hz)?;
        self.update_all_servos()?;
        Ok(())
    }

    fn set_pwm_freq(&mut self, mut freq_hz: f32) -> Result<(), I2C::Error> {
        // One global PWM base frequency shared by all 16 channels.
        // Datasheet practical range for PCA9685 is about 40..1000 Hz.
        if freq_hz < MIN_PWM_FREQ_HZ {
            freq_hz = MIN_PWM_FREQ_HZ;
        }
        if freq_hz > MAX_PWM_FREQ_HZ {
            freq_hz = MAX_PWM_FREQ_HZ;
        }

        // Calculate prescale value with proper rounding
        let prescaleval = (self.oscilator_freq / (freq_hz * PWM_RESOLUTION)) - 1.0;
        let prescale = prescaleval as u8;
        debug_assert!(
            (3..=255).contains(&prescale),
            "Prescale out of range: {}",
            prescale
        );

        // Read current MODE1
        let mut oldmode = [0u8];
        self.i2c.write(self.addr, &[0x00])?; // Set register pointer
        // If your I2C trait supports read, use it here. Otherwise, assume default 0x20 (AI enabled)
        self.i2c.read(self.addr, &mut oldmode)?;
        // For portability, use 0x20 as default
        oldmode[0] = 0x20;

        let newmode = (oldmode[0] & !0x80) | 0x10; // Clear RESTART, set SLEEP
        self.i2c.write(self.addr, &[0x00, newmode])?; // Go to sleep
        self.i2c.write(self.addr, &[0xFE, prescale])?; // Set prescaler
        self.i2c.write(self.addr, &[0x00, oldmode[0]])?; // Wake up
        // Delay 5ms (use busy-wait for no_std)
        for _ in 0..300_000 {
            cortex_m::asm::nop();
        }
        // Set RESTART and AI bits
        self.i2c
            .write(self.addr, &[0x00, oldmode[0] | 0x80 | 0x20])?;
        self.pwm_freq_hz = freq_hz;
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
        self.i2c.write(self.addr, &data)?;
        Ok(())
    }

    pub fn set_servo_angle(
        &mut self,
        channel: u8,
        angle: f32,
    ) -> Result<(), Pca9685Error<I2C::Error>> {
        let angle = angle.clamp(0.0, 180.0);
        let servo = self.get_servo_for_channel(channel).unwrap();
        servo.current_angle = angle;
        Ok(())
    }

    pub fn update_all_servos(&mut self) -> Result<(), I2C::Error> {
        for channel in 0..16 {
            if self.servos[channel as usize].is_some() {
                self.update_servo(channel)?;
            }
        }
        Ok(())
    }

    pub fn update_servo(&mut self, channel: u8) -> Result<(), I2C::Error> {
        let payload = self.create_payload_for_channel(channel).unwrap();
        self.i2c.write(self.addr, &payload)?;
        Ok(())
    }

    fn create_payload_for_channel(
        &mut self,
        channel: u8,
    ) -> Result<[u8; 5], Pca9685Error<I2C::Error>> {
        let servo = self.get_servo_for_channel(channel)?;
        let pulse_range = servo.max_pulse - servo.min_pulse;
        let pulse_width =
            servo.min_pulse as f32 + (servo.current_angle / 180.0) * pulse_range as f32;
        let ticks = ((pulse_width / 1_000_000.0) * self.pwm_freq_hz * PWM_RESOLUTION) as u16;

        let reg = 0x06 + 4 * channel;
        Ok([reg, 0, 0, (ticks & 0xFF) as u8, (ticks >> 8) as u8])
    }

    fn get_servo_for_channel(
        &mut self,
        channel: u8,
    ) -> Result<&mut ServoConfig, Pca9685Error<I2C::Error>> {
        match self.servos[channel as usize].as_mut() {
            Some(servo) => Ok(servo),
            None => Err(Pca9685Error::ServoNotConfigured),
        }
    }
}
