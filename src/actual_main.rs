use crate::hal::uart::{DataBits, StopBits, UartConfig};
use crate::{hal, hal::Clock, hal::fugit::RateExtU32};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

use crate::XTAL_FREQ_HZ;
use crate::controls::{
    ControlCommand, InputDevice, bt_controls::HC05, buttons_controls::ButtonsControls,
};
use crate::pca9685::{Pca9685, ServoConfig};

pub fn main(mut pac: hal::pac::Peripherals) -> ! {
    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    #[cfg(rp2040)]
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    #[cfg(rp2350)]
    let timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO25 as an output
    let led_pin = pins.gpio25.into_push_pull_output();

    let sda = pins
        .gpio4
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();
    let scl = pins
        .gpio5
        .reconfigure::<hal::gpio::FunctionI2C, hal::gpio::PullUp>();

    let i2c = hal::i2c::I2C::i2c0(
        pac.I2C0,
        sda,
        scl,
        100.kHz(),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    let mut servos = [None; 16];
    servos[0] = Some(ServoConfig {
        min_pulse: 0,
        max_pulse: 20_000,
        current_angle: 0.0,
    });
    servos[1] = Some(ServoConfig {
        min_pulse: 1000,
        max_pulse: 2000,
        current_angle: 0.0,
    });

    let mut pca9685 = Pca9685::new_default(i2c, servos);
    pca9685.init().unwrap();

    let uart_pins = (
        pins.gpio0.into_function::<hal::gpio::FunctionUart>(),
        pins.gpio1.into_function::<hal::gpio::FunctionUart>(),
    );
    let uart = hal::uart::UartPeripheral::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            UartConfig::new(9600.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    let hc_05 = HC05::new(uart);

    // let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    // let pwm = &mut pwm_slices.pwm2;
    // pwm.set_ph_correct();
    // pwm.enable();
    // let pwm_pin = pins.gpio21.into_function::<hal::gpio::FunctionPwm>();
    // let channel = &mut pwm.channel_b;
    // channel.output_to(pwm_pin);
    let buttons_contr = ButtonsControls::new(
        pins.gpio6.into_pull_up_input(),
        pins.gpio7.into_pull_up_input(),
        pins.gpio8.into_pull_up_input(),
        pins.gpio9.into_pull_up_input(),
    );

    program_loop(timer, led_pin, pca9685, buttons_contr);
}

fn program_loop<I2C>(
    mut timer: hal::Timer,
    mut led_pin: impl OutputPin,
    mut pca9685: Pca9685<I2C>,
    mut input_device: impl InputDevice,
    // uart: hal::uart::UartPeripheral<hal::uart::Enabled, D, P>,
    // mut channel: &mut hal::pwm::Channel<
    //     hal::pwm::Slice<hal::pwm::Pwm2, hal::pwm::FreeRunning>,
    //     hal::pwm::B,
    // >,
) -> !
where
    I2C: embedded_hal::i2c::I2c,
{
    loop {
        // Animate LED0 as before
        defmt::info!("on!");
        led_pin.set_high().unwrap();
        pca9685.set_servo_angle(1, 90.0).unwrap();
        pca9685.set_servo_angle(0, 90.0).unwrap();

        pca9685.update_all_servos().unwrap();
        // for i in (LOW..=HIGH).step_by(25) {
        //     timer.delay_us(500);
        //     channel.set_duty(i);
        // }
        timer.delay_ms(500);

        defmt::info!("off!");
        led_pin.set_low().unwrap();
        pca9685.set_servo_angle(1, 45.0).unwrap();
        pca9685.set_servo_angle(0, 45.0).unwrap();
        pca9685.update_all_servos().unwrap();
        // Ramp brightness down
        // for i in (LOW..=HIGH).rev().step_by(25) {
        //     timer.delay_us(50);
        //     channel.set_duty(i);
        // }
        timer.delay_ms(500);

        let input = input_device.read_input().unwrap();
        match input {
            ControlCommand::Servo(cmd) => {
                pca9685
                    .set_servo_angle(cmd.servo_index, cmd.step as f32 * 10.0)
                    .unwrap();
            }
            ControlCommand::Effector(cmd) => {
                // Handle effector command
            }
            ControlCommand::Config(cmd) => {
                // Handle configuration command
            }
        }
    }
}
